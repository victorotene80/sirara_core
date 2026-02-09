use serde::{Deserialize};

#[derive(Debug, Deserialize)]
pub struct MonnifyEnvelope<T> {
    pub requestSuccessful: bool,
    pub responseMessage: String,
    pub responseCode: String,
    pub responseBody: Option<T>,
}