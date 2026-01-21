use anyhow::Context;

use crate::utils::configuration::{database};

#[derive(Debug, Clone)]
pub struct Config {
    pub database: database::DatabaseConfig,
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let _ = dotenvy::dotenv();
        let toml = load_toml()?;
        Ok(Self {
            database: database::load(&toml)?,
        })
    }
}

#[derive(Debug, serde::Deserialize, Clone)]
pub(crate) struct TomlConfig {
    pub db: database::DatabaseToml,
}

fn load_toml() -> anyhow::Result<TomlConfig> {
    let cfg = config::Config::builder()
        .add_source(config::File::with_name("config"))
        .build()
        .context("failed to load config.toml")?;

    cfg.try_deserialize().context("invalid config.toml")
}
