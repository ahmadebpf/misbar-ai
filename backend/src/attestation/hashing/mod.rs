use anyhow::Result;
use sha2::{Digest, Sha256};
use std::{fs, path::Path};

pub fn hash_bytes(data: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(data);
    hex::encode(h.finalize())
}

pub fn hash_str(data: &str) -> String {
    hash_bytes(data.as_bytes())
}

/// Read a file and return SHA256 of its bytes.
/// Used at startup by OnnxSession to compute model_hash from model.onnx.
pub fn hash_file(path: &Path) -> Result<String> {
    let bytes = fs::read(path)?;
    Ok(hash_bytes(&bytes))
}
