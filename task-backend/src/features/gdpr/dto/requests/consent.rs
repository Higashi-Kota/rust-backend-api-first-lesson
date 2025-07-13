use crate::features::gdpr::models::user_consent::ConsentType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;

/// Consent update request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ConsentUpdateRequest {
    pub consents: HashMap<ConsentType, bool>,
    #[validate(length(max = 500, message = "Reason cannot exceed 500 characters"))]
    pub reason: Option<String>,
}

/// Single consent update request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SingleConsentUpdateRequest {
    pub consent_type: ConsentType,
    pub is_granted: bool,
    #[validate(length(max = 500, message = "Reason cannot exceed 500 characters"))]
    pub reason: Option<String>,
}
