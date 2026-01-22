use std::collections::HashMap;
use async_trait::async_trait;
use bigdecimal::BigDecimal;
use sqlx::{PgPool, Postgres, Transaction};

use crate::domain::aggregate::{PostedJournal, ValidatedJournal};
use crate::domain::entities::{AccountType, LedgerAccount};
use crate::domain::repository::{LedgerRepository, NewLedgerAccountSpec, RepoError};
use crate::domain::value_objects::{ExternalRef, ExternalRefType};

use crate::infrastructure::persistence::error_map::map_sqlx;
use crate::infrastructure::persistence::mappers::{i128_to_bigdecimal, map_posted_journal};
use crate::infrastructure::persistence::models::{JournalLineRow, JournalTxRow, LedgerAccountRow};
use crate::application::contracts::repository::LedgerRepositoryTx;

//type BoxFut<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub struct PgLedgerRepository {
    pool: PgPool,
}

impl PgLedgerRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    /*async fn with_tx<T>(
        &self,
        f: impl for<'a> FnOnce(&'a mut Transaction<'_, Postgres>) -> BoxFut<'a, Result<T, RepoError>>
        + Send,
    ) -> Result<T, RepoError> {
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
    }*/

    fn is_spendable_bucket(t: AccountType) -> bool {
        matches!(
            t,
            AccountType::UserAvailable | AccountType::TreasuryAvailable | AccountType::InventoryAvailable
        )
    }

    async fn fetch_accounts_for_update(
        tx: &mut Transaction<'_, Postgres>,
        account_ids: &[i64],
    ) -> Result<Vec<LedgerAccountRow>, RepoError> {
        if account_ids.is_empty() {
            return Ok(vec![]);
        }

        let rows = sqlx::query_as::<_, LedgerAccountRow>(
            r#"
            SELECT id, public_id, owner_type, owner_id, account_type, asset_id, is_active
            FROM ledger_accounts
            WHERE id = ANY($1)
            ORDER BY id
            FOR UPDATE
            "#,
        )
            .bind(account_ids)
            .fetch_all(&mut **tx)
            .await
            .map_err(map_sqlx)?;

        Ok(rows)
    }

    async fn insert_or_get_tx_id(
        tx: &mut Transaction<'_, Postgres>,
        posting: &ValidatedJournal,
    ) -> Result<i64, RepoError> {
        let inserted = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO journal_transactions
                (public_id, external_ref, external_ref_type, description, created_by)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (external_ref_type, external_ref) DO NOTHING
            RETURNING id
            "#,
        )
            .bind(posting.public_id.value())
            .bind(posting.external_ref.as_str())
            .bind(posting.external_ref_type.as_code())
            .bind(&posting.description)
            .bind(&posting.created_by)
            .fetch_optional(&mut **tx)
            .await
            .map_err(map_sqlx)?;

        if let Some(id) = inserted {
            return Ok(id);
        }

        let existing = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT id
            FROM journal_transactions
            WHERE external_ref_type = $1 AND external_ref = $2
            "#,
        )
            .bind(posting.external_ref_type.as_code())
            .bind(posting.external_ref.as_str())
            .fetch_one(&mut **tx)
            .await
            .map_err(map_sqlx)?;

        Ok(existing)
    }

    async fn tx_has_lines(tx: &mut Transaction<'_, Postgres>, tx_id: i64) -> Result<bool, RepoError> {
        let has = sqlx::query_scalar::<_, bool>(
            r#"SELECT EXISTS (SELECT 1 FROM journal_lines WHERE journal_tx_id = $1)"#,
        )
            .bind(tx_id)
            .fetch_one(&mut **tx)
            .await
            .map_err(map_sqlx)?;

        Ok(has)
    }

    async fn load_posted_by_tx_id_tx(
        tx: &mut Transaction<'_, Postgres>,
        tx_id: i64,
    ) -> Result<PostedJournal, RepoError> {
        let header = sqlx::query_as::<_, JournalTxRow>(
            r#"
            SELECT id, public_id, external_ref_type, external_ref, description, created_by
            FROM journal_transactions
            WHERE id = $1
            "#,
        )
            .bind(tx_id)
            .fetch_one(&mut **tx)
            .await
            .map_err(map_sqlx)?;

        let lines = sqlx::query_as::<_, JournalLineRow>(
            r#"
            SELECT account_id, amount
            FROM journal_lines
            WHERE journal_tx_id = $1
            ORDER BY id ASC
            "#,
        )
            .bind(tx_id)
            .fetch_all(&mut **tx)
            .await
            .map_err(map_sqlx)?;

        let asset_id = if let Some(first) = lines.first() {
            sqlx::query_scalar::<_, i16>("SELECT asset_id FROM ledger_accounts WHERE id = $1")
                .bind(first.account_id)
                .fetch_one(&mut **tx)
                .await
                .map_err(map_sqlx)?
        } else {
            0
        };

        map_posted_journal(header, lines, asset_id)
    }

    async fn load_posted_by_tx_id(pool: &PgPool, tx_id: i64) -> Result<PostedJournal, RepoError> {
        let header = sqlx::query_as::<_, JournalTxRow>(
            r#"
            SELECT id, public_id, external_ref_type, external_ref, description, created_by
            FROM journal_transactions
            WHERE id = $1
            "#,
        )
            .bind(tx_id)
            .fetch_one(pool)
            .await
            .map_err(map_sqlx)?;

        let lines = sqlx::query_as::<_, JournalLineRow>(
            r#"
            SELECT account_id, amount
            FROM journal_lines
            WHERE journal_tx_id = $1
            ORDER BY id ASC
            "#,
        )
            .bind(tx_id)
            .fetch_all(pool)
            .await
            .map_err(map_sqlx)?;

        let asset_id = if let Some(first) = lines.first() {
            sqlx::query_scalar::<_, i16>("SELECT asset_id FROM ledger_accounts WHERE id = $1")
                .bind(first.account_id)
                .fetch_one(pool)
                .await
                .map_err(map_sqlx)?
        } else {
            0
        };

        map_posted_journal(header, lines, asset_id)
    }

    async fn bulk_insert_lines(
        tx: &mut Transaction<'_, Postgres>,
        tx_id: i64,
        account_ids: &[i64],
        amounts: &[BigDecimal],
    ) -> Result<u64, RepoError> {
        let res = sqlx::query(
            r#"
        INSERT INTO journal_lines (journal_tx_id, account_id, amount)
        SELECT $1, x.account_id, x.amount
        FROM UNNEST($2::bigint[], $3::numeric[]) AS x(account_id, amount)
        ON CONFLICT (journal_tx_id, account_id) DO NOTHING
        "#,
        )
            .bind(tx_id)
            .bind(account_ids)
            .bind(amounts)
            .execute(&mut **tx)
            .await
            .map_err(map_sqlx)?;

        Ok(res.rows_affected())
    }


    fn numeric0_to_i128_strict(v: &BigDecimal, account_id: i64) -> Result<i128, RepoError> {
        let s = v.to_string();
        if s.contains('.') {
            return Err(RepoError::Integrity {
                message: format!("non-integer numeric found (account_id={account_id})"),
            });
        }
        s.parse::<i128>().map_err(|_| RepoError::Integrity {
            message: format!("numeric out of i128 range (account_id={account_id})"),
        })
    }

    async fn lock_and_fetch_balances(
        tx: &mut Transaction<'_, Postgres>,
        account_ids: &[i64],
    ) -> Result<HashMap<i64, i128>, RepoError> {
        #[derive(sqlx::FromRow)]
        struct BalRow {
            account_id: i64,
            balance: BigDecimal,
        }

        if account_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let rows = sqlx::query_as::<_, BalRow>(
            r#"
            SELECT account_id, balance
            FROM ledger_account_balances
            WHERE account_id = ANY($1)
            ORDER BY account_id
            FOR UPDATE
            "#,
        )
            .bind(account_ids)
            .fetch_all(&mut **tx)
            .await
            .map_err(map_sqlx)?;

        if rows.len() != account_ids.len() {
            return Err(RepoError::Integrity {
                message: "missing balance row for one or more accounts (ensure trigger or insert row on account create)"
                    .into(),
            });
        }

        let mut out = HashMap::with_capacity(account_ids.len());
        for r in rows {
            let val = Self::numeric0_to_i128_strict(&r.balance, r.account_id)?;
            out.insert(r.account_id, val);
        }
        Ok(out)
    }

    async fn ensure_single_asset(
        tx: &mut Transaction<'_, Postgres>,
        account_ids: &[i64],
    ) -> Result<i16, RepoError> {
        if account_ids.is_empty() {
            return Err(RepoError::Integrity {
                message: "posting has no accounts".into(),
            });
        }

        let distinct = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(DISTINCT asset_id)
            FROM ledger_accounts
            WHERE id = ANY($1)
            "#,
        )
            .bind(account_ids)
            .fetch_one(&mut **tx)
            .await
            .map_err(map_sqlx)?;

        if distinct != 1 {
            return Err(RepoError::Integrity {
                message: "posting spans multiple assets; split into separate journals per asset".into(),
            });
        }

        let asset_id = sqlx::query_scalar::<_, i16>(
            r#"
            SELECT asset_id
            FROM ledger_accounts
            WHERE id = ANY($1)
            ORDER BY id
            LIMIT 1
            "#,
        )
            .bind(account_ids)
            .fetch_one(&mut **tx)
            .await
            .map_err(map_sqlx)?;

        Ok(asset_id)
    }
    async fn apply_balance_deltas(
        tx: &mut Transaction<'_, Postgres>,
        delta: &HashMap<i64, i128>,
    ) -> Result<(), RepoError> {
        if delta.is_empty() {
            return Ok(());
        }

        let mut ids: Vec<i64> = Vec::with_capacity(delta.len());
        let mut deltas: Vec<BigDecimal> = Vec::with_capacity(delta.len());

        for (id, d) in delta {
            ids.push(*id);
            deltas.push(i128_to_bigdecimal(*d));
        }

        let res = sqlx::query(
            r#"
        UPDATE ledger_account_balances b
        SET balance = b.balance + x.delta,
            updated_at = now()
        FROM UNNEST($1::bigint[], $2::numeric[]) AS x(account_id, delta)
        WHERE b.account_id = x.account_id
        "#,
        )
            .bind(&ids)
            .bind(&deltas)
            .execute(&mut **tx)
            .await
            .map_err(map_sqlx)?;

        let expected = ids.len() as u64;
        let actual = res.rows_affected();

        if actual != expected {
            return Err(RepoError::Integrity {
                message: format!(
                    "balance update mismatch: expected to update {expected} rows, updated {actual}. \
                 missing balance rows or type mismatch? account_ids={ids:?}"
                ),
            });
        }

        Ok(())
    }
}
    #[async_trait]
