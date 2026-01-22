use crate::application::dtos::{PostJournalRequestDTO, JournalLineDTO};
use crate::domain::aggregate::JournalDraft;
use crate::domain::value_objects::{ExternalRef, ExternalRefType, Money, PublicId};
use crate::application::AppError;

pub fn map_post_journal_request(dto: PostJournalRequestDTO) -> Result<JournalDraft, AppError> {
    let public_id = PublicId::new(uuid::Uuid::parse_str(&dto.public_id)?);

    let external_ref_type =
        ExternalRefType::from_code(&dto.external_ref_type)
            .map_err(AppError::from)?;

    let external_ref = ExternalRef::new(dto.external_ref.clone())
        .map_err(AppError::from)?;

    let mut draft = JournalDraft::new(
        public_id,
        external_ref_type,
        external_ref,
        dto.created_by,
        dto.description,
    ).map_err(AppError::from)?;

    for l in dto.lines {
        let money = if l.amount_minor > 0 {
            Money::debit(l.amount_minor)
        } else {
            Money::credit(l.amount_minor.abs())
        };

        draft.add_line(l.account_id, money.map_err(AppError::from)?);
    }

    Ok(draft)
}
