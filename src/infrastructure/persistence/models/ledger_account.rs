use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct LedgerAccountRow {
    pub id: i64,
    pub public_id: Uuid,
    pub owner_type: String,
    pub owner_id: Option<Uuid>,
    pub account_type: String,
    pub asset_id: i16,
    pub region_code: Option<String>,
    pub is_active: bool,
}
