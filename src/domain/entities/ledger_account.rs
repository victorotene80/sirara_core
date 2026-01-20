use uuid::Uuid;

use crate::domain::error::DomainError;
use crate::domain::value_objects::PublicId;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum OwnerType {
    User,
    Platform,
    Treasury,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AccountType {
    UserAvailable,
    UserLocked,
    PlatformClearing,
    TreasuryAvailable,
    TreasuryLocked,
    InventoryAvailable,
    InventoryLocked,
}

#[derive(Debug, Clone)]
pub struct LedgerAccount {
    id: i64, // DB id (internal)
    public_id: PublicId,
    owner_type: OwnerType,
    owner_id: Option<Uuid>,
    account_type: AccountType,
    asset_id: i16,
    is_active: bool,
}

impl LedgerAccount {
    pub fn new(
        id: i64,
        public_id: PublicId,
        owner_type: OwnerType,
        owner_id: Option<Uuid>,
        account_type: AccountType,
        asset_id: i16,
        is_active: bool,
    ) -> Self {
        Self {
            id,
            public_id,
            owner_type,
            owner_id,
            account_type,
            asset_id,
            is_active,
        }
    }

    pub fn ensure_active(&self) -> Result<(), DomainError> {
        if !self.is_active {
            return Err(DomainError::LedgerAccountInactive);
        }
        Ok(())
    }

    pub fn id(&self) -> i64 { self.id }
    pub fn public_id(&self) -> PublicId { self.public_id }
    pub fn owner_type(&self) -> OwnerType { self.owner_type }
    pub fn owner_id(&self) -> Option<Uuid> { self.owner_id }
    pub fn account_type(&self) -> AccountType { self.account_type }
    pub fn asset_id(&self) -> i16 { self.asset_id }
    pub fn is_active(&self) -> bool { self.is_active }
}
