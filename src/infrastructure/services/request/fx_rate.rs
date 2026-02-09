use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FxProviderQuoteRequest {
    pub from: String,
    pub to: String,
    pub via: Vec<String>,
    pub amount_minor: i128,
}
