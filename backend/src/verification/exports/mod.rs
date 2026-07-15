use serde::{Deserialize, Serialize};

use crate::domain::{ReceiptRecord, VerificationResult};

#[derive(Debug, Serialize, Deserialize)]
pub struct VerificationPackage {
    pub receipt: ReceiptRecord,
    pub verified: bool,
    pub signature_valid: bool,
    pub proof_valid: bool,
}

pub fn build(receipt: ReceiptRecord, result: VerificationResult) -> VerificationPackage {
    VerificationPackage {
        verified: result.verified,
        signature_valid: result.signature_valid,
        proof_valid: result.zk_proof_valid.unwrap_or(false),
        receipt,
    }
}
