use chrono::{DateTime, Utc};
use crate::domain::value_objects::{AssetCode, Chain, RegionCode};

#[derive(Debug, Clone)]
pub struct CreateHotWalletCommand {
    pub chain: Chain,
    pub asset_code: AssetCode,
    pub region_code: RegionCode,
    //pub private_key_hex: String,
    pub max_balance_minor: i128,
    pub now: DateTime<Utc>,
}
