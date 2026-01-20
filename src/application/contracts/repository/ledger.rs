use crate::domain::repository::RepoError;
use crate::domain::value_objects::{PublicId, ExternalRef, ExternalRefType};
use crate::domain::entities::OwnerType;
use crate::application::dtos::{
    LedgerAccountDTO,
    JournalDTO,
    AccountBalanceDTO,
    OwnerBalancesDTO,
    ListJournalsFilterDTO,
};

pub trait LedgerQueryRepository {
    fn get_account_by_public_id(
        &self,
        public_id: PublicId,
    ) -> Result<Option<LedgerAccountDTO>, RepoError>;

    fn list_accounts_by_owner(
        &self,
        owner_type: OwnerType,
        owner_id: Option<uuid::Uuid>,
    ) -> Result<Vec<LedgerAccountDTO>, RepoError>;

    fn get_journal_by_public_id(
        &self,
        public_id: PublicId,
    ) -> Result<Option<JournalDTO>, RepoError>;

    fn get_journal_by_external_ref(
        &self,
        external_ref_type: ExternalRefType,
        external_ref: &ExternalRef,
    ) -> Result<Option<JournalDTO>, RepoError>;

    fn list_journals(
        &self,
        filter: ListJournalsFilterDTO,
    ) -> Result<Vec<JournalDTO>, RepoError>;

    fn get_account_balance(
        &self,
        account_id: i64,
    ) -> Result<AccountBalanceDTO, RepoError>;

    fn get_owner_balances(
        &self,
        owner_type: OwnerType,
        owner_id: Option<uuid::Uuid>,
    ) -> Result<OwnerBalancesDTO, RepoError>;
}
