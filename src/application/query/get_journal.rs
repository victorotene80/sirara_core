use crate::domain::value_objects::PublicId;

pub struct GetJournalQuery {
    pub public_id: PublicId,
}