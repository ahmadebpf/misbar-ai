use anyhow::{bail, Context, Result};
use chrono::Utc;
use std::path::Path;
use uuid::Uuid;

use crate::attestation::{hashing, receipts, signatures::Keypair, zk, zk::ZkConfig};
use crate::domain::{ModelInput, ModelOutput, ReceiptRecord, VerificationResult, ZkTrace};
use crate::execution::{features, inference::OnnxSession};
use crate::registry::{ModelEntry, PolicyEntry, Registry};
use crate::store::Store;
use crate::verification::verifier;

pub struct Gateway {
    session: OnnxSession,
    keypair: Keypair,
    pub policy_id: Uuid,
    pub policy_hash: String,
    store: Store,
    registry: Registry,
    zk_config: ZkConfig,
}

pub struct ExecutionResult {
    pub decision_id: Uuid,
    pub receipt_id: Uuid,
    pub output: ModelOutput,
}

pub struct GatewayInfo {
    pub model_id: Uuid,
    pub model_hash: String,
    pub policy_id: Uuid,
    pub policy_hash: String,
}

impl Gateway {
    /// `policy_artifact` is the canonical string for the active policy.
    /// The gateway only stores its hash — any string is accepted.
    /// Model name/version come from `session` (read out of model.onnx's own
    /// metadata — see `OnnxSession::load`), not passed in here — distinct
    /// from `model_id` (a stable UUID) and `model_hash` (content hash of
    /// `model.onnx`).
    /// `key_path` is the file used to persist the Ed25519 signing key across restarts.
    pub fn new(
        session: OnnxSession,
        store: Store,
        registry: Registry,
        policy_artifact: &str,
        key_path: &Path,
        circuit_dir: &Path,
    ) -> Result<Self> {
        let policy_hash = hashing::hash_str(policy_artifact);

        // Stable policy_id derived from policy content — same policy, same ID.
        const POLICY_NS: Uuid = Uuid::from_bytes([
            0x6b, 0xa7, 0xb8, 0x14, 0x9d, 0xad, 0x11, 0xd1,
            0x80, 0xb4, 0x00, 0xc0, 0x4f, 0xd4, 0x30, 0xc8,
        ]);
        let policy_id = Uuid::new_v5(&POLICY_NS, policy_hash.as_bytes());

        registry.register_model(ModelEntry {
            id: session.model_id,
            name: session.model_name.clone(),
            version: session.model_version.clone(),
            artifact_hash: session.model_hash.clone(),
            registered_at: Utc::now(),
        });

        registry.register_policy(PolicyEntry {
            id: policy_id,
            name: "credit-scoring-policy".to_string(),
            version: "v1.0.0".to_string(),
            artifact_hash: policy_hash.clone(),
            registered_at: Utc::now(),
        });

        let keypair = Keypair::load_or_generate(key_path)?;

        let zk_config = ZkConfig::new(circuit_dir);
        zk_config
            .verify_artifacts_exist()
            .context("zk circuit artifacts")?;

        tracing::info!(
            model_id = %session.model_id,
            model_hash = %session.model_hash,
            policy_id = %policy_id,
            policy_hash = %policy_hash,
            "gateway initialised"
        );

        Ok(Self {
            session,
            keypair,
            policy_id,
            policy_hash,
            store,
            registry,
            zk_config,
        })
    }

