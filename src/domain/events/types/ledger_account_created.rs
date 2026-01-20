use uuid::Uuid;

use crate::domain::entities::{AccountType, OwnerType};
use crate::domain::value_objects::PublicId;

#[derive(Debug, Clone)]
pub struct LedgerAccountCreated {
    //pub account_db_id: i64,
    pub public_id: PublicId,
    pub owner_type: OwnerType,
    pub owner_id: Option<Uuid>,
    pub account_type: AccountType,
    pub asset_id: i16,
    pub is_active: bool,
}
