use serde::{Deserialize, Serialize};

/// GDPR data export request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataExportRequest {
    pub include_tasks: bool,
    pub include_teams: bool,
    pub include_subscription_history: bool,
    pub include_activity_logs: bool,
}
