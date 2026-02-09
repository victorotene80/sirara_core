use async_trait::async_trait;

use crate::application::AppError;
use crate::application::commands::*;
use crate::domain::aggregate::TransferIntent;

#[async_trait]
pub trait TransferService: Send + Sync {
    async fn create_intent(&self, cmd: CreateTransferIntentCommand) -> Result<TransferIntent, AppError>;
    async fn lock_rate(&self, cmd: LockRateCommand) -> Result<TransferIntent, AppError>;
    async fn lock_funds(&self, cmd: LockFundsCommand) -> Result<TransferIntent, AppError>;
    async fn reserve_inventory(&self, cmd: ReserveInventoryIntentCommand) -> Result<TransferIntent, AppError>;
    async fn mark_debit_posted(&self, cmd: MarkDebitPostedCommand) -> Result<TransferIntent, AppError>;
    async fn mark_rail_submitted(&self, cmd: MarkRailSubmittedCommand) -> Result<TransferIntent, AppError>;
    async fn settle(&self, cmd: SettleCommand) -> Result<TransferIntent, AppError>;
    async fn fail_with_policy(&self, cmd: FailWithPolicyCommand) -> Result<TransferIntent, AppError>;
}
