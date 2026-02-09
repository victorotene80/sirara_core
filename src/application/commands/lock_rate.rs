use chrono::{DateTime, Utc};

use crate::domain::value_objects::{
     FxQuote, PublicId,
};

#[derive(Debug, Clone)]
pub struct LockRateCommand {
    pub public_id: PublicId,
    pub quote: FxQuote,
    pub required_usdt_minor: i128,
    pub now: DateTime<Utc>,
}