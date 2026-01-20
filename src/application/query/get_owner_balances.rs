use crate::domain::entities::OwnerType;

pub struct GetOwnerBalancesQuery {
    pub owner_type: OwnerType,
    pub owner_id: Option<uuid::Uuid>,
}