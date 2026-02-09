use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureInfoJson {
    pub(crate) class: String,
    pub(crate) reason: String,
    pub(crate) message: Option<String>,
    pub(crate) failed_at: DateTime<Utc>,
    pub(crate) compensation: Vec<String>,
}
