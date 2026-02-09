use uuid::Uuid;

use crate::domain::value_objects::{
    AccountNumber, AssetCode, BankCode, ExternalRef, ExternalRefType, Money, PublicId, RegionCode,
};

#[derive(Debug, Clone)]
pub struct CreateInterBankTransferCommand {
    pub public_id: PublicId,
    pub external_ref_type: ExternalRefType,
    pub external_ref: ExternalRef,
    pub from_user_id: Uuid,
    pub region: RegionCode,
    pub bank_code: BankCode,
    pub account_number: AccountNumber,
    pub asset: AssetCode,
    pub amount: Money,
    pub initiated_by: Option<String>,
}
