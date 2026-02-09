use chrono::{DateTime, Utc};

use crate::domain::value_objects::{
    PublicId, FailureClass, FailureReason
};
#[derive(Debug, Clone)]
pub struct FailWithPolicyCommand {
    pub public_id: PublicId,
    pub class: FailureClass,
    pub reason: FailureReason,
    pub message: String,
    pub now: DateTime<Utc>,
}