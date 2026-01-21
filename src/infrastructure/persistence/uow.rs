use sqlx::{PgPool, Postgres, Transaction};
use crate::domain::repository::RepoError;
use crate::infrastructure::persistence::error_map::map_sqlx;
use crate::application::contracts::repository::uow::{UnitOfWork, BoxFut};

pub struct PgUnitOfWork {
    pool: PgPool,
}

impl PgUnitOfWork {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl UnitOfWork for PgUnitOfWork {
    fn with_tx<'u, T: Send + 'u>(
        &'u self,
        f: impl for<'a> FnOnce(&'a mut Transaction<'_, Postgres>) -> BoxFut<'a, Result<T, RepoError>>
        + Send
        + 'u,
    ) -> BoxFut<'u, Result<T, RepoError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(map_sqlx)?;
            let res = f(&mut tx).await;

            match res {
                Ok(v) => {
                    tx.commit().await.map_err(map_sqlx)?;
                    Ok(v)
                }
                Err(e) => {
                    tx.rollback().await.map_err(map_sqlx)?;
                    Err(e)
                }
            }
        })
    }
}
