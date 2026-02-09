use sqlx::FromRow;
use uuid::Uuid;
#[derive(Debug, FromRow)]
pub struct TransferTransitionRow {
    pub(crate) id: i64,
    pub(crate) intent_id: Uuid,
    pub(crate) from_state: String,
    pub(crate) to_state: String,
    pub(crate) at: chrono::DateTime<chrono::Utc>,
    pub(crate) reason: Option<String>,
    pub(crate) evidence: Option<String>,
}