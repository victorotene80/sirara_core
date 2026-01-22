use anyhow::Context;
use bigdecimal::BigDecimal;
use num_traits::Zero;
use serial_test::serial;
use sqlx::{PgPool, Postgres, Transaction};
use std::collections::HashMap;
use uuid::Uuid;

use sirara_core::application::contracts::repository::LedgerRepositoryTx;
use sirara_core::domain::aggregate::JournalDraft;
use sirara_core::domain::entities::LedgerAccount;
use sirara_core::domain::value_objects::{ExternalRef, ExternalRefType, Money, PublicId};
use sirara_core::infrastructure::persistence::ledger::PgLedgerRepository;
use sirara_core::infrastructure::persistence::models::LedgerAccountRow;

// Persistence helpers

fn persist_enabled() -> bool {
    std::env::var("PERSIST_TEST_DB").ok().as_deref() == Some("1")
}

fn reset_enabled() -> bool {
    std::env::var("RESET_TEST_DB").ok().as_deref() == Some("1")
}


async fn finish_tx(mut tx: Transaction<'_, Postgres>) -> anyhow::Result<()> {
    if persist_enabled() {
        tx.commit().await?;
    } else {
        tx.rollback().await.ok();
    }
    Ok(())
}

// DB helpers

async fn pool() -> PgPool {
    let _ = dotenvy::dotenv();
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for integration tests");
    PgPool::connect(&url).await.expect("connect failed")
}

async fn begin_tx(pool: &PgPool) -> Transaction<'_, Postgres> {
    pool.begin().await.expect("begin tx failed")
}

async fn reset_db(pool: &PgPool) -> anyhow::Result<()> {
    sqlx::query(
        r#"
        TRUNCATE TABLE
            journal_lines,
            journal_transactions,
            ledger_account_balances,
            ledger_accounts,
            assets
        RESTART IDENTITY CASCADE
        "#,
    )
        .execute(pool)
        .await?;
    Ok(())
}

async fn reset_db_if_requested(pool: &PgPool) -> anyhow::Result<()> {
    if reset_enabled() {
        reset_db(pool).await?;
    }
    Ok(())
}

// Seed helpers (per-test, inside the tx)

async fn seed_minimal(tx: &mut Transaction<'_, Postgres>) -> anyhow::Result<()> {
    sqlx::query(
        r#"
        insert into assets(code, decimals) values
          ('NGN', 2),
          ('USDT', 6)
        on conflict (code) do nothing
        "#,
    )
        .execute(tx.as_mut())
        .await?;

    let ngn_id: i16 = sqlx::query_scalar::<_, i16>("select id from assets where code = 'NGN'")
        .fetch_one(tx.as_mut())
        .await?;

    let usdt_id: i16 = sqlx::query_scalar::<_, i16>("select id from assets where code = 'USDT'")
        .fetch_one(tx.as_mut())
        .await?;

    let user_id = Uuid::new_v4();

    let user_avail_id: i64 = sqlx::query_scalar::<_, i64>(
        r#"
        insert into ledger_accounts(public_id, owner_type, owner_id, account_type, asset_id, is_active)
        values ($1, 'USER', $2, 'USER_AVAILABLE', $3, true)
        returning id
        "#,
    )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(ngn_id)
        .fetch_one(tx.as_mut())
        .await?;

    let user_locked_id: i64 = sqlx::query_scalar::<_, i64>(
        r#"
        insert into ledger_accounts(public_id, owner_type, owner_id, account_type, asset_id, is_active)
        values ($1, 'USER', $2, 'USER_LOCKED', $3, true)
        returning id
        "#,
    )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(ngn_id)
        .fetch_one(tx.as_mut())
        .await?;

    let platform_clear_id: i64 = sqlx::query_scalar::<_, i64>(
        r#"
        insert into ledger_accounts(public_id, owner_type, owner_id, account_type, asset_id, is_active)
        values ($1, 'PLATFORM', null, 'PLATFORM_CLEARING', $2, true)
        on conflict (owner_type, account_type, asset_id)
          where owner_id is null
        do update set is_active = excluded.is_active
        returning id
        "#,
    )
        .bind(Uuid::new_v4())
        .bind(ngn_id)
        .fetch_one(tx.as_mut())
        .await?;

    let platform_clear_usdt_id: i64 = sqlx::query_scalar::<_, i64>(
        r#"
        insert into ledger_accounts(public_id, owner_type, owner_id, account_type, asset_id, is_active)
        values ($1, 'PLATFORM', null, 'PLATFORM_CLEARING', $2, true)
        on conflict (owner_type, account_type, asset_id)
          where owner_id is null
        do update set is_active = excluded.is_active
        returning id
        "#,
    )
        .bind(Uuid::new_v4())
        .bind(usdt_id)
        .fetch_one(tx.as_mut())
        .await?;

    sqlx::query("create temp table if not exists t_seed(k text primary key, v bigint)")
        .execute(tx.as_mut())
        .await?;

    for (k, v) in [
        ("user_avail", user_avail_id),
        ("user_locked", user_locked_id),
        ("plat_clear", platform_clear_id),
        ("plat_clear_usdt", platform_clear_usdt_id),
    ] {
        sqlx::query(
            "insert into t_seed(k,v) values ($1,$2) on conflict (k) do update set v=excluded.v",
        )
            .bind(k)
            .bind(v)
            .execute(tx.as_mut())
            .await?;
    }

    Ok(())
}

