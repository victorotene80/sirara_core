use async_trait::async_trait;
use crate::application::AppError;
use crate::domain::value_objects::FxQuote;

#[derive(Debug, Clone)]
pub struct FxQuoteRequest {
    pub from: String,
    pub to: String,
    pub via: Vec<String>,
    pub amount_minor: i128,
}

#[async_trait]
pub trait FxQuoteService: Send + Sync {
    async fn get_firm_quote(&self, req: FxQuoteRequest) -> Result<FxQuote, AppError>;
}
