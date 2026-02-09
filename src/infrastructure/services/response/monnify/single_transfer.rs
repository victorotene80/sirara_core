use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub struct SingleTransferBody {
    pub amount: f64,
    pub reference: String,
    pub status: String, // e.g. SUCCESS, PENDING_AUTHORIZATION, PENDING
    pub dateCreated: Option<String>,
    pub totalFee: Option<f64>,
    pub sessionId: Option<String>,

    pub destinationAccountName: Option<String>,
    pub destinationBankName: Option<String>,
    pub destinationAccountNumber: Option<String>,
    pub destinationBankCode: Option<String>,
}