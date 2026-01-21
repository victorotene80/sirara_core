use sqlx::{postgres::PgPoolOptions, PgPool, Postgres, Transaction};

use crate::infrastructure::error::InfraError;
use crate::utils::configuration::DatabaseConfig;

#[derive(Clone)]
pub struct Db {
    pool: PgPool,
}

impl Db {
    pub async fn new(cfg: &DatabaseConfig) -> Result<Self, InfraError> {
        let pool = PgPoolOptions::new()
            .max_connections(cfg.max_connections)
            .min_connections(cfg.min_connections)
            .acquire_timeout(cfg.acquire_timeout)
            .connect(&cfg.database_url)
            .await?;

        Ok(Self { pool })
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn begin(&self) -> Result<Transaction<'_, Postgres>, InfraError> {
        Ok(self.pool.begin().await?)
    }
}
