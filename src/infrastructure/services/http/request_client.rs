use async_trait::async_trait;
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::application::AppError;
use super::client::HttpClient;

pub struct RequestHttpClient {
    client: Client,
}

impl RequestHttpClient {
    pub fn new() -> Self {
        Self { client: Client::new() }
    }
}

#[async_trait]
impl HttpClient for RequestHttpClient {
    async fn get_json<T: DeserializeOwned + Send>(
        &self,
        url: &str,
        query: &[(&str, &str)],
        bearer: Option<&str>,
    ) -> Result<T, AppError> {
        let mut req = self.client.get(url).query(query);
        if let Some(token) = bearer {
            req = req.bearer_auth(token);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| AppError::Unexpected(format!("http get error: {e}")))?;

        if !resp.status().is_success() {
            return Err(AppError::Unexpected(format!("http status: {}", resp.status())));
        }

        resp.json::<T>()
            .await
            .map_err(|e| AppError::Unexpected(format!("bad json: {e}")))
    }

    async fn post_json<B: Serialize + Send + Sync, T: DeserializeOwned + Send>(
        &self,
        url: &str,
        body: &B,
        bearer: Option<&str>,
    ) -> Result<T, AppError> {
        let mut req = self.client.post(url).json(body);
        if let Some(token) = bearer {
            req = req.bearer_auth(token);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| AppError::Unexpected(format!("http post error: {e}")))?;

        if !resp.status().is_success() {
            return Err(AppError::Unexpected(format!("http status: {}", resp.status())));
        }

        resp.json::<T>()
            .await
            .map_err(|e| AppError::Unexpected(format!("bad json: {e}")))
    }

    async fn post_json_with_headers<B: Serialize + Send + Sync, T: DeserializeOwned + Send>(
        &self,
        url: &str,
        body: &B,
        headers: &[(&str, &str)],
    ) -> Result<T, AppError> {
        let mut req = self.client.post(url).json(body);

        for (k, v) in headers {
            req = req.header(*k, *v);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| AppError::Unexpected(format!("http post error: {e}")))?;

        if !resp.status().is_success() {
            return Err(AppError::Unexpected(format!("http status: {}", resp.status())));
        }

        resp.json::<T>()
            .await
            .map_err(|e| AppError::Unexpected(format!("bad json: {e}")))
    }
}
