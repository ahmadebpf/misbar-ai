CREATE TABLE IF NOT EXISTS zk_traces (
    trace_id              TEXT PRIMARY KEY,
    decision_id           TEXT NOT NULL,
    receipt_id            TEXT,
    phase                 TEXT NOT NULL,
    source                TEXT NOT NULL,
    success               INTEGER NOT NULL,
    witness_input         TEXT,
    circuit_public_values TEXT,
    error_message         TEXT,
    stdout_tail           TEXT,
    duration_ms           INTEGER NOT NULL,
    created_at            TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_zk_traces_decision_id ON zk_traces (decision_id);
CREATE INDEX IF NOT EXISTS idx_zk_traces_created_at  ON zk_traces (created_at DESC);
