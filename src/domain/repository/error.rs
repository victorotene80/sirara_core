use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum RepoError {
    #[error("not found: {entity}")]
    NotFound { entity: String },

    #[error("conflict: {message}")]
    Conflict { message: String },

    #[error("integrity error: {message}")]
    Integrity { message: String },

    #[error("transient failure: {message}")]
    Transient { message: String },

    #[error("unexpected persistence error: {message}")]
    Unexpected { message: String },
}
