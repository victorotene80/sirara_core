use std::sync::Arc;

use async_trait::async_trait;

use crate::application::AppError;
use crate::infrastructure::services::fx_quote::fx_rate_gateway::FxRatesGateway;
use crate::infrastructure::services::http::HttpClient;
use crate::infrastructure::services::response::fx_rate::FxRatesResponse;
use crate::utils::configuration::Config;

pub struct ForexRateApiGateway<C: HttpClient> {
    http: Arc<C>,
    base_url: String,
    api_key: String,
}

impl<C: HttpClient> ForexRateApiGateway<C> {
    pub fn from_config(http: Arc<C>, cfg: &Config) -> Result<Self, AppError> {
        let base_url = cfg.fx.base_url.trim_end_matches('/').to_string();
        let api_key = cfg.fx.api_key.clone().ok_or_else(|| {
            AppError::Unexpected("missing fx api key (FOREXRATEAPI_KEY or fx.api_key)".to_string())
        })?;

        Ok(Self { http, base_url, api_key })
    }

    fn latest_url(&self) -> String {
        format!("{}/v1/latest", self.base_url)
    }

    async fn fetch_latest(
        &self,
        base: &str,
        currencies: &[&str],
    ) -> Result<FxRatesResponse, AppError> {
        let currencies_csv = currencies.join(",");

        let query = [
            ("api_key", self.api_key.as_str()),
            ("base", base),
            ("currencies", currencies_csv.as_str()),
        ];

        self.http
            .get_json::<FxRatesResponse>(&self.latest_url(), &query, None)
            .await
    }
}

#[async_trait]
impl<C: HttpClient> FxRatesGateway for ForexRateApiGateway<C> {
    async fn latest(&self, base: &str, currencies: &[&str]) -> Result<FxRatesResponse, AppError> {
        if base.trim().is_empty() {
            return Err(AppError::Unexpected("base must not be empty".to_string()));
        }
        if currencies.is_empty() {
            return Err(AppError::Unexpected("currencies must not be empty".to_string()));
        }

        let resp = self.fetch_latest(base, currencies).await?;

        if !resp.success {
            return Err(AppError::Unexpected("fx provider returned success=false".to_string()));
        }

        Ok(resp)
    }
}
