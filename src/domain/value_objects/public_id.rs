use uuid::Uuid;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct PublicId(Uuid);

impl PublicId {
    pub fn new(id: Uuid) -> Self {
        Self(id)
    }

    pub fn value(&self) -> Uuid {
        self.0
    }
}
