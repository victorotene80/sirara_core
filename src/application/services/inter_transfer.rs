use chrono::Utc;
use uuid::Uuid;

use crate::application::contracts::repository::{ReposInTx, UnitOfWork};
use crate::application::contracts::repository::{TransferRepositoryTx};
use crate::application::contracts::repository::OutboxRepositoryTx;
use crate::application::AppError;

use crate::domain::aggregate::TransferIntent;
use crate::domain::value_objects::{ExternalRef, ExternalRefType, Money, PublicId};
use crate::domain::value_objects::TransferRoute;

pub struct InterTransferOrchestrator<U: UnitOfWork> {
    uow: U,
}

impl<U: UnitOfWork> InterTransferOrchestrator<U> {
    pub fn new(uow: U) -> Self { Self { uow } }

    pub async fn initiate_inter(
        &self,
        route: TransferRoute,         
        asset_code: String,            
        amount_minor: i128,
        external_ref: String,
    ) -> Result<Uuid, AppError> {
        let now = Utc::now();
        let public_id = PublicId::new(Uuid::new_v4());

        self.uow.with_tx(move |repos: &mut dyn ReposInTx| {
            Box::pin(async move {
                // you must expose repos.transfer() + repos.outbox() similar to repos.ledger()
                let mut transfer: Box<dyn TransferRepositoryTx> = repos.transfer(); // <-- add to ReposInTx
                let mut outbox: Box<dyn OutboxRepositoryTx> = repos.outbox();       // <-- add to ReposInTx

                let intent = TransferIntent::new_created(
                    public_id,
                    ExternalRefType::TransferIntent,
                    ExternalRef::new(external_ref)?,
                    route,
                    Money::from_signed_minor(amount_minor)?,
                    asset_code,
                    now,
                )?;

                let saved = transfer.insert_intent_if_absent(intent).await?;

                // Append initial transition row (CREATED) for audit
                transfer.append_transition(
                    saved.db_id,
                    crate::domain::value_objects::TransferState::Created,
                    crate::domain::value_objects::TransferState::Created,
                    "created".into(),
                    None,
                ).await?;

                // Enqueue async processing step
                outbox.enqueue(
                    "transfer.intent.created",
                    &saved.public_id.value().to_string(),
                    serde_json::json!({ "intent_public_id": saved.public_id.value() }),
                ).await?;

                Ok(saved.public_id.value())
            })
        }).await
    }
}
