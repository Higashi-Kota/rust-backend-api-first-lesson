use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// GDPR deletion response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataDeletionResponse {
    pub user_id: Uuid,
    pub deleted_at: DateTime<Utc>,
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