impl LedgerRepository for PgLedgerRepository {
    async fn create_account(&self, spec: NewLedgerAccountSpec) -> Result<LedgerAccount, RepoError> {
        let row = sqlx::query_as::<_, LedgerAccountRow>(
            r#"
            INSERT INTO ledger_accounts
                (public_id, owner_type, owner_id, account_type, asset_id, is_active)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, public_id, owner_type, owner_id, account_type, asset_id, is_active
            "#,
        )
            .bind(spec.public_id.value())
            .bind(match spec.owner_type {
                crate::domain::entities::OwnerType::User => "USER",
                crate::domain::entities::OwnerType::Platform => "PLATFORM",
                crate::domain::entities::OwnerType::Treasury => "TREASURY",
            })
            .bind(spec.owner_id)
            .bind(match spec.account_type {
                AccountType::UserAvailable => "USER_AVAILABLE",
                AccountType::UserLocked => "USER_LOCKED",
                AccountType::PlatformClearing => "PLATFORM_CLEARING",
                AccountType::TreasuryAvailable => "TREASURY_AVAILABLE",
                AccountType::TreasuryLocked => "TREASURY_LOCKED",
                AccountType::InventoryAvailable => "INVENTORY_AVAILABLE",
                AccountType::InventoryLocked => "INVENTORY_LOCKED",
            })
            .bind(spec.asset_id)
            .bind(spec.is_active)
            .fetch_one(&self.pool)
            .await
            .map_err(map_sqlx)?;

        // If you didn't create the trigger to auto-create balance rows,
        // then insert a row here. If you DID create the trigger, this is safe anyway:
        sqlx::query(
            r#"
            INSERT INTO ledger_account_balances(account_id, balance)
            VALUES ($1, 0)
            ON CONFLICT (account_id) DO NOTHING
            "#,
        )
            .bind(row.id)
            .execute(&self.pool)
            .await
            .map_err(map_sqlx)?;

        row.to_domain()
    }

