use anyhow::{Context, Result};
use chrono::Utc;
use std::path::Path;
use uuid::Uuid;

use crate::attestation::{hashing, receipts, signatures::Keypair, zk::ZkConfig};
use crate::domain::{ModelInput, ModelOutput};
use crate::execution::{features, inference::OnnxSession};
use crate::registry::{ModelEntry, PolicyEntry, Registry};
use crate::store::Store;

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
            name: "misbar-model".to_string(),
            version: "v1.0.0".to_string(),
            artifact_hash: session.model_hash.clone(),
            registered_at: Utc::now(),
        });

        registry.register_policy(PolicyEntry {
            id: policy_id,
            name: "misbar-policy".to_string(),
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
        let timestamp = Utc::now();

        let input_hash = features::input_hash(&input)?;
        let output = self.session.run(&input)?;
        let output_hash = features::output_hash(&output)?;

        // Proves the compiled circuit produces this output from this input —
        // a stronger claim than the signature below, which only attests that
        // misbar signed this record. Synchronous: the request waits for the
        // proof (small model, expected well under a few seconds).
        let zk_proof = crate::attestation::zk::prove(&self.zk_config, decision_id, &input)
            .await
            .context("zk proof generation")?;

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

        let receipt_id = receipt.receipt_id;
        self.store.save_receipt(&receipt).await?;

        Ok(ExecutionResult { decision_id, receipt_id, output })
    }

    pub fn info(&self) -> GatewayInfo {
        GatewayInfo {
            model_id: self.session.model_id,
            model_hash: self.session.model_hash.clone(),
            policy_id: self.policy_id,
            policy_hash: self.policy_hash.clone(),
        }
    }

    pub fn keypair(&self) -> &Keypair {
        &self.keypair
    }

    pub fn store(&self) -> &Store {
        &self.store
    }

    pub fn registry(&self) -> &Registry {
        &self.registry
    }

    pub fn zk_config(&self) -> &ZkConfig {
        &self.zk_config
    }
}
