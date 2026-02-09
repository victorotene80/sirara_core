use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::application::AppError;
use crate::application::commands::create_international_transfer::CreateInternationalTransferCommand;
use crate::application::contracts::repository::TxContext;
use crate::domain::aggregate::TransferIntent;

#[async_trait]
pub trait CryptoTransferService: Send + Sync {
    async fn create_crypto_transfer(
        &self,
        ctx: &mut dyn TxContext,
        cmd: CreateInternationalTransferCommand,
        now: DateTime<Utc>,
    ) -> Result<TransferIntent, AppError>;
}
