use anyhow::{anyhow, Result};
use ort::{inputs, session::Session, value::Tensor};
use std::{path::Path, sync::Mutex};
use uuid::Uuid;

use crate::attestation::hashing;
use crate::domain::{ModelInput, ModelOutput};

pub struct OnnxSession {
    // Mutex because Session::run requires &mut self,
    // but OnnxSession is shared via Arc<Gateway>.
    session: Mutex<Session>,
    pub model_id: Uuid,
    pub model_hash: String,
}

impl OnnxSession {
    /// Load the ONNX model from disk. Computes model_hash from file bytes.
    /// Call once at startup — the session is shared via Arc<Gateway>.
    pub fn load(model_path: &Path, model_id: Uuid) -> Result<Self> {
        let model_hash = hashing::hash_file(model_path)?;
        let session = Session::builder()?.commit_from_file(model_path)?;

        tracing::info!("model loaded: {}", model_path.display());
        tracing::info!("model_hash: {}", model_hash);

        Ok(Self { session: Mutex::new(session), model_id, model_hash })
    }

    pub fn run(&self, input: &ModelInput) -> Result<ModelOutput> {
        let features = self.extract_features(input)?;

        // Build a [1, 4] f32 tensor from the feature array.
        // Tensor::from_array takes (shape, flat_data) — no ndarray needed.
        let tensor = Tensor::<f32>::from_array(([1_usize, 4_usize], features.to_vec()))?;
        let mut session = self.session.lock().unwrap();
        let outputs = session.run(inputs!["input" => tensor])?;

        // "probabilities": flat &[f32] of length 2 — [P(bad), P(good)]
        let (_, probs) = outputs["probabilities"].try_extract_tensor::<f32>()?;
        let prob_good = probs[1];

        // score = round(300 + prob_good * 550) — mirrors model/scoring.py
        let score = (300.0_f32 + prob_good * 550.0).round() as i32;

        Ok(serde_json::json!({ "prob_good": prob_good, "score": score }))
    }

    /// Feature order: [income, debt_ratio, missed_payments, credit_history_months]
    /// Must match model/generate.py, model/train.py, and model/export_onnx.py exactly.
    fn extract_features(&self, input: &ModelInput) -> Result<[f32; 4]> {
        let f = |key: &str| -> Result<f32> {
            input
                .get(key)
                .and_then(|v| v.as_f64())
                .map(|v| v as f32)
                .ok_or_else(|| anyhow!("missing or invalid field: {}", key))
        };
        Ok([
            f("income")?,
            f("debt_ratio")?,
            f("missed_payments")?,
            f("credit_history_months")?,
        ])
    }
}
