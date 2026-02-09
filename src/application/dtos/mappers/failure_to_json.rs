use serde::{Deserialize, Serialize};

use crate::domain::error::DomainError;
use crate::domain::value_objects::{CompensationAction, FailureClass, FailureInfo, FailureReason};
use crate::application::dtos::FailureInfoDto;

impl From<&FailureInfo> for FailureInfoDto {
     fn from(f: &FailureInfo) -> Self {
        Self {
            class: f.class().as_str().to_string(),
            reason: f.reason().as_str().to_string(),
            message: f.message().to_string(),
            compensation: f
                .compensation()
                .iter()
                .map(|c| c.as_str().to_string())
                .collect(),
        }
    }
}

impl TryFrom<FailureInfoDto> for FailureInfo {
    type Error = DomainError;

    fn try_from(dto: FailureInfoDto) -> Result<Self, Self::Error> {
        let class = dto.class.parse::<FailureClass>()?;
        let reason = dto.reason.parse::<FailureReason>()?;
        let compensation = dto
            .compensation
            .into_iter()
            .map(|s| s.parse::<CompensationAction>())
            .collect::<Result<Vec<_>, _>>()?;

        FailureInfo::new(class, reason, dto.message, compensation)
    }
}

pub fn failure_to_json(f: &FailureInfo) -> Result<serde_json::Value, DomainError> {
    serde_json::to_value(FailureInfoDto::from(f)).map_err(|e| DomainError::InvalidFailure {
        message: format!("failed to serialize failure info: {e}"),
    })
}

pub fn failure_from_json(v: &serde_json::Value) -> Result<FailureInfo, DomainError> {
    let dto: FailureInfoDto = serde_json::from_value(v.clone()).map_err(|e| DomainError::InvalidFailure {
        message: format!("failed to parse failure info: {e}"),
    })?;
    FailureInfo::try_from(dto)
}