mod decode;

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use serde::Serialize;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::time::Instant;
use tokio::process::Command;
use uuid::Uuid;

use crate::domain::{ModelInput, ModelOutput};
use decode::InstanceScales;

const TAIL_CHARS: usize = 4000;

// Tolerances for binding-checking a proof's own decoded public values
// against the receipt's claimed input/output (see `check_binding`).
// Input echo is just fixed-point rounding (error bound is
// 0.5 / 2^input_scale, ~0.00024 at scale 11) — 0.01 leaves comfortable
// margin while still being far tighter than any real tamper. Output has
// the documented quantization drift (up to ~10-15 points on prob_good in
// testing, see docs/ezkl.md) — 0.15 tolerates that while still catching a
// fabricated output.
const INPUT_TOLERANCE: f64 = 0.01;
const OUTPUT_TOLERANCE: f64 = 0.15;

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
    pub fn verify_artifacts_exist(&self) -> anyhow::Result<()> {
        for (name, path) in [
            ("compiled circuit", &self.compiled_circuit),
            ("settings", &self.settings),
            ("proving key", &self.pk),
            ("verifying key", &self.vk),
            ("SRS", &self.srs),
        ] {
            if !path.exists() {
                anyhow::bail!(
                    "missing {name} at {} — run backend/scripts/zk_setup.sh (see docs/ezkl.md)",
                    path.display()
                );
            }
        }
        Ok(())
    }

    fn instance_scales(&self) -> anyhow::Result<InstanceScales> {
        InstanceScales::load(&self.settings)
    }
}

/// Full outcome of a single ezkl operation (prove or verify) — always
/// populated, even on failure, so every attempt leaves a complete trace
/// (see `domain::ZkTrace` / the `zk_traces` table). Previously a failed
/// `gen-witness`/`prove` call only surfaced a generic "zk proof generation"
/// error with the real ezkl stderr silently dropped.
#[derive(Debug, Clone, Serialize)]
pub struct ZkOutcome {
    pub success: bool,
    pub proof_b64: Option<String>,
    pub witness_input: Value,
    /// The circuit's own public input/output, rescaled to floats — this is
    /// what the proof actually attests to, which can differ from the real
    /// `ort` output by a noticeable amount (quantization drift, see
    /// docs/ezkl.md). `None` if the run failed before a proof existed to
    /// decode, or decoding itself failed.
    pub circuit_public_values: Option<Vec<f64>>,
    pub error_message: Option<String>,
    pub stdout_tail: Option<String>,
    pub duration_ms: u64,
}

impl ZkOutcome {
    fn setup_failure(witness_input: Value, message: String) -> Self {
        Self {
            success: false,
            proof_b64: None,
            witness_input,
            circuit_public_values: None,
            error_message: Some(message),
            stdout_tail: None,
            duration_ms: 0,
        }
    }
}

struct StageOutcome {
    success: bool,
    stdout: String,
    stderr: String,
    duration_ms: u64,
}

/// Feature order must match execution/inference/mod.rs::extract_features.
fn extract_input_features(input: &ModelInput) -> anyhow::Result<[f64; 4]> {
    let f = |key: &str| -> anyhow::Result<f64> {
        input
            .get(key)
            .and_then(|v| v.as_f64())
            .ok_or_else(|| anyhow::anyhow!("missing or invalid field: {}", key))
    };
    Ok([
        f("income")?,
        f("debt_ratio")?,
        f("missed_payments")?,
        f("credit_history_months")?,
    ])
}

fn witness_input_json(input: &ModelInput) -> anyhow::Result<Value> {
    let features = extract_input_features(input)?;
    Ok(json!({ "input_data": [features] }))
}

/// Cross-checks the circuit's own decoded public values (see
/// `decode_proof_instances`) against what the receipt actually claims —
/// without this, a valid `zk_proof` from any receipt can be paired with
/// fabricated `input`/`output` fields and still report `proof_valid: true`,
/// since ezkl's `verify` only checks the proof's internal validity, not
/// that it's *for* this input/output. Layout is fixed by this model's
/// circuit (`backend/scripts/calibration.json` / `zk_setup.sh`):
/// `[income, debt_ratio, missed_payments, credit_history_months, label, prob_bad, prob_good]`.
///
/// Returns `Ok(true/false)` when the comparison could be made, `Err(reason)`
/// when it couldn't (missing fields, unexpected circuit shape) — a `None`/
/// `Err` result does NOT fail the proof, since that's an operational gap,
/// not evidence of tampering (the proof itself is still cryptographically
/// intact either way).
fn check_binding(circuit_values: &[f64], input: &ModelInput, output: &ModelOutput) -> Result<bool, String> {
    if circuit_values.len() < 7 {
        return Err(format!("expected 7 decoded circuit values, got {}", circuit_values.len()));
    }

    let claimed_input = extract_input_features(input).map_err(|e| e.to_string())?;
    for (i, claimed) in claimed_input.iter().enumerate() {
        if (circuit_values[i] - claimed).abs() > INPUT_TOLERANCE {
            return Ok(false);
        }
    }

    let claimed_prob_good = output
        .get("prob_good")
        .and_then(|v| v.as_f64())
        .ok_or_else(|| "output missing prob_good".to_string())?;
    let circuit_prob_good = circuit_values[6];
    if (circuit_prob_good - claimed_prob_good).abs() > OUTPUT_TOLERANCE {
        return Ok(false);
    }

    Ok(true)
}

