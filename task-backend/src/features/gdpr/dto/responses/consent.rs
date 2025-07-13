use crate::features::gdpr::models::user_consent::ConsentType;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Consent status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentStatusResponse {
    pub user_id: Uuid,
    pub consents: Vec<ConsentStatus>,
    pub last_updated: DateTime<Utc>,
}

/// Individual consent status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentStatus {
    pub consent_type: ConsentType,
    pub is_granted: bool,
    pub granted_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub last_updated: DateTime<Utc>,
    pub display_name: String,
    pub description: String,
    pub is_required: bool,
}

/// Consent history response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentHistoryResponse {
    pub user_id: Uuid,
    pub history: Vec<ConsentHistoryEntry>,
    pub total_count: u64,
}

/// Consent history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentHistoryEntry {
    pub id: Uuid,
    pub consent_type: ConsentType,
    pub action: String, // "granted" or "revoked"
    pub is_granted: bool,
    pub timestamp: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}
