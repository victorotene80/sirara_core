use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct JournalTxRow {
    pub id: i64,
    pub public_id: Uuid,
    pub external_ref_type: String,
    pub external_ref: String,
    pub description: Option<String>,
    pub created_by: String,
}
