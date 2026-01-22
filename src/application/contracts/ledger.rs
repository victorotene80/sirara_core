use async_trait::async_trait;
use crate::application::dtos::{CreateAccountDTO, LedgerAccountDTO, PostJournalRequestDTO, PostedJournalDTO};
use crate::application::AppError;
#[async_trait]
pub trait LedgerService: Send + Sync {
    async fn create_account(&self, req: CreateAccountDTO) -> Result<LedgerAccountDTO, AppError>;

    async fn post_journal_atomic(&self, req: PostJournalRequestDTO) -> Result<PostedJournalDTO, AppError>;

    async fn find_posted_by_external_ref(
        &self,
        external_ref_type: String,
        external_ref: String,
    ) -> Result<Option<PostedJournalDTO>, AppError>;
}