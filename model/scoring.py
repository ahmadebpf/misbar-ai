"""
Defines the probability -> score conversion used to turn the model's raw
output (a 0-1 "probability of good credit") into a familiar 300-850 style
score, plus the risk_band bucketing.

IMPORTANT: This logic must be reimplemented in Rust (in policy.rs or
receipt.rs, after inference.rs gets the raw probability back from ONNX).
It deliberately lives in Python too -- as documentation and as the source
of truth -- but Python does NOT run in the request path. Keep both
implementations in sync; if you change this formula, update the Rust
mirror and bump the policy/schema version.

Formula: linear scaling from [0, 1] probability to [300, 850] score range.
This is a simplification for the POC -- real bureau scores use far more
complex, regulator-reviewed scorecards. Good enough to demonstrate the
receipt/audit infrastructure end-to-end.
"""

SCORE_MIN = 300
SCORE_MAX = 850


def probability_to_score(prob_good: float) -> int:
  """Linearly scale a 0-1 probability into the 300-850 score range."""
  return round(SCORE_MIN + prob_good * (SCORE_MAX - SCORE_MIN))


def score_to_risk_band(score: int) -> str:
  """Bucket a score into a risk band. Thresholds are illustrative --
  in production these come from config/policy.json, not hardcoded here.
  """
  if score >= 700:
    return "LOW"
  elif score >= 580:
    return "MEDIUM"
  else:
    return "HIGH"


if __name__ == "__main__":
  # Quick sanity table
  for p in [0.05, 0.25, 0.5, 0.75, 0.95, 0.99]:
    s = probability_to_score(p)
    band = score_to_risk_band(s)
    print(f"prob={p:.2f} -> score={s} -> risk_band={band}")