    async fn set_account_active(&self, account_id: i64, active: bool) -> Result<(), RepoError> {
        let n = sqlx::query(r#"UPDATE ledger_accounts SET is_active = $2 WHERE id = $1"#)
            .bind(account_id)
            .bind(active)
            .execute(&self.pool)
            .await
            .map_err(map_sqlx)?
            .rows_affected();

        if n == 0 {
            return Err(RepoError::NotFound {
                entity: format!("ledger_account id={account_id}"),
            });
        }
        Ok(())
    }

    async fn get_accounts_by_ids(&self, ids: &[i64]) -> Result<Vec<LedgerAccount>, RepoError> {
        if ids.is_empty() {
            return Ok(vec![]);
        }

        let rows = sqlx::query_as::<_, LedgerAccountRow>(
            r#"
            SELECT id, public_id, owner_type, owner_id, account_type, asset_id, is_active
            FROM ledger_accounts
            WHERE id = ANY($1)
            "#,
        )
            .bind(ids)
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx)?;

        let mut out = Vec::with_capacity(rows.len());
        for r in rows {
            out.push(r.to_domain()?);
        }
        Ok(out)
    }

    async fn find_posted_by_external_ref(
        &self,
        external_ref_type: ExternalRefType,
        external_ref: &ExternalRef,
    ) -> Result<Option<PostedJournal>, RepoError> {
        let tx_id = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT id
            FROM journal_transactions
            WHERE external_ref_type = $1 AND external_ref = $2
            "#,
        )
            .bind(external_ref_type.as_code())
            .bind(external_ref.as_str())
            .fetch_optional(&self.pool)
            .await
            .map_err(map_sqlx)?;

