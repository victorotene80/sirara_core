use crate::application::contracts::repository::BoxFut;
use crate::domain::aggregate::TransferIntent;
use crate::domain::repository::RepoError;
use crate::domain::value_objects::{
    ExternalRef, ExternalRefType, PublicId, TransferState, FxQuote, FailureInfo,
};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct InsertIntentResult {
    pub intent: TransferIntent,
    pub inserted: bool,
}

#[derive(Debug, Clone, Default)]
pub struct IntentPatch {
    pub quote: Option<FxQuote>,
    pub quote_expires_at: Option<DateTime<Utc>>,
    pub required_usdt_minor: Option<i128>,
    pub failure: Option<FailureInfo>,
}

pub trait TransferRepositoryTx: Send {
    fn insert_intent_if_absent(
        &mut self,
        intent: TransferIntent,
    ) -> BoxFut<'_, Result<InsertIntentResult, RepoError>>;

    fn get_intent_by_public_id(
        &mut self,
        public_id: PublicId,
    ) -> BoxFut<'_, Result<TransferIntent, RepoError>>;

    fn get_intent_for_update_by_public_id(
        &mut self,
        public_id: PublicId,
    ) -> BoxFut<'_, Result<TransferIntent, RepoError>>;

    fn append_transition_if_current(
        &mut self,
        intent_db_id: i64,
        expected_version: i32,
        expected_from: TransferState,
        to: TransferState,
        reason: String,
        data_json: Option<serde_json::Value>,
    ) -> BoxFut<'_, Result<bool, RepoError>>;

    fn update_intent_state_cas(
        &mut self,
        intent_db_id: i64,
        expected_version: i32,
        expected_state: TransferState,
        new_state: TransferState,
        patch: IntentPatch,
    ) -> BoxFut<'_, Result<(), RepoError>>;

    fn find_by_external_ref(
        &mut self,
        t: ExternalRefType,
        r: &ExternalRef,
    ) -> BoxFut<'_, Result<Option<TransferIntent>, RepoError>>;
}
