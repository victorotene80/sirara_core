use anyhow::Context;

use super::database::{self, DatabaseConfig, DatabaseToml};
use super::ledger::{self, LedgerConfig, LedgerToml};

#[derive(Debug, Clone)]
pub struct Config {
    pub database: DatabaseConfig,
    pub ledger: LedgerConfig,
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let _ = dotenvy::dotenv();
        let toml = load_toml()?;

        let database = database::load(&toml)?;
        let ledger = ledger::load(&toml)?;

        Ok(Self { database, ledger })
    }
}

#[derive(Debug, serde::Deserialize, Clone)]
pub(crate) struct TomlConfig {
    pub db: DatabaseToml,
    pub ledger: LedgerToml,
}

fn load_toml() -> anyhow::Result<TomlConfig> {
    let cfg = config::Config::builder()
        .add_source(config::File::with_name("config"))
        .build()
        .context("failed to load config.toml")?;

    cfg.try_deserialize().context("invalid config.toml")
}
