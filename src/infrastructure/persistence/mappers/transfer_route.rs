use crate::domain::error::DomainError;
use crate::domain::value_objects::{Chain, Memo, OnchainAddress, RegionCode, TransferRoute};

use crate::infrastructure::persistence::mappers::destination::{
    from_json as dest_from_json,
    to_json as dest_to_json,
};
use crate::infrastructure::persistence::models::{TransferRouteJson};

pub fn from_json(dto: TransferRouteJson) -> Result<TransferRoute, DomainError> {
    match dto {
        TransferRouteJson::INTRA { region_code, from_user_id, to_user_id } => {
            let region = RegionCode::new(&region_code)?;
            TransferRoute::intra_transfer(region, from_user_id, to_user_id)
        }

        TransferRouteJson::BANK { region_code, from_user_id, destination } => {
            let region = RegionCode::new(&region_code)?;
            let dest = dest_from_json(destination)?;
            TransferRoute::bank_transfer(region, from_user_id, dest)
        }

        TransferRouteJson::CRYPTO { region_code, from_user_id, chain, to_address, memo } => {
            let region = RegionCode::new(&region_code)?;

            let chain = Chain::new(&chain)?;
            let address = OnchainAddress::new(&to_address)?;
            let memo = match memo {
                None => None,
                Some(s) => Some(Memo::new(&s)?),
            };

            let dest = crate::domain::value_objects::Destination::onchain(chain, address, memo)?;
            TransferRoute::crypto_transfer(region, from_user_id, dest)
        }
    }
}

pub fn to_json(route: &TransferRoute) -> Result<TransferRouteJson, DomainError> {
    Ok(match route {
        TransferRoute::IntraTransfer { region, from_user_id, to_user_id } => TransferRouteJson::INTRA {
            region_code: region.as_str().to_string(),
            from_user_id: *from_user_id,
            to_user_id: *to_user_id,
        },

        TransferRoute::BankTransfer { region, from_user_id, destination } => TransferRouteJson::BANK {
            region_code: region.as_str().to_string(),
            from_user_id: *from_user_id,
            destination: dest_to_json(destination),
        },

        TransferRoute::CryptoTransfer { region, from_user_id, destination } => {
            match destination {
                crate::domain::value_objects::Destination::Onchain { chain, address, memo } => {
                    TransferRouteJson::CRYPTO {
                        region_code: region.as_str().to_string(),
                        from_user_id: *from_user_id,
                        chain: chain.as_str().to_string(),
                        to_address: address.as_str().to_string(),
                        memo: memo.as_ref().map(|m| m.as_str().to_string()),
                    }
                }
                _ => return Err(DomainError::InvalidTransferRoute),
            }
        }
    })
}
