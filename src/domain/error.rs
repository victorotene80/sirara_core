use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum DomainError {
    #[error("asset code length invalid (must be between {min} and {max})")]
    AssetCodeInvalidLength { min: usize, max: usize },

    #[error("asset code must be uppercase")]
    AssetCodeNotUppercase,

    #[error("debit amount must be greater than zero")]
    InvalidDebitAmount,

    #[error("credit amount must be greater than zero")]
    InvalidCreditAmount,

    #[error("external_ref cannot be empty")]
    ExternalRefEmpty,

    #[error("external_ref too long (max {max} chars)")]
    ExternalRefTooLong { max: usize },

    #[error("invalid external_ref_type: {value}")]
    InvalidExternalRefType { value: String },

    #[error("ledger account is inactive")]
    LedgerAccountInactive,

    #[error("created_by cannot be empty")]
    CreatedByEmpty,

    #[error("journal transaction must have at least 2 lines")]
    JournalTooFewLines,

    #[error("account not found: {account_id}")]
    LedgerAccountNotFound { account_id: i64 },

    #[error("multi-asset journal transactions are not allowed")]
    MultiAssetJournalNotAllowed,

    #[error("journal transaction is not balanced")]
    JournalNotBalanced,

    #[error("cross-asset posting not allowed")]
    CrossAssetPostingNotAllowed,

    #[error("journal must have at least one line")]
    JournalEmpty,

    #[error("journal line amount cannot be zero")]
    JournalLineAmountZero,

    #[error("journal contains too many lines: maximum allowed is {max}")]
    JournalTooManyLines { max: usize },

    #[error("posting not allowed: from {from} to {to}")]
    PostingNotAllowed { from: String, to: String },

    #[error("posting shape not allowed (senders={senders}, receivers={receivers})")]
    PostingShapeNotAllowed { senders: usize, receivers: usize },

    #[error("account owner type mismatch for account {account_id} (expected {expected}, got {actual})")]
    AccountOwnerTypeMismatch { account_id: i64, expected: String, actual: String },

    #[error("owner_id required for user account {account_id}")]
    OwnerIdRequiredForUserAccount { account_id: i64 },

    #[error("hold posting ambiguous (found {available_accounts} available accounts and {locked_accounts} locked accounts)")]
    HoldPostingAmbiguous {  available_accounts: usize, locked_accounts: usize },

    #[error("hold must be between accounts of the same user (available_account_id={available_account_id}, locked_account_id={locked_account_id})")]
    HoldMustBeSameUser { available_account_id: i64, locked_account_id: i64 },

    #[error("money cannot be zero")]
    MoneyZeroNotAllowed,

}
