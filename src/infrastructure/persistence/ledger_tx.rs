use sqlx::{Postgres, Transaction};

use crate::application::contracts::repository::{BoxFut, LedgerRepositoryTx};
use crate::domain::aggregate::{PostedJournal, ValidatedJournal};
use crate::domain::repository::RepoError;
use crate::domain::entities::LedgerAccount;
use crate::infrastructure::persistence::models::LedgerAccountRow;
use crate::infrastructure::persistence::error_map::map_sqlx;
use crate::infrastructure::persistence::ledger::PgLedgerRepository;
use uuid::Uuid;
pub struct PgLedgerTxRepo<'a, 'c> {
    tx: &'a mut Transaction<'c, Postgres>,
}

impl<'a, 'c> PgLedgerTxRepo<'a, 'c> {
    pub fn new(tx: &'a mut Transaction<'c, Postgres>) -> Self {
        Self { tx }
    }
}

impl<'a, 'c> LedgerRepositoryTx for PgLedgerTxRepo<'a, 'c> {
    fn insert_posting_atomic(
        &mut self,
        posting: ValidatedJournal,
    ) -> BoxFut<'_, Result<PostedJournal, RepoError>> {
        Box::pin(async move {
            PgLedgerRepository::insert_posting_atomic_on_tx(self.tx, posting).await
        })
    }

    fn get_accounts_by_ids_for_validation(
        &mut self,
        ids: &[i64],
    ) -> BoxFut<'_, Result<Vec<LedgerAccount>, RepoError>> {
        let ids = ids.to_vec();

        Box::pin(async move {
            if ids.is_empty() {
                return Ok(vec![]);
            }

            let rows = sqlx::query_as::<_, LedgerAccountRow>(
                r#"
            SELECT id, public_id, owner_type, owner_id, account_type, asset_id, is_active
            FROM ledger_accounts
            WHERE id = ANY($1)
            ORDER BY id
            "#,
            )
                .bind(&ids)
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

    fn resolve_user_hold_accounts(
        &mut self,
        user_id: Uuid,
        asset_code: String,
    ) -> BoxFut<'_, Result<(i64, i64), RepoError>> {
        let asset_code = asset_code.trim().to_uppercase();

        Box::pin(async move {
            // Get both USER_AVAILABLE and USER_LOCKED for (user_id, asset_code)
            #[derive(sqlx::FromRow)]
            struct Row {
                id: i64,
                account_type: String,
            }

            let rows = sqlx::query_as::<_, Row>(
                r#"
                SELECT la.id, la.account_type
                FROM ledger_accounts la
                JOIN assets a ON a.id = la.asset_id
                WHERE la.owner_type = 'USER'
                  AND la.owner_id = $1
                  AND a.code = $2
                  AND la.account_type IN ('USER_AVAILABLE','USER_LOCKED')
                  AND la.is_active = true
                "#,
            )
                .bind(user_id)
                .bind(&asset_code)
                .fetch_all(&mut **self.tx)
                .await
                .map_err(map_sqlx)?;

            let mut available: Option<i64> = None;
            let mut locked: Option<i64> = None;

            for r in rows {
                match r.account_type.as_str() {
                    "USER_AVAILABLE" => available = Some(r.id),
                    "USER_LOCKED" => locked = Some(r.id),
                    _ => {}
                }
            }

            let a = available.ok_or_else(|| RepoError::NotFound {
                entity: format!("USER_AVAILABLE for user_id={user_id} asset={asset_code}"),
            })?;
            let l = locked.ok_or_else(|| RepoError::NotFound {
                entity: format!("USER_LOCKED for user_id={user_id} asset={asset_code}"),
            })?;

            Ok((a, l))
        })
    }
}
