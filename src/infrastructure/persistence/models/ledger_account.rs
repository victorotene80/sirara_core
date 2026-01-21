use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct LedgerAccountRow {
    pub id: i64,
    pub public_id: Uuid,
    pub owner_type: String,   // 'USER' | 'PLATFORM' | 'TREASURY'
    pub owner_id: Option<Uuid>,
    pub account_type: String, // 'USER_AVAILABLE' etc
    pub asset_id: i16,
    pub is_active: bool,
}
