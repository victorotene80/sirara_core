use crate::domain::value_objects::AssetCode;

#[derive(Debug, Clone)]
pub struct Asset {
    id: i16,
    code: AssetCode,
    decimals: i16,
    is_active: bool,
}

impl Asset {
    pub fn new(id: i16, code: AssetCode, decimals: i16, is_active: bool) -> Self {
        Self { id, code, decimals, is_active }
    }

    pub fn id(&self) -> i16 { self.id }
    pub fn code(&self) -> &AssetCode { &self.code }
    pub fn decimals(&self) -> i16 { self.decimals }
    pub fn is_active(&self) -> bool { self.is_active }
}
