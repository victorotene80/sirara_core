use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct EmptyBody {}

#[derive(Debug, Serialize)]
pub struct InitiateSingleTransferReq<'a> {
    pub amount: f64,
    pub reference: &'a str,
    pub narration: &'a str,
    pub destinationBankCode: &'a str,
    pub destinationAccountNumber: &'a str,
    pub currency: &'a str, // "NGN"
    pub sourceAccountNumber: &'a str,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub destinationAccountName: Option<&'a str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#async: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub senderInfo: Option<SenderInfo<'a>>,
}
#[derive(Debug, Serialize)]
pub struct SenderInfo<'a> {
    pub name: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phoneNumber: Option<&'a str>,
}