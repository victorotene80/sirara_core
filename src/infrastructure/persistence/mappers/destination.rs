use crate::domain::error::DomainError;
use crate::domain::value_objects::{AccountNumber, BankCode, Chain, Destination, Memo, OnchainAddress};

use crate::infrastructure::persistence::models::DestinationJson;

pub fn to_json(dest: &Destination) -> DestinationJson {
    match dest {
        Destination::Bank { bank_code, account_number } => DestinationJson::BANK {
            bank_code: bank_code.as_str().to_string(),
            account_number: account_number.as_str().to_string(),
        },

        Destination::Onchain { chain, address, memo } => DestinationJson::ONCHAIN {
            chain: chain.as_str().to_string(),
            address: address.as_str().to_string(),
            memo: memo.as_ref().map(|m| m.as_str().to_string()),
        },
    }
}

pub fn from_json(dto: DestinationJson) -> Result<Destination, DomainError> {
    match dto {
        DestinationJson::BANK { bank_code, account_number } => {
            let bank_code = BankCode::new(&bank_code)?;
            let account_number = AccountNumber::new(&account_number)?;
            Destination::bank(bank_code, account_number)
        }

        DestinationJson::ONCHAIN { chain, address, memo } => {
            let chain = Chain::new(&chain)?;
            let address = OnchainAddress::new(&address)?;

            let memo = match memo {
                None => None,
                Some(m) => Some(Memo::new(&m)?),
            };

            Destination::onchain(chain, address, memo)
        }
    }
}

