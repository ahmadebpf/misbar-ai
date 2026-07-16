use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize)]
struct Settings {
    model_instance_shapes: Vec<Vec<usize>>,
    model_input_scales: Vec<i32>,
    model_output_scales: Vec<i32>,
}

/// Rescales a circuit's public `instances` (from `proof.json`) back into
/// human-readable floats, using the per-tensor scales `zk_setup.sh` baked
/// into `settings.json`. Instance blocks are input tensors first, then
/// output tensors, in the order fixed at circuit-compile time.
pub struct InstanceScales {
    shapes: Vec<Vec<usize>>,
    scales: Vec<i32>,
}

impl InstanceScales {
    pub fn load(settings_path: &Path) -> Result<Self> {
        let bytes = std::fs::read(settings_path).context("read settings.json")?;
        let settings: Settings = serde_json::from_slice(&bytes).context("parse settings.json")?;

        let mut scales = Vec::with_capacity(settings.model_input_scales.len() + settings.model_output_scales.len());
        scales.extend(settings.model_input_scales.iter().copied());
        scales.extend(settings.model_output_scales.iter().copied());

        Ok(Self { shapes: settings.model_instance_shapes, scales })
    }

    /// Decodes little-endian hex field elements into a flat `Vec<f64>`,
    /// dividing each block by `2^scale` (scale 0 = integer as-is).
    ///
    /// Scoped to this model: every public value (features, probabilities, a
    /// 0/1 class label) is non-negative, so BN254 field-wraparound for
    /// negative fixed-point numbers isn't handled here — see docs/ezkl.md's
    /// other demo-grade caveats.
    pub fn decode(&self, instances: &[String]) -> Vec<f64> {
        let mut out = Vec::with_capacity(instances.len());
        let mut idx = 0;

        for (block, &scale) in self.shapes.iter().zip(self.scales.iter()) {
            let len: usize = block.iter().product();
            let divisor = (1u64 << scale.max(0)) as f64;
            for _ in 0..len {
                let Some(hex) = instances.get(idx) else { return out };
                out.push(decode_field_element(hex) / divisor);
                idx += 1;
            }
        }
        out
    }
}

fn decode_field_element(hex: &str) -> f64 {
    let Ok(bytes) = hex_to_bytes(hex) else { return f64::NAN };
    // Little-endian; the low 16 bytes are plenty for the small integers
    // this model's public values decode to (no field-wraparound handling).
    let mut buf = [0u8; 16];
    let n = bytes.len().min(16);
    buf[..n].copy_from_slice(&bytes[..n]);
    u128::from_le_bytes(buf) as f64
}

fn hex_to_bytes(hex: &str) -> std::result::Result<Vec<u8>, std::num::ParseIntError> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..(i + 2).min(hex.len())], 16))
        .collect()
}
