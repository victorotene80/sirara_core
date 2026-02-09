use async_trait::async_trait;
use crate::infrastructure::services::response::fx_rate::FxRatesResponse;

use crate::application::AppError;

#[async_trait]
pub trait FxRatesGateway: Send + Sync {
    async fn latest(&self, base: &str, currencies: &[&str]) -> Result<FxRatesResponse, AppError>;

}
