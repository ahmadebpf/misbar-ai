use anyhow::Result;
use chrono::Utc;
use uuid::Uuid;

use crate::attestation::{hashing, receipts, signatures::Keypair};
use crate::domain::{ModelInput, ModelOutput};
use crate::execution::{features, inference::OnnxSession};
use crate::registry::{ModelEntry, PolicyEntry, Registry};
use crate::store::Store;

pub struct Gateway {
    session: OnnxSession,
    keypair: Keypair,
    policy_id: Uuid,
    policy_hash: String,
    store: Store,
    registry: Registry,
}

pub struct ExecutionResult {
    pub decision_id: Uuid,
    pub receipt_id: Uuid,
    pub output: ModelOutput,
}

impl Gateway {
    /// `policy_artifact` is the canonical string for the active policy.
    /// The gateway only stores its hash — any string is accepted.
    pub fn new(
        session: OnnxSession,
        store: Store,
        registry: Registry,
        policy_artifact: &str,
    ) -> Result<Self> {
        let policy_hash = hashing::hash_str(policy_artifact);

        registry.register_model(ModelEntry {
            id: session.model_id,
            name: "misbar-model".to_string(),
            version: "v1.0.0".to_string(),
            artifact_hash: session.model_hash.clone(),
            registered_at: Utc::now(),
        });

        let policy_id = Uuid::new_v4();
        registry.register_policy(PolicyEntry {
            id: policy_id,
            name: "misbar-policy".to_string(),
            version: "v1.0.0".to_string(),
            artifact_hash: policy_hash.clone(),
            registered_at: Utc::now(),
        });

        tracing::info!(
            model_id = %session.model_id,
            model_hash = %session.model_hash,
            policy_id = %policy_id,
            policy_hash = %policy_hash,
            "gateway initialised"
        );

        Ok(Self {
            session,
            keypair: Keypair::generate(),
            policy_id,
            policy_hash,
            store,
            registry,
        })
    }

    pub fn execute(&self, input: ModelInput) -> Result<ExecutionResult> {
        let decision_id = Uuid::new_v4();
        let timestamp = Utc::now();

        let input_hash = features::input_hash(&input)?;
        let output = self.session.run(&input)?;
        let output_hash = features::output_hash(&output)?;

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
            input_hash,
            output_hash,
            timestamp,
            signature,
        );

        let receipt_id = receipt.receipt_id;
        self.store.save_receipt(receipt);

        Ok(ExecutionResult { decision_id, receipt_id, output })
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
}
