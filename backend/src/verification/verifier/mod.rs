use crate::attestation::{
    receipts::signable_payload,
    signatures::Keypair,
    zk::{self, ZkConfig, ZkOutcome},
};
use crate::domain::{ReceiptRecord, VerificationResult};

/// `result` is the public-facing verdict; `zk_outcome` carries the raw ezkl
/// verify trace (stdout/stderr/timing/decoded circuit values) for the
/// caller to persist as a `ZkTrace` — kept separate so `VerificationResult`
/// itself stays a clean, stable API response shape.
pub struct VerifyOutcome {
    pub result: VerificationResult,
    pub zk_outcome: Option<ZkOutcome>,
}

/// `verified` is true only if both the signature and (when present) the zk
/// proof check out — it reflects the strongest claim the receipt supports.
pub async fn verify(receipt: &ReceiptRecord, keypair: &Keypair, zk_config: &ZkConfig) -> VerifyOutcome {
    let payload = signable_payload(
        &receipt.decision_id,
        &receipt.model_id,
        &receipt.policy_id,
        &receipt.input_hash,
        &receipt.output_hash,
        &receipt.timestamp,
    );

    let signature_valid = keypair.verify(&payload, &receipt.signature);

    let zk_outcome = match &receipt.zk_proof {
        Some(proof) => Some(zk::verify(zk_config, proof).await),
        None => None,
    };
    let zk_proof_valid = zk_outcome.as_ref().map(|o| o.success);

    let result = VerificationResult {
        receipt_id: receipt.receipt_id,
        verified: signature_valid && zk_proof_valid.unwrap_or(false),
        signature_valid,
        zk_proof_valid,
        timestamp: receipt.timestamp,
    };

    VerifyOutcome { result, zk_outcome }
}
