use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::application::AppError;
use crate::application::commands::{
    CreateTransferIntentCommand, FailWithPolicyCommand, LockFundsCommand, LockRateCommand,
    MarkDebitPostedCommand, MarkRailSubmittedCommand, ReserveInventoryIntentCommand, SettleCommand,
};
use crate::application::contracts::repository::{IntentPatch, TxContext, UnitOfWork};
use crate::application::contracts::TransferService;

use crate::domain::aggregate::TransferIntent;
use crate::domain::value_objects::{PublicId, TransferState};

pub struct TransferServiceImpl<U: UnitOfWork> {
    uow: U,
}

impl<U: UnitOfWork> TransferServiceImpl<U> {
    pub fn new(uow: U) -> Self {
        Self { uow }
    }

    async fn apply_transition_cas_only(
        ctx: &mut dyn TxContext,
        public_id: PublicId,
        _now: DateTime<Utc>,
        mutator: impl FnOnce(&mut TransferIntent) -> Result<IntentPatch, AppError> + Send,
        to_state: TransferState,
        reason: &str,
    ) -> Result<TransferIntent, AppError> {
        let mut repo = ctx.transfer();

        let mut intent = repo
            .get_intent_by_public_id(public_id)
            .await
            .map_err(AppError::from)?;

        let expected_version = intent.version();
        let expected_state = intent.state();

        let patch = mutator(&mut intent)?;

        let appended = repo
            .append_transition_if_current(
                intent.db_id(),
                expected_version,
                expected_state,
                to_state,
                reason.to_string(),
                None,
            )
            .await
            .map_err(AppError::from)?;

        if !appended {
            return Err(AppError::Conflict {
                message: format!(
                    "transition append conflict (public_id={}, expected_state={}, expected_version={})",
                    public_id.value(),
                    expected_state.as_str(),
                    expected_version
                ),
            });
        }

        repo.update_intent_state_cas(
            intent.db_id(),
            expected_version,
            expected_state,
            to_state,
            patch,
        )
            .await
            .map_err(AppError::from)?;

        let reloaded = repo
            .get_intent_by_public_id(public_id)
            .await
            .map_err(AppError::from)?;

        Ok(reloaded)
    }

    async fn apply_transition_with_lock(
        ctx: &mut dyn TxContext,
        public_id: PublicId,
        _now: DateTime<Utc>,
        mutator: impl FnOnce(&mut TransferIntent) -> Result<IntentPatch, AppError> + Send,
        to_state: TransferState,
        reason: &str,
    ) -> Result<TransferIntent, AppError> {
        let mut repo = ctx.transfer();

        let mut intent = repo
            .get_intent_for_update_by_public_id(public_id)
            .await
            .map_err(AppError::from)?;

        let expected_version = intent.version();
        let expected_state = intent.state();

        let patch = mutator(&mut intent)?;

        let appended = repo
            .append_transition_if_current(
                intent.db_id(),
                expected_version,
                expected_state,
                to_state,
                reason.to_string(),
                None,
            )
            .await
            .map_err(AppError::from)?;

        if !appended {
            return Err(AppError::Conflict {
                message: format!(
                    "transition append conflict (public_id={}, expected_state={}, expected_version={})",
                    public_id.value(),
                    expected_state.as_str(),
                    expected_version
                ),
            });
        }

        repo.update_intent_state_cas(
            intent.db_id(),
            expected_version,
            expected_state,
            to_state,
            patch,
        )
            .await
            .map_err(AppError::from)?;

        let reloaded = repo
            .get_intent_for_update_by_public_id(public_id)
            .await
            .map_err(AppError::from)?;

        Ok(reloaded)
    }
}

