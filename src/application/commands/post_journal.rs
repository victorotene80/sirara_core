use crate::domain::value_objects::{PublicId, ExternalRef, ExternalRefType};
pub struct PostJournalCommand {
    pub journal_public_id: PublicId,
    pub external_ref_type: ExternalRefType,
    pub external_ref: ExternalRef,
    pub created_by: String,
    pub description: Option<String>,
    pub lines: Vec<PostJournalLine>,
}

pub struct PostJournalLine {
    pub account_id: i64,
    pub amount_minor: i128,
}
