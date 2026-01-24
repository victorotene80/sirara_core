use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]

pub enum DestinationDTO {
    Onchain { chain: String, address: String, memo: Option<String> },
    Bank { bank_code: String, account_number: String, account_name: Option<String> },
}