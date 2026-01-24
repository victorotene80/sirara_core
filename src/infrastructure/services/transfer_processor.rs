use chrono::Utc;

use crate::application::AppError;
use crate::application::contracts::repository::{IntentPatch, TransferRepositoryTx};
use crate::application::contracts::repository::OutboxRepositoryTx;
use crate::domain::value_objects::FxQuote;
use crate::infrastructure::persistence::PgReposInTx;

use crate::domain::value_objects::{PublicId, TransferState};

pub async fn process_created(repos: &mut PgReposInTx<'_, '_>, payload: serde_json::Value) -> Result<(), AppError> {
    let intent_public_id = payload["intent_public_id"]
        .as_str()
        .ok_or_else(|| AppError::InvalidRequest { message: "missing intent_public_id".into() })?;

    let public_id = PublicId::new(
        uuid::Uuid::parse_str(intent_public_id)
            .map_err(|_| AppError::InvalidRequest { message: "bad uuid".into() })?
    );

    let mut transfer = repos.transfer();
    let mut outbox = repos.outbox();
    let mut ledger = repos.ledger();

    // 1) lock intent row
    let mut intent = transfer.get_intent_for_update_by_public_id(public_id).await?;

    // If already progressed, idempotent no-op
    if intent.state != TransferState::Created { return Ok(()); }

    // 2) FX quote (TODO external port; for now deterministic stub)
    // TODO(epic): real FX adapter + firm quote + TTL
    let now = Utc::now();
    let expires_at = now + chrono::Duration::seconds(30);
    let quote_json = serde_json::json!({ "rate": "1.0000", "path": ["NGN","USDT","KES"] });
    let required_usdt_minor: i128 = 100; // TODO compute

    // CAS update intent + append transition
    let expected = intent.version;
    intent.lock_rate(
        FxQuote::new( "1.0000".into(), expires_at,  vec!["NGN".into(),"USDT".into(),"KES".into()]),// { rate: "1.0000".into(), expires_at, path: vec!["NGN".into(),"USDT".into(),"KES".into()] },
        required_usdt_minor,
        now
    )?;

    transfer.append_transition(intent.db_id, TransferState::Created, TransferState::RateLocked, "rate locked".into(), Some(quote_json.clone())).await?;
    transfer.update_intent_state_cas(
        intent.db_id,
        expected,
        TransferState::RateLocked,
        intent.version,
        IntentPatch { quote_json: Some(quote_json), quote_expires_at: Some(expires_at), required_usdt_minor: Some(required_usdt_minor), ..Default::default() }
    ).await?;

    // 3) reserve inventory (TODO: should be its own service + ledger movement)
    // TODO(epic): inventory reservation, multi-provider, reserve idempotency
    // For now: enqueue next step
    outbox.enqueue(
        "transfer.intent.reserve_inventory",
        &intent.public_id.value().to_string(),
        serde_json::json!({ "intent_public_id": intent.public_id.value() })
    ).await?;

    Ok(())
}

pub async fn process_check(repos: &mut PgReposInTx<'_, '_>, payload: serde_json::Value) -> Result<(), AppError> {
    // TODO: check blockchain/bank status and settle/unwind
    let _ = payload;
    Ok(())
}
