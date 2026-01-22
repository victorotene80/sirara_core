use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAccountDTO {
    pub owner_type: String,
    pub owner_id: Option<String>,
    pub account_type: String,
    pub asset_id: i16,
    pub is_active: bool,
}