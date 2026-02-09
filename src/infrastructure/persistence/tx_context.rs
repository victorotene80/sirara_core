use sqlx::{Postgres, Transaction};

use crate::application::contracts::repository::{LedgerRepositoryTx, OutboxRepositoryTx, TransferRepositoryTx, TxContext, OnchainRepositoryTx, HotWalletRepositoryTx};
use crate::infrastructure::persistence::{PgLedgerTxRepo, PgOutboxTxRepo, PgTransferTxRepo, PgOnchainTxRepo, PgHotWalletTxRepo};

pub struct PgTxContext<'a, 'c> {
    tx: &'a mut Transaction<'c, Postgres>,
}

impl<'a, 'c> PgTxContext<'a, 'c> {
    pub fn new(tx: &'a mut Transaction<'c, Postgres>) -> Self {
        Self { tx }
    }
}

impl<'a, 'c> TxContext for PgTxContext<'a, 'c> {
    fn ledger<'s>(&'s mut self) -> Box<dyn LedgerRepositoryTx + 's> {
        Box::new(PgLedgerTxRepo::new(&mut *self.tx))
    }

    fn transfer<'s>(&'s mut self) -> Box<dyn TransferRepositoryTx + 's> {
        Box::new(PgTransferTxRepo::new(&mut *self.tx))
    }

    fn outbox<'s>(&'s mut self) -> Box<dyn OutboxRepositoryTx + 's> {
        Box::new(PgOutboxTxRepo::new(&mut *self.tx))
    }

    fn onchain<'s>(&'s mut self) -> Box<dyn OnchainRepositoryTx + 's> {
        Box::new(PgOnchainTxRepo::new(&mut *self.tx))
    }

    fn wallet<'s>(&'s mut self) -> Box<dyn HotWalletRepositoryTx + 's> {
        Box::new(PgHotWalletTxRepo::new(&mut *self.tx))
    }
}
