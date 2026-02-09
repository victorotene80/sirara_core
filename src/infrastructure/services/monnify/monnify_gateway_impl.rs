use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use base64::{engine::general_purpose, Engine as _};

use crate::infrastructure::error::InfraError;
use crate::infrastructure::services::http::HttpClient;
use crate::utils::configuration::{Config, MonnifyConfig};

use crate::infrastructure::services::response::monnify::{
    MonnifyEnvelope, MonnifyLoginBody, SingleTransferBody,
};
use crate::infrastructure::services::request::monnify::{
    EmptyBody, InitiateSingleTransferReq, ResendOtpReq, ValidateOtpReq,
};
use crate::infrastructure::services::monnify::monnify_gateway::MonnifyDisbursementGateway;
use crate::infrastructure::services::token_store::TokenStore;
use crate::infrastructure::services::token_store::token_store_impl::InMemoryTokenStore;

pub struct MonnifyApiGateway<C: HttpClient> {
    http: Arc<C>,
    cfg: MonnifyConfig,
    tokens: Arc<dyn TokenStore>,
}

impl<C: HttpClient> MonnifyApiGateway<C> {
    pub fn from_config(http: Arc<C>, cfg: &Config) -> Result<Self, InfraError> {
        Ok(Self {
            http,
            cfg: cfg.monnify.clone(),
            tokens: Arc::new(InMemoryTokenStore::new()),
        })
    }

    pub fn new(http: Arc<C>, cfg: MonnifyConfig, tokens: Arc<dyn TokenStore>) -> Self {
        Self { http, cfg, tokens }
    }

    fn basic_auth_header_value(&self) -> String {
        let raw = format!("{}:{}", self.cfg.api_key, self.cfg.secret_key);
        format!("Basic {}", general_purpose::STANDARD.encode(raw.as_bytes()))
    }

    fn ensure_non_empty(name: &str, value: &str) -> Result<(), InfraError> {
        if value.trim().is_empty() {
            return Err(InfraError::Unexpected {
                message: format!("{name} must not be empty"),
            });
        }
        Ok(())
    }

    fn validate_transfer_req(req: &InitiateSingleTransferReq<'_>) -> Result<(), InfraError> {
        Self::ensure_non_empty("reference", req.reference)?;
        Self::ensure_non_empty("narration", req.narration)?;
        Self::ensure_non_empty("destinationBankCode", req.destinationBankCode)?;
        Self::ensure_non_empty("destinationAccountNumber", req.destinationAccountNumber)?;
        Self::ensure_non_empty("sourceAccountNumber", req.sourceAccountNumber)?;
        Self::ensure_non_empty("currency", req.currency)?;

        if req.amount <= 0.0 {
            return Err(InfraError::Unexpected {
                message: "amount must be > 0".to_string(),
            });
        }

        Ok(())
    }

    async fn ensure_token(&self) -> Result<String, InfraError> {
        if let Some(t) = self.tokens.get_valid().await? {
            return Ok(t);
        }

        let auth = self.basic_auth_header_value();
        let headers = [("Authorization", auth.as_str())];

        let env: MonnifyEnvelope<MonnifyLoginBody> = self
            .http
            .post_json_with_headers(&self.cfg.login_url(), &EmptyBody {}, &headers)
            .await?;

        if !env.requestSuccessful {
            return Err(InfraError::Unexpected {
                message: format!(
                    "monnify login failed: {} ({})",
                    env.responseMessage, env.responseCode
                ),
            });
        }

        let body = env.responseBody.ok_or_else(|| InfraError::Unexpected {
            message: "monnify login missing responseBody".to_string(),
        })?;

        let ttl = Duration::from_secs(body.expiresIn.max(0) as u64)
            .saturating_sub(self.cfg.token_skew);

        self.tokens.set(body.accessToken.clone(), ttl).await?;

        Ok(body.accessToken)
    }

    fn ensure_success<T>(env: MonnifyEnvelope<T>, ctx: &str) -> Result<MonnifyEnvelope<T>, InfraError> {
        if env.requestSuccessful {
            Ok(env)
        } else {
            Err(InfraError::Unexpected {
                message: format!(
                    "{ctx}: {} ({})",
                    env.responseMessage, env.responseCode
                ),
            })
        }
    }
}

#[async_trait]
impl<C: HttpClient> MonnifyDisbursementGateway for MonnifyApiGateway<C> {
    async fn login(&self) -> Result<MonnifyEnvelope<MonnifyLoginBody>, InfraError> {
        let auth = self.basic_auth_header_value();
        let headers = [("Authorization", auth.as_str())];

        let env: MonnifyEnvelope<MonnifyLoginBody> = self
            .http
            .post_json_with_headers(&self.cfg.login_url(), &EmptyBody {}, &headers)
            .await?;

        Self::ensure_success(env, "monnify login failed")
    }

    async fn initiate_single_transfer(
        &self,
        req: InitiateSingleTransferReq<'_>,
    ) -> Result<MonnifyEnvelope<SingleTransferBody>, InfraError> {
        Self::validate_transfer_req(&req)?;

        let token = self.ensure_token().await?;
        let env: MonnifyEnvelope<SingleTransferBody> = self
            .http
            .post_json(&self.cfg.single_transfer_url(), &req, Some(&token))
            .await?;

        Self::ensure_success(env, "monnify initiate transfer failed")
    }

    async fn validate_single_transfer_otp(
        &self,
        reference: &str,
        authorization_code: &str,
    ) -> Result<MonnifyEnvelope<SingleTransferBody>, InfraError> {
        Self::ensure_non_empty("reference", reference)?;
        Self::ensure_non_empty("authorizationCode", authorization_code)?;

        let token = self.ensure_token().await?;
        let body = ValidateOtpReq {
            reference,
            authorizationCode: authorization_code,
        };

        let env: MonnifyEnvelope<SingleTransferBody> = self
            .http
            .post_json(&self.cfg.validate_otp_url(), &body, Some(&token))
            .await?;

        Self::ensure_success(env, "monnify validate otp failed")
    }

    async fn resend_single_transfer_otp(
        &self,
        reference: &str,
    ) -> Result<MonnifyEnvelope<SingleTransferBody>, InfraError> {
        Self::ensure_non_empty("reference", reference)?;

        let token = self.ensure_token().await?;
        let body = ResendOtpReq { reference };

        let env: MonnifyEnvelope<SingleTransferBody> = self
            .http
            .post_json(&self.cfg.resend_otp_url(), &body, Some(&token))
            .await?;

        Self::ensure_success(env, "monnify resend otp failed")
    }

    async fn get_single_transfer_summary(
        &self,
        reference: &str,
    ) -> Result<MonnifyEnvelope<SingleTransferBody>, InfraError> {
        Self::ensure_non_empty("reference", reference)?;

        let token = self.ensure_token().await?;
        let query = [("reference", reference)];

        let env: MonnifyEnvelope<SingleTransferBody> = self
            .http
            .get_json(&self.cfg.single_summary_url(), &query, Some(&token))
            .await?;

        Self::ensure_success(env, "monnify transfer summary failed")
    }
}