    pub async fn execute(&self, input: ModelInput) -> Result<ExecutionResult> {
        let decision_id = Uuid::new_v4();
        let receipt_id = Uuid::new_v4();
        let timestamp = Utc::now();

        let input_hash = features::input_hash(&input)?;
        let output = self.session.run(&input)?;
        let output_hash = features::output_hash(&output)?;

        // Proves the compiled circuit produces this output from this input —
        // a stronger claim than the signature below, which only attests that
        // saleem signed this record. Synchronous: the request waits for the
        // proof (small model, expected well under a few seconds).
        let outcome = zk::prove(&self.zk_config, decision_id, &input).await;

        // Persisted regardless of success — a failed proof previously left
        // zero trace anywhere (not even decision_id), with the real ezkl
        // error dropped by `.context("zk proof generation")`. Trace-write
        // failures are logged, not fatal to the request.
        let trace = ZkTrace {
            trace_id: Uuid::new_v4(),
            decision_id,
            receipt_id: outcome.success.then_some(receipt_id),
            phase: "prove".to_string(),
            source: "gateway".to_string(),
            success: outcome.success,
            witness_input: Some(outcome.witness_input.clone()),
            circuit_public_values: outcome.circuit_public_values.clone(),
            error_message: outcome.error_message.clone(),
            stdout_tail: outcome.stdout_tail.clone(),
            duration_ms: outcome.duration_ms as i64,
            created_at: Utc::now(),
        };
        if let Err(e) = self.store.save_zk_trace(&trace).await {
            tracing::error!(error = %e, %decision_id, "failed to persist zk trace");
        }

        let Some(zk_proof) = outcome.proof_b64 else {
            bail!(
                "zk proof generation failed: {}",
                outcome.error_message.unwrap_or_default()
            );
        };

        let payload = receipts::signable_payload(
            &decision_id,
            &self.session.model_id,
            &self.policy_id,
            &input_hash,
            &output_hash,
            &timestamp,
        );
        let signature = self.keypair.sign(&payload);

        let receipt = receipts::assemble(
            receipt_id,
            decision_id,
            self.session.model_id,
            self.session.model_hash.clone(),
            self.policy_id,
            self.policy_hash.clone(),
            input,
            input_hash,
            output.clone(),
            output_hash,
            timestamp,
            signature,
            Some(zk_proof),
        );

        self.store.save_receipt(&receipt).await?;

        Ok(ExecutionResult { decision_id, receipt_id, output })
    }

    /// Verifies a receipt already in the store (`GET /verify/{id}`),
    /// persisting a `source: "lookup"` trace of the attempt.
    pub async fn verify_receipt(&self, receipt_id: &Uuid) -> Result<Option<(ReceiptRecord, VerificationResult)>> {
        let Some(receipt) = self.store.get_receipt(receipt_id).await? else {
            return Ok(None);
        };
        let result = self.verify_and_trace(&receipt, "lookup").await;
        Ok(Some((receipt, result)))
    }

    /// Verifies a standalone receipt that isn't necessarily in this
    /// instance's store (`POST /verify` with a pasted Verification
    /// Package) — same verification logic, `source: "pasted"` trace.
    pub async fn verify_external(&self, receipt: ReceiptRecord) -> (ReceiptRecord, VerificationResult) {
        let result = self.verify_and_trace(&receipt, "pasted").await;
        (receipt, result)
    }

    async fn verify_and_trace(&self, receipt: &ReceiptRecord, source: &str) -> VerificationResult {
        let outcome = verifier::verify(receipt, &self.keypair, &self.zk_config).await;

        if let Some(zk) = &outcome.zk_outcome {
            let trace = ZkTrace {
                trace_id: Uuid::new_v4(),
                decision_id: receipt.decision_id,
                receipt_id: Some(receipt.receipt_id),
                phase: "verify".to_string(),
                source: source.to_string(),
                success: zk.success,
                witness_input: None,
                circuit_public_values: zk.circuit_public_values.clone(),
                error_message: zk.error_message.clone(),
                stdout_tail: zk.stdout_tail.clone(),
                duration_ms: zk.duration_ms as i64,
                created_at: Utc::now(),
            };
            if let Err(e) = self.store.save_zk_trace(&trace).await {
                tracing::error!(error = %e, decision_id = %receipt.decision_id, "failed to persist zk trace");
            }
        }

        outcome.result
    }

    pub fn info(&self) -> GatewayInfo {
        GatewayInfo {
            model_id: self.session.model_id,
            model_hash: self.session.model_hash.clone(),
            policy_id: self.policy_id,
            policy_hash: self.policy_hash.clone(),
        }
    }

    pub fn store(&self) -> &Store {
        &self.store
    }

    pub fn registry(&self) -> &Registry {
        &self.registry
    }
}
