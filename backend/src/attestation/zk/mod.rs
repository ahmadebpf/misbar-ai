use anyhow::{bail, Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use serde_json::json;
use std::path::{Path, PathBuf};
use tokio::process::Command;
use uuid::Uuid;

use crate::domain::ModelInput;

/// Paths to the circuit artifacts produced once by `scripts/zk_setup.sh`
/// (see docs/ezkl.md). Nothing in this module generates these — it only
/// consumes them per-request via `ezkl gen-witness` / `prove` / `verify`.
#[derive(Clone)]
pub struct ZkConfig {
    ezkl_bin: String,
    compiled_circuit: PathBuf,
    settings: PathBuf,
    pk: PathBuf,
    vk: PathBuf,
    srs: PathBuf,
}

impl ZkConfig {
    /// `circuit_dir` is the directory written by `scripts/zk_setup.sh`
    /// (backend/circuit by default). The `ezkl` binary is resolved from
    /// PATH unless overridden via the `EZKL_BIN` env var.
    pub fn new(circuit_dir: &Path) -> Self {
        Self {
            ezkl_bin: std::env::var("EZKL_BIN").unwrap_or_else(|_| "ezkl".to_string()),
            compiled_circuit: circuit_dir.join("network.compiled"),
            settings: circuit_dir.join("settings.json"),
            pk: circuit_dir.join("pk.key"),
            vk: circuit_dir.join("vk.key"),
            srs: circuit_dir.join("kzg.srs"),
        }
    }

    /// Fails fast at startup if `scripts/zk_setup.sh` hasn't been run yet.
    pub fn verify_artifacts_exist(&self) -> Result<()> {
        for (name, path) in [
            ("compiled circuit", &self.compiled_circuit),
            ("settings", &self.settings),
            ("proving key", &self.pk),
            ("verifying key", &self.vk),
            ("SRS", &self.srs),
        ] {
            if !path.exists() {
                bail!(
                    "missing {name} at {} — run backend/scripts/zk_setup.sh (see docs/ezkl.md)",
                    path.display()
                );
            }
        }
        Ok(())
    }
}

/// Feature order must match execution/inference/mod.rs::extract_features.
fn witness_input_json(input: &ModelInput) -> Result<serde_json::Value> {
    let f = |key: &str| -> Result<f64> {
        input
            .get(key)
            .and_then(|v| v.as_f64())
            .ok_or_else(|| anyhow::anyhow!("missing or invalid field: {}", key))
    };
    Ok(json!({
        "input_data": [[
            f("income")?,
            f("debt_ratio")?,
            f("missed_payments")?,
            f("credit_history_months")?,
        ]]
    }))
}

fn work_dir(id: &Uuid) -> PathBuf {
    std::env::temp_dir().join(format!("misbar-zk-{id}"))
}

fn path_arg(p: &Path) -> String {
    p.to_string_lossy().into_owned()
}

async fn run(cfg: &ZkConfig, subcommand: &str, args: &[String]) -> Result<()> {
    let mut full_args = vec![subcommand.to_string()];
    full_args.extend_from_slice(args);

    let output = Command::new(&cfg.ezkl_bin)
        .args(&full_args)
        .output()
        .await
        .with_context(|| format!("failed to spawn `{}`", cfg.ezkl_bin))?;

    if !output.status.success() {
        bail!(
            "ezkl {subcommand} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}

/// Generates a witness for `input` against the compiled circuit, proves it,
/// and returns the base64-encoded proof. Runs `gen-witness` then `prove` as
/// subprocesses — ezkl isn't published on crates.io and pins its own nightly
/// toolchain, so it's invoked as an external CLI rather than a Cargo dependency.
///
/// This is CPU-bound work done synchronously in the request path (the model
/// is tiny — 4 scalar features — so this is expected to add well under a
/// few seconds per decision; see docs/ezkl.md for context).
pub async fn prove(cfg: &ZkConfig, decision_id: Uuid, input: &ModelInput) -> Result<String> {
    let dir = work_dir(&decision_id);
    tokio::fs::create_dir_all(&dir)
        .await
        .context("create zk work dir")?;

    let result = prove_in(cfg, &dir, input).await;
    let _ = tokio::fs::remove_dir_all(&dir).await;
    result
}

async fn prove_in(cfg: &ZkConfig, dir: &Path, input: &ModelInput) -> Result<String> {
    let input_path = dir.join("input.json");
    let witness_path = dir.join("witness.json");
    let proof_path = dir.join("proof.json");

    tokio::fs::write(&input_path, serde_json::to_vec(&witness_input_json(input)?)?)
        .await
        .context("write witness input")?;

    run(
        cfg,
        "gen-witness",
        &[
            "-D".into(),
            path_arg(&input_path),
            "-M".into(),
            path_arg(&cfg.compiled_circuit),
            "-O".into(),
            path_arg(&witness_path),
        ],
    )
    .await
    .context("ezkl gen-witness")?;

    run(
        cfg,
        "prove",
        &[
            "-W".into(),
            path_arg(&witness_path),
            "-M".into(),
            path_arg(&cfg.compiled_circuit),
            "--pk-path".into(),
            path_arg(&cfg.pk),
            "--proof-path".into(),
            path_arg(&proof_path),
            "--srs-path".into(),
            path_arg(&cfg.srs),
        ],
    )
    .await
    .context("ezkl prove")?;

    let proof_bytes = tokio::fs::read(&proof_path).await.context("read proof.json")?;
    Ok(BASE64.encode(&proof_bytes))
}

/// Verifies a base64-encoded proof against the fixed verifying key. Returns
/// `Ok(false)` (not `Err`) for a structurally invalid or rejected proof —
/// only I/O/process failures are surfaced as `Err`.
pub async fn verify(cfg: &ZkConfig, proof_b64: &str) -> Result<bool> {
    let Ok(proof_bytes) = BASE64.decode(proof_b64) else {
        return Ok(false);
    };

    let dir = work_dir(&Uuid::new_v4());
    tokio::fs::create_dir_all(&dir)
        .await
        .context("create zk work dir")?;

    let proof_path = dir.join("proof.json");
    let write_result = tokio::fs::write(&proof_path, &proof_bytes).await;

    let verified = if write_result.is_ok() {
        run(
            cfg,
            "verify",
            &[
                "-S".into(),
                path_arg(&cfg.settings),
                "--proof-path".into(),
                path_arg(&proof_path),
                "--vk-path".into(),
                path_arg(&cfg.vk),
                "--srs-path".into(),
                path_arg(&cfg.srs),
            ],
        )
        .await
        .is_ok()
    } else {
        false
    };

    let _ = tokio::fs::remove_dir_all(&dir).await;
    Ok(verified)
}
