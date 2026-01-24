use async_trait::async_trait;
use crate::application::AppError;

#[derive(Debug, Clone)]
pub struct SubmitResult {
    pub reference: String, // tx hash or bank ref
}

#[derive(Debug, Clone)]
pub enum CheckStatus {
    Confirmed,
    Failed(String),
    Pending,
}

#[async_trait]
pub trait TransferRail: Send + Sync {
    async fn submit(&self, intent_public_id: uuid::Uuid, idempotency_key: String) -> Result<SubmitResult, AppError>;
    async fn check(&self, reference: String) -> Result<CheckStatus, AppError>;
}