        let Some(id) = tx_id else { return Ok(None); };

        // Read method: use pool loader (avoids any tx lifetime gymnastics)
        Ok(Some(Self::load_posted_by_tx_id(&self.pool, id).await?))
    }

   /* async fn insert_posting_atomic(&self, posting: ValidatedJournal) -> Result<PostedJournal, RepoError> {
        self.with_tx(|tx| {
            Box::pin(async move {
                // 1) Idempotent header insert/get
                let tx_id = Self::insert_or_get_tx_id(tx, &posting).await?;

                // If already posted (lines exist), return existing
                if Self::tx_has_lines(tx, tx_id).await? {
                    return Self::load_posted_by_tx_id_tx(tx, tx_id).await;
                }

                // 2) Lock affected accounts in stable order
                let mut account_ids: Vec<i64> = posting.lines.iter().map(|l| l.account_id).collect();
                account_ids.sort_unstable();
                account_ids.dedup();

                let locked_rows = Self::fetch_accounts_for_update(tx, &account_ids).await?;
                if locked_rows.len() != account_ids.len() {
                    return Err(RepoError::NotFound {
                        entity: "one or more ledger accounts missing".into(),
                    });
                }

                // 3) Map rows -> domain and validate active
                let mut locked: HashMap<i64, LedgerAccount> = HashMap::with_capacity(locked_rows.len());
                for r in locked_rows {
                    let acct = r.to_domain()?;
                    if !acct.is_active() {
                        return Err(RepoError::Integrity {
                            message: format!("ledger account is inactive (account_id={})", acct.id()),
                        });
                    }
                    locked.insert(acct.id(), acct);
                }

                // 4) Enforce single-asset journal
                let _asset_id = Self::ensure_single_asset(tx, &account_ids).await?;

                // 5) Lock+fetch running balances (FOR UPDATE)
                let current = Self::lock_and_fetch_balances(tx, &account_ids).await?;

                // 6) Compute deltas (overflow-safe)
                let mut delta: HashMap<i64, i128> = HashMap::with_capacity(posting.lines.len());
                for l in &posting.lines {
                    let entry = delta.entry(l.account_id).or_insert(0);
                    *entry = entry
                        .checked_add(l.amount.minor())
                        .ok_or_else(|| RepoError::Integrity {
                            message: format!("delta overflow (account_id={})", l.account_id),
                        })?;
                }

                // 7) Validate spendability against running balances
                for (account_id, d) in &delta {
                    let acct = locked.get(account_id).ok_or_else(|| RepoError::NotFound {
                        entity: format!("ledger_account id={account_id}"),
                    })?;

                    if Self::is_spendable_bucket(acct.account_type()) {
                        let cur = *current.get(account_id).unwrap_or(&0);
                        let next = cur.checked_add(*d).ok_or_else(|| RepoError::Integrity {
                            message: format!("balance overflow (account_id={account_id})"),
                        })?;

                        if next < 0 {
                            return Err(RepoError::Conflict {
                                message: format!(
                                    "insufficient funds (account_id={account_id}, current={cur}, delta={d})"
                                ),
                            });
                        }
                    }
                }

                // 8) Bulk insert journal lines (append-only)
                let mut line_account_ids: Vec<i64> = Vec::with_capacity(posting.lines.len());
                let mut line_amounts: Vec<BigDecimal> = Vec::with_capacity(posting.lines.len());
                for l in &posting.lines {
                    line_account_ids.push(l.account_id);
                    line_amounts.push(i128_to_bigdecimal(l.amount.minor()));
                }
                Self::bulk_insert_lines(tx, tx_id, &line_account_ids, &line_amounts).await?;

                // 9) Update running balances
                Self::apply_balance_deltas(tx, &delta).await?;

                // 10) Return posted journal from same tx snapshot
                Self::load_posted_by_tx_id_tx(tx, tx_id).await
            })
        })
            .await
    }*/
}

