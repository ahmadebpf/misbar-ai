"""
Trains the Misbar POC credit scoring model.

Model choice: GradientBoostingClassifier (scikit-learn).
Why not a neural net: four tabular features, small dataset, need an
explainable model for a regulated-finance demo, and gradient-boosted trees
export cleanly to ONNX. This is the right amount of complexity for a POC --
not a toy (logistic regression alone would underfit the nonlinear
debt_ratio x missed_payments interaction) and not overkill.

Feature order MUST match generate.py and the committed
input_schema.json. This is the single biggest source of silent bugs in this
pipeline -- if you change the order here, update both of those too.

Run:
    python train.py
Output:
    model.pkl        -- sklearn model (for evaluate.py / debugging only)
    feature_info.json -- means/stds and feature order, used at export time
"""

import json

import joblib
import numpy as np
import pandas as pd
from sklearn.ensemble import GradientBoostingClassifier
from sklearn.metrics import accuracy_score, roc_auc_score, classification_report
from sklearn.model_selection import train_test_split

FEATURE_ORDER = ["income", "debt_ratio",
                 "missed_payments", "credit_history_months"]
RANDOM_STATE = 42


def load_data(path: str = "synthetic_credit.csv") -> tuple[pd.DataFrame, pd.Series]:
  df = pd.read_csv(path)
  X = df[FEATURE_ORDER]
  y = df["label"]
  return X, y


def train(X: pd.DataFrame, y: pd.Series) -> GradientBoostingClassifier:
  clf = GradientBoostingClassifier(
      n_estimators=150,
      max_depth=3,
      learning_rate=0.08,
      subsample=0.9,
      random_state=RANDOM_STATE,
  )
  clf.fit(X, y)
  return clf


def evaluate(clf: GradientBoostingClassifier, X_test: pd.DataFrame, y_test: pd.Series) -> None:
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
