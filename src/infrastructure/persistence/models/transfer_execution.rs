use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub struct TransferExecutionRow {
    pub id: i64,
    pub intent_id: Uuid,
    pub provider: String,
    pub provider_ref: Option<String>,
    pub status: String,
    pub request: Option<Value>,
    pub response: Option<Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: i32,
}
