use crate::domain::entities::OwnerType;

pub struct ListAccountsByOwnerQuery {
    pub owner_type: OwnerType,
    pub owner_id: Option<uuid::Uuid>,
}