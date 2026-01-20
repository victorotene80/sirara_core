use crate::domain::error::DomainError;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AssetCode(String);

impl AssetCode {
    pub fn new(code: impl Into<String>) -> Result<Self, DomainError> {
        let code = code.into().trim().to_string();

        if code.len() < 2 || code.len() > 10 {
            return Err(DomainError::AssetCodeInvalidLength { min: 2, max: 10 });
        }

        if code != code.to_uppercase() {
            return Err(DomainError::AssetCodeNotUppercase);
        }

        Ok(Self(code))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

