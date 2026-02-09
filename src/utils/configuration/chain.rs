use anyhow::Context;

#[derive(Debug, Clone)]
pub struct ChainConfig {
    pub solana: SolanaConfig,
}

#[derive(Debug, Clone)]
pub struct SolanaConfig {
    pub rpc_url: String,
    pub usdt_mint: String,
    pub payer_secret_handle: Option<String>,
    pub payer_keypair_b64: Option<String>,
    pub min_confirmations: i32,
}

#[derive(Debug, serde::Deserialize, Clone)]
pub(crate) struct ChainToml {
    pub solana: SolanaToml,
}

#[derive(Debug, serde::Deserialize, Clone)]
pub(crate) struct SolanaToml {
    pub network: String,
    pub rpc_url: String,
    pub usdt_mint: String,
    pub min_confirmations: Option<i32>,
}

pub fn load(toml: &crate::utils::configuration::config::TomlConfig) -> anyhow::Result<ChainConfig> {
    let sol_toml = &toml.chain.solana;

    let rpc_url = std::env::var("SOLANA_RPC_URL").unwrap_or_else(|_| sol_toml.rpc_url.clone());
    let usdt_mint = std::env::var("SOLANA_USDT_MINT").unwrap_or_else(|_| sol_toml.usdt_mint.clone());

    let payer_secret_handle = std::env::var("SOLANA_PAYER_SECRET_HANDLE").ok();
    let payer_keypair_b64 = std::env::var("SOLANA_PAYER_KEYPAIR_B64").ok();

    let min_confirmations = std::env::var("SOLANA_MIN_CONFIRMATIONS")
        .ok()
        .and_then(|v| v.parse::<i32>().ok())
        .or(sol_toml.min_confirmations)
        .unwrap_or(1);

    Ok(ChainConfig {
        solana: SolanaConfig {
            rpc_url,
            usdt_mint,
            payer_secret_handle,
            payer_keypair_b64,
            min_confirmations,
        },
    })
}

