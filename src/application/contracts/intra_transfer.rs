use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::application::AppError;
use crate::application::commands::create_internal_transfer::CreateInternalTransferCommand;
use crate::application::contracts::repository::TxContext;
use crate::application::dtos::transfer_intent_result::InternalTransferResult;

#[async_trait]
pub trait InternalTransferService: Send + Sync {
    async fn create_internal_transfer(
        &self,
        ctx: &mut dyn TxContext,
        cmd: CreateInternalTransferCommand,
        now: DateTime<Utc>,
    ) -> Result<InternalTransferResult, AppError>;
}
