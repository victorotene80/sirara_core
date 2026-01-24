use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalLineDTO {
    pub account_id: i64,
    pub amount_minor: i128,
}