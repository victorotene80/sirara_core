use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterbankInstruction {
    pub intent_id: Uuid,
    pub external_ref: String,
    pub asset_code: String,     // "NGN"
    pub amount_minor: i128,
    pub bank_code: String,
    pub account_number: String,
    pub account_name: Option<String>,
    pub narration: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterbankInitiationResult {
    pub provider: String,
    pub provider_ref: String,
    pub raw: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InterbankExecutionStatus {
    Pending { raw: serde_json::Value },
    Success { raw: serde_json::Value },
    Failed { reason: String, raw: serde_json::Value },
}
