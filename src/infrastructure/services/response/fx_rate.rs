use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FxRatesResponse {
    pub success: bool,
    pub base: String,
    pub timestamp: i64,
    pub rates: HashMap<String, f64>,
}
