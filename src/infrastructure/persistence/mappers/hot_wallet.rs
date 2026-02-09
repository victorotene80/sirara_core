use crate::domain::entities::HotWallet;
use crate::domain::error::DomainError;
use crate::domain::repository::RepoError;
use crate::domain::value_objects::{Chain, PublicId, RegionCode};
use crate::infrastructure::persistence::mappers::numeric0_to_i128_strict;
use crate::infrastructure::persistence::models::HotWalletRow;

pub fn row_to_domain(row: HotWalletRow) -> Result<HotWallet, RepoError> {
    let chain = Chain::new(&row.chain).map_err(|_| RepoError::Integrity {
        message: "bad chain in hot_wallets".into(),
    })?;

    let region = RegionCode::new(&row.region_code).map_err(|_| RepoError::Integrity {
        message: "bad region_code in hot_wallets".into(),
    })?;

    let max_balance_minor =
        crate::infrastructure::persistence::mappers::numeric::bd_to_i128(
            &row.max_balance_minor,
            "hot_wallets.max_balance_minor",
        )?;

    HotWallet::rehydrate(
        row.id,
        PublicId::new(row.public_id),
        chain,
        row.asset_code,
        region,
        row.address_base58,
        row.address_hex,
        max_balance_minor,
        row.is_active,
        row.version,
        row.created_at,
        row.updated_at,
    )
        .map_err(|e: DomainError| RepoError::Integrity {
            message: format!("rehydrate hot wallet: {e:?}"),
        })
}

impl HotWalletRow {
    pub fn to_domain(self) -> Result<HotWallet, RepoError> {
        let chain = Chain::new(&self.chain).map_err(|e| RepoError::Integrity {
            message: format!("invalid chain in hot_wallets(id={}): {:?}", self.id, e),
        })?;

        let region_code = RegionCode::new(&self.region_code).map_err(|e| RepoError::Integrity {
            message: format!("invalid region_code in hot_wallets(id={}): {:?}", self.id, e),
        })?;

        let max = numeric0_to_i128_strict(&self.max_balance_minor, self.id)?;

        HotWallet::rehydrate(
            self.id,
            PublicId::new(self.public_id),
            chain,
            self.asset_code,
            region_code,
            self.address_base58,
            self.address_hex,
            max,
            self.is_active,
            self.version,     
            self.created_at,  
            self.updated_at,
        )
            .map_err(|e| RepoError::Integrity {
                message: format!("failed to rehydrate HotWallet(id={}): {:?}", self.id, e),
            })
    }
}
