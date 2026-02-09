use crate::domain::value_objects::{PublicId, ExternalRef, AssetCode};

pub struct HoldUserFundsCommand {
    pub journal_public_id: PublicId,
    pub external_ref: ExternalRef,
    pub created_by: String,
    pub user_id: uuid::Uuid,
    pub asset_code: AssetCode,
    pub description: Option<String>,
    pub amount_minor: i128,
}