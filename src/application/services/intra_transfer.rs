use chrono::Utc;
use uuid::Uuid;

use crate::application::contracts::repository::{ReposInTx, UnitOfWork};
use crate::application::AppError;
use crate::domain::aggregate::{JournalDraft};
use crate::domain::value_objects::{ExternalRef, ExternalRefType, Money, PublicId, TransferRoute};

pub struct IntraTransferOrchestrator<U: UnitOfWork> {
    uow: U,
}

impl<U: UnitOfWork> IntraTransferOrchestrator<U> {
    pub fn new(uow: U) -> Self { Self { uow } }

    pub async fn initiate_intra(
        &self,
        from_user_id: Uuid,
        to_user_id: Uuid,
        asset_code: String,
        amount_minor: i128,
        external_ref: String,
        created_by: String,
    ) -> Result<Uuid, AppError> {
        let now = Utc::now();
        let public_id = PublicId::new(Uuid::new_v4());

        self.uow.with_tx(move |repos: &mut dyn ReposInTx| {
            Box::pin(async move {
                let mut ledger = repos.ledger();

                // Resolve accounts (fast, stable, no orchestration here)
                let (from_avail, _from_locked) = ledger.resolve_user_hold_accounts(from_user_id, asset_code.clone()).await?;
                let (to_avail, _to_locked) = ledger.resolve_user_hold_accounts(to_user_id, asset_code.clone()).await?;

                // Build route
                let route = TransferRoute::IntraTransfer {
                    from_user_id,
                    to_user_id,
                    asset: crate::domain::value_objects::AssetCode::new(asset_code.clone())?,
                    amount: Money::from_signed_minor(amount_minor)?,
                };

                // Create intent (persist via transfer repo once you add it; for now TODO)
                // TODO(epic): persist TransferIntent + transitions (inter uses it heavily too)

                // Ledger posting (atomic)
                let mut draft = JournalDraft::new(
                    PublicId::new(Uuid::new_v4()),
                    ExternalRefType::TransferIntent,
                    ExternalRef::new(external_ref)?,
                    created_by,
                    Some("intra transfer".into()),
                )?;

                // Debit sender, credit receiver
                draft.add_line(from_avail, Money::from_signed_minor(-amount_minor)?); // credit
                draft.add_line(to_avail, Money::from_signed_minor(amount_minor)?);   // debit

                let ids: Vec<i64> = draft.lines().iter().map(|l| l.account_id).collect();
                let accounts = ledger.get_accounts_by_ids_for_validation(&ids).await?;
                let mut by_id = std::collections::HashMap::new();
                for a in &accounts { by_id.insert(a.id(), a); }

                let validated = draft.validate_with_accounts(&by_id)?;
                let _posted = ledger.insert_posting_atomic(validated).await?;

                Ok(public_id.value())
            })
        }).await
    }
}
