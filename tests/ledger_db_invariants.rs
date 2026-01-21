use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

async fn pool() -> PgPool {
    let url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set for integration tests");
    PgPool::connect(&url).await.expect("connect failed")
}

async fn begin_tx(pool: &PgPool) -> Transaction<'_, Postgres> {
    pool.begin().await.expect("begin tx failed")
}

async fn seed_minimal(tx: &mut Transaction<'_, Postgres>) -> anyhow::Result<()> {
    // Seed assets
    sqlx::query(
        r#"
        insert into assets(code, decimals) values
          ('NGN', 2),
          ('USDT', 6)
        on conflict (code) do nothing
        "#
    )
        .execute(&mut **tx).await?;

    // Get asset ids
    let ngn_id: i16 = sqlx::query_scalar("select id from assets where code = 'NGN'")
        .fetch_one(&mut **tx).await?;
    let usdt_id: i16 = sqlx::query_scalar("select id from assets where code = 'USDT'")
        .fetch_one(&mut **tx).await?;

    // Create a USER with available/locked NGN accounts
    let user_id = Uuid::new_v4();
    let user_avail_id: i64 = sqlx::query_scalar(
        r#"
        insert into ledger_accounts(public_id, owner_type, owner_id, account_type, asset_id, is_active)
        values ($1, 'USER', $2, 'USER_AVAILABLE', $3, true)
        returning id
        "#
    )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(ngn_id)
        .fetch_one(&mut **tx).await?;

    let user_locked_id: i64 = sqlx::query_scalar(
        r#"
        insert into ledger_accounts(public_id, owner_type, owner_id, account_type, asset_id, is_active)
        values ($1, 'USER', $2, 'USER_LOCKED', $3, true)
        returning id
        "#
    )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(ngn_id)
        .fetch_one(&mut **tx).await?;

    // Platform clearing NGN
    let platform_clear_id: i64 = sqlx::query_scalar(
        r#"
        insert into ledger_accounts(public_id, owner_type, owner_id, account_type, asset_id, is_active)
        values ($1, 'PLATFORM', null, 'PLATFORM_CLEARING', $2, true)
        returning id
        "#
    )
        .bind(Uuid::new_v4())
        .bind(ngn_id)
        .fetch_one(&mut **tx).await?;

    // Another asset account for cross-asset test: PLATFORM_CLEARING USDT
    let platform_clear_usdt_id: i64 = sqlx::query_scalar(
        r#"
        insert into ledger_accounts(public_id, owner_type, owner_id, account_type, asset_id, is_active)
        values ($1, 'PLATFORM', null, 'PLATFORM_CLEARING', $2, true)
        returning id
        "#
    )
        .bind(Uuid::new_v4())
        .bind(usdt_id)
        .fetch_one(&mut **tx).await?;

    // Put ids somewhere accessible to tests: simplest is temp table or just return Ok
    // We'll store them in session-local temp table to fetch later:
    sqlx::query("create temp table if not exists t_seed(k text primary key, v bigint)")
        .execute(&mut **tx).await?;
    sqlx::query("insert into t_seed(k,v) values ($1,$2) on conflict (k) do update set v=excluded.v")
        .bind("user_avail").bind(user_avail_id)
        .execute(&mut **tx).await?;
    sqlx::query("insert into t_seed(k,v) values ($1,$2) on conflict (k) do update set v=excluded.v")
        .bind("user_locked").bind(user_locked_id)
        .execute(&mut **tx).await?;
    sqlx::query("insert into t_seed(k,v) values ($1,$2) on conflict (k) do update set v=excluded.v")
        .bind("plat_clear").bind(platform_clear_id)
        .execute(&mut **tx).await?;
    sqlx::query("insert into t_seed(k,v) values ($1,$2) on conflict (k) do update set v=excluded.v")
        .bind("plat_clear_usdt").bind(platform_clear_usdt_id)
        .execute(&mut **tx).await?;

    Ok(())
}

async fn seeded_id(tx: &mut Transaction<'_, Postgres>, key: &str) -> i64 {
    sqlx::query_scalar("select v from t_seed where k = $1")
        .bind(key)
        .fetch_one(&mut **tx)
        .await
        .expect("seeded id missing")
}

#[tokio::test]
async fn invariant_balanced_and_min_2_lines_enforced() -> anyhow::Result<()> {
    let pool = pool().await;
    let mut tx = begin_tx(&pool).await;

    seed_minimal(&mut tx).await?;
    let a1 = seeded_id(&mut tx, "user_avail").await;
    let a2 = seeded_id(&mut tx, "plat_clear").await;

    // Create header
    let jtx_id: i64 = sqlx::query_scalar(
        r#"
        insert into journal_transactions(public_id, external_ref, external_ref_type, description, created_by)
        values ($1, $2, 'TRANSFER_INTENT', null, 'test')
        returning id
        "#
    )
        .bind(Uuid::new_v4())
        .bind(format!("test:{}", Uuid::new_v4()))
        .fetch_one(&mut **tx).await?;

    // Insert ONLY ONE line => should fail at COMMIT due to >=2 lines trigger.
    sqlx::query(
        r#"insert into journal_lines(journal_tx_id, account_id, amount) values ($1,$2,$3)"#
    )
        .bind(jtx_id)
        .bind(a1)
        .bind(100_i64) // debit +100
        .execute(&mut **tx).await?;

    // Commit should fail because constraint trigger runs deferred.
    let err = tx.commit().await.unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("must have at least 2 lines"),
        "expected min-lines error, got: {msg}"
    );

    Ok(())
}

