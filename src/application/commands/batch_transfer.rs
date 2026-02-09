use crate::domain::value_objects::{PublicId, ExternalRef, AssetCode};

pub struct BatchTransferCommand {
    pub journal_public_id: PublicId,
    pub external_ref: ExternalRef,
    pub created_by: String,
    pub description: Option<String>,
    pub asset_code: AssetCode,
    pub funding_account_id: i64,
    pub payouts: Vec<BatchPayout>,
}

pub struct BatchPayout {
    pub user_id: uuid::Uuid,
    pub amount_minor: i128,
}
