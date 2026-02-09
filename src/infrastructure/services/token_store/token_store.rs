use async_trait::async_trait;
use std::time::Duration;

use crate::infrastructure::error::InfraError;

#[async_trait]
pub trait TokenStore: Send + Sync {
    async fn get_valid(&self) -> Result<Option<String>, InfraError>;
    async fn set(&self, token: String, ttl: Duration) -> Result<(), InfraError>;
    async fn clear(&self) -> Result<(), InfraError>;
}
