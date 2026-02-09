use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct JournalTxRow {
    pub(crate) id: i64,
    pub(crate) public_id: Uuid,
    pub(crate) external_ref_type: String,
    pub(crate) external_ref: String,
    pub(crate) description: Option<String>,
    pub(crate) created_by: String,
}