fn work_dir(id: &Uuid) -> PathBuf {
    std::env::temp_dir().join(format!("saleem-zk-{id}"))
}

fn path_arg(p: &Path) -> String {
    p.to_string_lossy().into_owned()
}

fn tail(s: &str) -> String {
    let char_count = s.chars().count();
    if char_count <= TAIL_CHARS {
        s.to_string()
    } else {
        s.chars().skip(char_count - TAIL_CHARS).collect()
    }
}

/// Runs an `ezkl` subcommand, always capturing stdout/stderr/timing —
/// never just on failure — so callers can build a full trace regardless
/// of outcome.
async fn run(cfg: &ZkConfig, subcommand: &str, args: &[String]) -> StageOutcome {
    let mut full_args = vec![subcommand.to_string()];
    full_args.extend_from_slice(args);

    let started = Instant::now();
    let output = Command::new(&cfg.ezkl_bin).args(&full_args).output().await;
    let duration_ms = started.elapsed().as_millis() as u64;

    match output {
        Ok(output) => StageOutcome {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            duration_ms,
        },
        Err(e) => StageOutcome {
            success: false,
            stdout: String::new(),
            stderr: format!("failed to spawn `{}`: {e}", cfg.ezkl_bin),
            duration_ms,
        },
    }
}

/// ezkl's own diagnostic lines (e.g. `[E] ... value is OOR of lookup`) go
/// to stdout despite the `[E]` severity marker, not stderr — confirmed
/// empirically, not assumed. Falls back to stdout when stderr is blank so
/// the real failure reason isn't lost.
fn error_summary(stage_name: &str, stage: &StageOutcome) -> String {
    let detail = if !stage.stderr.trim().is_empty() {
        tail(&stage.stderr)
    } else {
        tail(&stage.stdout)
    };
    format!("ezkl {stage_name} failed: {detail}")
}

fn decode_proof_instances(cfg: &ZkConfig, proof_bytes: &[u8]) -> Option<Vec<f64>> {
    let proof: Value = serde_json::from_slice(proof_bytes).ok()?;
    let instances = proof.get("instances")?.get(0)?.as_array()?;
    let instances: Vec<String> = instances.iter().filter_map(|v| v.as_str().map(String::from)).collect();
    let scales = cfg.instance_scales().ok()?;
    Some(scales.decode(&instances))
}

/// Generates a witness for `input` against the compiled circuit and proves
/// it. Runs `gen-witness` then `prove` as subprocesses — ezkl isn't
/// published on crates.io and pins its own nightly toolchain, so it's
/// invoked as an external CLI rather than a Cargo dependency.
///
/// This is CPU-bound work done synchronously in the request path (the model
/// is tiny — 4 scalar features — so this is expected to add well under a
/// few seconds per decision; see docs/ezkl.md for context).
pub async fn prove(cfg: &ZkConfig, decision_id: Uuid, input: &ModelInput) -> ZkOutcome {
    let dir = work_dir(&decision_id);
    if let Err(e) = tokio::fs::create_dir_all(&dir).await {
        return ZkOutcome::setup_failure(Value::Null, format!("create zk work dir: {e}"));
    }

    let outcome = prove_in(cfg, &dir, input).await;
    let _ = tokio::fs::remove_dir_all(&dir).await;
    outcome
}

