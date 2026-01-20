use crate::domain::events::DomainEvent;
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct AggregateRoot {
    id: String,
    version: i32,
    created_at: SystemTime,
    updated_at: SystemTime,
    uncommitted_events: Vec<DomainEvent>,
}

impl AggregateRoot {
    pub fn new(id: impl Into<String>) -> Self {
        let now = SystemTime::now();
        Self {
            id: id.into(),
            version: 0,
            created_at: now,
            updated_at: now,
            uncommitted_events: Vec::new(),
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn version(&self) -> i32 {
        self.version
    }

    pub fn created_at(&self) -> SystemTime {
        self.created_at
    }

    pub fn updated_at(&self) -> SystemTime {
        self.updated_at
    }

    pub fn raise_event(&mut self, event: DomainEvent) {
        self.uncommitted_events.push(event);
        self.updated_at = SystemTime::now();
    }

    pub fn pull_events(&self) -> Vec<DomainEvent> {
        self.uncommitted_events.clone()
    }

    pub fn clear_events(&mut self) {
        self.uncommitted_events.clear();
    }

    pub fn commit_version(&mut self) {
        self.version += 1;
    }
}
