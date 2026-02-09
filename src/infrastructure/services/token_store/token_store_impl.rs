use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use crate::infrastructure::error::InfraError;
use super::TokenStore;

#[derive(Debug, Clone)]
struct CachedToken {
    token: String,
    expires_at: Instant,
}

pub struct InMemoryTokenStore {
    inner: RwLock<Option<CachedToken>>,
}

impl InMemoryTokenStore {
    pub fn new() -> Self {
        Self { inner: RwLock::new(None) }
    }
}

#[async_trait::async_trait]
impl TokenStore for InMemoryTokenStore {
    async fn get_valid(&self) -> Result<Option<String>, InfraError> {
        let guard = self.inner.read().await;
        if let Some(t) = guard.as_ref() {
            if Instant::now() < t.expires_at {
                return Ok(Some(t.token.clone()));
            }
        }
        Ok(None)
    }

    async fn set(&self, token: String, ttl: Duration) -> Result<(), InfraError> {
        let mut guard = self.inner.write().await;
        *guard = Some(CachedToken {
            token,
            expires_at: Instant::now() + ttl,
        });
        Ok(())
    }

    async fn clear(&self) -> Result<(), InfraError> {
        let mut guard = self.inner.write().await;
        *guard = None;
        Ok(())
    }
}
