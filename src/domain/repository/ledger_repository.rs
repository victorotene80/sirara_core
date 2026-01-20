use crate::domain::aggregate::PostedJournal;
//use crate::domain::services::PolicyValidatedJournal;
use crate::domain::entities::{LedgerAccount, AccountType, OwnerType};
use crate::domain::value_objects::{PublicId, ExternalRef, ExternalRefType};
use crate::domain::repository::error::RepoError;

#[derive(Debug, Clone)]
pub struct NewLedgerAccountSpec {
    pub public_id: PublicId,
    pub owner_type: OwnerType,
    pub owner_id: Option<uuid::Uuid>,
    pub account_type: AccountType,
    pub asset_id: i16,
    pub is_active: bool,
}

pub trait LedgerRepository {
    fn create_account(&self, spec: NewLedgerAccountSpec)
                      -> Result<LedgerAccount, RepoError>;

    fn set_account_active(&self, account_id: i64, active: bool)
                          -> Result<(), RepoError>;

    fn get_accounts_by_ids(&self, ids: &[i64])
                           -> Result<Vec<LedgerAccount>, RepoError>;

    fn find_posted_by_external_ref(
        &self,
        external_ref_type: ExternalRefType,
        external_ref: &ExternalRef,
    ) -> Result<Option<PostedJournal>, RepoError>;

    /*fn insert_posting_atomic(
        &self,
        posting: PolicyValidatedJournal,
    ) -> Result<PostedJournal, RepoError>;*/
}
