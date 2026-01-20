#[derive(Debug, Clone)]
pub struct SetLedgerAccountActiveCommand {
    pub account_id: i64,
    pub is_active: bool,
}