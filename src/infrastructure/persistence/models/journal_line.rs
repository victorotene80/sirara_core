use bigdecimal::BigDecimal;
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct JournalLineRow {
    pub(crate) account_id: i64,
    pub(crate) asset_id: i16,
    pub(crate) amount: BigDecimal,
}
