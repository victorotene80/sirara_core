use crate::domain::aggregate::TransferIntent;
#[derive(Debug, Clone)]
pub enum InternalTransferResult {
    Success {
        intent: TransferIntent,
    },
    RejectedInsufficientFunds {
        available_minor: i128,
        required_minor: i128,
    },
}
