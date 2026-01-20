use crate::domain::value_objects::{ExternalRef, ExternalRefType, Money, PublicId};

#[derive(Debug, Clone)]
pub struct ReverseJournalCommand {
    pub original_public_id: PublicId,

    pub reversal_public_id: PublicId,
    pub reversal_external_ref_type: ExternalRefType,
    pub reversal_external_ref: ExternalRef,

    pub created_by: String,
    pub description: Option<String>,

    pub max_lines: usize,
}
