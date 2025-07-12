use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// GDPR compliance status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceStatusResponse {
    pub user_id: Uuid,
    pub data_retention_days: u32,
    pub last_data_export: Option<DateTime<Utc>>,
    pub deletion_requested: bool,
    pub deletion_scheduled_for: Option<DateTime<Utc>>,
}
