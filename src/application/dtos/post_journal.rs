use serde::{Deserialize, Serialize};
use crate::application::dtos::JournalLineDTO;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostJournalRequestDTO {
    pub public_id: String,
    pub external_ref_type: String,
    pub external_ref: String,
    pub description: Option<String>,
    pub created_by: String,
    pub lines: Vec<JournalLineDTO>,
}