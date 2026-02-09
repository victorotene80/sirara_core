use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DestinationJson {
    BANK {
        bank_code: String,
        account_number: String,
    },
    ONCHAIN {
        chain: String,
        address: String,
        memo: Option<String>,
    },
}