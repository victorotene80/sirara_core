use crate::domain::value_objects::{PublicId, ExternalRefType};
use crate::application::dtos::JournalLineDTO;

#[derive(Debug, Clone)]
pub struct JournalDTO {
    pub db_id: i64,
    pub public_id: PublicId,
    pub external_ref_type: ExternalRefType,
    pub external_ref: String,
    pub description: Option<String>,
    pub created_by: String,
    pub asset_id: i16,
    pub lines: Vec<JournalLineDTO>,
}