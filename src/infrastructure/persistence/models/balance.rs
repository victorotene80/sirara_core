use bigdecimal::BigDecimal;

#[derive(sqlx::FromRow)]
pub struct BalanceRow {
    pub(crate) account_id: i64,
    pub(crate) balance: BigDecimal,
}