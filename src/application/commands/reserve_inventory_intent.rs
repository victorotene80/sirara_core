use chrono::{DateTime, Utc};
use crate::domain::value_objects::PublicId;

pub struct ReserveInventoryIntentCommand {
    pub public_id: PublicId,
    pub now: DateTime<Utc>,
}
