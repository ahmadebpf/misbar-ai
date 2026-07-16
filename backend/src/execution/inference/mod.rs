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
    pub model_name: String,
    pub model_version: String,
}

impl OnnxSession {
    /// Load the ONNX model from disk. Computes model_hash from file bytes and
    /// derives a stable model_id (UUID v5) from that hash — same file, same ID.
    ///
    /// model_name/model_version come from the ONNX file's own metadata_props
    /// (set by model/export_onnx.py, read back via `ort`'s
    /// `Session::metadata()`/`ModelMetadata::custom()`) — the model's identity
    /// is a property of the model artifact, not a string hardcoded or guessed
    /// at in backend code. Falls back to something visibly generic (not a
    /// fabricated product name) if the file predates this or wasn't exported
    /// via export_onnx.py.
    pub fn load(model_path: &Path) -> Result<Self> {
        let model_hash = hashing::hash_file(model_path)?;

        // Stable namespace for Saleem model IDs.
        const MODEL_NS: Uuid = Uuid::from_bytes([
            0x6b, 0xa7, 0xb8, 0x11, 0x9d, 0xad, 0x11, 0xd1,
            0x80, 0xb4, 0x00, 0xc0, 0x4f, 0xd4, 0x30, 0xc8,
        ]);
        let model_id = Uuid::new_v5(&MODEL_NS, model_hash.as_bytes());

        let session = Session::builder()?.commit_from_file(model_path)?;

        let metadata = session.metadata().ok();
        let model_name = metadata.as_ref().and_then(|m| m.custom("model_name")).unwrap_or_else(|| {
            model_path
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| "unnamed-model".to_string())
        });
        let model_version = metadata
            .as_ref()
            .and_then(|m| m.custom("model_version"))
            .unwrap_or_else(|| "unversioned".to_string());
        // ModelMetadata borrows `session` and its Drop impl needs that
        // borrow to still be valid, so it must go out of scope explicitly
        // before `session` moves into the Mutex below.
        drop(metadata);

        tracing::info!("model loaded: {}", model_path.display());
        tracing::info!("model_id:      {}", model_id);
        tracing::info!("model_hash:    {}", model_hash);
        tracing::info!("model_name:    {}", model_name);
        tracing::info!("model_version: {}", model_version);

        Ok(Self { session: Mutex::new(session), model_id, model_hash, model_name, model_version })
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
