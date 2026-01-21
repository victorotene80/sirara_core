use bigdecimal::BigDecimal;
use num_traits::ToPrimitive;

use crate::domain::aggregate::{JournalLine, PostedJournal};
use crate::domain::repository::RepoError;
use crate::domain::value_objects::{ExternalRef, ExternalRefType, Money, PublicId};
use crate::infrastructure::persistence::models::{JournalLineRow, JournalTxRow};

pub fn bigdecimal_to_i128(d: &BigDecimal) -> Result<i128, RepoError> {
    d.to_i128().ok_or_else(|| RepoError::Integrity {
        message: "db numeric amount is not a valid i128 integer (out of range or non-integer)".into(),
    })
}

pub fn i128_to_bigdecimal(v: i128) -> BigDecimal {
    BigDecimal::from(v)
}

pub fn map_posted_journal(
    header: JournalTxRow,
    lines: Vec<JournalLineRow>,
    asset_id: i16,
) -> Result<PostedJournal, RepoError> {
    let tx_id = header.id;

    let ext_type = ExternalRefType::from_code(&header.external_ref_type).map_err(|e| RepoError::Integrity {
        message: format!(
            "invalid external_ref_type in db (tx_id={tx_id}, value='{}'): {e}",
            header.external_ref_type
        ),
    })?;

    let external_ref = ExternalRef::new(header.external_ref).map_err(|e| RepoError::Integrity {
        message: format!("invalid external_ref in db (tx_id={tx_id}): {e}"),
    })?;

    let mut out_lines: Vec<JournalLine> = Vec::with_capacity(lines.len());

    for l in lines {
        let account_id = l.account_id;

        let minor = bigdecimal_to_i128(&l.amount).map_err(|_| RepoError::Integrity {
            message: format!(
                "invalid journal line amount in db (tx_id={tx_id}, account_id={account_id}): not i128"
            ),
        })?;

        let money = Money::from_signed_minor(minor).map_err(|e| RepoError::Integrity {
            message: format!(
                "invalid money in db (tx_id={tx_id}, account_id={account_id}, minor={minor}): {e}"
            ),
        })?;

        out_lines.push(JournalLine { account_id, amount: money });
    }

    Ok(PostedJournal {
        db_id: header.id,
        public_id: PublicId::new(header.public_id),
        external_ref_type: ext_type,
        external_ref,
        description: header.description,
        created_by: header.created_by,
        asset_id,
        lines: out_lines,
    })
}