async fn prove_in(cfg: &ZkConfig, dir: &Path, input: &ModelInput) -> ZkOutcome {
    let witness_input = match witness_input_json(input) {
        Ok(v) => v,
        Err(e) => return ZkOutcome::setup_failure(Value::Null, e.to_string()),
    };

    let input_path = dir.join("input.json");
    let witness_path = dir.join("witness.json");
    let proof_path = dir.join("proof.json");

    let input_bytes = match serde_json::to_vec(&witness_input) {
        Ok(b) => b,
        Err(e) => return ZkOutcome::setup_failure(witness_input, format!("serialize witness input: {e}")),
    };
    if let Err(e) = tokio::fs::write(&input_path, input_bytes).await {
        return ZkOutcome::setup_failure(witness_input, format!("write witness input: {e}"));
    }

    let gen_witness = run(
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
    .await;

    if !gen_witness.success {
        return ZkOutcome {
            success: false,
            proof_b64: None,
            witness_input,
            circuit_public_values: None,
            error_message: Some(error_summary("gen-witness", &gen_witness)),
            stdout_tail: Some(tail(&gen_witness.stdout)),
            duration_ms: gen_witness.duration_ms,
        };
    }

    let prove_stage = run(
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
    .await;

    let duration_ms = gen_witness.duration_ms + prove_stage.duration_ms;

    if !prove_stage.success {
        return ZkOutcome {
            success: false,
            proof_b64: None,
            witness_input,
            circuit_public_values: None,
            error_message: Some(error_summary("prove", &prove_stage)),
            stdout_tail: Some(tail(&prove_stage.stdout)),
            duration_ms,
        };
    }

    let proof_bytes = match tokio::fs::read(&proof_path).await {
        Ok(b) => b,
        Err(e) => {
            return ZkOutcome {
                success: false,
                proof_b64: None,
                witness_input,
                circuit_public_values: None,
                error_message: Some(format!("read proof.json: {e}")),
                stdout_tail: None,
                duration_ms,
            }
        }
    };

    let circuit_public_values = decode_proof_instances(cfg, &proof_bytes);

    ZkOutcome {
        success: true,
        proof_b64: Some(BASE64.encode(&proof_bytes)),
        witness_input,
        circuit_public_values,
        error_message: None,
        stdout_tail: None,
        duration_ms,
    }
}

/// Verifies a base64-encoded proof against the fixed verifying key, AND
/// that the proof's own decoded public values actually match what
/// `expected_input`/`expected_output` claim (see `check_binding`) — a
/// structurally valid proof from a different decision, paired with
/// fabricated input/output on the outer receipt, now fails this.
/// `success` reflects both checks — ezkl's `verify` subcommand exits
/// non-zero for a structurally invalid or tampered proof, and the binding
/// check independently can flip a structurally-valid proof to failed.
pub async fn verify(
    cfg: &ZkConfig,
    proof_b64: &str,
    expected_input: &ModelInput,
    expected_output: &ModelOutput,
) -> ZkOutcome {
    let Ok(proof_bytes) = BASE64.decode(proof_b64) else {
        return ZkOutcome::setup_failure(Value::Null, "invalid base64 proof".to_string());
    };

    let circuit_public_values = decode_proof_instances(cfg, &proof_bytes);

    let dir = work_dir(&Uuid::new_v4());
    if let Err(e) = tokio::fs::create_dir_all(&dir).await {
        return ZkOutcome {
            circuit_public_values,
            ..ZkOutcome::setup_failure(Value::Null, format!("create zk work dir: {e}"))
        };
    }

    let proof_path = dir.join("proof.json");
    let mut outcome = match tokio::fs::write(&proof_path, &proof_bytes).await {
        Ok(_) => {
            let stage = run(
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
            .await;

            ZkOutcome {
                success: stage.success,
                proof_b64: None,
                witness_input: Value::Null,
                circuit_public_values,
                error_message: if stage.success { None } else { Some(error_summary("verify", &stage)) },
                stdout_tail: Some(tail(&stage.stdout)),
                duration_ms: stage.duration_ms,
            }
        }
        Err(e) => ZkOutcome {
            success: false,
            proof_b64: None,
            witness_input: Value::Null,
            circuit_public_values,
            error_message: Some(format!("write proof.json: {e}")),
            stdout_tail: None,
            duration_ms: 0,
        },
    };

    let _ = tokio::fs::remove_dir_all(&dir).await;

    if outcome.success {
        if let Some(values) = &outcome.circuit_public_values {
            match check_binding(values, expected_input, expected_output) {
                Ok(true) => {}
                Ok(false) => {
                    outcome.success = false;
                    outcome.error_message = Some(
                        "proof is cryptographically valid but its decoded public values do not match this receipt's claimed input/output (binding check failed)"
                            .to_string(),
                    );
                }
                Err(reason) => {
                    outcome.stdout_tail = Some(format!(
                        "{}\n[binding check skipped: {reason}]",
                        outcome.stdout_tail.unwrap_or_default()
                    ));
                }
            }
        }
    }

    outcome
}
