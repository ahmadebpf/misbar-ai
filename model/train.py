"""
Trains the Misbar POC credit scoring model.

Model choice: MLPClassifier (scikit-learn), preceded by a StandardScaler in
the same pipeline. Previously this was a GradientBoostingClassifier, but
tree ensembles export to ONNX as a single opaque `TreeEnsembleClassifier`
op, which EZKL's ONNX frontend (tract) cannot parse at all -- there's no
way to turn a decision tree's branching logic into an arithmetic circuit
via EZKL. An MLP exports as plain Gemm/Add/Relu/Sigmoid ops, which EZKL
can turn into a provable circuit. The StandardScaler is included in the
exported ONNX graph (as Sub/Div ops) rather than done in Rust, so the
scaling itself is part of what gets proven and the Rust side still just
sends raw feature values.

A small hidden-layer MLP still captures the nonlinear
debt_ratio x missed_payments interaction that a plain logistic regression
would underfit -- see generate.py's label formula -- while staying
provable, unlike the tree ensemble it replaces.

Feature order MUST match generate.py and the committed
input_schema.json. This is the single biggest source of silent bugs in this
pipeline -- if you change the order here, update both of those too.

Run:
    python train.py
Output:
    model.pkl        -- sklearn pipeline (for evaluate.py / debugging only)
    feature_info.json -- means/stds and feature order, used at export time
"""

import json

import joblib
import numpy as np
import pandas as pd
from sklearn.metrics import accuracy_score, roc_auc_score, classification_report
from sklearn.model_selection import train_test_split
from sklearn.neural_network import MLPClassifier
from sklearn.pipeline import Pipeline
from sklearn.preprocessing import StandardScaler

FEATURE_ORDER = ["income", "debt_ratio",
                 "missed_payments", "credit_history_months"]
RANDOM_STATE = 42


def load_data(path: str = "synthetic_credit.csv") -> tuple[pd.DataFrame, pd.Series]:
  df = pd.read_csv(path)
  X = df[FEATURE_ORDER]
  y = df["label"]
  return X, y


def train(X: pd.DataFrame, y: pd.Series) -> Pipeline:
  clf = Pipeline([
      ("scaler", StandardScaler()),
      ("mlp", MLPClassifier(
          hidden_layer_sizes=(16, 8),
          activation="relu",
          alpha=1e-3,
          max_iter=2000,
          random_state=RANDOM_STATE,
      )),
  ])
  clf.fit(X, y)
  return clf


def evaluate(clf: Pipeline, X_test: pd.DataFrame, y_test: pd.Series) -> None:
  y_pred = clf.predict(X_test)
  y_proba = clf.predict_proba(X_test)[:, 1]

  print(f"Accuracy: {accuracy_score(y_test, y_pred):.4f}")
  print(f"ROC AUC:  {roc_auc_score(y_test, y_proba):.4f}")
  print()
  print(classification_report(y_test, y_pred, target_names=["bad", "good"]))


def main() -> None:
  X, y = load_data()
  X_train, X_test, y_train, y_test = train_test_split(
      X, y, test_size=0.2, random_state=RANDOM_STATE, stratify=y
  )

  print(f"Training on {len(X_train)} rows, testing on {len(X_test)} rows")
  print(f"Feature order: {FEATURE_ORDER}")
  print()

  clf = train(X_train, y_train)
  evaluate(clf, X_test, y_test)

  # Persist the raw sklearn model -- used by export_onnx.py and
  # evaluate.py. This file is NOT what the Rust backend loads.
  joblib.dump(clf, "model.pkl")

  # Persist feature order + basic stats alongside the model. The Rust
  # backend doesn't read this directly, but it's the reference for
  # input_schema.json and for anyone debugging a mismatch later.
  feature_info = {
      "feature_order": FEATURE_ORDER,
      "feature_stats": {
          col: {"min": float(X[col].min()), "max": float(
              X[col].max()), "mean": float(X[col].mean())}
          for col in FEATURE_ORDER
      },
      "training_rows": len(X_train),
      "test_rows": len(X_test),
  }
  with open("feature_info.json", "w") as f:
    json.dump(feature_info, f, indent=2)

  print("\nWrote model.pkl and feature_info.json")


if __name__ == "__main__":
  main()
