use bigdecimal::BigDecimal;
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct JournalLineRow {
    pub account_id: i64,
    pub amount: BigDecimal, // numeric(38,0)
}
