use crate::domain::value_objects::ExternalRefType;

pub struct ListJournalsQuery {
    pub external_ref_type: Option<ExternalRefType>,
    pub created_by: Option<String>,
    pub limit: usize,
    pub offset: usize,
}