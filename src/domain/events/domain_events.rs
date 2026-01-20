use crate::domain::events::types::LedgerAccountCreated;

#[derive(Debug, Clone)]
pub enum DomainEvent {
    LedgerAccountCreated(LedgerAccountCreated),
}

impl DomainEvent {
    pub fn event_type(&self) -> &'static str {
        match self {
            DomainEvent::LedgerAccountCreated(_) => "ledger.account.created",
        }
    }
}
