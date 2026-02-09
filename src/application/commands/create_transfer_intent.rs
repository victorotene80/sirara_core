use chrono::{DateTime, Utc};

use crate::domain::value_objects::{
    AssetCode, ExternalRef, ExternalRefType, Money, PublicId,
    TransferRoute,
};

#[derive(Debug, Clone)]
pub struct CreateTransferIntentCommand {
    pub public_id: PublicId,
    pub external_ref_type: ExternalRefType,
    pub external_ref: ExternalRef,
    pub route: TransferRoute,
    pub asset: AssetCode,
    pub amount: Money,
    pub now: DateTime<Utc>,
}