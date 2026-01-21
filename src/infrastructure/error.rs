#[derive(Debug, thiserror::Error)]
pub enum InfraError {
    #[error("database error")]
    Db(#[source] sqlx::Error),

    #[error("http error")]
    Http(#[source] reqwest::Error),

    #[error("configuration error")]
    Config(#[source] anyhow::Error),
}

impl From<sqlx::Error> for InfraError {
    fn from(e: sqlx::Error) -> Self { InfraError::Db(e) }
}
impl From<reqwest::Error> for InfraError {
    fn from(e: reqwest::Error) -> Self { InfraError::Http(e) }
}
impl From<anyhow::Error> for InfraError {
    fn from(e: anyhow::Error) -> Self { InfraError::Config(e) }
}
