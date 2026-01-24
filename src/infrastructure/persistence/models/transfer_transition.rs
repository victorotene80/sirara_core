use sqlx::FromRow;
use uuid::Uuid;
#[derive(Debug, FromRow)]
pub struct TransferTransitionRow {
    pub id: i64,
    pub intent_id: Uuid,
    pub from_state: String,
    pub to_state: String,
    pub at: chrono::DateTime<chrono::Utc>,
    pub reason: Option<String>,
    pub evidence: Option<String>,
}