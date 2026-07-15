use anyhow::Result;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::domain::{ModelInput, ModelOutput};

/// Canonical JSON: keys sorted alphabetically, no whitespace, recursive.
/// Handles arbitrarily nested objects and arrays.
pub fn canonicalize(value: &Value) -> Result<String> {
    match value {
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let pairs = keys
                .into_iter()
                .map(|k| Ok(format!("\"{}\":{}", k, canonicalize(&map[k])?)))
                .collect::<Result<Vec<_>>>()?;
            Ok(format!("{{{}}}", pairs.join(",")))
        }
        Value::Array(arr) => {
            let items = arr.iter().map(canonicalize).collect::<Result<Vec<_>>>()?;
            Ok(format!("[{}]", items.join(",")))
        }
        _ => Ok(value.to_string()),
    }
}

fn sha256_of(s: &str) -> String {
    let mut h = Sha256::new();
    h.update(s.as_bytes());
    hex::encode(h.finalize())
}

pub fn input_hash(input: &ModelInput) -> Result<String> {
    Ok(sha256_of(&canonicalize(input)?))
}

pub fn output_hash(output: &ModelOutput) -> Result<String> {
    Ok(sha256_of(&canonicalize(output)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn hash_is_deterministic() {
        let input = json!({"income": 90000.0, "debt_ratio": 0.23});
        assert_eq!(input_hash(&input).unwrap(), input_hash(&input).unwrap());
    }

    #[test]
    fn key_order_does_not_affect_hash() {
        let a = input_hash(&json!({"income": 90000.0, "debt_ratio": 0.23})).unwrap();
        let b = input_hash(&json!({"debt_ratio": 0.23, "income": 90000.0})).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn different_inputs_produce_different_hashes() {
        let a = input_hash(&json!({"income": 90000.0})).unwrap();
        let b = input_hash(&json!({"income": 50000.0})).unwrap();
        assert_ne!(a, b);
    }
}
