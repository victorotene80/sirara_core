use async_trait::async_trait;

use crate::infrastructure::error::InfraError;
use crate::infrastructure::services::request::monnify::{InitiateSingleTransferReq};
use crate::infrastructure::services::response::monnify::{MonnifyEnvelope, MonnifyLoginBody, SingleTransferBody};

#[async_trait]
pub trait MonnifyDisbursementGateway: Send + Sync {
    async fn login(&self) -> Result<MonnifyEnvelope<MonnifyLoginBody>, InfraError>;

    async fn initiate_single_transfer(
        &self,
        req: InitiateSingleTransferReq<'_>,
    ) -> Result<MonnifyEnvelope<SingleTransferBody>, InfraError>;

    async fn validate_single_transfer_otp(
        &self,
        reference: &str,
        authorization_code: &str,
    ) -> Result<MonnifyEnvelope<SingleTransferBody>, InfraError>;

    async fn resend_single_transfer_otp(
        &self,
        reference: &str,
    ) -> Result<MonnifyEnvelope<SingleTransferBody>, InfraError>;

    async fn get_single_transfer_summary(
        &self,
        reference: &str,
    ) -> Result<MonnifyEnvelope<SingleTransferBody>, InfraError>;
}
