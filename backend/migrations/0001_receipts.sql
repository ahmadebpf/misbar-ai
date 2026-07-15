CREATE TABLE IF NOT EXISTS receipts (
    receipt_id   TEXT PRIMARY KEY,
    decision_id  TEXT NOT NULL,
    model_id     TEXT NOT NULL,
    model_hash   TEXT NOT NULL,
    policy_id    TEXT NOT NULL,
    policy_hash  TEXT NOT NULL,
    input_hash   TEXT NOT NULL,
    output_hash  TEXT NOT NULL,
    timestamp    TEXT NOT NULL,
    signature    TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_receipts_timestamp    ON receipts (timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_receipts_decision_id  ON receipts (decision_id);
