use chrono::{DateTime, Utc};
use sqlx::{Postgres, Transaction};

use crate::application::contracts::repository::{BoxFut, OutboxRepositoryTx};
use crate::domain::repository::RepoError;
use crate::infrastructure::persistence::error_map::map_sqlx;
use crate::application::contracts::repository::OutboxMessage;

pub struct PgOutboxTxRepo<'a, 'c> {
    tx: &'a mut Transaction<'c, Postgres>,
}

impl<'a, 'c> PgOutboxTxRepo<'a, 'c> {
    pub fn new(tx: &'a mut Transaction<'c, Postgres>) -> Self {
        Self { tx }
    }
}

impl<'a, 'c> OutboxRepositoryTx for PgOutboxTxRepo<'a, 'c> {
    fn enqueue(&mut self, topic: &str, key: &str, payload: serde_json::Value) -> BoxFut<'_, Result<(), RepoError>> {
        let topic = topic.to_string();
        let key = key.to_string();

        Box::pin(async move {
            sqlx::query(
                r#"
                INSERT INTO outbox_messages (topic, key, payload_json, status, attempts, next_attempt_at, created_at, updated_at)
                VALUES ($1, $2, $3, 'PENDING', 0, now(), now(), now())
                ON CONFLICT (topic, key) DO NOTHING
                "#,
            )
                .bind(&topic)
                .bind(&key)
                .bind(payload)
                .execute(&mut **self.tx)
                .await
                .map_err(map_sqlx)?;

            Ok(())
        })
    }

    fn claim_due(&mut self, batch: i64) -> BoxFut<'_, Result<Vec<OutboxMessage>, RepoError>> {
        Box::pin(async move {
            // Claim messages safely across multiple workers:
            // - pick due PENDING rows
            // - lock with SKIP LOCKED
            // - mark PROCESSING and increment attempts
            // - return claimed rows
            let rows = sqlx::query_as::<_, (i64, String, String, serde_json::Value, i32)>(
                r#"
                WITH cte AS (
                    SELECT id
                    FROM outbox_messages
                    WHERE status = 'PENDING'
                      AND next_attempt_at <= now()
                    ORDER BY next_attempt_at ASC, id ASC
                    LIMIT $1
                    FOR UPDATE SKIP LOCKED
                )
                UPDATE outbox_messages m
                SET status = 'PROCESSING',
                    attempts = m.attempts + 1,
                    updated_at = now()
                FROM cte
                WHERE m.id = cte.id
                RETURNING m.id, m.topic, m.key, m.payload_json, m.attempts
                "#,
            )
                .bind(batch)
                .fetch_all(&mut **self.tx)
                .await
                .map_err(map_sqlx)?;

            let mut out = Vec::with_capacity(rows.len());
            for (id, topic, key, payload_json, attempts) in rows {
                out.push(OutboxMessage { id, topic, key, payload_json, attempts });
            }
            Ok(out)
        })
    }

    fn mark_sent(&mut self, id: i64) -> BoxFut<'_, Result<(), RepoError>> {
        Box::pin(async move {
            let n = sqlx::query(
                r#"
                UPDATE outbox_messages
                SET status = 'SENT',
                    updated_at = now()
                WHERE id = $1
                "#,
            )
                .bind(id)
                .execute(&mut **self.tx)
                .await
                .map_err(map_sqlx)?
                .rows_affected();

            if n == 0 {
                return Err(RepoError::NotFound { entity: format!("outbox_message id={id}") });
            }

            Ok(())
        })
    }

    fn mark_failed_retry(
        &mut self,
        id: i64,
        err: String,
        next_attempt_at: DateTime<Utc>,
    ) -> BoxFut<'_, Result<(), RepoError>> {
        Box::pin(async move {
            let n = sqlx::query(
                r#"
                UPDATE outbox_messages
                SET status = 'PENDING',
                    next_attempt_at = $2,
                    last_error = $3,
                    updated_at = now()
                WHERE id = $1
                "#,
            )
                .bind(id)
                .bind(next_attempt_at)
                .bind(err)
                .execute(&mut **self.tx)
                .await
                .map_err(map_sqlx)?
                .rows_affected();

            if n == 0 {
                return Err(RepoError::NotFound { entity: format!("outbox_message id={id}") });
            }

            Ok(())
        })
    }
}
