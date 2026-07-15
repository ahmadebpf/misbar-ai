use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use sqlx::{sqlite::SqliteConnectOptions, FromRow, SqlitePool};
use std::str::FromStr;
use uuid::Uuid;

use crate::domain::ReceiptRecord;

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
}
