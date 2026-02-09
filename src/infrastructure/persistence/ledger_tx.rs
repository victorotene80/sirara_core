use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::application::contracts::repository::{BoxFut, LedgerRepositoryTx};
use crate::domain::aggregate::{PostedJournal, ValidatedJournal};
use crate::domain::entities::LedgerAccount;
use crate::domain::repository::RepoError;
use crate::domain::value_objects::{AssetCode, RegionCode};
use crate::infrastructure::persistence::error_map::map_sqlx;
use crate::infrastructure::persistence::ledger::PgLedgerRepository;
use crate::infrastructure::persistence::models::{LedgerAccountRow};
use crate::infrastructure::persistence::mappers::numeric0_to_i128_strict;
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

    fn peek_balance_minor(&mut self, account_id: i64) -> BoxFut<'_, Result<i128, RepoError>> {
        Box::pin(async move {
            #[derive(sqlx::FromRow)]
            struct Row {
                balance: bigdecimal::BigDecimal,
            }

            let row = sqlx::query_as::<_, Row>(
                r#"
                SELECT balance
                FROM ledger_account_balances
                WHERE account_id = $1
                "#,
            )
                .bind(account_id)
                .fetch_one(&mut **self.tx)
                .await
                .map_err(map_sqlx)?;

            numeric0_to_i128_strict(&row.balance, account_id)
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
                SELECT id, public_id, owner_type, owner_id, account_type, asset_id, region_code, is_active
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
        asset_code: &AssetCode,
    ) -> BoxFut<'_, Result<(i64, i64), RepoError>> {
        let asset_code = asset_code.as_str().trim().to_uppercase();
        Box::pin(async move {
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

            let mut available = None;
            let mut locked = None;

            for r in rows {
                match r.account_type.as_str() {
                    "USER_AVAILABLE" => available = Some(r.id),
                    "USER_LOCKED" => locked = Some(r.id),
                    _ => {}
                }
            }

            Ok((
                available.ok_or_else(|| RepoError::NotFound {
                    entity: format!("USER_AVAILABLE user_id={user_id} asset={asset_code}"),
                })?,
                locked.ok_or_else(|| RepoError::NotFound {
                    entity: format!("USER_LOCKED user_id={user_id} asset={asset_code}"),
                })?,
            ))
        })
    }

    fn resolve_platform_inventory_accounts(
        &mut self,
        asset_code: &AssetCode,
        region_code: &RegionCode,
    ) -> BoxFut<'_, Result<(i64, i64), RepoError>> {
        let asset_code = asset_code.as_str().trim().to_uppercase();
        let region_code = region_code.as_str().trim().to_uppercase();

        Box::pin(async move {
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
                WHERE la.owner_type = 'PLATFORM'
                  AND la.owner_id IS NULL
                  AND a.code = $1
                  AND la.region_code = $2
                  AND la.account_type IN ('INVENTORY_AVAILABLE','INVENTORY_LOCKED')
                  AND la.is_active = true
                "#,
            )
                .bind(&asset_code)
                .bind(&region_code)
                .fetch_all(&mut **self.tx)
                .await
                .map_err(map_sqlx)?;

            let mut available: Option<i64> = None;
            let mut locked: Option<i64> = None;

            for r in rows {
                match r.account_type.as_str() {
                    "INVENTORY_AVAILABLE" => available = Some(r.id),
                    "INVENTORY_LOCKED" => locked = Some(r.id),
                    _ => {}
                }
            }

            let a = available.ok_or_else(|| RepoError::NotFound {
                entity: format!("INVENTORY_AVAILABLE asset={asset_code} region={region_code}"),
            })?;
            let l = locked.ok_or_else(|| RepoError::NotFound {
                entity: format!("INVENTORY_LOCKED asset={asset_code} region={region_code}"),
            })?;

            Ok((a, l))
        })
    }

    fn resolve_user_available_accounts(
        &mut self,
        from_user: Uuid,
        to_user: Uuid,
        asset_code: &AssetCode,
    ) -> BoxFut<'_, Result<(LedgerAccount, LedgerAccount), RepoError>> {
        let asset_code = asset_code.as_str().trim().to_uppercase();

        Box::pin(async move {
            let rows = sqlx::query_as::<_, LedgerAccountRow>(
                r#"
            SELECT la.id, la.public_id, la.owner_type, la.owner_id,
                   la.account_type, la.asset_id, la.region_code, la.is_active
            FROM ledger_accounts la
            JOIN assets a ON a.id = la.asset_id
            WHERE la.owner_type = 'USER'
              AND la.owner_id = ANY($1)
              AND la.account_type = 'USER_AVAILABLE'
              AND la.is_active = true
              AND la.region_code IS NULL
              AND a.code = $2
            "#,
            )
                .bind(&[from_user, to_user])
                .bind(&asset_code)
                .fetch_all(&mut **self.tx)
                .await
                .map_err(map_sqlx)?;

            let mut from_acc: Option<LedgerAccountRow> = None;
            let mut to_acc: Option<LedgerAccountRow> = None;

            for r in rows {
                if r.owner_id == Some(from_user) {
                    from_acc = Some(r);
                } else if r.owner_id == Some(to_user) {
                    to_acc = Some(r);
                }
            }

            let from_acc = from_acc.ok_or_else(|| RepoError::NotFound {
                entity: format!("USER_AVAILABLE user_id={from_user} asset={asset_code}"),
            })?;

            let to_acc = to_acc.ok_or_else(|| RepoError::NotFound {
                entity: format!("USER_AVAILABLE user_id={to_user} asset={asset_code}"),
            })?;

            Ok((from_acc.to_domain()?, to_acc.to_domain()?))
        })
    }

}
