use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Opaque input from the caller — the gateway never inspects field semantics.
pub type ModelInput = serde_json::Value;

/// Opaque output from the model — passed through to the caller as-is.
pub type ModelOutput = serde_json::Value;

/// The immutable audit record produced for every decision.
///
/// `model_hash` = SHA256(model.onnx bytes) — always present (local ONNX only).
/// `input_hash`  = SHA256(canonical input JSON)  — per request
/// `output_hash` = SHA256(canonical output JSON) — per request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptRecord {
    pub receipt_id: Uuid,
    pub decision_id: Uuid,
    pub model_id: Uuid,
    pub model_hash: String,
    pub policy_id: Uuid,
    pub policy_hash: String,
    pub input: ModelInput,
    pub input_hash: String,
    pub output: ModelOutput,
    pub output_hash: String,
    pub timestamp: DateTime<Utc>,
    pub signature: String,
    /// Base64-encoded EZKL proof that the model actually produced `output`
    /// from `input` — a stronger claim than the signature, which only
    /// attests that saleem signed this record. See attestation/zk/mod.rs.
    pub zk_proof: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub receipt_id: Uuid,
    pub verified: bool,
    pub signature_valid: bool,
    pub zk_proof_valid: Option<bool>,
    pub timestamp: DateTime<Utc>,
}

/// A single ezkl attempt (prove or verify) — persisted for every attempt,
/// success or failure, keyed by `decision_id` rather than `receipt_id`
/// since a failed prove never produces a receipt. See attestation/zk/mod.rs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkTrace {
    pub trace_id: Uuid,
    pub decision_id: Uuid,
    pub receipt_id: Option<Uuid>,
    /// "prove" | "verify"
    pub phase: String,
    /// "gateway" (a live `/decision` call) | "lookup" (`GET /verify/{id}`)
    /// | "pasted" (`POST /verify` with a standalone receipt)
    pub source: String,
    pub success: bool,
    pub witness_input: Option<serde_json::Value>,
    /// The circuit's own decoded public input/output — see
    /// attestation/zk/decode.rs.
    pub circuit_public_values: Option<Vec<f64>>,
    pub error_message: Option<String>,
    pub stdout_tail: Option<String>,
    pub duration_ms: i64,
    pub created_at: DateTime<Utc>,
}
