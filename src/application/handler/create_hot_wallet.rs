use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::application::AppError;
use crate::application::commands::CreateHotWalletCommand;
use crate::application::contracts::chain_::ChainGateway;
use crate::application::contracts::repository::UnitOfWork;
use crate::domain::entities::HotWallet;
use crate::domain::value_objects::PublicId;

pub struct CreateHotWalletHandler<U: UnitOfWork> {
    pub uow: U,
    pub chain_gateway: std::sync::Arc<dyn ChainGateway>,
}

impl<U: UnitOfWork> CreateHotWalletHandler<U> {
    pub async fn handle(&self, cmd: CreateHotWalletCommand) -> Result<HotWallet, AppError> {
        let now: DateTime<Utc> = cmd.now;

        let created = self.chain_gateway.create_wallet().await?;

        self.chain_gateway
            .ensure_token_account(&created.address, &cmd.asset_code)
            .await?;

        let wallet = HotWallet::new(
            PublicId::new(),
            cmd.chain,
            cmd.asset_code,
            cmd.region_code,
            created.address,
            created.secret_handle.0,
            cmd.max_balance_minor,
        ).map_err(AppError::from)?;

        let saved = self.uow
            .with_tx(|mut tx| Box::pin(async move {
                tx.wallet().insert_if_absent(wallet).await
            }))
            .await?;

        Ok(saved)
    }
}
