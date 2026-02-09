use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::application::AppError;
use crate::application::commands::HoldUserFundsCommand;
use crate::application::commands::create_crypto_transfer::CreateInterCryptoTransferCommand;
use crate::application::contracts::LedgerService;
use crate::application::contracts::repository::TxContext;
use crate::application::dtos::transfer_intent_result::InternalTransferResult;

use crate::domain::aggregate::TransferIntent;
use crate::domain::value_objects::{Destination, TransferRoute};

#[async_trait]
pub trait InterCryptoTransferService: Send + Sync {
    async fn create_inter_crypto_transfer(
        &self,
        ctx: &mut dyn TxContext,
        cmd: CreateInterCryptoTransferCommand,
        now: DateTime<Utc>,
    ) -> Result<InternalTransferResult, AppError>;
}

pub struct InterCryptoTransferServiceImpl<L: LedgerService> {
    ledger: L,
}

impl<L: LedgerService> InterCryptoTransferServiceImpl<L> {
    pub fn new(ledger: L) -> Self {
        Self { ledger }
    }
}

#[async_trait]
impl<L> InterCryptoTransferService for InterCryptoTransferServiceImpl<L>
where
    L: LedgerService + Send + Sync,
{
    async fn create_inter_crypto_transfer(
        &self,
        ctx: &mut dyn TxContext,
        cmd: CreateInterCryptoTransferCommand,
        now: DateTime<Utc>,
    ) -> Result<InternalTransferResult, AppError> {
        let CreateInterCryptoTransferCommand {
            public_id,
            external_ref_type,
            external_ref,
            from_user_id,
            region,
            chain,
            to_address,
            memo,
            asset,
            amount,
            initiated_by,
        } = cmd;

        let (from_available, _from_locked) = {
            let mut ledger_tx = ctx.ledger();
            ledger_tx
                .resolve_user_hold_accounts(from_user_id, &asset)
                .await
                .map_err(AppError::from)?
        };

        let available_minor = self
            .ledger
            .peek_balance_minor(from_available)
            .await?;

        let required_minor = amount.minor();

        if available_minor < required_minor {
            return Ok(InternalTransferResult::RejectedInsufficientFunds {
                available_minor,
                required_minor,
            });
        }

        let destination = Destination::onchain(chain, to_address, memo).map_err(AppError::from)?;

        let route = TransferRoute::crypto_transfer(region.clone(), from_user_id, destination)
            .map_err(AppError::from)?;

        let intent = {
            let mut repo = ctx.transfer();

            let intent = TransferIntent::new(
                public_id,
                external_ref_type,
                external_ref,
                route,
                asset.clone(),
                amount.clone(),
                now,
            )?;

            repo.insert_intent_if_absent(intent)
                .await
                .map_err(AppError::from)?
                .intent
        };

        let hold_cmd = HoldUserFundsCommand {
            journal_public_id: intent.public_id(),
            external_ref: intent.external_ref().clone(),
            created_by: initiated_by.clone().unwrap_or_else(|| "system".into()),
            user_id: from_user_id,
            asset_code: asset,
            amount_minor: required_minor,
            description: Some("inter crypto transfer hold".to_string()),
        };

        self.ledger.hold_user_funds(hold_cmd).await?;

        Ok(InternalTransferResult::Success { intent })
    }
}
