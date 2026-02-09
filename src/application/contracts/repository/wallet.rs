use chrono::{DateTime, Utc};

use crate::application::contracts::repository::BoxFut;
use crate::domain::entities::HotWallet;
use crate::domain::repository::RepoError;
use crate::domain::value_objects::{AssetCode, Chain, PublicId, RegionCode};

pub trait HotWalletRepositoryTx: Send {
    fn insert_if_absent(&mut self, wallet: HotWallet) -> BoxFut<'_, Result<HotWallet, RepoError>>;

    fn update_active_flag(
        &mut self,
        wallet_public_id: PublicId,
        is_active: bool,
        expected_version: i32,
        now: DateTime<Utc>,
    ) -> BoxFut<'_, Result<i32, RepoError>>;

    fn find_active_by_chain_asset_region(
        &mut self,
        chain: &Chain,
        asset_code: &AssetCode,
        region_code: &RegionCode,
    ) -> BoxFut<'_, Result<Option<HotWallet>, RepoError>>;
}
