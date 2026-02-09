use chrono::{DateTime, Utc};
use sqlx::{Postgres, Transaction};

use crate::application::contracts::repository::{BoxFut, OnchainRepositoryTx};
use crate::domain::entities::OnchainTransaction;
use crate::domain::repository::RepoError;
use crate::domain::value_objects::{FailureInfo, OnchainTxStatus, PublicId, TxHash};

use crate::infrastructure::persistence::error_map::map_sqlx;
use crate::infrastructure::persistence::mappers::{i128_to_bd};
use crate::infrastructure::persistence::models::OnchainTxRow;
use crate::infrastructure::persistence::mappers::failure_info::{from_json, to_json};

pub struct PgOnchainTxRepo<'a, 'c> {
    tx: &'a mut Transaction<'c, Postgres>,
}

impl<'a, 'c> PgOnchainTxRepo<'a, 'c> {
    pub fn new(tx: &'a mut Transaction<'c, Postgres>) -> Self {
        Self { tx }
    }


    async fn cas_bump_version_or_conflict(
        tx: &mut Transaction<'_, Postgres>,
        intent_id: i64,
        expected_version: i32,
        sql: &str,
        binds: impl FnOnce(sqlx::query::Query<'_, Postgres, sqlx::postgres::PgArguments>)
            -> sqlx::query::Query<'_, Postgres, sqlx::postgres::PgArguments>,
    ) -> Result<i32, RepoError> {
        let q = sqlx::query(sql).bind(intent_id).bind(expected_version);
        let q = binds(q);

        let res = q.execute(&mut **tx).await.map_err(map_sqlx)?;
        let affected = res.rows_affected() as i32;

        if affected == 1 {
            return Ok(expected_version + 1);
        }

        let exists = sqlx::query_scalar::<_, bool>(
            r#"SELECT EXISTS (SELECT 1 FROM onchain_transactions WHERE intent_id = $1)"#,
        )
            .bind(intent_id)
            .fetch_one(&mut **tx)
            .await
            .map_err(map_sqlx)?;

        if !exists {
            return Err(RepoError::NotFound {
                entity: format!("onchain_transaction intent_id={}", intent_id),
            });
        }

        Err(RepoError::Conflict {
            message: format!(
                "onchain_transaction version conflict (intent_id={}, expected_version={})",
                intent_id, expected_version
            ),
        })
    }
}

impl<'a, 'c> OnchainRepositoryTx for PgOnchainTxRepo<'a, 'c> {
    fn insert_if_absent(
        &mut self,
        tx_entity: OnchainTransaction,
    ) -> BoxFut<'_, Result<OnchainTransaction, RepoError>> {
        Box::pin(async move {
            let intent_id = tx_entity.intent_db_id();
            let chain_s = tx_entity.chain().as_str().to_string();
            let idemp = tx_entity.idempotency_key().to_string();

            let insert_res = sqlx::query_as::<_, OnchainTxRow>(
                r#"
                INSERT INTO onchain_transactions
                  (public_id, intent_id,
                   chain, asset_code, region_code,
                   contract_address, from_address, to_address,
                   amount_minor,
                   status, tx_hash, confirmations,
                   idempotency_key,
                   submit_attempts, last_submit_at, last_checked_at,
                   failure_json, version)
                VALUES
                  ($1, $2,
                   $3, $4, $5,
                   $6, $7, $8,
                   $9,
                   $10, $11, $12,
                   $13,
                   $14, $15, $16,
                   $17, $18)
                RETURNING
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
                "#,
            )
                .bind(tx_entity.public_id().value())
                .bind(intent_id)
                .bind(tx_entity.chain().as_str())
                .bind(tx_entity.asset_code().as_str().trim().to_uppercase())
                .bind(tx_entity.region_code().as_str().trim().to_uppercase())
                .bind(tx_entity.contract_address().as_str())
                .bind(tx_entity.from_address().as_str())
                .bind(tx_entity.to_address().as_str())
                .bind(i128_to_bd(tx_entity.amount().minor()))
                .bind(tx_entity.status().as_str())
                .bind(tx_entity.tx_hash().map(|h| h.as_str().to_string()))
                .bind(tx_entity.confirmations())
                .bind(idemp.clone())
                .bind(tx_entity.submit_attempts())
                .bind(tx_entity.last_submit_at().cloned())
                .bind(tx_entity.last_checked_at().cloned())
                .bind(tx_entity.failure().map(|f| to_json(f)))
                .bind(tx_entity.version())
                .fetch_one(&mut **self.tx)
                .await;

            if let Ok(row) = insert_res {
                return row.to_domain();
            }

            if let Some(existing) = sqlx::query_as::<_, OnchainTxRow>(
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
                FOR UPDATE
                "#,
            )
                .bind(intent_id)
                .fetch_optional(&mut **self.tx)
                .await
                .map_err(map_sqlx)?
            {
                return existing.to_domain();
            }

            // 2) Otherwise, check idempotency conflict (chain, idempotency_key)
            if let Some(existing) = sqlx::query_as::<_, OnchainTxRow>(
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
                WHERE chain = $1
                  AND idempotency_key = $2
                LIMIT 1
                "#,
            )
                .bind(&chain_s)
                .bind(&idemp)
                .fetch_optional(&mut **self.tx)
                .await
                .map_err(map_sqlx)?
            {
                return Err(RepoError::Conflict {
                    message: format!(
                        "idempotency conflict: chain={} idempotency_key={} already used (existing intent_id={})",
                        existing.chain, existing.idempotency_key, existing.intent_id
                    ),
                });
            }

            Err(map_sqlx(insert_res.err().unwrap()))
        })
    }

