use crate::domain::aggregate::{JournalDraft, PostedJournal};
use crate::domain::value_objects::{
    PublicId,
    ExternalRef,
    ExternalRefType
};
pub struct PostJournalCommand {
    pub public_id: PublicId,
    pub external_ref_type: ExternalRefType,
    pub external_ref: ExternalRef,
    pub description: Option<String>,
    pub created_by: String,
    pub lines: Vec<PostJournalLineCommand>,
}

pub struct PostJournalLineCommand {
    pub account_id: i64,
    pub amount_minor: i128,
}
