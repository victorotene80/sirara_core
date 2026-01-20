use crate::domain::value_objects::PublicId;

pub struct GetAccountQuery {
    pub public_id: PublicId,
}