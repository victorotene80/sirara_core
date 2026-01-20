use crate::domain::entities::{OwnerType, AccountType};
use crate::domain::value_objects::PublicId;

#[derive(Debug, Clone)]
pub struct LedgerAccountDTO {
    pub id: i64,
    pub public_id: PublicId,
    pub owner_type: OwnerType,
    pub owner_id: Option<uuid::Uuid>,
    pub account_type: AccountType,
    pub asset_id: i16,
    pub is_active: bool,
}