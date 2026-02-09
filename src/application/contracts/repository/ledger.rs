use uuid::Uuid;

use crate::application::contracts::repository::BoxFut;
use crate::domain::aggregate::{PostedJournal, ValidatedJournal};
use crate::domain::entities::LedgerAccount;
use crate::domain::repository::RepoError;
use crate::domain::value_objects::{AssetCode, RegionCode};

pub trait LedgerRepositoryTx: Send {
    fn insert_posting_atomic(
        &mut self,
        posting: ValidatedJournal,
    ) -> BoxFut<'_, Result<PostedJournal, RepoError>>;

    fn get_accounts_by_ids_for_validation(
        &mut self,
        ids: &[i64],
    ) -> BoxFut<'_, Result<Vec<LedgerAccount>, RepoError>>;

    fn resolve_user_hold_accounts(
        &mut self,
        user_id: Uuid,
        asset_code: &AssetCode,
    ) -> BoxFut<'_, Result<(i64, i64), RepoError>>;

    fn resolve_platform_inventory_accounts(
        &mut self,
        asset_code: &AssetCode,
        region_code: &RegionCode,
    ) -> BoxFut<'_, Result<(i64, i64), RepoError>>;

    fn resolve_user_available_accounts(
        &mut self,
        from_user: Uuid,
        to_user: Uuid,
        asset_code: &AssetCode,
    ) -> BoxFut<'_, Result<(LedgerAccount, LedgerAccount), RepoError>>;

    fn peek_balance_minor(&mut self, account_id: i64) -> BoxFut<'_, Result<i128, RepoError>>;

}
