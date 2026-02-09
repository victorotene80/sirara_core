use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::application::AppError;

#[async_trait]
pub trait HttpClient: Send + Sync {
    async fn get_json<T: DeserializeOwned + Send>(
        &self,
        url: &str,
        query: &[(&str, &str)],
        bearer: Option<&str>,
    ) -> Result<T, AppError>;

    async fn post_json<B: Serialize + Send + Sync, T: DeserializeOwned + Send>(
        &self,
        url: &str,
        body: &B,
        bearer: Option<&str>,
    ) -> Result<T, AppError>;

    async fn post_json_with_headers<B: Serialize + Send + Sync, T: DeserializeOwned + Send>(
        &self,
        url: &str,
        body: &B,
        headers: &[(&str, &str)],
    ) -> Result<T, AppError>;
}
