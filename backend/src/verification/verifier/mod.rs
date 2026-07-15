use crate::attestation::{receipts::signable_payload, signatures::Keypair, zk::ZkConfig};
use crate::domain::{ReceiptRecord, VerificationResult};

/// `verified` is true only if both the signature and (when present) the zk
/// proof check out — it reflects the strongest claim the receipt supports.
pub async fn verify(
    receipt: &ReceiptRecord,
    keypair: &Keypair,
    zk_config: &ZkConfig,
) -> VerificationResult {
    let payload = signable_payload(
        &receipt.decision_id,
        &receipt.model_id,
        &receipt.policy_id,
        &receipt.input_hash,
        &receipt.output_hash,
        &receipt.timestamp,
    );

    let signature_valid = keypair.verify(&payload, &receipt.signature);

    let zk_proof_valid = match &receipt.zk_proof {
        Some(proof) => Some(crate::attestation::zk::verify(zk_config, proof).await.unwrap_or(false)),
        None => None,
    };

    VerificationResult {
        receipt_id: receipt.receipt_id,
        verified: signature_valid && zk_proof_valid.unwrap_or(false),
        signature_valid,
        zk_proof_valid,
        timestamp: receipt.timestamp,
    }
}
