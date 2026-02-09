use sqlx::{Postgres, Transaction};
use bigdecimal::BigDecimal;

use crate::application::contracts::repository::{
    BoxFut, InsertIntentResult, IntentPatch, TransferRepositoryTx,
};
use crate::domain::aggregate::TransferIntent;
use crate::domain::repository::RepoError;
use crate::domain::value_objects::{ExternalRef, ExternalRefType, PublicId, TransferState};
use crate::infrastructure::persistence::error_map::map_sqlx;
use crate::infrastructure::persistence::mappers;
use crate::infrastructure::persistence::mappers::map_row_to_intent;
use crate::infrastructure::persistence::models::TransferIntentRow;

pub struct PgTransferTxRepo<'a, 'c> {
    tx: &'a mut Transaction<'c, Postgres>,
}

impl<'a, 'c> PgTransferTxRepo<'a, 'c> {
    pub fn new(tx: &'a mut Transaction<'c, Postgres>) -> Self {
        Self { tx }
    }
}

impl<'a, 'c> TransferRepositoryTx for PgTransferTxRepo<'a, 'c> {
    fn insert_intent_if_absent(
        &mut self,
        intent: TransferIntent,
    ) -> BoxFut<'_, Result<InsertIntentResult, RepoError>> {
        Box::pin(async move {
            let route_dto = mappers::transfer_route::to_json(intent.route()).map_err(|e| {
                RepoError::Integrity {
                    message: format!("failed to map route to dto: {e}"),
                }
            })?;

            let route_json = serde_json::to_value(route_dto).map_err(|e| RepoError::Integrity {
                message: format!("failed to serialize route dto: {e}"),
            })?;

            let inserted_row = sqlx::query_as::<_, TransferIntentRow>(
                r#"
                INSERT INTO transfer_intents
                    (public_id, external_ref_type, external_ref, route_json, amount_minor, asset_code, current_state, version, created_at, updated_at)
                VALUES
                    ($1, $2, $3, $4, $5, $6, $7, 0, now(), now())
                ON CONFLICT (external_ref_type, external_ref) DO NOTHING
                RETURNING
                    id, public_id, external_ref_type, external_ref,
                    route_json, amount_minor, asset_code,
                    current_state, version,
                    quote_json, quote_expires_at, required_usdt_minor, failure_json,
                    created_at, updated_at
                "#,
            )
                .bind(intent.public_id().value())
                .bind(intent.external_ref_type().as_code())
                .bind(intent.external_ref().as_str())
                .bind(route_json)
                .bind(BigDecimal::from(intent.amount_minor()))
                .bind(intent.asset_code())
                .bind(intent.state().as_str())
                .fetch_optional(&mut **self.tx)
                .await
                .map_err(map_sqlx)?;

            if let Some(row) = inserted_row {
                let intent = map_row_to_intent(row)?;
                return Ok(InsertIntentResult {
                    intent,
                    inserted: true,
                });
            }

            let row = sqlx::query_as::<_, TransferIntentRow>(
                r#"
                SELECT
                    id, public_id, external_ref_type, external_ref,
                    route_json, amount_minor, asset_code,
                    current_state, version,
                    quote_json, quote_expires_at, required_usdt_minor, failure_json,
                    created_at, updated_at
                FROM transfer_intents
                WHERE external_ref_type = $1 AND external_ref = $2
                "#,
            )
                .bind(intent.external_ref_type().as_code())
                .bind(intent.external_ref().as_str())
                .fetch_one(&mut **self.tx)
                .await
                .map_err(map_sqlx)?;

            let intent = map_row_to_intent(row)?;
            Ok(InsertIntentResult {
                intent,
                inserted: false,
            })
        })
    }

    fn get_intent_by_public_id(
        &mut self,
        public_id: PublicId,
    ) -> BoxFut<'_, Result<TransferIntent, RepoError>> {
        Box::pin(async move {
            let row = sqlx::query_as::<_, TransferIntentRow>(
                r#"
                SELECT
                    id, public_id, external_ref_type, external_ref,
                    route_json, amount_minor, asset_code,
                    current_state, version,
                    quote_json, quote_expires_at, required_usdt_minor, failure_json,
                    created_at, updated_at
                FROM transfer_intents
                WHERE public_id = $1
                "#,
            )
                .bind(public_id.value())
                .fetch_one(&mut **self.tx)
                .await
                .map_err(map_sqlx)?;

            map_row_to_intent(row)
        })
    }

    fn get_intent_for_update_by_public_id(
        &mut self,
        public_id: PublicId,
    ) -> BoxFut<'_, Result<TransferIntent, RepoError>> {
        Box::pin(async move {
            let row = sqlx::query_as::<_, TransferIntentRow>(
                r#"
                SELECT
                    id, public_id, external_ref_type, external_ref,
                    route_json, amount_minor, asset_code,
                    current_state, version,
                    quote_json, quote_expires_at, required_usdt_minor, failure_json,
                    created_at, updated_at
                FROM transfer_intents
                WHERE public_id = $1
                FOR UPDATE
                "#,
            )
                .bind(public_id.value())
                .fetch_one(&mut **self.tx)
                .await
                .map_err(map_sqlx)?;

            map_row_to_intent(row)
        })
    }

    fn append_transition_if_current(
        &mut self,
        intent_db_id: i64,
        expected_version: i32,
        expected_from: TransferState,
        to: TransferState,
        reason: String,
        data_json: Option<serde_json::Value>,
    ) -> BoxFut<'_, Result<bool, RepoError>> {
        Box::pin(async move {
            let res = sqlx::query(
                r#"
            WITH ok AS (
                SELECT 1
                FROM transfer_intents
                WHERE id = $1
                  AND current_state = $2
                  AND version = $6
            ),
            ins AS (
                INSERT INTO transfer_intent_transitions
                    (intent_id, from_state, to_state, reason, data_json, at)
                SELECT
                    $1, $2, $3, $4, $5, now()
                FROM ok
                RETURNING 1
            )
            UPDATE transfer_intents
            SET updated_at = now()
            WHERE id = $1
              AND EXISTS (SELECT 1 FROM ins)
            "#,
            )
                .bind(intent_db_id)
                .bind(expected_from.as_str())
                .bind(to.as_str())
                .bind(reason)
                .bind(data_json)
                .bind(expected_version)
                .execute(&mut **self.tx)
                .await
                .map_err(map_sqlx)?;

            Ok(res.rows_affected() == 1)
        })
    }


    fn update_intent_state_cas(
        &mut self,
        intent_db_id: i64,
        expected_version: i32,
        expected_state: TransferState,
        new_state: TransferState,
        patch: IntentPatch,
    ) -> BoxFut<'_, Result<(), RepoError>> {
        Box::pin(async move {
            let new_version = expected_version
                .checked_add(1)
                .ok_or_else(|| RepoError::Integrity {
                    message: "version overflow".into(),
                })?;

            let required_usdt_bd: Option<BigDecimal> =
                patch.required_usdt_minor.map(BigDecimal::from);

            let quote_json: Option<serde_json::Value> = match patch.quote.as_ref() {
                None => None,
                Some(q) => {
                    let dto = mappers::fx_quote::to_json(q); // FxQuoteJson
                    Some(serde_json::to_value(dto).map_err(|e| RepoError::Integrity {
                        message: format!("failed to serialize quote dto: {e}"),
                    })?)
                }
            };

            let failure_json: Option<serde_json::Value> = patch
                .failure
                .as_ref()
                .map(|f| mappers::failure_info::to_json(f)); // already Value

            let res = sqlx::query(
                r#"
                UPDATE transfer_intents
                SET current_state = $4,
                    version = $5,
                    quote_json = COALESCE($6, quote_json),
                    quote_expires_at = COALESCE($7, quote_expires_at),
                    required_usdt_minor = COALESCE($8, required_usdt_minor),
                    failure_json = COALESCE($9, failure_json),
                    updated_at = now()
                WHERE id = $1
                  AND version = $2
                  AND current_state = $3
                "#,
            )
                .bind(intent_db_id)
                .bind(expected_version)
                .bind(expected_state.as_str())
                .bind(new_state.as_str())
                .bind(new_version)
                .bind(quote_json)
                .bind(patch.quote_expires_at)
                .bind(required_usdt_bd)
                .bind(failure_json)
                .execute(&mut **self.tx)
                .await
                .map_err(map_sqlx)?;

            if res.rows_affected() == 0 {
                return Err(RepoError::Conflict {
                    message: format!(
                        "intent CAS conflict (intent_id={intent_db_id}, expected_version={expected_version}, expected_state={})",
                        expected_state.as_str()
                    ),
                });
            }

            Ok(())
        })
    }

    fn find_by_external_ref(
        &mut self,
        t: ExternalRefType,
        r: &ExternalRef,
    ) -> BoxFut<'_, Result<Option<TransferIntent>, RepoError>> {
        let ext_type = t.as_code().to_string();
        let ext_ref = r.as_str().to_string();

        Box::pin(async move {
            let row = sqlx::query_as::<_, TransferIntentRow>(
                r#"
                SELECT
                    id, public_id, external_ref_type, external_ref,
                    route_json, amount_minor, asset_code,
                    current_state, version,
                    quote_json, quote_expires_at, required_usdt_minor, failure_json,
                    created_at, updated_at
                FROM transfer_intents
                WHERE external_ref_type = $1 AND external_ref = $2
                "#,
            )
                .bind(ext_type)
                .bind(ext_ref)
                .fetch_optional(&mut **self.tx)
                .await
                .map_err(map_sqlx)?;

            Ok(row.map(map_row_to_intent).transpose()?)
        })
    }
}
