use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub struct TransferExecutionRow {
    pub(crate) id: i64,
    pub(crate) intent_id: Uuid,
    pub(crate) provider: String,
    pub(crate) provider_ref: Option<String>,
    pub(crate) status: String,
    pub(crate) request: Option<Value>,
    pub(crate) response: Option<Value>,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) updated_at: DateTime<Utc>,
    pub(crate) version: i32,
}
