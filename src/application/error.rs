use thiserror::Error;

use crate::domain::error::DomainError;
use crate::domain::repository::RepoError;

#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    Domain(#[from] DomainError),

    #[error(transparent)]
    Repo(#[from] RepoError),

    #[error(transparent)]
    Uuid(#[from] uuid::Error),

    #[error("query repository error")]
    QueryRepo {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync + 'static>,
    },

    #[error("invalid request: {message}")]
    InvalidRequest { message: String },

    #[error("entity not found: {entity}")]
    NotFound { entity: String },

    #[error("transient error: {0}")]
    Transient(String),

    #[error("rejected: {0}")]
    Rejected(String),

    #[error("unexpected error: {0}")]
    Unexpected(String),

    #[error("conflict: {message}")]
    Conflict { message: String },

    #[error("invariant violated: {message}")]
    Invariant { message: String },

    #[error("insufficient funds (available_minor={available_minor}, required_minor={required_minor})")]
    InsufficientFunds {
        available_minor: i128,
        required_minor: i128,
    },
}