async fn seeded_id(tx: &mut Transaction<'_, Postgres>, key: &str) -> anyhow::Result<i64> {
    let v: i64 = sqlx::query_scalar::<_, i64>("select v from t_seed where k = $1")
        .bind(key)
        .fetch_one(tx.as_mut())
        .await?;
    Ok(v)
}

// Projection helpers

fn numeric0_to_i128_strict(v: &BigDecimal) -> anyhow::Result<i128> {
    let s = v.to_string();
    if s.contains('.') {
        anyhow::bail!("fractional numeric not allowed for numeric(38,0): {s}");
    }
    Ok(s.parse::<i128>()?)
}

async fn fetch_balance_i128(tx: &mut Transaction<'_, Postgres>, account_id: i64) -> anyhow::Result<i128> {
    let bal: BigDecimal =
        sqlx::query_scalar::<_, BigDecimal>("select balance from ledger_account_balances where account_id = $1")
            .bind(account_id)
            .fetch_one(tx.as_mut())
            .await?;
    Ok(numeric0_to_i128_strict(&bal)?)
}

async fn fetch_drift(tx: &mut Transaction<'_, Postgres>, account_id: i64) -> anyhow::Result<BigDecimal> {
    let drift: BigDecimal =
        sqlx::query_scalar::<_, BigDecimal>("select drift from v_ledger_balance_drift where account_id = $1")
            .bind(account_id)
            .fetch_one(tx.as_mut())
            .await?;
    Ok(drift)
}

// Helper: load accounts + build ValidatedJournal using aggregate

async fn load_accounts_by_ids(
    tx: &mut Transaction<'_, Postgres>,
    ids: &[i64],
) -> anyhow::Result<HashMap<i64, LedgerAccount>> {
    let rows = sqlx::query_as::<_, LedgerAccountRow>(
        r#"
        SELECT id, public_id, owner_type, owner_id, account_type, asset_id, is_active
        FROM ledger_accounts
        WHERE id = ANY($1)
        "#,
    )
        .bind(ids)
        .fetch_all(tx.as_mut())
        .await?;

    let mut out = HashMap::with_capacity(rows.len());
    for r in rows {
        out.insert(r.id, r.to_domain()?);
    }
    Ok(out)
}

fn make_validated_posting(
    a1: i64,
    a2: i64,
    accounts: &HashMap<i64, LedgerAccount>,
    external_ref: Option<ExternalRef>,
) -> anyhow::Result<sirara_core::domain::aggregate::ValidatedJournal> {
    let mut refs: HashMap<i64, &LedgerAccount> = HashMap::new();
    for (id, acct) in accounts.iter() {
        refs.insert(*id, acct);
    }

    let ext = external_ref.unwrap_or_else(|| ExternalRef::new(format!("test:{}", Uuid::new_v4())).unwrap());

    let mut draft = JournalDraft::new(
        PublicId::new(Uuid::new_v4()),
        ExternalRefType::TransferIntent,
        ext,
        "test",
        None,
    )?;

    draft.add_line(a1, Money::debit(100)?);
    draft.add_line(a2, Money::credit(100)?);

    Ok(draft.validate_with_accounts(&refs)?)
}

// Tests: DB invariants

