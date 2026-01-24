use serde::{Deserialize, Serialize};

use crate::application::dtos::DestinationDTO;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransferKindDTO { 
    Intra {
        from_user_id: String,
        to_user_id: String,
        asset_code: String,
        amount_minor: i128,
        from_user_available_acct_id: i64,
        from_user_locked_acct_id: i64,
        to_user_available_acct_id: i64,
    },

    Inter {
        from_user_id: String,
        pay_asset_code: String,
        pay_amount_minor: i128,
        deliver_asset_code: String,
        deliver_amount_minor: i128,
        from_user_available_acct_id: i64,
        from_user_locked_acct_id: i64,
        platform_clearing_acct_id: i64,
        destination: DestinationDTO,
    },
}
