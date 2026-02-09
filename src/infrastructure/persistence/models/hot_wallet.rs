use chrono::{DateTime, Utc};
use bigdecimal::BigDecimal;
use sqlx::FromRow;
#[derive(sqlx::FromRow)]
pub struct HotWalletRow {
    pub id: i64,
    pub public_id: uuid::Uuid,
    pub chain: String,
    pub asset_code: String,
    pub region_code: String,
    pub address_base58: String,
    pub address_hex: Option<String>,
    pub secret_handle: Option<String>,
    pub max_balance_minor: BigDecimal,
    pub is_active: bool,
    pub version: i32,
    pub created_at:DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

