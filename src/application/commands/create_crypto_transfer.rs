use uuid::Uuid;

use crate::domain::value_objects::{
    AssetCode, Chain, ExternalRef, ExternalRefType, Memo, Money, OnchainAddress, PublicId, RegionCode,
};

#[derive(Debug, Clone)]
pub struct CreateInterCryptoTransferCommand {
    pub public_id: PublicId,
    pub external_ref_type: ExternalRefType,
    pub external_ref: ExternalRef,
    pub from_user_id: Uuid,
    pub region: RegionCode,
    pub chain: Chain,
    pub to_address: OnchainAddress,
    pub memo: Option<Memo>,
    pub asset: AssetCode,
    pub amount: Money,
    pub initiated_by: Option<String>,
}


