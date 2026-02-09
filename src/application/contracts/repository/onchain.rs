use chrono::{DateTime, Utc};

use crate::application::contracts::repository::BoxFut;
use crate::domain::entities::OnchainTransaction;
use crate::domain::repository::RepoError;
use crate::domain::value_objects::{FailureInfo, OnchainTxStatus, PublicId, TxHash};

pub trait OnchainRepositoryTx: Send {
    fn insert_if_absent(
        &mut self,
        tx: OnchainTransaction,
    ) -> BoxFut<'_, Result<OnchainTransaction, RepoError>>;

    fn find_by_intent_id_for_update(
        &mut self,
        intent_db_id: i64,
    ) -> BoxFut<'_, Result<Option<OnchainTransaction>, RepoError>>;

    fn find_by_public_id_for_update(
        &mut self,
        public_id: PublicId,
    ) -> BoxFut<'_, Result<Option<OnchainTransaction>, RepoError>>;

    fn mark_submitted(
        &mut self,
        intent_db_id: i64,
        expected_version: i32,
        tx_hash: TxHash,
        now: DateTime<Utc>,
    ) -> BoxFut<'_, Result<i32, RepoError>>;

    fn update_confirmations(
        &mut self,
        intent_db_id: i64,
        expected_version: i32,
        confirmations: i32,
        now: DateTime<Utc>,
    ) -> BoxFut<'_, Result<i32, RepoError>>;

    fn mark_confirmed(
        &mut self,
        intent_db_id: i64,
        expected_version: i32,
        now: DateTime<Utc>,
    ) -> BoxFut<'_, Result<i32, RepoError>>;

    fn mark_failed(
        &mut self,
        intent_db_id: i64,
        expected_version: i32,
        failure: FailureInfo,
        now: DateTime<Utc>,
    ) -> BoxFut<'_, Result<i32, RepoError>>;

    fn list_submitted_due_for_check(
        &mut self,
        limit: i64,
        now: DateTime<Utc>,
    ) -> BoxFut<'_, Result<Vec<OnchainTransaction>, RepoError>>;

    fn list_by_status(
        &mut self,
        status: OnchainTxStatus,
        limit: i64,
    ) -> BoxFut<'_, Result<Vec<OnchainTransaction>, RepoError>>;
}
