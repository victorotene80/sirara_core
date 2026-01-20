/*use std::collections::HashMap;

use crate::domain::aggregate::ValidatedJournal;
use crate::domain::entities::{AccountType, LedgerAccount, OwnerType};
use crate::domain::error::DomainError;

#[derive(Debug, Clone)]
pub struct PolicyValidatedJournal(pub ValidatedJournal);

pub struct PostingPolicy;

impl PostingPolicy {
    /// O(L) where L = number of journal lines.
    /// - One pass over lines to compute net per account
    /// - One pass over distinct accounts involved to validate + compute masks
    pub fn validate(
        posting: ValidatedJournal,
        accounts_by_id: &HashMap<i64, &LedgerAccount>,
    ) -> Result<PolicyValidatedJournal, DomainError> {
        // Net per account_id: debit +, credit -
        let mut net_by_account: HashMap<i64, i128> = HashMap::new();
        for line in &posting.lines {
            *net_by_account.entry(line.account_id).or_insert(0) += line.amount.minor();
        }

        // Bitmasks of AccountType appearing on sender/receiver sides
        let mut sender_mask: u16 = 0;
        let mut receiver_mask: u16 = 0;

        // Validate account config + fill masks
        for (account_id, net) in net_by_account {
            if net == 0 {
                continue;
            }

            let acct = accounts_by_id
                .get(&account_id)
                .ok_or(DomainError::LedgerAccountNotFound { account_id })?;

            Self::ensure_owner_matches_bucket(acct)?;

            let bit = 1u16 << acct.account_type().idx();

            if net < 0 {
                sender_mask |= bit;
            } else {
                receiver_mask |= bit;
            }
        }

        let sender_count = sender_mask.count_ones() as usize;
        let receiver_count = receiver_mask.count_ones() as usize;

        // Keep postings unambiguous + simple (first principles)
        if sender_count != 1 || receiver_count != 1 {
            return Err(DomainError::PostingShapeNotAllowed {
                senders: sender_count,
                receivers: receiver_count,
            });
        }

        let from = account_type_from_single_bit(sender_mask)?;
        let to = account_type_from_single_bit(receiver_mask)?;

        if !Self::is_allowed(from, to) {
            return Err(DomainError::PostingNotAllowed {
                from: format!("{:?}", from),
                to: format!("{:?}", to),
            });
        }

        Ok(PolicyValidatedJournal(posting))
    }

    fn ensure_owner_matches_bucket(acct: &LedgerAccount) -> Result<(), DomainError> {
        use AccountType::*;
        use OwnerType::*;

        let expected_owner = match acct.account_type() {
            UserAvailable | UserLocked => User,
            PlatformClearing => Platform,
            TreasuryAvailable | TreasuryLocked => Treasury,
            InventoryAvailable | InventoryLocked => Platform, // typical
        };

        if acct.owner_type() != expected_owner {
            return Err(DomainError::AccountOwnerTypeMismatch {
                account_id: acct.id(),
                expected: format!("{:?}", expected_owner),
                actual: format!("{:?}", acct.owner_type()),
            });
        }

        if matches!(acct.account_type(), UserAvailable | UserLocked) && acct.owner_id().is_none() {
            return Err(DomainError::OwnerIdRequiredForUserAccount {
                account_id: acct.id(),
            });
        }

        Ok(())
    }

    /// Allowed movement edges.
    fn is_allowed(from: AccountType, to: AccountType) -> bool {
        use AccountType::*;

        matches!(
            (from, to),
            // hold / unhold
            (UserAvailable, UserLocked)
                | (UserLocked, UserAvailable)
            // user paying platform
                | (UserAvailable, PlatformClearing)
                | (UserLocked, PlatformClearing)
            // platform internal
                | (PlatformClearing, TreasuryAvailable)
                | (TreasuryAvailable, TreasuryLocked)
                | (TreasuryLocked, TreasuryAvailable)
            // inventory reserve / release
                | (InventoryAvailable, InventoryLocked)
                | (InventoryLocked, InventoryAvailable)
            // treasury funds inventory (optional)
                | (TreasuryAvailable, InventoryAvailable)
        )
    }
}

/// Map AccountType to a stable bit index (0..N-1).
impl AccountType {
    #[inline]
    pub fn idx(self) -> u8 {
        match self {
            AccountType::UserAvailable => 0,
            AccountType::UserLocked => 1,
            AccountType::PlatformClearing => 2,
            AccountType::TreasuryAvailable => 3,
            AccountType::TreasuryLocked => 4,
            AccountType::InventoryAvailable => 5,
            AccountType::InventoryLocked => 6,
        }
    }
}

/// Convert a single-bit mask into an AccountType.
/// Assumes mask.count_ones() == 1.
#[inline]
fn account_type_from_single_bit(mask: u16) -> Result<AccountType, DomainError> {
    let idx = mask.trailing_zeros() as u8;

    match idx {
        0 => Ok(AccountType::UserAvailable),
        1 => Ok(AccountType::UserLocked),
        2 => Ok(AccountType::PlatformClearing),
        3 => Ok(AccountType::TreasuryAvailable),
        4 => Ok(AccountType::TreasuryLocked),
        5 => Ok(AccountType::InventoryAvailable),
        6 => Ok(AccountType::InventoryLocked),
        _ => Err(DomainError::Validation(format!(
            "invalid account type bit index: {}",
            idx
        ))),
    }
}
*/