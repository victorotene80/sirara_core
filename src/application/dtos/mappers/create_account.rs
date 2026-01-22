use crate::application::dtos::{CreateAccountDTO, LedgerAccountDTO};
use crate::domain::entities::{AccountType, LedgerAccount, OwnerType};
use crate::domain::repository::NewLedgerAccountSpec;
use crate::domain::value_objects::PublicId;
use crate::application::AppError;

pub fn map_create_account_to_spec(dto: CreateAccountDTO) -> Result<NewLedgerAccountSpec, AppError> {
    let owner_type = match dto.owner_type.as_str() {
        "USER" => OwnerType::User,
        "PLATFORM" => OwnerType::Platform,
        "TREASURY" => OwnerType::Treasury,
        _ => {
            return Err(AppError::InvalidRequest {
                message: "Unknown account type".to_string(),
            })
        }

    };

    let account_type = match dto.account_type.as_str() {
        "USER_AVAILABLE" => AccountType::UserAvailable,
        "USER_LOCKED" => AccountType::UserLocked,
        "TREASURY_AVAILABLE" => AccountType::TreasuryAvailable,
        "TREASURY_LOCKED" => AccountType::TreasuryLocked,
        "INVENTORY_AVAILABLE" => AccountType::InventoryAvailable,
        "INVENTORY_LOCKED" => AccountType::InventoryLocked,
        _ => {
            return Err(AppError::InvalidRequest {
                message: "Unknown owner type".to_string(),
            })
        }
    };

    Ok(NewLedgerAccountSpec {
        public_id: PublicId::new(uuid::Uuid::new_v4()),
        owner_type,
        owner_id: dto.owner_id.map(|s| uuid::Uuid::parse_str(&s)).transpose()?,
        account_type,
        asset_id: dto.asset_id,
        is_active: dto.is_active,
    })
}

pub fn map_account_to_dto(a: &LedgerAccount) -> LedgerAccountDTO {
    LedgerAccountDTO {
        id: a.id(),
        public_id: a.public_id().value().to_string(),
        owner_type: a.owner_type().as_str().to_string(),
        owner_id: a.owner_id().map(|id| id.to_string()),
        account_type: a.account_type().as_str().to_string(),
        asset_id: a.asset_id(),
        is_active: a.is_active(),
    }
}
