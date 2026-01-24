use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransferRouteJson {
    INTRA {
        from_user_id: Uuid,
        to_user_id: Uuid,
        asset_code: String,
        amount_minor: i128,
    },

    PAYOUT {
        from_user_id: Uuid,
        asset_code: String,
        amount_minor: i128,
        destination: DestinationJson,
    },

    INTER {
        from_user_id: Uuid,
        pay_asset_code: String,
        pay_amount_minor: i128,
        deliver_asset_code: String,
        deliver_amount_minor: i128,
        destination: DestinationJson,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DestinationJson {
    ONCHAIN {
        chain: ChainJson,
        address: String,
        memo: Option<String>,
    },
    BANK {
        bank_code: String,
        account_number: String,
        account_name: Option<String>,
        narration: Option<String>,
    },
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ChainJson {
    TRON,
    ETHEREUM,
    SOLANA,
    BITCOIN,
}
