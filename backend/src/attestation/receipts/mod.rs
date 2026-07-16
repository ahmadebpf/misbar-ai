use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::{ModelInput, ModelOutput, ReceiptRecord};

/// The canonical payload that gets signed.
///
/// Format: "decision_id|model_id|policy_id|input_hash|output_hash|unix_timestamp"
///
/// Must be reproduced identically in verification/verifier/mod.rs.
pub fn signable_payload(
    decision_id: &Uuid,
    model_id: &Uuid,
    policy_id: &Uuid,
    input_hash: &str,
    output_hash: &str,
    timestamp: &DateTime<Utc>,
) -> Vec<u8> {
    format!(
        "{}|{}|{}|{}|{}|{}",
        decision_id,
        model_id,
        policy_id,
        input_hash,
        output_hash,
        timestamp.timestamp()
    )
    .into_bytes()
}

#[allow(clippy::too_many_arguments)]
pub fn assemble(
    receipt_id: Uuid,
    decision_id: Uuid,
    model_id: Uuid,
    model_hash: String,
    policy_id: Uuid,
    policy_hash: String,
    input: ModelInput,
    input_hash: String,
    output: ModelOutput,
    output_hash: String,
    timestamp: DateTime<Utc>,
    signature: String,
    zk_proof: Option<String>,
) -> ReceiptRecord {
    ReceiptRecord {
        receipt_id,
        decision_id,
        model_id,
        model_hash,
        policy_id,
        policy_hash,
        input,
        input_hash,
        output,
        output_hash,
        timestamp,
        signature,
        zk_proof,
    }
}
