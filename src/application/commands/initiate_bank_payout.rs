use uuid::Uuid;

use crate::domain::value_objects::{AccountNumber, AssetCode, BankCode, ExternalRef, Money};

#[derive(Debug, Clone)]
pub struct InitiateBankPayoutCommand {
    pub external_ref: ExternalRef,     // idempotency/correlation
    pub from_user_id: Uuid,
    pub asset: AssetCode,              // must be NGN for Bit 1
    pub amount: Money,
    pub bank_code: BankCode,
    pub account_number: AccountNumber,
    pub account_name: Option<String>,
    pub narration: Option<String>,

    pub created_by: String,
}
