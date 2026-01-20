use crate::domain::entities::{OwnerType};
use crate::application::dtos::AccountBalanceDTO;

#[derive(Debug, Clone)]
pub struct OwnerBalancesDTO {
    pub owner_type: OwnerType,
    pub owner_id: Option<uuid::Uuid>,
    pub balances: Vec<AccountBalanceDTO>,
}