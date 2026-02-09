use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FxQuoteJson {
    pub(crate) rate: String,
    pub(crate) path: Vec<String>,
}
