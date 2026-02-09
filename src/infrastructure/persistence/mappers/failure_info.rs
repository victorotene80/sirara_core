use serde_json::json;

use crate::domain::error::DomainError;
use crate::domain::value_objects::{FailureInfo, FailureClass, FailureReason, CompensationAction};

pub fn to_json(f: &FailureInfo) -> serde_json::Value {
    json!({
        "class": f.class().as_str(),
        "reason": f.reason().as_str(),
        "message": f.message(),
        "compensation": f.compensation().iter().map(|c| c.as_str()).collect::<Vec<_>>()
    })
}

pub fn from_json(v: &serde_json::Value) -> Result<FailureInfo, DomainError> {
    let class = v.get("class").and_then(|x| x.as_str()).ok_or_else(|| DomainError::InvalidFailure {
        message: "missing failure.class".into()
    })?;
    let reason = v.get("reason").and_then(|x| x.as_str()).ok_or_else(|| DomainError::InvalidFailure {
        message: "missing failure.reason".into()
    })?;
    let message = v.get("message").and_then(|x| x.as_str()).ok_or_else(|| DomainError::InvalidFailure {
        message: "missing failure.message".into()
    })?;

    let comp = v.get("compensation")
        .and_then(|x| x.as_array())
        .ok_or_else(|| DomainError::InvalidFailure { message: "missing failure.compensation".into() })?
        .iter()
        .map(|x| x.as_str().ok_or_else(|| DomainError::InvalidFailure { message: "bad compensation entry".into() }))
        .collect::<Result<Vec<_>, _>>()?;

    let class = class.parse::<FailureClass>()?;
    let reason = reason.parse::<FailureReason>()?;
    let compensation = comp.into_iter()
        .map(|s| s.parse::<CompensationAction>())
        .collect::<Result<Vec<_>, _>>()?;

    FailureInfo::new(class, reason, message, compensation)
}
