#!/usr/bin/env bash
set -euo pipefail

# One-time EZKL circuit setup for model/model.onnx.
#
# Prereqs: the `ezkl` CLI installed and on PATH (see docs/ezkl.md).
# Re-run this whenever model/model.onnx changes.
#
# Produces backend/circuit/{settings.json, network.compiled, pk.key, vk.key, kzg.srs}.
# These are the fixed, reusable artifacts the backend loads at startup (see
# backend/src/attestation/zk/mod.rs) — nothing here runs per-request.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"   # backend/scripts
BACKEND_DIR="$(dirname "$SCRIPT_DIR")"                        # backend/
CIRCUIT_DIR="$BACKEND_DIR/circuit"
MODEL="$BACKEND_DIR/../model/model.onnx"
CALIBRATION="$SCRIPT_DIR/calibration.json"

if ! command -v ezkl >/dev/null 2>&1; then
  echo "error: ezkl CLI not found on PATH. See docs/ezkl.md for install steps." >&2
  exit 1
fi

mkdir -p "$CIRCUIT_DIR"
cd "$CIRCUIT_DIR"

echo "==> gen-settings"
# decomp-legs=3 (default 2): with base=16384, legs=2 can only represent
# integers up to 16384^2 (~2.68e8) inside the circuit. `income` scaled by
# input_scale (~2^11) can exceed that for realistic values ($131k+), which
# fails with "decomposition error: integer too large" at gen-witness time.
# legs=3 raises the ceiling to 16384^3 (~4.4e12).
ezkl gen-settings \
  -M "$MODEL" \
  -O settings.json \
  --input-visibility public \
  --output-visibility public \
  --param-visibility private \
  --decomp-legs 3

echo "==> calibrate-settings"
# lookup-safety-margin gives headroom above what's observed in the
# calibration data, since real requests can fall slightly outside it.
ezkl calibrate-settings \
  -M "$MODEL" \
  -O settings.json \
  -D "$CALIBRATION" \
  --target resources \
  --lookup-safety-margin 6

echo "==> compile-circuit"
ezkl compile-circuit \
  -M "$MODEL" \
  -S settings.json \
  --compiled-circuit network.compiled

echo "==> get-srs"
ezkl get-srs \
  -S settings.json \
  --srs-path kzg.srs

echo "==> setup (proving + verifying keys)"
ezkl setup \
  -M network.compiled \
  --srs-path kzg.srs \
  --vk-path vk.key \
  --pk-path pk.key

echo "Done. Artifacts written to $CIRCUIT_DIR"
