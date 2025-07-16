// task-backend/src/api/dto/gdpr_dto.rs

use crate::domain::user_consent_model::ConsentType;
use crate::types::{optional_timestamp, Timestamp};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use validator::Validate;

/// GDPR data export request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataExportRequest {
    pub include_tasks: bool,
    pub include_teams: bool,
    pub include_subscription_history: bool,
    pub include_activity_logs: bool,
}

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
    pub exported_at: Timestamp,
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
    #[serde(default, with = "optional_timestamp")]
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

/// Task data export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDataExport {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    #[serde(default, with = "optional_timestamp")]
    pub due_date: Option<DateTime<Utc>>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

/// Team data export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamDataExport {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub role_in_team: String,
    pub joined_at: Timestamp,
}

/// Subscription history export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionHistoryExport {
    pub id: Uuid,
    pub previous_tier: Option<String>,
    pub new_tier: String,
    pub changed_at: Timestamp,
    pub reason: Option<String>,
}

/// Activity log export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityLogExport {
    pub id: Uuid,
    pub action: String,
    pub resource_type: Option<String>,
    pub resource_id: Option<Uuid>,
    pub timestamp: Timestamp,
}

/// GDPR deletion request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataDeletionRequest {
    pub confirm_deletion: bool,
    pub reason: Option<String>,
}

/// GDPR deletion response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataDeletionResponse {
    pub user_id: Uuid,
    pub deleted_at: Timestamp,
    pub deleted_records: DeletedRecordsSummary,
}

/// Summary of deleted records
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletedRecordsSummary {
    pub user_data: bool,
    pub tasks_count: u64,
    pub teams_count: u64,
    pub subscription_history_count: u64,
    pub activity_logs_count: u64,
    pub refresh_tokens_count: u64,
}

/// GDPR compliance status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceStatusResponse {
    pub user_id: Uuid,
    pub data_retention_days: u32,
    #[serde(default, with = "optional_timestamp")]
    pub last_data_export: Option<DateTime<Utc>>,
    pub deletion_requested: bool,
    #[serde(default, with = "optional_timestamp")]
    pub deletion_scheduled_for: Option<DateTime<Utc>>,
}

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

/// Consent status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentStatusResponse {
    pub user_id: Uuid,
    pub consents: Vec<ConsentStatus>,
    pub last_updated: Timestamp,
}

/// Individual consent status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentStatus {
    pub consent_type: ConsentType,
    pub is_granted: bool,
    #[serde(default, with = "optional_timestamp")]
    pub granted_at: Option<DateTime<Utc>>,
    #[serde(default, with = "optional_timestamp")]
    pub revoked_at: Option<DateTime<Utc>>,
    pub last_updated: Timestamp,
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
    pub timestamp: Timestamp,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}
