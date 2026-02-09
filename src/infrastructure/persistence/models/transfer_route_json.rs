use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::infrastructure::persistence::models::DestinationJson;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransferRouteJson {
    INTRA {
        region_code: String,
        from_user_id: Uuid,
        to_user_id: Uuid,
    },
    BANK {
        region_code: String,
        from_user_id: Uuid,
        destination: DestinationJson,
    },
    CRYPTO {
        region_code: String,
        from_user_id: Uuid,
        chain: String,
        to_address: String,
        memo: Option<String>,
    },
}
