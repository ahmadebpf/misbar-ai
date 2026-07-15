use serde::{Deserialize, Serialize};

use crate::domain::{ReceiptRecord, VerificationResult};

#[derive(Debug, Serialize, Deserialize)]
pub struct VerificationPackage {
    pub receipt: ReceiptRecord,
    pub verified: bool,
    pub signature_valid: bool,
    /// Always false until zk proofs are implemented
    pub proof_valid: bool,
}

pub fn build(receipt: ReceiptRecord, result: VerificationResult) -> VerificationPackage {
    VerificationPackage {
        verified: result.verified,
        signature_valid: result.signature_valid,
        proof_valid: false,
        receipt,
    }
}
