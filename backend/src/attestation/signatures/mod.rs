use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;

pub struct Keypair {
    signing_key: SigningKey,
    pub verifying_key: VerifyingKey,
}

impl Keypair {
    pub fn generate() -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        Self { signing_key, verifying_key }
    }

    /// Sign arbitrary bytes. Returns base64-encoded signature.
    pub fn sign(&self, payload: &[u8]) -> String {
        let signature: Signature = self.signing_key.sign(payload);
        BASE64.encode(signature.to_bytes())
    }

    /// Verify a base64-encoded Ed25519 signature against the stored verifying key.
    pub fn verify(&self, payload: &[u8], signature_b64: &str) -> bool {
        let Ok(sig_bytes) = BASE64.decode(signature_b64) else {
            return false;
        };
        let Ok(sig_array) = <[u8; 64]>::try_from(sig_bytes.as_slice()) else {
            return false;
        };
        let signature = Signature::from_bytes(&sig_array);
        self.verifying_key.verify(payload, &signature).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_and_verify_roundtrip() {
        let kp = Keypair::generate();
        let payload = b"decision|model_id|policy_id|input_hash|output_hash|1234567890";
        let sig = kp.sign(payload);
        assert!(kp.verify(payload, &sig));
    }

    #[test]
    fn tampered_payload_fails_verification() {
        let kp = Keypair::generate();
        let sig = kp.sign(b"original payload");
        assert!(!kp.verify(b"tampered payload", &sig));
    }
}
