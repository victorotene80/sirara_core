use crate::application::contracts::repository::BoxFut;
use crate::domain::aggregate::{PostedJournal, ValidatedJournal};
use crate::domain::repository::RepoError;
use crate::domain::entities::LedgerAccount;
use uuid::Uuid;

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
        asset_code: String,
    ) -> BoxFut<'_, Result<(i64, i64), RepoError>>; 

}
