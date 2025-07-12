use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// GDPR data export response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataExportResponse {
    pub user_data: UserDataExport,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tasks: Option<Vec<TaskDataExport>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub teams: Option<Vec<TeamDataExport>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscription_history: Option<Vec<SubscriptionHistoryExport>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activity_logs: Option<Vec<ActivityLogExport>>,
    pub exported_at: DateTime<Utc>,
}

/// User data export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDataExport {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub is_active: bool,
    pub email_verified: bool,
    pub role_name: String,
    pub subscription_tier: String,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Task data export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDataExport {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub due_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Team data export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamDataExport {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub role_in_team: String,
    pub joined_at: DateTime<Utc>,
}

/// Subscription history export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionHistoryExport {
    pub id: Uuid,
    pub previous_tier: Option<String>,
    pub new_tier: String,
    pub changed_at: DateTime<Utc>,
    pub reason: Option<String>,
}

/// Activity log export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityLogExport {
    pub id: Uuid,
    pub action: String,
    pub resource_type: Option<String>,
    pub resource_id: Option<Uuid>,
    pub timestamp: DateTime<Utc>,
}
