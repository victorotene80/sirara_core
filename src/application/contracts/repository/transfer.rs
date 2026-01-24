use crate::application::contracts::repository::BoxFut;
use crate::domain::aggregate::TransferIntent;
use crate::domain::value_objects::{ExternalRef, ExternalRefType, PublicId, TransferState};
use crate::domain::repository::RepoError;

pub trait TransferRepositoryTx: Send {
    fn insert_intent_if_absent(
        &mut self,
        intent: TransferIntent,
    ) -> BoxFut<'_, Result<TransferIntent, RepoError>>;

    fn get_intent_for_update_by_public_id(
        &mut self,
        public_id: PublicId,
    ) -> BoxFut<'_, Result<TransferIntent, RepoError>>;

    fn append_transition(
        &mut self,
        intent_db_id: i64,
        from: TransferState,
        to: TransferState,
        reason: String,
        data_json: Option<serde_json::Value>,
    ) -> BoxFut<'_, Result<(), RepoError>>;

    fn update_intent_state_cas(
        &mut self,
        intent_db_id: i64,
        expected_version: i32,
        new_state: TransferState,
        new_version: i32,
        patch: IntentPatch,
    ) -> BoxFut<'_, Result<(), RepoError>>;

    fn find_by_external_ref(
        &mut self,
        t: ExternalRefType,
        r: &ExternalRef,
    ) -> BoxFut<'_, Result<Option<TransferIntent>, RepoError>>;
}

#[derive(Debug, Clone, Default)]
pub struct IntentPatch {
    pub quote_json: Option<serde_json::Value>,
    pub quote_expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub required_usdt_minor: Option<i128>,
    pub tx_hash: Option<String>,
    pub failure_json: Option<serde_json::Value>,
}