#[async_trait]
impl LedgerRepositoryTx for PgLedgerRepository {
    async fn insert_posting_atomic_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        posting: ValidatedJournal,
    ) -> Result<PostedJournal, RepoError> {
        // 1) Idempotent header insert/get
        let tx_id = Self::insert_or_get_tx_id(tx, &posting).await?;

        // If already posted (lines exist), return existing
        if Self::tx_has_lines(tx, tx_id).await? {
            return Self::load_posted_by_tx_id_tx(tx, tx_id).await;
        }

        // 2) Lock affected accounts in stable order
        let mut account_ids: Vec<i64> = posting.lines.iter().map(|l| l.account_id).collect();
        account_ids.sort_unstable();
        account_ids.dedup();

        let locked_rows = Self::fetch_accounts_for_update(tx, &account_ids).await?;
        if locked_rows.len() != account_ids.len() {
            return Err(RepoError::NotFound {
                entity: "one or more ledger accounts missing".into(),
            });
        }

        let mut locked: std::collections::HashMap<i64, LedgerAccount> =
            std::collections::HashMap::with_capacity(locked_rows.len());

        for r in locked_rows {
            let acct = r.to_domain()?;
            if !acct.is_active() {
                return Err(RepoError::Integrity {
                    message: format!("ledger account is inactive (account_id={})", acct.id()),
                });
            }
            locked.insert(acct.id(), acct);
        }

        let _asset_id = Self::ensure_single_asset(tx, &account_ids).await?;

        let current = Self::lock_and_fetch_balances(tx, &account_ids).await?;

        let mut delta: std::collections::HashMap<i64, i128> =
            std::collections::HashMap::with_capacity(posting.lines.len());

        for l in &posting.lines {
            let entry = delta.entry(l.account_id).or_insert(0);
            *entry = entry.checked_add(l.amount.minor()).ok_or_else(|| RepoError::Integrity {
                message: format!("delta overflow (account_id={})", l.account_id),
            })?;
        }

        for (account_id, d) in &delta {
            let acct = locked.get(account_id).ok_or_else(|| RepoError::NotFound {
                entity: format!("ledger_account id={account_id}"),
            })?;

            if Self::is_spendable_bucket(acct.account_type()) {
                let cur = *current.get(account_id).unwrap_or(&0);
                let next = cur.checked_add(*d).ok_or_else(|| RepoError::Integrity {
                    message: format!("balance overflow (account_id={account_id})"),
                })?;

                if next < 0 {
                    return Err(RepoError::Conflict {
                        message: format!(
                            "insufficient funds (account_id={account_id}, current={cur}, delta={d})"
                        ),
                    });
                }
            }
        }

        let mut line_account_ids = Vec::with_capacity(posting.lines.len());
        let mut line_amounts = Vec::with_capacity(posting.lines.len());
        for l in &posting.lines {
            line_account_ids.push(l.account_id);
            line_amounts.push(i128_to_bigdecimal(l.amount.minor()));
        }

        // 8) Bulk insert journal lines (idempotent w/ unique (tx_id, account_id))
        let inserted = Self::bulk_insert_lines(tx, tx_id, &line_account_ids, &line_amounts).await?;

        // If 0 inserted, someone else already posted (idempotent replay / concurrent winner)
        // Return the existing posted journal without applying deltas again.
        if inserted == 0 {
            return Self::load_posted_by_tx_id_tx(tx, tx_id).await;
        }

        // Safety: partial insert should never happen if your ValidatedJournal is compressed,
        // but protect against mismatch anyway.
        let expected = line_account_ids.len() as u64;
        if inserted != expected {
            return Err(RepoError::Integrity {
                message: format!(
                    "partial insert for journal_lines: expected {expected}, inserted {inserted}. \
             This indicates inconsistent posting input or schema mismatch."
                ),
            });
        }

        // 9) Update running balances ONLY if we inserted lines now
        Self::apply_balance_deltas(tx, &delta).await?;

        // 10) Return posted journal
        Self::load_posted_by_tx_id_tx(tx, tx_id).await


        /*
        Self::bulk_insert_lines(tx, tx_id, &line_account_ids, &line_amounts).await?;

        Self::apply_balance_deltas(tx, &delta).await?;

        Self::load_posted_by_tx_id_tx(tx, tx_id).await*/
    }
}