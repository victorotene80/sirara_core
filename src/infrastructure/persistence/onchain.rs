use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::entities::OnchainTransaction;
use crate::domain::repository::{OnchainRepository, RepoError};
use crate::domain::value_objects::{OnchainTxStatus, PublicId};

use crate::infrastructure::persistence::error_map::map_sqlx;
use crate::infrastructure::persistence::models::OnchainTxRow;

pub struct PgOnchainRepository {
    pool: PgPool,
}

impl PgOnchainRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl OnchainRepository for PgOnchainRepository {
    async fn find_by_intent_db_id(
        &self,
        intent_db_id: i64,
    ) -> Result<Option<OnchainTransaction>, RepoError> {
        let row = sqlx::query_as::<_, OnchainTxRow>(
            r#"
            SELECT
              id, public_id,
              intent_id,
              chain, asset_code, region_code,
              contract_address, from_address, to_address,
              amount_minor,
              status, tx_hash, confirmations,
              idempotency_key,
              submit_attempts, last_submit_at, last_checked_at,
              failure_json,
              version, created_at, updated_at
            FROM onchain_transactions
            WHERE intent_id = $1
            LIMIT 1
            "#,
        )
            .bind(intent_db_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_sqlx)?;

        match row {
            None => Ok(None),
            Some(r) => Ok(Some(r.to_domain()?)),
        }
    }

    async fn find_by_public_id(
        &self,
        public_id: PublicId,
    ) -> Result<Option<OnchainTransaction>, RepoError> {
        let row = sqlx::query_as::<_, OnchainTxRow>(
            r#"
            SELECT
              id, public_id,
              intent_id,
              chain, asset_code, region_code,
              contract_address, from_address, to_address,
              amount_minor,
              status, tx_hash, confirmations,
              idempotency_key,
              submit_attempts, last_submit_at, last_checked_at,
              failure_json,
              version, created_at, updated_at
            FROM onchain_transactions
            WHERE public_id = $1
            LIMIT 1
            "#,
        )
            .bind(public_id.value())
            .fetch_optional(&self.pool)
            .await
            .map_err(map_sqlx)?;

        match row {
            None => Ok(None),
            Some(r) => Ok(Some(r.to_domain()?)),
        }
    }

    async fn list_by_status(
        &self,
        status: OnchainTxStatus,
        limit: i64,
    ) -> Result<Vec<OnchainTransaction>, RepoError> {
        let rows = sqlx::query_as::<_, OnchainTxRow>(
            r#"
            SELECT
              id, public_id,
              intent_id,
              chain, asset_code, region_code,
              contract_address, from_address, to_address,
              amount_minor,
              status, tx_hash, confirmations,
              idempotency_key,
              submit_attempts, last_submit_at, last_checked_at,
              failure_json,
              version, created_at, updated_at
            FROM onchain_transactions
            WHERE status = $1
            ORDER BY id ASC
            LIMIT $2
            "#,
        )
            .bind(status.as_str())
            .bind(limit)
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx)?;

        let mut out = Vec::with_capacity(rows.len());
        for r in rows {
            out.push(r.to_domain()?);
        }
        Ok(out)
    }
}
