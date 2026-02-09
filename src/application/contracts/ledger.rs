use async_trait::async_trait;

use crate::application::AppError;
use crate::application::commands::{
    HoldUserFundsCommand,
    ReleaseUserFundsCommand,
    ReserveInventoryCommand,
    ReleaseInventoryCommand,
};
use crate::domain::aggregate::PostedJournal;

#[async_trait]
pub trait LedgerService: Send + Sync {
    async fn hold_user_funds(
        &self,
        cmd: HoldUserFundsCommand,
    ) -> Result<PostedJournal, AppError>;

    async fn release_user_funds(
        &self,
        cmd: ReleaseUserFundsCommand,
    ) -> Result<PostedJournal, AppError>;

    async fn reserve_inventory(
        &self,
        cmd: ReserveInventoryCommand,
    ) -> Result<PostedJournal, AppError>;

    async fn release_inventory(
        &self,
        cmd: ReleaseInventoryCommand,
    ) -> Result<PostedJournal, AppError>;

    async fn peek_balance_minor(&self, account_id: i64) -> Result<i128, AppError>;
}
