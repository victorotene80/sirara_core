use anyhow::Context;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub database_url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout: Duration,
}

#[derive(Debug, serde::Deserialize, Clone)]
pub(crate) struct DatabaseToml {
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout_secs: u64,
}

pub(crate) fn load(toml: &crate::utils::configuration::config::TomlConfig) -> anyhow::Result<DatabaseConfig> {
    let database_url =
        std::env::var("DATABASE_URL").context("DATABASE_URL is required (env only)")?;

    Ok(DatabaseConfig {
        database_url,
        max_connections: toml.db.max_connections,
        min_connections: toml.db.min_connections,
        acquire_timeout: Duration::from_secs(toml.db.acquire_timeout_secs),
    })
}
