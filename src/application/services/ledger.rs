use async_trait::async_trait;

use crate::application::contracts::LedgerService;
use crate::application::contracts::repository::{LedgerRepositoryTx, UnitOfWork};
use crate::application::dtos::{
    CreateAccountDTO, LedgerAccountDTO,
    PostJournalRequestDTO, PostedJournalDTO,
};
use crate::application::dtos::mappers::{
    map_create_account_to_spec, map_account_to_dto,
    map_post_journal_request, posted_to_dto,
};
use crate::application::AppError;

use crate::domain::repository::LedgerRepository;

pub struct LedgerServiceImpl<R: LedgerRepository, RX: LedgerRepositoryTx, U: UnitOfWork> {
    repo: R,
    repo_tx: RX,
    uow: U,
}

impl<R, RX, U> LedgerServiceImpl<R, RX, U>
where
    R: LedgerRepository + Send + Sync,
    RX: LedgerRepositoryTx + Send + Sync,
    U: UnitOfWork + Send + Sync,
{
    pub fn new(repo: R, repo_tx: RX, uow: U) -> Self {
        Self { repo, repo_tx, uow }
    }
}

#[async_trait]
impl<R, RX, U> LedgerService for LedgerServiceImpl<R, RX, U>
where
    R: LedgerRepository + Send + Sync,
    RX: LedgerRepositoryTx + Send + Sync,
    U: UnitOfWork + Send + Sync,
{
    async fn create_account(&self, req: CreateAccountDTO) -> Result<LedgerAccountDTO, AppError> {
        let spec = map_create_account_to_spec(req)?;
        let account = self.repo.create_account(spec).await?;
        Ok(map_account_to_dto(&account))
    }

    async fn post_journal_atomic(
        &self,
        req: PostJournalRequestDTO
    ) -> Result<PostedJournalDTO, AppError> {
        let draft = map_post_journal_request(req)?;

        let result = self.uow.with_tx(|tx| {
            Box::pin(async move {
                // TODO:
                // 1. Load accounts using repo inside tx
                // 2. Validate draft.validate_with_accounts()
                // 3. Call repo_tx.insert_posting_atomic_tx(tx, validated)
                // 4. Return PostedJournal

                todo!("Implement domain posting flow");
            })
        }).await?;

        let dto = posted_to_dto(&result);
        Ok(dto)
    }

    async fn find_posted_by_external_ref(
        &self,
        external_ref_type: String,
        external_ref: String
    ) -> Result<Option<PostedJournalDTO>, AppError> {
        let ext_type = crate::domain::value_objects::ExternalRefType::from_code(&external_ref_type)?;
        let ext_ref = crate::domain::value_objects::ExternalRef::new(external_ref)?;

        if let Some(j) = self.repo.find_posted_by_external_ref(ext_type, &ext_ref).await? {
            return Ok(Some(posted_to_dto(&j)));
        }

        Ok(None)
    }
}
