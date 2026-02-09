use serde::Serialize;
#[derive(Debug, Serialize)]
pub struct ValidateOtpReq<'a> {
    pub reference: &'a str,
    pub authorizationCode: &'a str,
}