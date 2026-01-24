use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct TransferIntentDTO {
    pub id: Uuid,
    pub external_ref: String,
    pub state: String,
    pub version: i32,
    pub created_at_rfc3339: String,
    pub updated_at_rfc3339: String,
}

#[derive(Debug, Clone)]
pub struct TransferIntentWithTransitionsDTO {
    pub intent: TransferIntentDTO,
    pub transitions: Vec<TransitionDTO>,
    pub failure: Option<FailureDTO>,
}

#[derive(Debug, Clone)]
pub struct TransitionDTO {
    pub from: String,
    pub to: String,
    pub at_rfc3339: String,
    pub reason: Option<String>,
    pub evidence: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FailureDTO {
    pub class: String,
    pub reason: String,
    pub message: Option<String>,
    pub failed_at_rfc3339: String,
    pub compensation: Vec<String>,
}