#[tokio::test]
#[serial]
async fn invariant_balanced_and_min_2_lines_enforced() -> anyhow::Result<()> {
    let pool = pool().await;
    reset_db_if_requested(&pool).await?;
    let mut tx = begin_tx(&pool).await;

    seed_minimal(&mut tx).await?;
    let a1 = seeded_id(&mut tx, "user_avail").await?;

    let jtx_id: i64 = sqlx::query_scalar::<_, i64>(
        r#"
        insert into journal_transactions(public_id, external_ref, external_ref_type, description, created_by)
        values ($1, $2, 'TRANSFER_INTENT', null, 'test')
        returning id
        "#,
    )
        .bind(Uuid::new_v4())
        .bind(format!("test:{}", Uuid::new_v4()))
        .fetch_one(tx.as_mut())
        .await?;

    sqlx::query("insert into journal_lines(journal_tx_id, account_id, amount) values ($1,$2,$3)")
        .bind(jtx_id)
        .bind(a1)
        .bind(BigDecimal::from(100_i64))
        .execute(tx.as_mut())
        .await?;

    // This test EXPECTS commit failure, so it will NEVER persist rows.
    let err = tx.commit().await.unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("must have at least 2 lines"), "expected min-lines error, got: {msg}");
    Ok(())
}

#[tokio::test]
#[serial]
async fn invariant_sum_zero_enforced() -> anyhow::Result<()> {
    let pool = pool().await;
    reset_db_if_requested(&pool).await?;
    let mut tx = begin_tx(&pool).await;

    seed_minimal(&mut tx).await?;
    let a1 = seeded_id(&mut tx, "user_avail").await?;
    let a2 = seeded_id(&mut tx, "plat_clear").await?;

    let jtx_id: i64 = sqlx::query_scalar::<_, i64>(
        r#"
        insert into journal_transactions(public_id, external_ref, external_ref_type, description, created_by)
        values ($1, $2, 'TRANSFER_INTENT', null, 'test')
        returning id
        "#,
    )
        .bind(Uuid::new_v4())
        .bind(format!("test:{}", Uuid::new_v4()))
        .fetch_one(tx.as_mut())
        .await?;

    sqlx::query("insert into journal_lines(journal_tx_id, account_id, amount) values ($1,$2,$3)")
        .bind(jtx_id)
        .bind(a1)
        .bind(BigDecimal::from(100_i64))
        .execute(tx.as_mut())
        .await?;

    sqlx::query("insert into journal_lines(journal_tx_id, account_id, amount) values ($1,$2,$3)")
        .bind(jtx_id)
        .bind(a2)
        .bind(BigDecimal::from(-50_i64))
        .execute(tx.as_mut())
        .await?;

    let err = tx.commit().await.unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("is not balanced"), "expected balance error, got: {msg}");
    Ok(())
}

#[tokio::test]
#[serial]
async fn invariant_single_asset_per_tx_enforced() -> anyhow::Result<()> {
    let pool = pool().await;
    reset_db_if_requested(&pool).await?;
    let mut tx = begin_tx(&pool).await;

    seed_minimal(&mut tx).await?;
    let ngn_user = seeded_id(&mut tx, "user_avail").await?;
    let usdt_plat = seeded_id(&mut tx, "plat_clear_usdt").await?;

    let jtx_id: i64 = sqlx::query_scalar::<_, i64>(
        r#"
        insert into journal_transactions(public_id, external_ref, external_ref_type, description, created_by)
        values ($1, $2, 'TRANSFER_INTENT', null, 'test')
        returning id
        "#,
    )
        .bind(Uuid::new_v4())
        .bind(format!("test:{}", Uuid::new_v4()))
        .fetch_one(tx.as_mut())
        .await?;

    sqlx::query("insert into journal_lines(journal_tx_id, account_id, amount) values ($1,$2,$3)")
        .bind(jtx_id)
        .bind(ngn_user)
        .bind(BigDecimal::from(100_i64))
        .execute(tx.as_mut())
        .await?;

    sqlx::query("insert into journal_lines(journal_tx_id, account_id, amount) values ($1,$2,$3)")
        .bind(jtx_id)
        .bind(usdt_plat)
        .bind(BigDecimal::from(-100_i64))
        .execute(tx.as_mut())
        .await?;

    let err = tx.commit().await.unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("spans multiple assets"), "expected single-asset error, got: {msg}");
    Ok(())
}

