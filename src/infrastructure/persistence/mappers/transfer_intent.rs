use crate::domain::aggregate::TransferIntent;
use crate::domain::repository::RepoError;
use crate::domain::value_objects::{
    AssetCode, ExternalRef, ExternalRefType, FxQuote, FailureInfo, Money, PublicId, TransferRoute,
    TransferState,
};

use crate::infrastructure::persistence::mappers;
use crate::infrastructure::persistence::mappers::numeric::bd_to_i128;
use crate::infrastructure::persistence::models::{FxQuoteJson, TransferIntentRow, TransferRouteJson};

pub fn map_row_to_intent(row: TransferIntentRow) -> Result<TransferIntent, RepoError> {
    let public_id = PublicId::new(row.public_id);

    let external_ref_type =
        ExternalRefType::from_code(row.external_ref_type.as_str()).map_err(|e| RepoError::Integrity {
            message: format!("invalid external_ref_type in db: {e:?}"),
        })?;

    let external_ref = ExternalRef::new(row.external_ref).map_err(|e| RepoError::Integrity {
        message: format!("invalid external_ref in db: {e:?}"),
    })?;

    let route_dto: TransferRouteJson =
        serde_json::from_value(row.route_json).map_err(|e| RepoError::Integrity {
            message: format!("invalid route_json (cannot deserialize): {e}"),
        })?;

    let route: TransferRoute =
        mappers::transfer_route::from_json(route_dto).map_err(|e| RepoError::Integrity {
            message: format!("invalid route_json (domain validation failed): {e:?}"),
        })?;

    let amount_minor = bd_to_i128(&row.amount_minor, "amount_minor")?;
    let amount = Money::from_signed_minor(amount_minor).map_err(|e| RepoError::Integrity {
        message: format!("invalid amount_minor in db: {e:?}"),
    })?;

    let asset = AssetCode::new(row.asset_code).map_err(|e| RepoError::Integrity {
        message: format!("invalid asset_code in db: {e:?}"),
    })?;

    let state = TransferState::from_str(row.current_state.as_str()).ok_or_else(|| RepoError::Integrity {
        message: format!("invalid current_state in db: {}", row.current_state),
    })?;

    let quote: Option<FxQuote> = match (row.quote_json, row.quote_expires_at) {
        (None, None) => None,
        (None, Some(_)) => {
            return Err(RepoError::Integrity {
                message: "quote_expires_at set but quote_json is NULL".into(),
            });
        }
        (Some(_), None) => {
            return Err(RepoError::Integrity {
                message: "quote_json set but quote_expires_at is NULL".into(),
            });
        }
        (Some(v), Some(expires_at)) => {
            let dto: FxQuoteJson = serde_json::from_value(v).map_err(|e| RepoError::Integrity {
                message: format!("invalid quote_json: {e}"),
            })?;

            Some(mappers::fx_quote::from_json(dto, expires_at).map_err(|e| RepoError::Integrity {
                message: format!("invalid quote_json (domain): {e:?}"),
            })?)
        }
    };

    let required_usdt_minor = match row.required_usdt_minor {
        None => None,
        Some(bd) => Some(bd_to_i128(&bd, "required_usdt_minor")?),
    };

    let failure: Option<FailureInfo> = match row.failure_json {
        None => None,
        Some(v) => Some(mappers::failure_info::from_json(&v).map_err(|e| RepoError::Integrity {
            message: format!("invalid failure_json (domain): {e:?}"),
        })?),
    };

    TransferIntent::rehydrate(
        row.id,
        public_id,
        external_ref_type,
        external_ref,
        route,
        asset,
        amount,
        state,
        quote,
        required_usdt_minor,
        failure,
        row.version,
        row.created_at,
        row.updated_at,
    )
}
