use uuid::Uuid;

use crate::application::dtos::{TransferKindDTO,DestinationDTO};

#[derive(Debug, Clone)]
pub struct CreateTransferIntentDTO {
    pub external_ref: String,
    pub transfer_type: TransferKindDTO,
    pub from_user_id: Uuid,
    pub to_user_id: Option<Uuid>,
    pub destination: Option<DestinationDTO>,
    pub asset: Option<String>,
    pub amount_minor: Option<i128>,
    pub pay_asset: Option<String>,
    pub pay_amount_minor: Option<i128>,
    pub deliver_asset: Option<String>,
    pub deliver_amount_minor: Option<i128>,
}
