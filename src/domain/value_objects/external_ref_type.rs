use crate::domain::error::DomainError;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ExternalRefType {
    TransferIntent,
    ManualAdjustment,
    Reversal,
    Fee,
    Settlement,
}

impl ExternalRefType {
    pub fn as_code(&self) -> &'static str {
        match self {
            ExternalRefType::TransferIntent => "TRANSFER_INTENT",
            ExternalRefType::ManualAdjustment => "MANUAL_ADJUSTMENT",
            ExternalRefType::Reversal => "REVERSAL",
            ExternalRefType::Fee => "FEE",
            ExternalRefType::Settlement => "SETTLEMENT",
        }
    }
    pub fn from_code(s: &str) -> Result<Self, DomainError> {
        match s {
            "TRANSFER_INTENT" => Ok(Self::TransferIntent),
            "MANUAL_ADJUSTMENT" => Ok(Self::ManualAdjustment),
            "REVERSAL" => Ok(Self::Reversal),
            "FEE" => Ok(Self::Fee),
            "SETTLEMENT" => Ok(Self::Settlement),
            other => Err(DomainError::InvalidExternalRefType {
                value: other.to_string(),
            }),
        }
    }
}
