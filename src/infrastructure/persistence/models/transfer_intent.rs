use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;
use bigdecimal::BigDecimal;

#[derive(sqlx::FromRow)]
pub struct TransferIntentRow {
    pub(crate) id: i64,
    pub(crate) public_id: Uuid,
    pub(crate) external_ref_type: String,
    pub(crate) external_ref: String,
    pub(crate) route_json: Value,
    pub(crate) amount_minor: BigDecimal,
    pub(crate) asset_code: String,
    pub(crate) current_state: String,
    pub(crate) version: i32,
    pub(crate) quote_json: Option<Value>,
    pub(crate) quote_expires_at: Option<DateTime<Utc>>,
    pub(crate) required_usdt_minor: Option<BigDecimal>,
    pub(crate) tx_hash: Option<String>,
    pub(crate) failure_json: Option<Value>,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) updated_at: DateTime<Utc>,
}

