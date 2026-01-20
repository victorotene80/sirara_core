use crate::domain::aggregate::{JournalDraft, PostedJournal};

#[derive(Debug, Clone)]
pub struct PostJournalCommand {
    pub draft: JournalDraft,
    /// Optional safety limit against abuse
    pub max_lines: usize,
}