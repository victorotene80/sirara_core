use crate::application::contracts::repository::BoxFut;
use crate::domain::repository::RepoError;

#[derive(Debug, Clone)]
pub struct OutboxMessage {
    pub id: i64,
    pub topic: String,
    pub key: String,
    pub payload_json: serde_json::Value,
    pub attempts: i32,
}

pub trait OutboxRepositoryTx: Send {
    fn enqueue(
        &mut self,
        topic: &str,
        key: &str,
        payload: serde_json::Value,
    ) -> BoxFut<'_, Result<(), RepoError>>;

    fn claim_due(
        &mut self,
        batch: i64,
    ) -> BoxFut<'_, Result<Vec<OutboxMessage>, RepoError>>;

    fn mark_sent(&mut self, id: i64) -> BoxFut<'_, Result<(), RepoError>>;

    fn mark_failed_retry(
        &mut self,
        id: i64,
        err: String,
        next_attempt_at: chrono::DateTime<chrono::Utc>,
    ) -> BoxFut<'_, Result<(), RepoError>>;
}
