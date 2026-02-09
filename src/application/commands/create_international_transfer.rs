use uuid::Uuid;
use crate::domain::value_objects::{
    AssetCode, ExternalRef, ExternalRefType, Money, PublicId, RegionCode, Destination,
};

#[derive(Debug, Clone)]
pub struct CreateInternationalTransferCommand {
    pub public_id: PublicId,
    pub external_ref_type: ExternalRefType,
    pub external_ref: ExternalRef,
    pub from_user_id: Uuid,
    pub region: RegionCode,
    pub destination: Destination,
    pub asset: AssetCode,
    pub amount: Money,
    pub initiated_by: Option<String>,
}
