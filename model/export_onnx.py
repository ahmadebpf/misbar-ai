"""
Exports the trained sklearn model to ONNX format -- the artifact the Rust
backend actually loads via the `ort` crate.

This is the hard boundary between the Python world and the Rust world.
Everything before this script (training, evaluation, sklearn) is offline
tooling. model.onnx is the only thing that crosses into backend/.

model.pkl holds a Pipeline(StandardScaler, MLPClassifier). We don't export
the Pipeline as-is: skl2onnx converts StandardScaler to the ai.onnx.ml
`Scaler` op, which EZKL's ONNX frontend (tract) doesn't implement (same
class of problem as the TreeEnsembleClassifier op this model used to use).

We also do NOT fold the scaler into the MLP's first-layer weights
(new_W1 = W1 / scale) even though that's mathematically equivalent --
`income` has a scale of ~tens of thousands, so dividing its weight column
by scaler.scale_ produces weights orders of magnitude smaller than the
other columns. EZKL quantizes an entire weight tensor to one fixed-point
scale, so those tiny income weights round to ~0, silently discarding the
feature (confirmed empirically: it flipped a 95%-good-credit prediction to
5% inside the circuit). Instead we prepend the standardization as its own
Add/Mul nodes (plain ops, not the ai.onnx.ml Scaler op) ahead of the bare
MLP's graph, so the weight matrix itself stays on one consistent, small
scale and only the (cheap, well-conditioned) input normalization step
carries the large income constant. The result is mathematically identical
to the pipeline -- verify_export.py checks this by comparing against the
*pipeline's* predict_proba, not the bare MLP's.

Output tensor shape: the model outputs two things in ONNX:
  1. "label"            -- int64, the predicted class (0 or 1)
  2. "probabilities"     -- a map/sequence of class probabilities

We care about probabilities[1] (probability of "good" credit) -- that's
what gets turned into the 300-850 style score. Run verify_export.py after
this to confirm the output names match what inference.rs expects.

Run:
    python export_onnx.py
Output:
    model.onnx
"""

import joblib
import numpy as np
from onnx import StringStringEntryProto, helper, numpy_helper
from skl2onnx import convert_sklearn
from skl2onnx.common.data_types import FloatTensorType

FEATURE_ORDER = ["income", "debt_ratio", "missed_payments", "credit_history_months"]

# Identifies this model artifact -- the Rust backend reads these back out of
# model.onnx itself (via ort's Session::metadata()/.custom(), see
# backend/src/execution/inference/mod.rs) rather than having a name/version
# hardcoded or guessed at in backend code. Bump MODEL_VERSION whenever the
# training data, architecture, or hyperparameters change meaningfully.
MODEL_NAME = "credit-scoring"
MODEL_VERSION = "v2.9.1"


def prepend_standardization(onnx_model, mean, scale):
  """Inserts Add/Mul nodes computing (x - mean) / scale ahead of the
  graph's first node, and rewires that first node to consume the result
  instead of the raw input. Plain ops (not ai.onnx.ml Scaler), and kept
  separate from the MLP weights -- see export_onnx.py's module docstring
  for why folding into the weights doesn't work here."""
  graph = onnx_model.graph
  input_name = graph.input[0].name

  neg_mean = numpy_helper.from_array((-mean).astype(np.float32), name="scaler_neg_mean")
  inv_scale = numpy_helper.from_array((1.0 / scale).astype(np.float32), name="scaler_inv_scale")
  graph.initializer.extend([neg_mean, inv_scale])

  add_node = helper.make_node("Add", [input_name, "scaler_neg_mean"], ["scaler_centered"], name="scaler_add")
  mul_node = helper.make_node("Mul", ["scaler_centered", "scaler_inv_scale"], ["scaler_scaled"], name="scaler_mul")

  for node in graph.node:
    for i, inp in enumerate(node.input):
      if inp == input_name:
        node.input[i] = "scaler_scaled"

  reordered = [add_node, mul_node] + list(graph.node)
  del graph.node[:]
  graph.node.extend(reordered)

  return onnx_model


def main() -> None:
  pipeline = joblib.load("model.pkl")
  scaler, mlp = pipeline.named_steps["scaler"], pipeline.named_steps["mlp"]

  # None in the batch dimension means the model accepts any batch size --
  # Rust will send batch size 1 per request, but this keeps the door open
  # for batch scoring later without re-exporting.
  initial_type = [("input", FloatTensorType([None, len(FEATURE_ORDER)]))]

  onnx_model = convert_sklearn(
      mlp,
      initial_types=initial_type,
      target_opset=17,
      # plain array output, not a dict -- much easier to parse in Rust
      options={id(mlp): {"zipmap": False}},
  )
  onnx_model = prepend_standardization(onnx_model, scaler.mean_, scaler.scale_)

  onnx_model.metadata_props.append(StringStringEntryProto(key="model_name", value=MODEL_NAME))
  onnx_model.metadata_props.append(StringStringEntryProto(key="model_version", value=MODEL_VERSION))

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
