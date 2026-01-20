use crate::domain::error::DomainError;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Money {
    minor: i128,
}

impl Money {
    pub fn debit(minor: i128) -> Result<Self, DomainError> {
        if minor <= 0 {
            return Err(DomainError::InvalidDebitAmount);
        }
        Ok(Self { minor })
    }

    pub fn credit(minor: i128) -> Result<Self, DomainError> {
        if minor <= 0 {
            return Err(DomainError::InvalidCreditAmount);
        }
        Ok(Self { minor: -minor })
    }

    pub fn from_signed_minor(minor: i128) -> Result<Self, DomainError> {
        if minor == 0 {
            return Err(DomainError::MoneyZeroNotAllowed);
        }
        Ok(Self { minor })
    }

    pub fn minor(&self) -> i128 {
        self.minor
    }
}
