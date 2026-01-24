use crate::domain::value_objects::FailureInfo;
use crate::domain::error::DomainError;
use crate::domain::value_objects::{CompensationAction, FailureClass, FailureReason};

use crate::infrastructure::persistence::models::FailureInfoJson;

pub fn from_json(dto: FailureInfoJson) -> Result<FailureInfo, DomainError> {
    Ok(FailureInfo {
        class: FailureClass::from_str(&dto.class)?,
        reason: FailureReason::from_str(&dto.reason)?,
        message: dto.message,
        failed_at: dto.failed_at,
        compensation: dto
            .compensation
            .into_iter()
            .map(|s| CompensationAction::from_str(&s))
            .collect::<Result<Vec<_>, _>>()?,
    })
}

pub fn to_json(f: &FailureInfo) -> FailureInfoJson {
    FailureInfoJson {
        class: f.class.as_str().to_string(),
        reason: f.reason.as_str().to_string(),
        message: f.message.clone(),
        failed_at: f.failed_at,
        compensation: f.compensation.iter().map(|c| c.as_str().to_string()).collect(),
    }
}
