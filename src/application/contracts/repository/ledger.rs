use async_trait::async_trait;
use sqlx::{Postgres, Transaction};

use crate::domain::aggregate::{PostedJournal, ValidatedJournal};
use crate::domain::repository::RepoError;

#[async_trait]
pub trait LedgerRepositoryTx: Send + Sync {
     async fn insert_posting_atomic_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        posting: ValidatedJournal,
    ) -> Result<PostedJournal, RepoError>;
}
