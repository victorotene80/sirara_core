use chrono::Utc;

use crate::domain::entities::OnchainTransaction;
use crate::domain::repository::RepoError;
use crate::domain::value_objects::{
    Chain, OnchainAddress, OnchainTxStatus, PublicId, RegionCode, TxHash, AssetCode, Money
};
use crate::infrastructure::persistence::mappers::{numeric0_to_i128_strict, failure_info};
use crate::infrastructure::persistence::models::OnchainTxRow;

impl OnchainTxRow {
    pub fn to_domain(self) -> Result<OnchainTransaction, RepoError> {
        let chain = Chain::new(self.chain).map_err(|e| RepoError::Integrity {
            message: format!("invalid chain in onchain_transactions(id={}): {:?}", self.id, e),
        })?;

        let asset_code = AssetCode::new(self.asset_code).map_err(|e| RepoError::Integrity {
            message: format!("invalid asset_code in onchain_transactions(id={}): {:?}", self.id, e),
        })?;

        let region_code = RegionCode::new(self.region_code).map_err(|e| RepoError::Integrity {
            message: format!("invalid region_code in onchain_transactions(id={}): {:?}", self.id, e),
        })?;

        let status = OnchainTxStatus::from_str(self.status.as_str()).ok_or_else(|| RepoError::Integrity {
            message: format!("invalid status '{}' in onchain_transactions(id={})", self.status, self.id),
        })?;

        let contract_address = OnchainAddress::new(self.contract_address.as_str())
            .map_err(|e| RepoError::Integrity {
                message: format!(
                    "invalid contract_address in onchain_transactions(id={}): {:?}",
                    self.id, e
                ),
            })?;

        let from_address = OnchainAddress::new(self.from_address.as_str())
            .map_err(|e| RepoError::Integrity {
                message: format!("invalid from_address: {}", e),
            })?;

        let to_address = OnchainAddress::new(self.to_address.as_str())
            .map_err(|e| RepoError::Integrity {
                message: format!(
                    "invalid to_address in onchain_transactions(id={}): {:?}",
                    self.id, e
                ),
            })?;

        contract_address.validate_for_chain(&chain).map_err(|e| RepoError::Integrity {
            message: format!("contract_address fails chain validation (id={}): {:?}", self.id, e),
        })?;
        from_address.validate_for_chain(&chain).map_err(|e| RepoError::Integrity {
            message: format!("from_address fails chain validation (id={}): {:?}", self.id, e),
        })?;
        to_address.validate_for_chain(&chain).map_err(|e| RepoError::Integrity {
            message: format!("to_address fails chain validation (id={}): {:?}", self.id, e),
        })?;

        let amount_minor = numeric0_to_i128_strict(&self.amount_minor, self.id)?;
        let amount = Money::debit(amount_minor).map_err(|e| RepoError::Integrity {
            message: format!("invalid amount_minor in onchain_transactions(id={}): {:?}", self.id, e),
        })?;

        let tx_hash = match self.tx_hash {
            None => None,
            Some(s) => Some(TxHash::new(s).map_err(|e| RepoError::Integrity {
                message: format!("invalid tx_hash in onchain_transactions(id={}): {:?}", self.id, e),
            })?),
        };

        let failure = match self.failure_json.as_ref() {
            None => None,
            Some(v) => Some(
                failure_info::from_json(v).map_err(|e| RepoError::Integrity {
                    message: format!("invalid failure_json in onchain_transactions(id={}): {}", self.id, e),
                })?
            ),
        };

        OnchainTransaction::rehydrate(
            self.id,
            PublicId::new(self.public_id),
            self.intent_id,
            chain,
            asset_code,
            region_code,
            contract_address,
            from_address,
            to_address,
            self.idempotency_key,
            amount_minor,
            status,
            tx_hash,
            self.confirmations,
            self.submit_attempts,
            self.last_submit_at,
            self.last_checked_at,
            failure,
            self.version,
            self.created_at,
            self.updated_at,
        )
            .map_err(|e| RepoError::Integrity {
                message: format!("failed to rehydrate OnchainTransaction(id={}): {:?}", self.id, e),
            })
    }
}
