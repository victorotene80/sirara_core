use async_trait::async_trait;
use std::collections::HashMap;

use crate::application::contracts::LedgerService;
use crate::application::contracts::repository::{ReposInTx, UnitOfWork};
use crate::application::dtos::{CreateAccountDTO, LedgerAccountDTO, PostedJournalDTO};
use crate::application::commands::PostJournalCommand;
use crate::application::dtos::mappers::{map_create_account_to_spec, map_account_to_dto, posted_to_dto};
use crate::application::AppError;
use crate::domain::value_objects::Money;
use crate::domain::aggregate::{JournalDraft, ValidatedJournal};
use crate::domain::entities::LedgerAccount;
use crate::domain::repository::LedgerRepository;
use crate::domain::services::LedgerPostingService;

use crate::utils::configuration::LedgerConfig;

pub struct LedgerServiceImpl<R: LedgerRepository, U: UnitOfWork> {
    repo: R,
    uow: U,

    max_lines_batch: usize,
    max_lines_normal: usize,
    max_abs_minor_by_asset: HashMap<i16, i128>, // asset_id -> limit
}

impl<R, U> LedgerServiceImpl<R, U>
where
    R: LedgerRepository + Send + Sync,
    U: UnitOfWork + Send + Sync,
{
    /// `max_abs_minor_by_asset` should already be asset_id -> max_abs_minor.
    /// (You can build it from cfg.max_post_amount_by_code + assets table during bootstrap.)
    pub fn new(repo: R, uow: U, cfg: &LedgerConfig, max_abs_minor_by_asset: HashMap<i16, i128>) -> Self {
        Self {
            repo,
            uow,
            max_lines_batch: cfg.max_lines_batch,
            max_lines_normal: cfg.max_lines_normal,
            max_abs_minor_by_asset,
        }
    }
}

#[async_trait]
impl<R, U> LedgerService for LedgerServiceImpl<R, U>
where
    R: LedgerRepository + Send + Sync,
    U: UnitOfWork + Send + Sync,
{
    async fn create_account(&self, req: CreateAccountDTO) -> Result<LedgerAccountDTO, AppError> {
        let spec = map_create_account_to_spec(req)?;
        let account = self.repo.create_account(spec).await?;
        Ok(map_account_to_dto(&account))
    }

    async fn post_journal_atomic(&self, cmd: PostJournalCommand) -> Result<PostedJournalDTO, AppError> {
        // copy config values into the tx closure (so we don't borrow self across await points)
        let max_lines_batch = self.max_lines_batch;
        let max_lines_normal = self.max_lines_normal;
        let max_abs_minor_by_asset = self.max_abs_minor_by_asset.clone();

        let posted = self
            .uow
            .with_tx(move |repos: &mut dyn ReposInTx| {
                Box::pin(async move {
                    // 1) Build JournalDraft from command
                    let mut draft = JournalDraft::new(
                        cmd.public_id,
                        cmd.external_ref_type,
                        cmd.external_ref,
                        cmd.created_by,
                        cmd.description,
                    )?;

                    for line in cmd.lines {
                        // NOTE: you’re passing signed i128 from application.
                        // JournalDraft wants Money, so convert here.
                        // If your convention is: debit = +minor, credit = -minor, this matches.
                        // Otherwise adjust accordingly.
                        let money = Money::from_signed_minor(line.amount_minor)?;
                        draft.add_line(line.account_id, money);
                    }

                    // 2) Load accounts needed for validation (IN TX)
                    let ids: Vec<i64> = draft.lines().iter().map(|l| l.account_id).collect();

                    let mut ledger_tx = repos.ledger();
                    let accounts: Vec<LedgerAccount> =
                        ledger_tx.get_accounts_by_ids_for_validation(&ids).await?;

                    // Build accounts_by_id: HashMap<i64, &LedgerAccount>
                    let mut accounts_by_id: HashMap<i64, &LedgerAccount> =
                        HashMap::with_capacity(accounts.len());
                    for a in &accounts {
                        accounts_by_id.insert(a.id(), a);
                    }

                    // 3) Domain validation using accounts
                    let validated: ValidatedJournal = draft.validate_with_accounts(&accounts_by_id)?;

                    // 4) Policy/guards validation (line limits, max amount, taxonomy, etc.)
                    let validated = LedgerPostingService::validate(
                        validated,
                        &accounts_by_id,
                        max_lines_batch,
                        max_lines_normal,
                        &max_abs_minor_by_asset,
                    )?;

                    // 5) Persist atomically
                    let posted = ledger_tx.insert_posting_atomic(validated).await?;

                    Ok(posted)
                })
            })
            .await?; // RepoError -> AppError should be mapped by your UnitOfWork boundary

        Ok(posted_to_dto(&posted))
    }

    async fn find_posted_by_external_ref(
        &self,
        external_ref_type: String,
        external_ref: String,
    ) -> Result<Option<PostedJournalDTO>, AppError> {
        let ext_type = crate::domain::value_objects::ExternalRefType::from_code(&external_ref_type)?;
        let ext_ref = crate::domain::value_objects::ExternalRef::new(external_ref)?;

        if let Some(j) = self.repo.find_posted_by_external_ref(ext_type, &ext_ref).await? {
            return Ok(Some(posted_to_dto(&j)));
        }
        Ok(None)
    }
}
