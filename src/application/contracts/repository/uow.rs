use crate::application::AppError;
use crate::application::contracts::repository::{BoxFut, TxContext};

pub trait UnitOfWork: Send + Sync {
    fn with_tx<'a, T: Send + 'a>(
        &'a self,
        f: impl for<'tx> FnOnce(&'tx mut dyn TxContext) -> BoxFut<'tx, Result<T, AppError>>
        + Send
        + 'a,
    ) -> BoxFut<'a, Result<T, AppError>>;
}
