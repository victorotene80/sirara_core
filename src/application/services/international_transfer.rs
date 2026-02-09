use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use crate::application::AppError;

use crate::application::commands::{
    HoldUserFundsCommand, LockFundsCommand, LockRateCommand, ReserveInventoryCommand,
    ReserveInventoryIntentCommand, CreateTransferIntentCommand,
};
use crate::application::commands::create_international_transfer::CreateInternationalTransferCommand;

use crate::application::contracts::{FxQuoteService, FxQuoteDetails, LedgerService, TransferService};
use crate::application::contracts::repository::TxContext;
use crate::application::contracts::international_transfer::CryptoTransferService;
use crate::domain::aggregate::TransferIntent;
use crate::domain::value_objects::{AssetCode, TransferRoute};

pub struct CryptoTransferServiceImpl {
    fx: Arc<dyn FxQuoteService>,
    transfer: Arc<dyn TransferService>,
    ledger: Arc<dyn LedgerService>,
}

impl CryptoTransferServiceImpl {
    pub fn new(
        fx: Arc<dyn FxQuoteService>,
        transfer: Arc<dyn TransferService>,
        ledger: Arc<dyn LedgerService>,
    ) -> Self {
        Self { fx, transfer, ledger }
    }

    fn required_usdt_minor_from_quote(
        quote: &crate::domain::value_objects::FxQuote,
        amount_minor: i128,
    ) -> Result<i128, AppError> {
        // Replace this with your real calculation once you expose it on FxQuote.
        // For now we enforce "must be positive" and treat it as same minor.
        if amount_minor <= 0 {
            return Err(AppError::InvalidRequest { message: "amount_minor must be > 0".into() });
        }
        Ok(amount_minor)
    }
}

#[async_trait]
impl CryptoTransferService for CryptoTransferServiceImpl {
    async fn create_crypto_transfer(
        &self,
        ctx: &mut dyn TxContext,
        cmd: CreateInternationalTransferCommand,
        now: DateTime<Utc>,
    ) -> Result<TransferIntent, AppError> {
        let CreateInternationalTransferCommand {
            public_id,
            external_ref_type,
            external_ref,
            from_user_id,
            region,
            destination,
            asset,
            amount,
            initiated_by,
        } = cmd;

        if !destination.is_onchain() {
            return Err(AppError::InvalidRequest { message: "destination must be onchain".into() });
        }

        let (from_available, from_locked) = {
            let mut ledger_tx = ctx.ledger();
            ledger_tx
                .resolve_user_hold_accounts(from_user_id, &asset)
                .await
                .map_err(AppError::from)?
        };

        let available_minor = {
            let mut ledger_tx = ctx.ledger();
            ledger_tx
                .peek_balance_minor(from_available)
                .await
                .map_err(AppError::from)?
        };

        let required_minor = amount.minor();
        if available_minor < required_minor {
            return Err(AppError::Conflict {
                message: format!(
                    "insufficient funds (available_minor={}, required_minor={})",
                    available_minor, required_minor
                ),
            });
        }

        let route = TransferRoute::crypto_transfer(region.clone(), from_user_id, destination)
            .map_err(AppError::from)?;

        let intent = self
            .transfer
            .create_intent(CreateTransferIntentCommand {
                public_id,
                external_ref_type,
                external_ref: external_ref.clone(),
                route,
                asset: asset.clone(),
                amount: amount.clone(),
                now,
            })
            .await?;

        let quote = self
            .fx
            .get_firm_quote(FxQuoteDetails {
                from: asset.as_str().to_string(),
                to: "USDT".to_string(),
                via: vec![],
                amount_minor: required_minor,
            })
            .await?;

        let required_usdt_minor = Self::required_usdt_minor_from_quote(&quote, required_minor)?;

        let intent = self
            .transfer
            .lock_rate(LockRateCommand {
                public_id: intent.public_id().clone(),
                quote: quote.clone(),
                required_usdt_minor,
                now,
            })
            .await?;

        self.ledger
            .reserve_inventory(ReserveInventoryCommand {
                journal_public_id: intent.public_id().clone(),
                external_ref: intent.external_ref().clone(),
                created_by: initiated_by.clone().unwrap_or_else(|| "system".into()),
                asset_code: AssetCode::new("USDT".to_string()).map_err(AppError::from)?,
                region_code: region.clone(),
                description: Some("inventory reserve".to_string()),
                amount_minor: required_usdt_minor,
            })
            .await?;

        let intent = self
            .transfer
            .reserve_inventory(ReserveInventoryIntentCommand {
                public_id: intent.public_id().clone(),
                now,
            })
            .await?;

        self.ledger
            .hold_user_funds(HoldUserFundsCommand {
                journal_public_id: intent.public_id().clone(),
                external_ref: intent.external_ref().clone(),
                created_by: initiated_by.clone().unwrap_or_else(|| "system".into()),
                user_id: from_user_id,
                asset_code: asset,
                amount_minor: required_minor,
                description: Some("user funds hold".to_string()),
            })
            .await?;

        let intent = self
            .transfer
            .lock_funds(LockFundsCommand {
                public_id: intent.public_id().clone(),
                now,
            })
            .await?;

        Ok(intent)
    }
}