    fn find_by_intent_id_for_update(
        &mut self,
        intent_db_id: i64,
    ) -> BoxFut<'_, Result<Option<OnchainTransaction>, RepoError>> {
        Box::pin(async move {
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
                FOR UPDATE
                "#,
            )
                .bind(intent_db_id)
                .fetch_optional(&mut **self.tx)
                .await
                .map_err(map_sqlx)?;

            Ok(row.map(|r| r.to_domain()).transpose()?)
        })
    }

    fn find_by_public_id_for_update(
        &mut self,
        public_id: PublicId,
    ) -> BoxFut<'_, Result<Option<OnchainTransaction>, RepoError>> {
        Box::pin(async move {
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
                FOR UPDATE
                "#,
            )
                .bind(public_id.value())
                .fetch_optional(&mut **self.tx)
                .await
                .map_err(map_sqlx)?;

            Ok(row.map(|r| r.to_domain()).transpose()?)
        })
    }

    fn mark_submitted(
        &mut self,
        intent_db_id: i64,
        expected_version: i32,
        tx_hash: TxHash,
        now: DateTime<Utc>,
    ) -> BoxFut<'_, Result<i32, RepoError>> {
        let tx_hash_s = tx_hash.as_str().to_string();
        Box::pin(async move {
            Self::cas_bump_version_or_conflict(
                self.tx,
                intent_db_id,
                expected_version,
                r#"
                UPDATE onchain_transactions
                SET
                  status = 'SUBMITTED',
                  tx_hash = $3,
                  submit_attempts = submit_attempts + 1,
                  last_submit_at = $4,
                  last_checked_at = $4,
                  version = version + 1,
                  updated_at = $4
                WHERE intent_id = $1
                  AND version = $2
                "#,
                |q| q.bind(tx_hash_s).bind(now),
            )
                .await
        })
    }

    fn update_confirmations(
        &mut self,
        intent_db_id: i64,
        expected_version: i32,
        confirmations: i32,
        now: DateTime<Utc>,
    ) -> BoxFut<'_, Result<i32, RepoError>> {
        Box::pin(async move {
            Self::cas_bump_version_or_conflict(
                self.tx,
                intent_db_id,
                expected_version,
                r#"
                UPDATE onchain_transactions
                SET
                  confirmations = $3,
                  last_checked_at = $4,
                  version = version + 1,
                  updated_at = $4
                WHERE intent_id = $1
                  AND version = $2
                "#,
                |q| q.bind(confirmations).bind(now),
            )
                .await
        })
    }

    fn mark_confirmed(
        &mut self,
        intent_db_id: i64,
        expected_version: i32,
        now: DateTime<Utc>,
    ) -> BoxFut<'_, Result<i32, RepoError>> {
        Box::pin(async move {
            Self::cas_bump_version_or_conflict(
                self.tx,
                intent_db_id,
                expected_version,
                r#"
                UPDATE onchain_transactions
                SET
                  status = 'CONFIRMED',
                  last_checked_at = $3,
                  version = version + 1,
                  updated_at = $3
                WHERE intent_id = $1
                  AND version = $2
                "#,
                |q| q.bind(now),
            )
                .await
        })
    }

    fn mark_failed(
        &mut self,
        intent_db_id: i64,
        expected_version: i32,
        failure: FailureInfo,
        now: DateTime<Utc>,
    ) -> BoxFut<'_, Result<i32, RepoError>> {
        let failure_json = to_json(&failure);
        Box::pin(async move {
            Self::cas_bump_version_or_conflict(
                self.tx,
                intent_db_id,
                expected_version,
                r#"
                UPDATE onchain_transactions
                SET
                  status = 'FAILED',
                  failure_json = $3,
                  last_checked_at = $4,
                  version = version + 1,
                  updated_at = $4
                WHERE intent_id = $1
                  AND version = $2
                "#,
                |q| q.bind(failure_json).bind(now),
            )
                .await
        })
    }

    fn list_submitted_due_for_check(
        &mut self,
        limit: i64,
        now: DateTime<Utc>,
    ) -> BoxFut<'_, Result<Vec<OnchainTransaction>, RepoError>> {
        const RECHECK_CONF0_SECS: i64 = 5;
        const RECHECK_CONF1PLUS_SECS: i64 = 8;

        Box::pin(async move {
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
                WHERE status = 'SUBMITTED'
                  AND $2 >= (
                    COALESCE(last_checked_at, last_submit_at, created_at)
                    + (
                        CASE
                          WHEN confirmations = 0 THEN make_interval(secs => $3)
                          ELSE make_interval(secs => $4)
                        END
                      )
                  )
                ORDER BY COALESCE(last_checked_at, last_submit_at, created_at) ASC
                FOR UPDATE SKIP LOCKED
                LIMIT $1
                "#,
            )
                .bind(limit)
                .bind(now)
                .bind(RECHECK_CONF0_SECS)
                .bind(RECHECK_CONF1PLUS_SECS)
                .fetch_all(&mut **self.tx)
                .await
                .map_err(map_sqlx)?;

            let mut out = Vec::with_capacity(rows.len());
            for r in rows {
                out.push(r.to_domain()?);
            }
            Ok(out)
        })
    }

    fn list_by_status(
        &mut self,
        status: OnchainTxStatus,
        limit: i64,
    ) -> BoxFut<'_, Result<Vec<OnchainTransaction>, RepoError>> {
        Box::pin(async move {
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
                .fetch_all(&mut **self.tx)
                .await
                .map_err(map_sqlx)?;

            let mut out = Vec::with_capacity(rows.len());
            for r in rows {
                out.push(r.to_domain()?);
            }
            Ok(out)
        })
    }
}