#[tokio::test]
#[serial]
async fn immutability_blocks_updates_and_deletes() -> anyhow::Result<()> {
    let pool = pool().await;
    reset_db_if_requested(&pool).await?;
    let mut tx = begin_tx(&pool).await;

    seed_minimal(&mut tx).await?;
    let a1 = seeded_id(&mut tx, "user_avail").await?;
    let a2 = seeded_id(&mut tx, "plat_clear").await?;

    let jtx_id: i64 = sqlx::query_scalar::<_, i64>(
        r#"
        insert into journal_transactions(public_id, external_ref, external_ref_type, description, created_by)
        values ($1, $2, 'TRANSFER_INTENT', null, 'test')
        returning id
        "#,
    )
        .bind(Uuid::new_v4())
        .bind(format!("test:{}", Uuid::new_v4()))
        .fetch_one(tx.as_mut())
        .await?;

    sqlx::query("insert into journal_lines(journal_tx_id, account_id, amount) values ($1,$2,$3)")
        .bind(jtx_id)
        .bind(a1)
        .bind(BigDecimal::from(100_i64))
        .execute(tx.as_mut())
        .await?;

    sqlx::query("insert into journal_lines(journal_tx_id, account_id, amount) values ($1,$2,$3)")
        .bind(jtx_id)
        .bind(a2)
        .bind(BigDecimal::from(-100_i64))
        .execute(tx.as_mut())
        .await?;

    let upd = sqlx::query("update journal_lines set amount = amount + 1 where journal_tx_id = $1")
        .bind(jtx_id)
        .execute(tx.as_mut())
        .await;
    assert!(upd.is_err(), "update should be blocked");

    let del = sqlx::query("delete from journal_lines where journal_tx_id = $1")
        .bind(jtx_id)
        .execute(tx.as_mut())
        .await;
    assert!(del.is_err(), "delete should be blocked");

    finish_tx(tx).await?;
    Ok(())
}

// Tests: Repo-driven projection correctness

#[tokio::test]
#[serial]
async fn projection_matches_truth_after_repo_posting() -> anyhow::Result<()> {
    let pool = pool().await;
    reset_db_if_requested(&pool).await?;
    let mut tx = begin_tx(&pool).await;

    seed_minimal(&mut tx).await?;
    let a1 = seeded_id(&mut tx, "user_avail").await?;
    let a2 = seeded_id(&mut tx, "plat_clear").await?;

    let accounts = load_accounts_by_ids(&mut tx, &[a1, a2]).await?;
    let posting = make_validated_posting(a1, a2, &accounts, None)?;

    let repo = PgLedgerRepository::new(pool.clone());
    repo.insert_posting_atomic_tx(&mut tx, posting)
        .await
        .context("repo posting failed")?;

    let drift1 = fetch_drift(&mut tx, a1).await?;
    let drift2 = fetch_drift(&mut tx, a2).await?;
    assert!(drift1.is_zero());
    assert!(drift2.is_zero());

    finish_tx(tx).await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn balances_updated_after_repo_posting() -> anyhow::Result<()> {
    let pool = pool().await;
    reset_db_if_requested(&pool).await?;
    let mut tx = begin_tx(&pool).await;

    seed_minimal(&mut tx).await?;
    let a1 = seeded_id(&mut tx, "user_avail").await?;
    let a2 = seeded_id(&mut tx, "plat_clear").await?;

    let accounts = load_accounts_by_ids(&mut tx, &[a1, a2]).await?;
    let posting = make_validated_posting(a1, a2, &accounts, None)?;

    let repo = PgLedgerRepository::new(pool.clone());
    repo.insert_posting_atomic_tx(&mut tx, posting).await?;

    let b1 = fetch_balance_i128(&mut tx, a1).await?;
    let b2 = fetch_balance_i128(&mut tx, a2).await?;
    assert_eq!(b1, 100);
    assert_eq!(b2, -100);

    finish_tx(tx).await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn idempotency_same_external_ref_does_not_double_apply() -> anyhow::Result<()> {
    let pool = pool().await;
    reset_db_if_requested(&pool).await?;
    let mut tx = begin_tx(&pool).await;

    seed_minimal(&mut tx).await?;
    let a1 = seeded_id(&mut tx, "user_avail").await?;
    let a2 = seeded_id(&mut tx, "plat_clear").await?;

    let accounts = load_accounts_by_ids(&mut tx, &[a1, a2]).await?;

    let same_ext = ExternalRef::new(format!("test:{}", Uuid::new_v4()))?;
    let p1 = make_validated_posting(a1, a2, &accounts, Some(same_ext.clone()))?;
    let p2 = make_validated_posting(a1, a2, &accounts, Some(same_ext))?;

    let repo = PgLedgerRepository::new(pool.clone());

    repo.insert_posting_atomic_tx(&mut tx, p1).await?;
    repo.insert_posting_atomic_tx(&mut tx, p2).await?; 

    let b1 = fetch_balance_i128(&mut tx, a1).await?;
    let b2 = fetch_balance_i128(&mut tx, a2).await?;
    assert_eq!(b1, 100);
    assert_eq!(b2, -100);

    let drift1 = fetch_drift(&mut tx, a1).await?;
    let drift2 = fetch_drift(&mut tx, a2).await?;
    assert!(drift1.is_zero());
    assert!(drift2.is_zero());

    finish_tx(tx).await?;
    Ok(())
}
