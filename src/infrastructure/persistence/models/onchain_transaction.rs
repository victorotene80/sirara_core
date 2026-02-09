use chrono::{DateTime, Utc};
use sqlx::types::BigDecimal;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct OnchainTxRow {
    pub id: i64,
    pub public_id: Uuid,
    pub intent_id: i64,
    pub chain: String,
    pub asset_code: String,
    pub region_code: String,
    pub contract_address: String,
    pub from_address: String,
    pub to_address: String,
    pub amount_minor: BigDecimal,
    pub status: String,
    pub tx_hash: Option<String>,
    pub confirmations: i32,
    pub idempotency_key: String,
    pub submit_attempts: i32,
    pub last_submit_at: Option<DateTime<Utc>>,
    pub last_checked_at: Option<DateTime<Utc>>,
    pub failure_json: Option<serde_json::Value>,
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
