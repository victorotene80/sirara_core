use chrono::{DateTime, Utc};

use crate::domain::value_objects::{
    PublicId,
};

#[derive(Debug, Clone)]
pub struct MarkRailSubmittedCommand {
    pub public_id: PublicId,
    pub now: DateTime<Utc>,
}