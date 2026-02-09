use std::collections::HashMap;

use crate::application::AppError;
use crate::application::contracts::repository::LedgerRepositoryTx;
use crate::domain::aggregate::{JournalDraft, PostedJournal};
use crate::domain::entities::LedgerAccount;
use crate::domain::services::LedgerPostingService;

pub async fn post_validated(
    ledger: &mut dyn LedgerRepositoryTx,
    draft: JournalDraft,
    policy: &super::LedgerPostingPolicy,
) -> Result<PostedJournal, AppError> {
    let mut ids: Vec<i64> = draft.lines().iter().map(|l| l.account_id).collect();
    ids.sort_unstable();
    ids.dedup();

    let accounts = ledger
        .get_accounts_by_ids_for_validation(&ids)
        .await
        .map_err(AppError::from)?;

    let mut by_id: HashMap<i64, &LedgerAccount> = HashMap::with_capacity(accounts.len());
    for a in &accounts {
        by_id.insert(a.id(), a);
    }

    let validated = draft.validate_with_accounts(&by_id).map_err(AppError::from)?;

    let validated = LedgerPostingService::validate(
        validated,
        &by_id,
        policy.max_lines_batch,
        policy.max_lines_normal,
        &policy.max_abs_minor_by_asset,
    )
        .map_err(AppError::from)?;

    ledger
        .insert_posting_atomic(validated)
        .await
        .map_err(AppError::from)
}
