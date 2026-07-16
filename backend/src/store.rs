use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use sqlx::{sqlite::SqliteConnectOptions, FromRow, SqlitePool};
use std::str::FromStr;
use uuid::Uuid;

use crate::domain::{ReceiptRecord, ZkTrace};

/// SQLite row — all fields stored as TEXT (input/output as JSON text).
#[derive(FromRow)]
struct ReceiptRow {
    receipt_id: String,
    decision_id: String,
    model_id: String,
    model_hash: String,
    policy_id: String,
    policy_hash: String,
    input: String,
    input_hash: String,
    output: String,
    output_hash: String,
    timestamp: String,
    signature: String,
    zk_proof: Option<String>,
}

impl TryFrom<ReceiptRow> for ReceiptRecord {
    type Error = anyhow::Error;

    fn try_from(r: ReceiptRow) -> Result<Self> {
        Ok(ReceiptRecord {
            receipt_id: Uuid::parse_str(&r.receipt_id).context("receipt_id")?,
            decision_id: Uuid::parse_str(&r.decision_id).context("decision_id")?,
            model_id: Uuid::parse_str(&r.model_id).context("model_id")?,
            model_hash: r.model_hash,
            policy_id: Uuid::parse_str(&r.policy_id).context("policy_id")?,
            policy_hash: r.policy_hash,
            input: serde_json::from_str(&r.input).context("input")?,
            input_hash: r.input_hash,
            output: serde_json::from_str(&r.output).context("output")?,
            output_hash: r.output_hash,
            timestamp: r.timestamp.parse::<DateTime<Utc>>().context("timestamp")?,
            signature: r.signature,
            zk_proof: r.zk_proof,
        })
    }
}

#[derive(Clone)]
pub struct Store {
    pool: SqlitePool,
}

impl Store {
    pub async fn new(database_url: &str) -> Result<Self> {
        let opts = SqliteConnectOptions::from_str(database_url)?.create_if_missing(true);
        let pool = SqlitePool::connect_with(opts).await?;
        sqlx::migrate!("./migrations").run(&pool).await?;
        Ok(Self { pool })
    }

    pub async fn save_receipt(&self, receipt: &ReceiptRecord) -> Result<()> {
        let input = serde_json::to_string(&receipt.input).context("serialize input")?;
        let output = serde_json::to_string(&receipt.output).context("serialize output")?;

        sqlx::query(
            "INSERT OR IGNORE INTO receipts
             (receipt_id, decision_id, model_id, model_hash, policy_id, policy_hash,
              input, input_hash, output, output_hash, timestamp, signature, zk_proof)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(receipt.receipt_id.to_string())
        .bind(receipt.decision_id.to_string())
        .bind(receipt.model_id.to_string())
        .bind(&receipt.model_hash)
        .bind(receipt.policy_id.to_string())
        .bind(&receipt.policy_hash)
        .bind(input)
        .bind(&receipt.input_hash)
        .bind(output)
        .bind(&receipt.output_hash)
        .bind(receipt.timestamp.to_rfc3339())
        .bind(&receipt.signature)
        .bind(&receipt.zk_proof)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_receipt(&self, id: &Uuid) -> Result<Option<ReceiptRecord>> {
        let row = sqlx::query_as::<_, ReceiptRow>(
            "SELECT * FROM receipts WHERE receipt_id = ?",
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        row.map(ReceiptRecord::try_from).transpose()
    }

    pub async fn list_receipts(&self, limit: i64, offset: i64) -> Result<Vec<ReceiptRecord>> {
        let rows = sqlx::query_as::<_, ReceiptRow>(
            "SELECT * FROM receipts ORDER BY timestamp DESC LIMIT ? OFFSET ?",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(ReceiptRecord::try_from).collect()
    }

    pub async fn count_receipts(&self) -> Result<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM receipts")
            .fetch_one(&self.pool)
            .await?;
        Ok(count)
    }

    pub async fn save_zk_trace(&self, trace: &ZkTrace) -> Result<()> {
        let witness_input = trace
            .witness_input
            .as_ref()
            .filter(|v| !v.is_null())
            .map(serde_json::to_string)
            .transpose()
            .context("serialize witness_input")?;
        let circuit_public_values = trace
            .circuit_public_values
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .context("serialize circuit_public_values")?;

        sqlx::query(
            "INSERT INTO zk_traces
             (trace_id, decision_id, receipt_id, phase, source, success,
              witness_input, circuit_public_values, error_message, stdout_tail,
              duration_ms, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(trace.trace_id.to_string())
        .bind(trace.decision_id.to_string())
        .bind(trace.receipt_id.map(|id| id.to_string()))
        .bind(&trace.phase)
        .bind(&trace.source)
        .bind(trace.success)
        .bind(witness_input)
        .bind(circuit_public_values)
        .bind(&trace.error_message)
        .bind(&trace.stdout_tail)
        .bind(trace.duration_ms)
        .bind(trace.created_at.to_rfc3339())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn list_zk_traces_for_decision(&self, decision_id: &Uuid) -> Result<Vec<ZkTrace>> {
        let rows = sqlx::query_as::<_, ZkTraceRow>(
            "SELECT * FROM zk_traces WHERE decision_id = ? ORDER BY created_at ASC",
        )
        .bind(decision_id.to_string())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(ZkTrace::try_from).collect()
    }

    pub async fn list_zk_traces(&self, limit: i64, offset: i64) -> Result<Vec<ZkTrace>> {
        let rows = sqlx::query_as::<_, ZkTraceRow>(
            "SELECT * FROM zk_traces ORDER BY created_at DESC LIMIT ? OFFSET ?",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(ZkTrace::try_from).collect()
    }
}

#[derive(FromRow)]
struct ZkTraceRow {
    trace_id: String,
    decision_id: String,
    receipt_id: Option<String>,
    phase: String,
    source: String,
    success: bool,
    witness_input: Option<String>,
    circuit_public_values: Option<String>,
    error_message: Option<String>,
    stdout_tail: Option<String>,
    duration_ms: i64,
    created_at: String,
}

impl TryFrom<ZkTraceRow> for ZkTrace {
    type Error = anyhow::Error;

    fn try_from(r: ZkTraceRow) -> Result<Self> {
        Ok(ZkTrace {
            trace_id: Uuid::parse_str(&r.trace_id).context("trace_id")?,
            decision_id: Uuid::parse_str(&r.decision_id).context("decision_id")?,
            receipt_id: r.receipt_id.map(|id| Uuid::parse_str(&id)).transpose().context("receipt_id")?,
            phase: r.phase,
            source: r.source,
            success: r.success,
            witness_input: r.witness_input.map(|s| serde_json::from_str(&s)).transpose().context("witness_input")?,
            circuit_public_values: r
                .circuit_public_values
                .map(|s| serde_json::from_str(&s))
                .transpose()
                .context("circuit_public_values")?,
            error_message: r.error_message,
            stdout_tail: r.stdout_tail,
            duration_ms: r.duration_ms,
            created_at: r.created_at.parse::<DateTime<Utc>>().context("created_at")?,
        })
    }
}
