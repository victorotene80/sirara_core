use crate::domain::value_objects::FxQuote;
pub struct FxQuoteResult {
    pub quote: FxQuote,
    pub required_usdt_minor: i128,
}
