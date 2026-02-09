use anyhow::Context;
use crate::utils::configuration::database::{self, DatabaseConfig, DatabaseToml};
use crate::utils::configuration::ledger::{self, LedgerConfig, LedgerToml};
use crate::utils::configuration::fx::{self, FxConfig, FxToml,};
use crate::utils::configuration::monnify::{self, MonnifyConfig, MonnifyToml};
use crate::utils::configuration::chain::{self, ChainToml, ChainConfig};

#[derive(Debug, Clone)]
pub struct Config {
    pub database: DatabaseConfig,
    pub ledger: LedgerConfig,
    pub fx: FxConfig,
    pub monnify: MonnifyConfig,
    pub chain: ChainConfig,
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let _ = dotenvy::dotenv();
        let toml = load_toml()?;

        let database = database::load(&toml)?;
        let ledger = ledger::load(&toml)?;
        let fx = fx::load(&toml)?;
        let monnify = monnify::load(&toml)?;
        let chain = chain::load(&toml)?;

        Ok(Self { database, ledger, fx, monnify, chain })
    }
}

#[derive(Debug, serde::Deserialize, Clone)]
pub(crate) struct TomlConfig {
    pub db: DatabaseToml,
    pub ledger: LedgerToml,
    pub fx: FxToml,
    pub monnify: MonnifyToml,
    pub chain: ChainToml,
}

fn load_toml() -> anyhow::Result<TomlConfig> {
    let cfg = config::Config::builder()
        .add_source(config::File::with_name("config"))
        .build()
        .context("failed to load config.toml")?;

    cfg.try_deserialize().context("invalid config.toml")
}
