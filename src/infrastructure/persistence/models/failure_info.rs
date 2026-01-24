use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureInfoJson {
    pub class: String,
    pub reason: String,
    pub message: Option<String>,
    pub failed_at: DateTime<Utc>,
    pub compensation: Vec<String>,
}
