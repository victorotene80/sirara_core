use std::collections::HashMap;

use anyhow::bail;

use super::config::TomlConfig;

#[derive(Debug, Clone)]
pub struct LedgerConfig {
    pub max_lines_normal: usize,
    pub max_lines_batch: usize,
    /// asset_code -> max_abs_minor (per journal line)
    pub max_post_amount_by_code: HashMap<String, i128>,
}

#[derive(Debug, serde::Deserialize, Clone)]
pub(crate) struct LedgerToml {
    pub max_lines_normal: usize,
    pub max_lines_batch: usize,
    #[serde(default)]
    pub max_post_amount: HashMap<String, i128>,
}

pub(crate) fn load(toml: &TomlConfig) -> anyhow::Result<LedgerConfig> {
    validate_ledger_toml(&toml.ledger)?;

    let mut by_code = HashMap::new();
    for (code, v) in toml.ledger.max_post_amount.iter() {
        by_code.insert(code.trim().to_uppercase(), *v);
    }

    Ok(LedgerConfig {
        max_lines_normal: toml.ledger.max_lines_normal,
        max_lines_batch: toml.ledger.max_lines_batch,
        max_post_amount_by_code: by_code,
    })
}

fn validate_ledger_toml(cfg: &LedgerToml) -> anyhow::Result<()> {
    if cfg.max_lines_normal < 2 {
        bail!("ledger.max_lines_normal must be >= 2 (got {})", cfg.max_lines_normal);
    }

    // Batch postings should never be smaller than normal (or you’ll surprise yourself later)
    if cfg.max_lines_batch < cfg.max_lines_normal {
        bail!(
            "ledger.max_lines_batch must be >= ledger.max_lines_normal (batch={}, normal={})",
            cfg.max_lines_batch,
            cfg.max_lines_normal
        );
    }

    for (code, limit) in cfg.max_post_amount.iter() {
        let c = code.trim();

        if c.is_empty() {
            bail!("ledger.max_post_amount has an empty asset code key");
        }

        if c != c.to_uppercase() {
            bail!("ledger.max_post_amount key must be uppercase (got '{c}')");
        }
        if !(2..=10).contains(&c.len()) {
            bail!("ledger.max_post_amount asset code length must be 2..=10 (got '{c}')");
        }

        if *limit <= 0 {
            bail!("ledger.max_post_amount[{c}] must be > 0 (got {limit})");
        }
    }

    Ok(())
}
