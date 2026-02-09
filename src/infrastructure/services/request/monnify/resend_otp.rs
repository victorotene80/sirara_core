use serde::Serialize;
#[derive(Debug, Serialize)]
pub struct ResendOtpReq<'a> {
    pub reference: &'a str,
}