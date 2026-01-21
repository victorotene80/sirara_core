use crate::domain::entities::{AccountType, LedgerAccount, OwnerType};
use crate::domain::repository::RepoError;
use crate::domain::value_objects::PublicId;

use crate::infrastructure::persistence::models::LedgerAccountRow;

impl LedgerAccountRow {
    pub fn to_domain(&self) -> Result<LedgerAccount, RepoError> {
        let owner_type = match self.owner_type.as_str() {
            "USER" => OwnerType::User,
            "PLATFORM" => OwnerType::Platform,
            "TREASURY" => OwnerType::Treasury,
            other => {
                return Err(RepoError::Integrity {
                    message: format!("unknown owner_type={other} for ledger_account_id={}", self.id),
                })
            }
        };

        let account_type = match self.account_type.as_str() {
            "USER_AVAILABLE" => AccountType::UserAvailable,
            "USER_LOCKED" => AccountType::UserLocked,
            "PLATFORM_CLEARING" => AccountType::PlatformClearing,
            "TREASURY_AVAILABLE" => AccountType::TreasuryAvailable,
            "TREASURY_LOCKED" => AccountType::TreasuryLocked,
            "INVENTORY_AVAILABLE" => AccountType::InventoryAvailable,
            "INVENTORY_LOCKED" => AccountType::InventoryLocked,
            other => {
                return Err(RepoError::Integrity {
                    message: format!("unknown account_type={other} for ledger_account_id={}", self.id),
                })
            }
        };

        Ok(LedgerAccount::new(
            self.id,
            PublicId::new(self.public_id),
            owner_type,
            self.owner_id,
            account_type,
            self.asset_id,
            self.is_active,
        ))
    }
}
