use crate::application::dtos::{PostedJournalDTO, JournalLineDTO};
use crate::domain::aggregate::PostedJournal;

pub fn posted_to_dto(p: &PostedJournal) -> PostedJournalDTO {
    PostedJournalDTO {
        db_id: p.db_id,
        public_id: p.public_id.value().to_string(),
        external_ref_type: p.external_ref_type.as_code().to_string(),
        external_ref: p.external_ref.as_str().to_string(),
        description: p.description.clone(),
        created_by: p.created_by.clone(),
        asset_id: p.asset_id,
        lines: p.lines.iter().map(|l| JournalLineDTO {
            account_id: l.account_id,
            amount_minor: l.amount.minor(),
        }).collect(),
    }
}
