use crate::application::dtos::{CreateAccountDTO, LedgerAccountDTO};
use crate::domain::entities::{AccountType, LedgerAccount, OwnerType};
use crate::domain::repository::NewLedgerAccountSpec;
use crate::domain::value_objects::{PublicId, RegionCode};
use crate::application::AppError;

pub fn map_create_account_to_spec(dto: CreateAccountDTO) -> Result<NewLedgerAccountSpec, AppError> {
    let owner_type = match dto.owner_type.as_str() {
        "USER" => OwnerType::User,
        "PLATFORM" => OwnerType::Platform,
        "TREASURY" => OwnerType::Treasury,
        _ => return Err(AppError::InvalidRequest{ message: "Unknown owner type".into() }),
    };

    let account_type = match dto.account_type.as_str() {
        "USER_AVAILABLE" => AccountType::UserAvailable,
        "USER_LOCKED" => AccountType::UserLocked,
        "PLATFORM_CLEARING" => AccountType::PlatformClearing,
        "TREASURY_AVAILABLE" => AccountType::TreasuryAvailable,
        "TREASURY_LOCKED" => AccountType::TreasuryLocked,
        "INVENTORY_AVAILABLE" => AccountType::InventoryAvailable,
        "INVENTORY_LOCKED" => AccountType::InventoryLocked,
        _ => return Err(AppError::InvalidRequest{ message: "Unknown account type".into() }),
    };

    let owner_id = dto.owner_id
        .as_deref()
        .map(uuid::Uuid::parse_str)
        .transpose()
        .map_err(|_| AppError::InvalidRequest{ message: "Invalid owner_id".into() })?;

    let region_code: Option<RegionCode> =
        match dto.region_code.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            None => None,
            Some(raw) => Some(
                RegionCode::new(raw)
                    .map_err(|e| AppError::InvalidRequest { message: e.to_string() })?
            ),
        };

    Ok(NewLedgerAccountSpec {
        public_id: PublicId::new(uuid::Uuid::new_v4()),
        owner_type,
        owner_id,
        account_type,
        asset_id: dto.asset_id,
        region_code,
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
        region_code: a.region_code().map(|rc| rc.as_str().to_string()),
        is_active: a.is_active(),
    }
}
