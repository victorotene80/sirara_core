use crate::domain::value_objects::{PublicId, ExternalRef, RegionCode, AssetCode};

pub struct ReleaseInventoryCommand {
    pub journal_public_id: PublicId,
    pub external_ref: ExternalRef,
    pub created_by: String,
    pub asset_code: AssetCode,
    pub region_code: RegionCode,
    pub description: Option<String>,
    pub amount_minor: i128,
}