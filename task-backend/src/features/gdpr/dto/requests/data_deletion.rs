use serde::{Deserialize, Serialize};

/// GDPR deletion request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataDeletionRequest {
    pub confirm_deletion: bool,
    pub reason: Option<String>,
}
