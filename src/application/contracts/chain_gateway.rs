use async_trait::async_trait;

use crate::application::AppError;
use crate::domain::value_objects::{AssetCode, Chain, Money, OnchainAddress, TxHash};

/// Handle to retrieve secret from your secret store (KMS/DB encryption/etc).
/// Keep this opaque; infra knows how to resolve it.
#[derive(Debug, Clone)]
pub struct SecretHandle(pub String);

#[derive(Debug, Clone)]
pub struct CreatedWallet {
    pub address: OnchainAddress,
    pub secret_handle: SecretHandle,
}

/// For Solana SPL tokens (USDT is SPL token mint).
#[derive(Debug, Clone)]
pub struct TokenMintAddress(pub String);

#[derive(Debug, Clone)]
pub struct SubmitTransferResult {
    pub tx_hash: TxHash,
}

#[async_trait]
pub trait SecretStore: Send + Sync {
    /// Returns raw bytes needed to sign (e.g., Solana Keypair bytes).
    async fn get_signing_key_bytes(&self, handle: &SecretHandle) -> Result<Vec<u8>, AppError>;
}

#[async_trait]
pub trait ChainGateway: Send + Sync {
    fn chain(&self) -> Chain;

    /// Create a custodial wallet (you control keys). For non-custodial, you’d accept addresses instead.
    async fn create_wallet(&self) -> Result<CreatedWallet, AppError>;

    /// Ensure token-account exists for (owner, asset). On Solana: create ATA if missing.
    async fn ensure_token_account(&self, owner: &OnchainAddress, asset: &AssetCode) -> Result<(), AppError>;

    /// Submit an onchain token transfer and return tx hash.
    /// `from` is a secret handle so infra can sign.
    async fn submit_token_transfer(
        &self,
        from: &SecretHandle,
        to: &OnchainAddress,
        asset: &AssetCode,
        amount: Money,
        idempotency_key: &str,
    ) -> Result<SubmitTransferResult, AppError>;

    /// Confirmation polling primitive (your confirm worker will use this).
    async fn get_tx_confirmations(&self, tx_hash: &TxHash) -> Result<i32, AppError>;

    /// Optional convenience: returns true when confirmed enough.
    async fn is_tx_confirmed(&self, tx_hash: &TxHash, min_confirmations: i32) -> Result<bool, AppError> {
        let c = self.get_tx_confirmations(tx_hash).await?;
        Ok(c >= min_confirmations)
    }

    // ------------------------
    // DEV-ONLY (optional)
    // ------------------------

    /// Create a test mint (like USDT_TEST) and return mint address.
    /// In production you will not call this; you use configured real USDT mint.
    async fn dev_create_test_mint(&self, _decimals: u8) -> Result<TokenMintAddress, AppError> {
        Err(AppError::Unexpected("dev_create_test_mint not supported".into()))
    }

    /// Mint test tokens to address (dev only).
    async fn dev_mint_to(
        &self,
        _mint: &TokenMintAddress,
        _to: &OnchainAddress,
        _amount_minor: u64,
    ) -> Result<(), AppError> {
        Err(AppError::Unexpected("dev_mint_to not supported".into()))
    }
}
