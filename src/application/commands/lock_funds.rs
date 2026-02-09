use crate::domain::value_objects::{PublicId, ExternalRef, AssetCode};
use chrono::{ Utc, DateTime};

#[derive(Debug, Clone)]
pub struct LockFundsCommand {
    pub public_id: PublicId,
    pub now: DateTime<Utc>,
}