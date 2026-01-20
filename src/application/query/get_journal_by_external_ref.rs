use crate::domain::value_objects::{ExternalRef, ExternalRefType};

pub struct GetJournalByExternalRefQuery {
    pub external_ref_type: ExternalRefType,
    pub external_ref: ExternalRef,
}