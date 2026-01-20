#[derive(Debug, Clone)]
pub struct AccountBalanceDTO {
    pub account_id: i64,
    pub asset_id: i16,
    pub balance_minor: i128,
}