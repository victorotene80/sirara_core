use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureInfoDto {
    pub class: String,
    pub reason: String,
    pub message: String,
    pub compensation: Vec<String>,
}