use std::collections::HashMap;

use crate::domain::entities::LedgerAccount;
use crate::domain::error::DomainError;
use crate::domain::value_objects::{ExternalRef, ExternalRefType, Money, PublicId};

#[derive(Debug, Clone)]
pub struct JournalLineDraft {
    pub account_id: i64,
    pub amount: Money, 
}

#[derive(Debug, Clone)]
pub struct JournalDraft {
    pub public_id: PublicId,
    pub external_ref_type: ExternalRefType,
    pub external_ref: ExternalRef,
    pub description: Option<String>,
    pub created_by: String,
    lines: Vec<JournalLineDraft>,
}

impl JournalDraft {
    pub fn new(
        public_id: PublicId,
        external_ref_type: ExternalRefType,
        external_ref: ExternalRef,
        created_by: impl Into<String>,
        description: Option<String>,
    ) -> Result<Self, DomainError> {
        let created_by = created_by.into();
        if created_by.trim().is_empty() {
            return Err(DomainError::CreatedByEmpty);
        }

        Ok(Self {
            public_id,
            external_ref_type,
            external_ref,
            description,
            created_by,
            lines: vec![],
        })
    }

    pub fn add_line(&mut self, account_id: i64, amount: Money) {
        self.lines.push(JournalLineDraft { account_id, amount });
    }

    pub fn lines(&self) -> &[JournalLineDraft] {
        &self.lines
    }

    // ---- pure invariants ----

    pub fn ensure_non_empty(&self) -> Result<(), DomainError> {
        if self.lines.is_empty() {
            return Err(DomainError::JournalEmpty);
        }
        Ok(())
    }

    pub fn ensure_balanced(&self) -> Result<(), DomainError> {
        let sum: i128 = self.lines.iter().map(|l| l.amount.minor()).sum();
        if sum != 0 {
            return Err(DomainError::JournalNotBalanced);
        }
        Ok(())
    }

    pub fn ensure_no_zero_lines(&self) -> Result<(), DomainError> {
        if self.lines.iter().any(|l| l.amount.minor() == 0) {
            return Err(DomainError::JournalLineAmountZero);
        }
        Ok(())
    }

    fn compress_lines(lines: Vec<JournalLineDraft>) -> Result<Vec<JournalLineDraft>, DomainError> {
        let mut net: HashMap<i64, i128> = HashMap::new();
        for l in lines {
            *net.entry(l.account_id).or_insert(0) += l.amount.minor();
        }

        let mut out: Vec<JournalLineDraft> = Vec::with_capacity(net.len());
        for (account_id, minor) in net {
            if minor == 0 {
                continue;
            }
            out.push(JournalLineDraft {
                account_id,
                amount: Money::from_signed_minor(minor)?,
            });
        }
        Ok(out)
    }
    
    pub fn validate_with_accounts(
        self,
        accounts_by_id: &HashMap<i64, &LedgerAccount>,
    ) -> Result<ValidatedJournal, DomainError> {
        self.ensure_non_empty()?;
        self.ensure_balanced()?;
        self.ensure_no_zero_lines()?;

        // compress lines to reduce ambiguity/noise
        let compressed = Self::compress_lines(self.lines)?;

        let mut asset_id: Option<i16> = None;

        for line in &compressed {
            let acct = accounts_by_id
                .get(&line.account_id)
                .ok_or(DomainError::LedgerAccountNotFound { account_id: line.account_id })?;

            acct.ensure_active()?;

            match asset_id {
                None => asset_id = Some(acct.asset_id()),
                Some(aid) if aid != acct.asset_id() => return Err(DomainError::CrossAssetPostingNotAllowed),
                _ => {}
            }
        }

        let lines = compressed
            .into_iter()
            .map(|l| JournalLine { account_id: l.account_id, amount: l.amount })
            .collect();

        let asset_id = asset_id.ok_or(DomainError::JournalEmpty)?;

        Ok(ValidatedJournal {
            public_id: self.public_id,
            external_ref_type: self.external_ref_type,
            external_ref: self.external_ref,
            description: self.description,
            created_by: self.created_by,
            asset_id,
            lines,
        })
    }
}

#[derive(Debug, Clone)]
pub struct JournalLine {
    pub account_id: i64,
    pub amount: Money,
}

#[derive(Debug, Clone)]
pub struct ValidatedJournal {
    pub public_id: PublicId,
    pub external_ref_type: ExternalRefType,
    pub external_ref: ExternalRef,
    pub description: Option<String>,
    pub created_by: String,
    pub asset_id: i16,
    pub lines: Vec<JournalLine>,
}

#[derive(Debug, Clone)]
pub struct PostedJournal {
    pub db_id: i64,
    pub public_id: PublicId,
    pub external_ref_type: ExternalRefType,
    pub external_ref: ExternalRef,
    pub description: Option<String>,
    pub created_by: String,
    pub asset_id: i16,
    pub lines: Vec<JournalLine>,
}

impl ValidatedJournal {
    pub fn into_posted(self, db_id: i64) -> PostedJournal {
        PostedJournal {
            db_id,
            public_id: self.public_id,
            external_ref_type: self.external_ref_type,
            external_ref: self.external_ref,
            description: self.description,
            created_by: self.created_by,
            asset_id: self.asset_id,
            lines: self.lines,
        }
    }
}
