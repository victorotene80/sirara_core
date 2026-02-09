use anyhow::{bail, Context};
use std::time::Duration;

use super::config::TomlConfig;

#[derive(Debug, Clone)]
pub struct MonnifyConfig {
    pub base_url: String,
    pub token_skew: Duration,
    pub api_key: String,
    pub secret_key: String,
    pub contract_code: Option<String>,
    pub paths: MonnifyPaths,
}

#[derive(Debug, Clone)]
pub struct MonnifyPaths {
    pub login: String,
    pub single_transfer: String,
    pub validate_otp: String,
    pub resend_otp: String,
    pub single_summary: String,
}

#[derive(Debug, serde::Deserialize, Clone)]
pub(crate) struct MonnifyToml {
    pub base_url: String,
    pub token_skew_secs: u64,
    pub paths: MonnifyPathsToml,
}

#[derive(Debug, serde::Deserialize, Clone)]
pub(crate) struct MonnifyPathsToml {
    pub login: String,
    pub single_transfer: String,
    pub validate_otp: String,
    pub resend_otp: String,
    pub single_summary: String,
}

pub(crate) fn load(toml: &TomlConfig) -> anyhow::Result<MonnifyConfig> {
    validate_monnify_toml(&toml.monnify)?;

    let base_url = toml.monnify.base_url.trim().trim_end_matches('/').to_string();

    let api_key =
        std::env::var("MONNIFY_API_KEY").context("MONNIFY_API_KEY is required (env only)")?;
    let secret_key =
        std::env::var("MONNIFY_SECRET_KEY").context("MONNIFY_SECRET_KEY is required (env only)")?;

    if api_key.trim().is_empty() {
        bail!("MONNIFY_API_KEY must not be empty");
    }
    if secret_key.trim().is_empty() {
        bail!("MONNIFY_SECRET_KEY must not be empty");
    }

    let contract_code = std::env::var("MONNIFY_CONTRACT_CODE").ok();

    let p = &toml.monnify.paths;
    let paths = MonnifyPaths {
        login: normalize_path("monnify.paths.login", &p.login)?,
        single_transfer: normalize_path("monnify.paths.single_transfer", &p.single_transfer)?,
        validate_otp: normalize_path("monnify.paths.validate_otp", &p.validate_otp)?,
        resend_otp: normalize_path("monnify.paths.resend_otp", &p.resend_otp)?,
        single_summary: normalize_path("monnify.paths.single_summary", &p.single_summary)?,
    };

    Ok(MonnifyConfig {
        base_url,
        token_skew: Duration::from_secs(toml.monnify.token_skew_secs),
        api_key,
        secret_key,
        contract_code,
        paths,
    })
}

fn validate_monnify_toml(cfg: &MonnifyToml) -> anyhow::Result<()> {
    let base = cfg.base_url.trim();
    if base.is_empty() {
        bail!("monnify.base_url must not be empty");
    }
    if !(base.starts_with("https://") || base.starts_with("http://")) {
        bail!("monnify.base_url must start with http:// or https://");
    }
    if base == "https://" || base == "http://" {
        bail!("monnify.base_url is invalid (missing host)");
    }

    if cfg.token_skew_secs > 300 {
        bail!("monnify.token_skew_secs too large (max 300)");
    }

    validate_path("monnify.paths.login", &cfg.paths.login)?;
    validate_path("monnify.paths.single_transfer", &cfg.paths.single_transfer)?;
    validate_path("monnify.paths.validate_otp", &cfg.paths.validate_otp)?;
    validate_path("monnify.paths.resend_otp", &cfg.paths.resend_otp)?;
    validate_path("monnify.paths.single_summary", &cfg.paths.single_summary)?;

    Ok(())
}

fn validate_path(name: &str, v: &str) -> anyhow::Result<()> {
    let s = v.trim();
    if s.is_empty() {
        bail!("{name} must not be empty");
    }
    if !s.starts_with('/') {
        bail!("{name} must start with '/' (got '{s}')");
    }
    if s.contains("http://") || s.contains("https://") {
        bail!("{name} must be a path only (do not include scheme/host): '{s}'");
    }
    Ok(())
}

fn normalize_path(name: &str, v: &str) -> anyhow::Result<String> {
    validate_path(name, v)?;
    Ok(v.trim().to_string())
}

impl MonnifyConfig {
    #[inline]
    pub fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    pub fn login_url(&self) -> String {
        self.url(&self.paths.login)
    }
    pub fn single_transfer_url(&self) -> String {
        self.url(&self.paths.single_transfer)
    }
    pub fn validate_otp_url(&self) -> String {
        self.url(&self.paths.validate_otp)
    }
    pub fn resend_otp_url(&self) -> String {
        self.url(&self.paths.resend_otp)
    }
    pub fn single_summary_url(&self) -> String {
        self.url(&self.paths.single_summary)
    }
}
