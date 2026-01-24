use async_trait::async_trait;
use crate::application::AppError;

#[async_trait]
pub trait InventoryService: Send + Sync {
    async fn reserve_usdt(&self, intent_public_id: uuid::Uuid, amount_minor: i128) -> Result<(), AppError>;
    async fn release_usdt(&self, intent_public_id: uuid::Uuid) -> Result<(), AppError>;
    async fn consume_usdt(&self, intent_public_id: uuid::Uuid) -> Result<(), AppError>;
}
