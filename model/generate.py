"""
Generates synthetic credit application data for the Saleem POC model.

This is NOT real credit data. It's a rule-based simulation used to give the
classifier something realistic to learn from. The ground-truth label is
computed from a weighted formula over the four input features, plus noise,
so the model has to actually learn a decision boundary rather than memorize
a lookup table.

Feature order is the single source of truth for the rest of the pipeline —
train.py and the Rust inference code must both respect this exact order:

    [income, debt_ratio, missed_payments, credit_history_months]

Run:
    python generate.py
Output:
    synthetic_credit.csv in this directory
"""

import numpy as np
import pandas as pd

RNG_SEED = 42
N_SAMPLES = 20_000

FEATURE_ORDER = ["income", "debt_ratio",
                 "missed_payments", "credit_history_months"]


def generate(n_samples: int = N_SAMPLES, seed: int = RNG_SEED) -> pd.DataFrame:
  rng = np.random.default_rng(seed)

  # Income: log-normal, roughly $20k-$300k, centered around $65k
  income = rng.lognormal(mean=11.0, sigma=0.45, size=n_samples)
  income = np.clip(income, 15_000, 400_000)

  # Debt-to-income ratio: beta distribution, mostly 0.1-0.5
  debt_ratio = rng.beta(a=2.2, b=4.0, size=n_samples)

  # Missed payments in the last 24 months: mostly 0, occasionally more
  missed_payments = rng.poisson(lam=0.6, size=n_samples)
  missed_payments = np.clip(missed_payments, 0, 12)

  # Credit history length in months: uniform-ish, 0-360 (30 years)
  credit_history_months = rng.gamma(shape=3.0, scale=40.0, size=n_samples)
  credit_history_months = np.clip(credit_history_months, 0, 420).round()

  df = pd.DataFrame(
      {
          "income": income.round(2),
          "debt_ratio": debt_ratio.round(4),
          "missed_payments": missed_payments.astype(int),
          "credit_history_months": credit_history_months.astype(int),
      }
  )

  # --- Ground truth label ---
  # A weighted "creditworthiness" signal, normalized into rough FICO-like
  # range, then thresholded into a binary good/bad label with injected
  # noise so the boundary isn't perfectly learnable (more realistic).
  income_z = (np.log(df["income"]) - np.log(df["income"]
                                            ).mean()) / np.log(df["income"]).std()
  history_z = (df["credit_history_months"] -
               df["credit_history_months"].mean()) / df["credit_history_months"].std()

  raw_signal = (
      0.35 * income_z
      - 1.8 * df["debt_ratio"]
      - 0.55 * df["missed_payments"]
      + 0.30 * history_z
  )

  noise = rng.normal(0, 0.35, size=n_samples)
  signal_with_noise = raw_signal + noise

  # Label: 1 = good credit (approve-worthy), 0 = bad credit
  threshold = np.median(signal_with_noise)
  df["label"] = (signal_with_noise > threshold).astype(int)

  return df[FEATURE_ORDER + ["label"]]


if __name__ == "__main__":
  df = generate()
  out_path = "synthetic_credit.csv"
  df.to_csv(out_path, index=False)
  print(f"Wrote {len(df)} rows to {out_path}")
  print(f"Label balance: {df['label'].mean():.3f} positive")
  print(df.describe())