#[async_trait]
impl<U> TransferService for TransferServiceImpl<U>
where
    U: UnitOfWork + Send + Sync,
{
    async fn create_intent(
        &self,
        cmd: CreateTransferIntentCommand,
    ) -> Result<TransferIntent, AppError> {
        self.uow
            .with_tx(move |ctx| {
                Box::pin(async move {
                    let mut repo = ctx.transfer();

                    let intent = TransferIntent::new(
                        cmd.public_id,
                        cmd.external_ref_type,
                        cmd.external_ref,
                        cmd.route,
                        cmd.asset,
                        cmd.amount,
                        cmd.now,
                    )?;

                    let res = repo.insert_intent_if_absent(intent).await?;
                    Ok(res.intent)
                })
            })
            .await
            .map_err(AppError::from)
    }

    async fn lock_rate(&self, cmd: LockRateCommand) -> Result<TransferIntent, AppError> {
        self.uow
            .with_tx(move |ctx| {
                Box::pin(async move {
                    Self::apply_transition_cas_only(
                        ctx,
                        cmd.public_id,
                        cmd.now,
                        move |intent| {
                            intent.lock_rate(cmd.quote.clone(), cmd.required_usdt_minor, cmd.now)?;

                            let q = intent.quote().ok_or_else(|| AppError::Invariant {
                                message: "RATE_LOCKED requires quote".into(),
                            })?;

                            Ok(IntentPatch {
                                quote: Some(q.clone()),
                                quote_expires_at: Some(q.expires_at()),
                                required_usdt_minor: intent.required_usdt_minor(),
                                failure: None,
                            })
                        },
                        TransferState::RateLocked,
                        "rate locked",
                    )
                        .await
                })
            })
            .await
            .map_err(AppError::from)
    }

    async fn lock_funds(&self, cmd: LockFundsCommand) -> Result<TransferIntent, AppError> {
        self.uow
            .with_tx(move |ctx| {
                Box::pin(async move {
                    Self::apply_transition_cas_only(
                        ctx,
                        cmd.public_id,
                        cmd.now,
                        move |intent| {
                            intent.lock_funds(cmd.now)?;
                            Ok(IntentPatch::default())
                        },
                        TransferState::FundsLocked,
                        "funds locked",
                    )
                        .await
                })
            })
            .await
            .map_err(AppError::from)
    }

    async fn reserve_inventory(
        &self,
        cmd: ReserveInventoryIntentCommand,
    ) -> Result<TransferIntent, AppError> {
        self.uow
            .with_tx(move |ctx| {
                Box::pin(async move {
                    Self::apply_transition_with_lock(
                        ctx,
                        cmd.public_id,
                        cmd.now,
                        move |intent| {
                            intent.reserve_inventory(cmd.now)?;
                            Ok(IntentPatch::default())
                        },
                        TransferState::InventoryReserved,
                        "inventory reserved",
                    )
                        .await
                })
            })
            .await
            .map_err(AppError::from)
    }

    async fn mark_debit_posted(
        &self,
        cmd: MarkDebitPostedCommand,
    ) -> Result<TransferIntent, AppError> {
        self.uow
            .with_tx(move |ctx| {
                Box::pin(async move {
                    Self::apply_transition_cas_only(
                        ctx,
                        cmd.public_id,
                        cmd.now,
                        move |intent| {
                            intent.mark_debit_posted(cmd.now)?;
                            Ok(IntentPatch::default())
                        },
                        TransferState::DebitPosted,
                        "debit posted",
                    )
                        .await
                })
            })
            .await
            .map_err(AppError::from)
    }

    async fn mark_rail_submitted(
        &self,
        cmd: MarkRailSubmittedCommand,
    ) -> Result<TransferIntent, AppError> {
        self.uow
            .with_tx(move |ctx| {
                Box::pin(async move {
                    Self::apply_transition_with_lock(
                        ctx,
                        cmd.public_id,
                        cmd.now,
                        move |intent| {
                            intent.mark_rail_submitted(cmd.now)?;
                            Ok(IntentPatch::default())
                        },
                        TransferState::RailSubmitted,
                        "rail submitted",
                    )
                        .await
                })
            })
            .await
            .map_err(AppError::from)
    }

    async fn settle(&self, cmd: SettleCommand) -> Result<TransferIntent, AppError> {
        self.uow
            .with_tx(move |ctx| {
                Box::pin(async move {
                    Self::apply_transition_cas_only(
                        ctx,
                        cmd.public_id,
                        cmd.now,
                        move |intent| {
                            intent.settle(cmd.now)?;
                            Ok(IntentPatch::default())
                        },
                        TransferState::Settled,
                        "settled",
                    )
                        .await
                })
            })
            .await
            .map_err(AppError::from)
    }

    async fn fail_with_policy(
        &self,
        cmd: FailWithPolicyCommand,
    ) -> Result<TransferIntent, AppError> {
        self.uow
            .with_tx(move |ctx| {
                Box::pin(async move {
                    Self::apply_transition_cas_only(
                        ctx,
                        cmd.public_id,
                        cmd.now,
                        move |intent| {
                            let info = intent.fail_with_policy(
                                cmd.class,
                                cmd.reason,
                                cmd.message,
                                cmd.now,
                            )?;

                            Ok(IntentPatch {
                                quote: None,
                                quote_expires_at: None,
                                required_usdt_minor: None,
                                failure: Some(info),
                            })
                        },
                        TransferState::Failed,
                        "failed",
                    )
                        .await
                })
            })
            .await
            .map_err(AppError::from)
    }
}
