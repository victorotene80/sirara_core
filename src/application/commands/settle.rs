use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::domain::value_objects::{
    PublicId
};

#[derive(Debug, Clone)]
pub struct SettleCommand {
    pub public_id: PublicId,
    pub now: DateTime<Utc>,
}