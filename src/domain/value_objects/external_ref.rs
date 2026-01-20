use crate::domain::error::DomainError;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExternalRef(String);

impl ExternalRef {
    pub fn new(s: impl Into<String>) -> Result<Self, DomainError> {
        let s = s.into().trim().to_string();

        if s.is_empty() {
            return Err(DomainError::ExternalRefEmpty);
        }
        if s.len() > 200 {
            return Err(DomainError::ExternalRefTooLong { max: 200 });
        }

        Ok(Self(s))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
