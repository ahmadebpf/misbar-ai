"""
Sanity check: confirms the exported model.onnx produces the same
predictions as the original sklearn model.pkl on a held-out sample.

This is the script to run any time you re-export the model -- it's cheap
insurance against silent export bugs (wrong opset, dtype mismatch, zipmap
option changing output shape, etc.) before the model ever reaches Rust.

Also prints the exact tensor shapes and dtypes ort will see, since that's
the part that's easy to get wrong on the Rust side.

Run:
    python verify_export.py
"""

import joblib
import numpy as np
import onnxruntime as ort
import pandas as pd

FEATURE_ORDER = ["income", "debt_ratio",
                 "missed_payments", "credit_history_months"]


def main() -> None:
  clf = joblib.load("model.pkl")
  session = ort.InferenceSession("model.onnx")

  print("ONNX session inputs:")
  for inp in session.get_inputs():
    print(f"  - {inp.name}: shape={inp.shape}, type={inp.type}")
  print("ONNX session outputs:")
  for out in session.get_outputs():
    print(f"  - {out.name}: shape={out.shape}, type={out.type}")
  print()

  # Pull a handful of real rows from the dataset to test against
  df = pd.read_csv("synthetic_credit.csv").sample(5, random_state=7)
  X = df[FEATURE_ORDER].values.astype(np.float32)

  sklearn_proba = clf.predict_proba(X)[:, 1]

  onnx_input = {"input": X}
  onnx_label, onnx_proba_raw = session.run(None, onnx_input)
  onnx_proba = onnx_proba_raw[:, 1]

  print("Row-by-row comparison (probability of 'good' credit):")
  print(f"{'sklearn':>10} {'onnx':>10} {'diff':>10}")
  max_diff = 0.0
  for sk, on in zip(sklearn_proba, onnx_proba):
    diff = abs(sk - on)
    max_diff = max(max_diff, diff)
    print(f"{sk:10.6f} {on:10.6f} {diff:10.8f}")

  print(f"\nMax absolute difference: {max_diff:.8f}")
  if max_diff < 1e-4:
    print("PASS -- ONNX export matches sklearn predictions")
  else:
    print("FAIL -- predictions diverge, investigate before shipping this export")


if __name__ == "__main__":
  main()
