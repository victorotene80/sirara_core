use async_trait::async_trait;

use crate::application::AppError;
use crate::application::contracts::chain::SecretHandle;
use crate::application::contracts::repository::TxContext;
use crate::domain::entities::HotWallet;
use crate::domain::value_objects::{AssetCode, Chain, PublicId, RegionCode};

#[async_trait]
pub trait HotWalletService: Send + Sync {
    async fn create_hot_wallet(
        &self,
        ctx: &mut dyn TxContext,
        public_id: PublicId,
        chain: Chain,
        asset: AssetCode,
        region: RegionCode,
        max_balance_minor: i128,
        is_active: bool,
    ) -> Result<(HotWallet, SecretHandle), AppError>;
}
