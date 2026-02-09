use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
#[derive(sqlx::FromRow)]
struct HotWalletRow {
    id: i64,
    public_id: uuid::Uuid,
    chain: String,
    asset_code: String,
    region_code: String,
    address_base58: String,
    address_hex: Option<String>,
    max_balance_minor: BigDecimal,
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}