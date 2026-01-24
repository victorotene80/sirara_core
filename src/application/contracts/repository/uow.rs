use std::future::Future;
use std::pin::Pin;

use crate::application::AppError;
use crate::application::contracts::repository::LedgerRepositoryTx;

pub type BoxFut<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub trait ReposInTx: Send {
    fn ledger<'a>(&'a mut self) -> Box<dyn LedgerRepositoryTx + 'a>;
}

pub trait UnitOfWork: Send + Sync {
    fn with_tx<'a, T: Send + 'a>(
        &'a self,
        f: impl for<'tx> FnOnce(&'tx mut dyn ReposInTx) -> BoxFut<'tx, Result<T, AppError>>
        + Send
        + 'a,
    ) -> BoxFut<'a, Result<T, AppError>>;
}
