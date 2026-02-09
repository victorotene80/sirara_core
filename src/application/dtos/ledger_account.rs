use uuid::uuid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerAccountDTO {
    pub id: i64,
    pub public_id: String,
    pub owner_type: String,
    pub owner_id: Option<String>,
    pub account_type: String,
    pub region_code: Option<String>,
    pub asset_id: i16,
    pub is_active: bool,
}