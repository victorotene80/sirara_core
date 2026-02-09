use chrono::{DateTime, Utc};
use crate::domain::error::DomainError;
use crate::domain::value_objects::FxQuote;
use crate::infrastructure::persistence::models::FxQuoteJson;

pub fn from_json(dto: FxQuoteJson, expires_at: DateTime<Utc>) -> Result<FxQuote, DomainError> {
    FxQuote::new(dto.rate, expires_at, dto.path)
}

pub fn to_json(q: &FxQuote) -> FxQuoteJson {
    FxQuoteJson {
        rate: q.rate().to_string(),
        path: q.path().to_vec(),
    }
}
