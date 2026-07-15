"""
Exports the trained sklearn model to ONNX format -- the artifact the Rust
backend actually loads via the `ort` crate.

This is the hard boundary between the Python world and the Rust world.
Everything before this script (training, evaluation, sklearn) is offline
tooling. model.onnx is the only thing that crosses into backend/.

Output tensor shape: the model outputs two things in ONNX:
  1. "label"            -- int64, the predicted class (0 or 1)
  2. "probabilities"     -- a map/sequence of class probabilities

We care about probabilities[1] (probability of "good" credit) -- that's
what gets turned into the 300-850 style score. Run inspect_onnx.py after
this to confirm the output names match what inference.rs expects.

Run:
    python export_onnx.py
Output:
    model.onnx
"""

import joblib
from skl2onnx import convert_sklearn
from skl2onnx.common.data_types import FloatTensorType

FEATURE_ORDER = ["income", "debt_ratio", "missed_payments", "credit_history_months"]


def main() -> None:
  clf = joblib.load("model.pkl")

  # None in the batch dimension means the model accepts any batch size --
  # Rust will send batch size 1 per request, but this keeps the door open
  # for batch scoring later without re-exporting.
  initial_type = [("input", FloatTensorType([None, len(FEATURE_ORDER)]))]

  onnx_model = convert_sklearn(
      clf,
      initial_types=initial_type,
      target_opset=17,
      # plain array output, not a dict -- much easier to parse in Rust
      options={id(clf): {"zipmap": False}},
  )

  with open("model.onnx", "wb") as f:
    f.write(onnx_model.SerializeToString())

  print("Wrote model.onnx")
  print(f"Input shape: [batch, {len(FEATURE_ORDER)}] in order {FEATURE_ORDER}")

  # Print the actual output names/shapes so they can be cross-checked
  # against inference.rs
  print("\nOutput tensors:")
  for output in onnx_model.graph.output:
    print(f"  - {output.name}")


if __name__ == "__main__":
  main()
