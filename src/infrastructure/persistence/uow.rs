use sqlx::PgPool;

use crate::application::AppError;
use crate::application::contracts::repository::{BoxFut, ReposInTx, UnitOfWork};
use crate::infrastructure::persistence::error_map::map_sqlx;
use crate::infrastructure::persistence::repos_in_tx::PgReposInTx;

pub struct PgUnitOfWork {
    pool: PgPool,
}

impl PgUnitOfWork {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl UnitOfWork for PgUnitOfWork {
    fn with_tx<'a, T>(
        &'a self,
        f: impl for<'tx> FnOnce(&'tx mut dyn ReposInTx) -> BoxFut<'tx, Result<T, AppError>>
        + Send
        + 'a,
    ) -> BoxFut<'a, Result<T, AppError>>
    where
        T: Send + 'a,
    {
        Box::pin(async move {
            let mut tx = self
                .pool
                .begin()
                .await
                .map_err(map_sqlx)
                .map_err(AppError::from)?;

            let res: Result<T, AppError> = {
                let mut repos = PgReposInTx::new(&mut tx);
                f(&mut repos).await
            };

            match res {
                Ok(v) => {
                    tx.commit()
                        .await
                        .map_err(map_sqlx)
                        .map_err(AppError::from)?;
                    Ok(v)
                }
                Err(e) => {
                    tx.rollback()
                        .await
                        .map_err(map_sqlx)
                        .map_err(AppError::from)?;
                    Err(e)
                }
            }
        })
    }
}
