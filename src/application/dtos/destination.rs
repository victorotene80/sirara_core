#[derive(Debug, Clone)]
pub enum DestinationDTO {
    Bank {
        bank_code: String,
        account_number: String,
    },
    Onchain {
        chain: String,
        address: String,
        memo: Option<String>,
    },
}

impl DestinationDTO {
    pub fn is_onchain(&self) -> bool {
        matches!(self, DestinationDTO::Onchain { .. })
    }
}
