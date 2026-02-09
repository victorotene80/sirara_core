use std::future::Future;
use std::pin::Pin;

use crate::domain::repository::RepoError;
use crate::application::contracts::repository::{
    LedgerRepositoryTx, OutboxRepositoryTx, TransferRepositoryTx, OnchainRepositoryTx,
    HotWalletRepositoryTx,
};

pub type BoxFut<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub trait TxContext: Send {
    fn ledger<'a>(&'a mut self) -> Box<dyn LedgerRepositoryTx + 'a>;
    fn transfer<'a>(&'a mut self) -> Box<dyn TransferRepositoryTx + 'a>;
    fn outbox<'a>(&'a mut self) -> Box<dyn OutboxRepositoryTx + 'a>;
    fn onchain<'a>(&'a mut self) -> Box<dyn OnchainRepositoryTx + 'a>;
    fn wallet<'a>(&'a mut self) -> Box<dyn HotWalletRepositoryTx + 'a>;
}

impl dyn TxContext + '_ {
    pub fn with_ledger<'tx, T: Send + 'tx>(
        &'tx mut self,
        f: impl for<'r> FnOnce(&'r mut (dyn LedgerRepositoryTx + 'r))
            -> BoxFut<'r, Result<T, RepoError>>
        + Send
        + 'tx,
    ) -> BoxFut<'tx, Result<T, RepoError>> {
        Box::pin(async move {
            let mut repo = self.ledger();
            f(&mut *repo).await
        })
    }

    pub fn with_transfer<'tx, T: Send + 'tx>(
        &'tx mut self,
        f: impl for<'r> FnOnce(&'r mut (dyn TransferRepositoryTx + 'r))
            -> BoxFut<'r, Result<T, RepoError>>
        + Send
        + 'tx,
    ) -> BoxFut<'tx, Result<T, RepoError>> {
        Box::pin(async move {
            let mut repo = self.transfer();
            f(&mut *repo).await
        })
    }

    pub fn with_outbox<'tx, T: Send + 'tx>(
        &'tx mut self,
        f: impl for<'r> FnOnce(&'r mut (dyn OutboxRepositoryTx + 'r))
            -> BoxFut<'r, Result<T, RepoError>>
        + Send
        + 'tx,
    ) -> BoxFut<'tx, Result<T, RepoError>> {
        Box::pin(async move {
            let mut repo = self.outbox();
            f(&mut *repo).await
        })
    }

    pub fn with_onchain<'tx, T: Send + 'tx>(
        &'tx mut self,
        f: impl for<'r> FnOnce(&'r mut (dyn OnchainRepositoryTx + 'r))
            -> BoxFut<'r, Result<T, RepoError>>
        + Send
        + 'tx,
    ) -> BoxFut<'tx, Result<T, RepoError>> {
        Box::pin(async move {
            let mut repo = self.onchain();
            f(&mut *repo).await
        })
    }

    pub fn with_wallet<'tx, T: Send + 'tx>(
        &'tx mut self,
        f: impl for<'r> FnOnce(&'r mut (dyn HotWalletRepositoryTx + 'r))
            -> BoxFut<'r, Result<T, RepoError>>
        + Send
        + 'tx,
    ) -> BoxFut<'tx, Result<T, RepoError>> {
        Box::pin(async move {
            let mut repo = self.wallet();
            f(&mut *repo).await
        })
    }
}
