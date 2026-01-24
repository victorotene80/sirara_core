use sqlx::{Postgres, Transaction};

use crate::application::contracts::repository::{LedgerRepositoryTx, ReposInTx};
use crate::infrastructure::persistence::{PgLedgerTxRepo};

pub struct PgReposInTx<'a, 'c> {
    tx: &'a mut Transaction<'c, Postgres>,
}

impl<'a, 'c> PgReposInTx<'a, 'c> {
    pub fn new(tx: &'a mut Transaction<'c, Postgres>) -> Self {
        Self { tx }
    }
}

impl<'a, 'c> ReposInTx for PgReposInTx<'a, 'c> {
    fn ledger<'s>(&'s mut self) -> Box<dyn LedgerRepositoryTx + 's> {
        Box::new(PgLedgerTxRepo::new(&mut *self.tx))
    }

}
