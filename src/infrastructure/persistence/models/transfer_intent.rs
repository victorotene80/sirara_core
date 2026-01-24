use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct TransferIntentRow {
    pub id: Uuid,
    pub external_ref: String,
    pub route: Value,
    pub current_state: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub failure: Option<Value>,
    pub version: i32,
}
