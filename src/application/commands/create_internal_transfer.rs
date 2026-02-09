use uuid::Uuid;
use crate::domain::value_objects::{AssetCode, ExternalRef, ExternalRefType, Money, PublicId, RegionCode};

#[derive(Debug, Clone)]
pub struct CreateInternalTransferCommand {
    pub public_id: PublicId,
    pub external_ref_type: ExternalRefType,
    pub external_ref: ExternalRef,
    pub from_user_id: Uuid,
    pub region: RegionCode,
    pub to_user_id: Uuid,
    pub asset: AssetCode,
    pub amount: Money,
    pub initiated_by: Option<String>,
}
