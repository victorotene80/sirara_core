use crate::application::AppError;
#[derive(Debug, thiserror::Error)]
pub enum InfraError {
    #[error("database error")]
    Db(#[source] sqlx::Error),

    #[error("http error")]
    Http(#[source] reqwest::Error),

    #[error("configuration error: {message}")]
    Config { message: String },

    #[error("integrity error: {message}")]
    Integrity { message: String },

    #[error("application error")]
    App(#[source] AppError),

    #[error("unexpected error: {message}")]
    Unexpected { message: String },
}

impl From<sqlx::Error> for InfraError {
    fn from(e: sqlx::Error) -> Self { InfraError::Db(e) }
}
impl From<reqwest::Error> for InfraError {
    fn from(e: reqwest::Error) -> Self { InfraError::Http(e) }
}

impl From<anyhow::Error> for InfraError {
    fn from(e: anyhow::Error) -> Self {
        InfraError::Config { message: e.to_string() }
    }
}
impl From<AppError> for InfraError {
    fn from(e: AppError) -> Self {
        InfraError::App(e)
    }
}

