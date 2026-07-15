use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::ReceiptRecord;

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

pub fn assemble(
    decision_id: Uuid,
    model_id: Uuid,
    model_hash: String,
    policy_id: Uuid,
    policy_hash: String,
    input_hash: String,
    output_hash: String,
    timestamp: DateTime<Utc>,
    signature: String,
) -> ReceiptRecord {
    ReceiptRecord {
        receipt_id: Uuid::new_v4(),
        decision_id,
        model_id,
        model_hash,
        policy_id,
        policy_hash,
        input_hash,
        output_hash,
        timestamp,
        signature,
    }
}
