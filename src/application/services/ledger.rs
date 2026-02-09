use std::{collections::HashMap, sync::Arc};
use async_trait::async_trait;

use crate::application::AppError;
use crate::application::commands::{
    HoldUserFundsCommand, ReleaseUserFundsCommand,
    ReserveInventoryCommand, ReleaseInventoryCommand,
};
use crate::application::contracts::repository::{TxContext, UnitOfWork, LedgerRepositoryTx};
use crate::application::contracts::LedgerService;
use crate::application::services::post_validated;
use crate::domain::aggregate::{JournalDraft, PostedJournal};
use crate::domain::value_objects::{ExternalRefType, Money};

#[derive(Debug, Clone)]
pub struct LedgerPostingPolicy {
    pub max_lines_batch: usize,
    pub max_lines_normal: usize,
    pub max_abs_minor_by_asset: HashMap<i16, i128>,
}

pub struct LedgerServiceImpl<U: UnitOfWork> {
    uow: U,
    policy: Arc<LedgerPostingPolicy>,
}

impl<U: UnitOfWork> LedgerServiceImpl<U> {
    pub fn new(uow: U, policy: LedgerPostingPolicy) -> Self {
        Self { uow, policy: Arc::new(policy) }
    }
}

#[async_trait]
impl<U> LedgerService for LedgerServiceImpl<U>
where
    U: UnitOfWork + Send + Sync,
{
    async fn hold_user_funds(&self, cmd: HoldUserFundsCommand) -> Result<PostedJournal, AppError> {
        if cmd.amount_minor <= 0 {
            return Err(AppError::InvalidRequest { message: "amount_minor must be > 0".into() });
        }

        let policy = Arc::clone(&self.policy);

        self.uow.with_tx(move |ctx| {
            Box::pin(async move {
                let mut ledger = ctx.ledger();

                let (avail, locked) = ledger
                    .resolve_user_hold_accounts(cmd.user_id, &cmd.asset_code)
                    .await
                    .map_err(AppError::from)?;

                let mut draft = JournalDraft::new(
                    cmd.journal_public_id,
                    ExternalRefType::TransferIntent,
                    cmd.external_ref,
                    cmd.created_by,
                    Some("user hold: available -> locked".into()),
                ).map_err(AppError::from)?;

                draft.add_line(avail, Money::from_signed_minor(-cmd.amount_minor).map_err(AppError::from)?);
                draft.add_line(locked, Money::from_signed_minor(cmd.amount_minor).map_err(AppError::from)?);

                post_validated(&mut *ledger, draft, policy.as_ref()).await
            })
        }).await
    }

    async fn release_user_funds(&self, cmd: ReleaseUserFundsCommand) -> Result<PostedJournal, AppError> {
        if cmd.amount_minor <= 0 {
            return Err(AppError::InvalidRequest { message: "amount_minor must be > 0".into() });
        }

        let policy = Arc::clone(&self.policy);

        self.uow.with_tx(move |ctx| {
            Box::pin(async move {
                let mut ledger = ctx.ledger();

                let (avail, locked) = ledger
                    .resolve_user_hold_accounts(cmd.user_id, &cmd.asset_code)
                    .await
                    .map_err(AppError::from)?;

                let mut draft = JournalDraft::new(
                    cmd.journal_public_id,
                    ExternalRefType::TransferIntent,
                    cmd.external_ref,
                    cmd.created_by,
                    Some("user release: locked -> available".into()),
                ).map_err(AppError::from)?;

                draft.add_line(locked, Money::from_signed_minor(-cmd.amount_minor).map_err(AppError::from)?);
                draft.add_line(avail, Money::from_signed_minor(cmd.amount_minor).map_err(AppError::from)?);

                post_validated(&mut *ledger, draft, policy.as_ref()).await
            })
        }).await
    }

    async fn reserve_inventory(&self, cmd: ReserveInventoryCommand) -> Result<PostedJournal, AppError> {
        if cmd.amount_minor <= 0 {
            return Err(AppError::InvalidRequest { message: "amount_minor must be > 0".into() });
        }

        let policy = Arc::clone(&self.policy);

        self.uow.with_tx(move |ctx| {
            Box::pin(async move {
                let mut ledger = ctx.ledger();

                let (avail, locked) = ledger
                    .resolve_platform_inventory_accounts(&cmd.asset_code, &cmd.region_code)
                    .await
                    .map_err(AppError::from)?;

                let mut draft = JournalDraft::new(
                    cmd.journal_public_id,
                    ExternalRefType::TransferIntent,
                    cmd.external_ref,
                    cmd.created_by,
                    Some("inventory reserve: available -> locked".into()),
                ).map_err(AppError::from)?;

                draft.add_line(avail, Money::from_signed_minor(-cmd.amount_minor).map_err(AppError::from)?);
                draft.add_line(locked, Money::from_signed_minor(cmd.amount_minor).map_err(AppError::from)?);

                post_validated(&mut *ledger, draft, policy.as_ref()).await
            })
        }).await
    }

    async fn peek_balance_minor(&self, account_id: i64) -> Result<i128, AppError> {
        self.uow
            .with_tx(move |ctx| {
                Box::pin(async move {
                    let mut ledger = ctx.ledger();
                    ledger
                        .peek_balance_minor(account_id)
                        .await
                        .map_err(AppError::from)
                })
            })
            .await
    }

    async fn release_inventory(&self, cmd: ReleaseInventoryCommand) -> Result<PostedJournal, AppError> {
        if cmd.amount_minor <= 0 {
            return Err(AppError::InvalidRequest { message: "amount_minor must be > 0".into() });
        }

        let policy = Arc::clone(&self.policy);

        self.uow.with_tx(move |ctx| {
            Box::pin(async move {
                let mut ledger = ctx.ledger();

                let (avail, locked) = ledger
                    .resolve_platform_inventory_accounts(&cmd.asset_code, &cmd.region_code)
                    .await
                    .map_err(AppError::from)?;

                let mut draft = JournalDraft::new(
                    cmd.journal_public_id,
                    ExternalRefType::TransferIntent,
                    cmd.external_ref,
                    cmd.created_by,
                    Some("inventory release: locked -> available".into()),
                ).map_err(AppError::from)?;

                draft.add_line(locked, Money::from_signed_minor(-cmd.amount_minor).map_err(AppError::from)?);
                draft.add_line(avail, Money::from_signed_minor(cmd.amount_minor).map_err(AppError::from)?);

                post_validated(&mut *ledger, draft, policy.as_ref()).await
            })
        }).await
    }
}
