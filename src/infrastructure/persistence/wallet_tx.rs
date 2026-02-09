use chrono::{DateTime, Utc};
use sqlx::{Postgres, Transaction};

use crate::application::contracts::repository::{BoxFut, HotWalletRepositoryTx};
use crate::domain::entities::HotWallet;
use crate::domain::repository::RepoError;
use crate::domain::value_objects::{AssetCode, Chain, PublicId, RegionCode};

use crate::infrastructure::persistence::error_map::map_sqlx;
use crate::infrastructure::persistence::mappers::i128_to_bd;
use crate::infrastructure::persistence::models::HotWalletRow;

pub struct PgHotWalletTxRepo<'a, 'c> {
    tx: &'a mut Transaction<'c, Postgres>,
}

impl<'a, 'c> PgHotWalletTxRepo<'a, 'c> {
    pub fn new(tx: &'a mut Transaction<'c, Postgres>) -> Self {
        Self { tx }
    }
}

impl<'a, 'c> HotWalletRepositoryTx for PgHotWalletTxRepo<'a, 'c> {
    fn insert_if_absent(&mut self, wallet: HotWallet) -> BoxFut<'_, Result<HotWallet, RepoError>> {
        Box::pin(async move {
            let chain_s = wallet.chain().as_str();
            let asset_s = wallet.asset_code().trim().to_uppercase();
            let region_s = wallet.region_code().as_str().trim().to_uppercase();

            let inserted = sqlx::query_as::<_, HotWalletRow>(
                r#"
                INSERT INTO hot_wallets
                    (public_id, chain, asset_code, region_code,
                     address_base58, address_hex,
                     secret_handle,
                     max_balance_minor, is_active, version)
                VALUES
                    ($1, $2, $3, $4,
                     $5, $6,
                     $7,
                     $8, $9, 0)
                ON CONFLICT (chain, asset_code, region_code) DO NOTHING
                RETURNING
                    id, public_id, chain, asset_code, region_code,
                    address_base58, address_hex,
                    secret_handle,
                    max_balance_minor, is_active,
                    version,
                    created_at, updated_at
                "#,
                )
                .bind(wallet.public_id().value())
                .bind(chain_s)
                .bind(&asset_s)
                .bind(wallet.secret_handle())
                .bind(&region_s)
                .bind(wallet.address_base58())
                .bind(wallet.address_hex())
                .bind(i128_to_bd(wallet.max_balance_minor()))
                .bind(wallet.is_active())
                .fetch_optional(&mut **self.tx)
                .await
                .map_err(map_sqlx)?;

            if let Some(row) = inserted {
                return row.to_domain();
            }

            let existing = sqlx::query_as::<_, HotWalletRow>(
                r#"
                SELECT
                    id, public_id, chain, asset_code, region_code,
                    address_base58, address_hex, secret_handle,
                    max_balance_minor, is_active,
                    version,
                    created_at, updated_at
                FROM hot_wallets
                WHERE chain = $1 AND asset_code = $2 AND region_code = $3
                LIMIT 1
                "#,
            )
                .bind(chain_s)
                .bind(&asset_s)
                .bind(&region_s)
                .fetch_one(&mut **self.tx)
                .await
                .map_err(map_sqlx)?;

            existing.to_domain()
        })
    }

    fn update_active_flag(
        &mut self,
        wallet_public_id: PublicId,
        is_active: bool,
        expected_version: i32,
        now: DateTime<Utc>,
    ) -> BoxFut<'_, Result<i32, RepoError>> {
        Box::pin(async move {
            let res = sqlx::query(
                r#"
                UPDATE hot_wallets
                SET
                    is_active = $2,
                    version = version + 1,
                    updated_at = $3
                WHERE public_id = $1
                  AND version = $4
                "#,
            )
                .bind(wallet_public_id.value())
                .bind(is_active)
                .bind(now)
                .bind(expected_version)
                .execute(&mut **self.tx)
                .await
                .map_err(map_sqlx)?;

            let affected = res.rows_affected() as i32;
            if affected == 1 {
                return Ok(expected_version + 1);
            }

            let exists = sqlx::query_scalar::<_, bool>(
                r#"SELECT EXISTS (SELECT 1 FROM hot_wallets WHERE public_id = $1)"#,
            )
                .bind(wallet_public_id.value())
                .fetch_one(&mut **self.tx)
                .await
                .map_err(map_sqlx)?;

            if !exists {
                return Err(RepoError::NotFound {
                    entity: format!("hot_wallet public_id={}", wallet_public_id.value()),
                });
            }

            Err(RepoError::Conflict {
                message: format!(
                    "hot_wallet version conflict (public_id={}, expected_version={})",
                    wallet_public_id.value(),
                    expected_version
                ),
            })
        })
    }

    fn find_active_by_chain_asset_region(
        &mut self,
        chain: &Chain,
        asset_code: &AssetCode,
        region_code: &RegionCode,
    ) -> BoxFut<'_, Result<Option<HotWallet>, RepoError>> {
        let chain_s = chain.as_str();
        let asset_s = asset_code.as_str().trim().to_uppercase();
        let region_s = region_code.as_str().trim().to_uppercase();

        Box::pin(async move {
            let row = sqlx::query_as::<_, HotWalletRow>(
                r#"
                SELECT
                    id, public_id, chain, asset_code, region_code,
                    address_base58, address_hex, secret_handle,
                    max_balance_minor, is_active,
                    version,
                    created_at, updated_at
                FROM hot_wallets
                WHERE is_active = true
                  AND chain = $1
                  AND asset_code = $2
                  AND region_code = $3
                LIMIT 1
                "#,
            )
                .bind(chain_s)
                .bind(&asset_s)
                .bind(&region_s)
                .fetch_optional(&mut **self.tx)
                .await
                .map_err(map_sqlx)?;

            Ok(row.map(|r| r.to_domain()).transpose()?)
        })
    }
}
