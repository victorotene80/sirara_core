use std::future::Future;
use std::pin::Pin;
use sqlx::{Postgres, Transaction};
use crate::domain::repository::RepoError;

pub type BoxFut<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub trait UnitOfWork: Send + Sync {
    fn with_tx<'u, T: Send + 'u>(
        &'u self,
        f: impl for<'a> FnOnce(&'a mut Transaction<'_, Postgres>) -> BoxFut<'a, Result<T, RepoError>>
        + Send
        + 'u,
    ) -> BoxFut<'u, Result<T, RepoError>>;
}
