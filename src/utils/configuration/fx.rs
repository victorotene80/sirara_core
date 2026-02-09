use std::time::Duration;
use anyhow::{bail, Context};
use super::config::TomlConfig;

#[derive(Debug, Clone)]
pub struct FxConfig {
    pub max_staleness: Duration,
    pub firm_ttl: Duration,
    pub base_url: String,
    pub api_key: Option<String>,
    pub enforce_direct_only: bool,
}

#[derive(Debug, serde::Deserialize, Clone)]
pub(crate) struct FxToml {
    pub base_url: String,
    pub api_key: Option<String>,
    pub max_staleness_secs: i64,
    pub firm_ttl_secs: i64,
    pub enforce_direct_only: Option<bool>,
}

pub(crate) fn load(toml: &TomlConfig) -> anyhow::Result<FxConfig> {
    if toml.fx.base_url.trim().is_empty() {
        bail!("fx.base_url must not be empty");
    }
    if toml.fx.max_staleness_secs <= 0 {
        bail!(
            "fx.max_staleness_secs must be > 0 (got {})",
            toml.fx.max_staleness_secs
        );
    }
    if toml.fx.firm_ttl_secs <= 0 {
        bail!(
            "fx.firm_ttl_secs must be > 0 (got {})",
            toml.fx.firm_ttl_secs
        );
    }

    let env_key = std::env::var("FOREXRATEAPI_KEY").ok();
    let env_key = env_key
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let api_key = env_key.or_else(|| toml.fx.api_key.clone());

    Ok(FxConfig {
        base_url: toml.fx.base_url.trim_end_matches('/').to_string(),  // <-- HERE
        api_key,
        max_staleness: Duration::from_secs(toml.fx.max_staleness_secs as u64),
        firm_ttl: Duration::from_secs(toml.fx.firm_ttl_secs as u64),
        enforce_direct_only: toml.fx.enforce_direct_only.unwrap_or(true),
    })
}

