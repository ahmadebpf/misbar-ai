use crate::attestation::{receipts::signable_payload, signatures::Keypair};
use crate::domain::{ReceiptRecord, VerificationResult};

pub fn verify(receipt: &ReceiptRecord, keypair: &Keypair) -> VerificationResult {
    let payload = signable_payload(
        &receipt.decision_id,
        &receipt.model_id,
        &receipt.policy_id,
        &receipt.input_hash,
        &receipt.output_hash,
        &receipt.timestamp,
    );

    let signature_valid = keypair.verify(&payload, &receipt.signature);

    VerificationResult {
        receipt_id: receipt.receipt_id,
        verified: signature_valid,
        signature_valid,
        timestamp: receipt.timestamp,
    }
}
