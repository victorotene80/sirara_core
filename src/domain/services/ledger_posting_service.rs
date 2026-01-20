use std::collections::HashMap;

use crate::domain::aggregate::ValidatedJournal;
use crate::domain::entities::{AccountType, LedgerAccount, OwnerType};
use crate::domain::error::DomainError;

#[derive(Debug, Clone)]
pub struct PolicyValidatedJournal(pub ValidatedJournal);

pub struct LedgerPostingService;

impl LedgerPostingService {
    pub fn validate(
        posting: ValidatedJournal,
        accounts_by_id: &HashMap<i64, &LedgerAccount>,
    ) -> Result<PolicyValidatedJournal, DomainError> {
        // 1) taxonomy integrity (owner matches bucket, required owner_id for user)
        for line in &posting.lines {
            let acct = accounts_by_id
                .get(&line.account_id)
                .ok_or(DomainError::LedgerAccountNotFound { account_id: line.account_id })?;
            Self::ensure_taxonomy(acct)?;
        }

        Self::enforce_hold_owner_constraint(&posting, accounts_by_id)?;

        Ok(PolicyValidatedJournal(posting))
    }

    fn ensure_taxonomy(acct: &LedgerAccount) -> Result<(), DomainError> {
        use AccountType::*;
        use OwnerType::*;

        let expected_owner = match acct.account_type() {
            UserAvailable | UserLocked => User,
            PlatformClearing => Platform,
            TreasuryAvailable | TreasuryLocked => Treasury,
            InventoryAvailable | InventoryLocked => Platform,
        };

        if acct.owner_type() != expected_owner {
            return Err(DomainError::AccountOwnerTypeMismatch {
                account_id: acct.id(),
                expected: format!("{:?}", expected_owner),
                actual: format!("{:?}", acct.owner_type()),
            });
        }

        if matches!(acct.account_type(), UserAvailable | UserLocked) && acct.owner_id().is_none() {
            return Err(DomainError::OwnerIdRequiredForUserAccount { account_id: acct.id() });
        }

        Ok(())
    }

    fn enforce_hold_owner_constraint(
        posting: &ValidatedJournal,
        accounts_by_id: &HashMap<i64, &LedgerAccount>,
    ) -> Result<(), DomainError> {
        use AccountType::*;

        let mut net: HashMap<i64, i128> = HashMap::new();
        for line in &posting.lines {
            *net.entry(line.account_id).or_insert(0) += line.amount.minor();
        }
        net.retain(|_, v| *v != 0);

        let mut avail: Vec<&LedgerAccount> = vec![];
        let mut locked: Vec<&LedgerAccount> = vec![];

        for (account_id, _) in &net {
            let acct = accounts_by_id
                .get(account_id)
                .ok_or(DomainError::LedgerAccountNotFound { account_id: *account_id })?;

            match acct.account_type() {
                UserAvailable => avail.push(*acct),
                UserLocked => locked.push(*acct),
                _ => {}
            }
        }

        if !avail.is_empty() && !locked.is_empty() {
            if avail.len() != 1 || locked.len() != 1 {
                return Err(DomainError::HoldPostingAmbiguous {
                    available_accounts: avail.len(),
                    locked_accounts: locked.len(),
                });
            }

            if avail[0].owner_id() != locked[0].owner_id() {
                return Err(DomainError::HoldMustBeSameUser {
                    available_account_id: avail[0].id(),
                    locked_account_id: locked[0].id(),
                });
            }
        }

        Ok(())
    }
}