#[tokio::test]
async fn invariant_sum_zero_enforced() -> anyhow::Result<()> {
    let pool = pool().await;
    let mut tx = begin_tx(&pool).await;

    seed_minimal(&mut tx).await?;
    let a1 = seeded_id(&mut tx, "user_avail").await;
    let a2 = seeded_id(&mut tx, "plat_clear").await;

    let jtx_id: i64 = sqlx::query_scalar(
        r#"
        insert into journal_transactions(public_id, external_ref, external_ref_type, description, created_by)
        values ($1, $2, 'TRANSFER_INTENT', null, 'test')
        returning id
        "#
    )
        .bind(Uuid::new_v4())
        .bind(format!("test:{}", Uuid::new_v4()))
        .fetch_one(&mut **tx).await?;

    // Two lines but not balanced: +100 and -50 => sum != 0
    sqlx::query("insert into journal_lines(journal_tx_id, account_id, amount) values ($1,$2,$3)")
        .bind(jtx_id).bind(a1).bind(100_i64)
        .execute(&mut **tx).await?;
    sqlx::query("insert into journal_lines(journal_tx_id, account_id, amount) values ($1,$2,$3)")
        .bind(jtx_id).bind(a2).bind(-50_i64)
        .execute(&mut **tx).await?;

    let err = tx.commit().await.unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("is not balanced"), "expected balance error, got: {msg}");
    Ok(())
}

#[tokio::test]
async fn invariant_single_asset_per_tx_enforced() -> anyhow::Result<()> {
    let pool = pool().await;
    let mut tx = begin_tx(&pool).await;

    seed_minimal(&mut tx).await?;
    let ngn_user = seeded_id(&mut tx, "user_avail").await;
    let usdt_plat = seeded_id(&mut tx, "plat_clear_usdt").await;

    let jtx_id: i64 = sqlx::query_scalar(
        r#"
        insert into journal_transactions(public_id, external_ref, external_ref_type, description, created_by)
        values ($1, $2, 'TRANSFER_INTENT', null, 'test')
        returning id
        "#
    )
        .bind(Uuid::new_v4())
        .bind(format!("test:{}", Uuid::new_v4()))
        .fetch_one(&mut **tx).await?;

    // Cross-asset: NGN account and USDT account in same tx.
    sqlx::query("insert into journal_lines(journal_tx_id, account_id, amount) values ($1,$2,$3)")
        .bind(jtx_id).bind(ngn_user).bind(100_i64)
        .execute(&mut **tx).await?;
    sqlx::query("insert into journal_lines(journal_tx_id, account_id, amount) values ($1,$2,$3)")
        .bind(jtx_id).bind(usdt_plat).bind(-100_i64)
        .execute(&mut **tx).await?;

    let err = tx.commit().await.unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("spans multiple assets"), "expected single-asset error, got: {msg}");
    Ok(())
}

#[tokio::test]
async fn immutability_blocks_updates_and_deletes() -> anyhow::Result<()> {
    let pool = pool().await;
    let mut tx = begin_tx(&pool).await;

    seed_minimal(&mut tx).await?;
    let a1 = seeded_id(&mut tx, "user_avail").await;
    let a2 = seeded_id(&mut tx, "plat_clear").await;

    let ext = format!("test:{}", Uuid::new_v4());
    let jtx_id: i64 = sqlx::query_scalar(
        r#"
        insert into journal_transactions(public_id, external_ref, external_ref_type, description, created_by)
        values ($1, $2, 'TRANSFER_INTENT', null, 'test')
        returning id
        "#
    )
        .bind(Uuid::new_v4())
        .bind(&ext)
        .fetch_one(&mut **tx).await?;

    sqlx::query("insert into journal_lines(journal_tx_id, account_id, amount) values ($1,$2,$3)")
        .bind(jtx_id).bind(a1).bind(100_i64)
        .execute(&mut **tx).await?;
    sqlx::query("insert into journal_lines(journal_tx_id, account_id, amount) values ($1,$2,$3)")
        .bind(jtx_id).bind(a2).bind(-100_i64)
        .execute(&mut **tx).await?;

    // Force triggers to validate by committing this tx successfully
    tx.commit().await?;

    // New tx for update attempt
    let mut tx2 = begin_tx(&pool).await;

    let res = sqlx::query("update journal_lines set amount = amount + 1 where journal_tx_id = $1")
        .bind(jtx_id)
        .execute(&mut *tx2)
        .await;
    assert!(res.is_err(), "update should be blocked");

    let res = sqlx::query("delete from journal_lines where journal_tx_id = $1")
        .bind(jtx_id)
        .execute(&mut *tx2)
        .await;
    assert!(res.is_err(), "delete should be blocked");

    // rollback tx2 (we don't need commit)
    tx2.rollback().await.ok();

    Ok(())
}
