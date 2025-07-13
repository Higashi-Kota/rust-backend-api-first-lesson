use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize)]
pub struct TrackFeatureUsageRequest {
    pub feature_name: String,
    pub action_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalyticsQuery {
    pub period_days: Option<i32>,
    pub include_details: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeatureUsageQuery {
    pub days: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportQuery {
    pub user_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalyticsTimeRangeRequest {
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub granularity: Option<String>,
    pub period_days: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskAnalyticsRequest {
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub user_id: Option<Uuid>,
    pub status_filter: Option<Vec<String>>,
    pub priority_filter: Option<Vec<String>>,
    pub include_details: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct AdvancedExportRequest {
    pub export_type: String,
    pub format: ExportFormat,
    pub filters: Option<HashMap<String, serde_json::Value>>,
    pub columns: Option<Vec<String>>,
    pub date_range: Option<DateRange>,
    #[validate(range(
        min = 1,
        max = 100000,
        message = "Max records must be between 1 and 100000"
    ))]
    pub max_records: Option<u32>,
    pub include_metadata: Option<bool>,
    pub custom_fields: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DateRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Csv,
    Json,
    Excel,
    Pdf,
}

// Re-export specific DTOs from admin module for compatibility
// pub use crate::features::admin::dto::analytics::{UserAnalyticsExport, UserAnalyticsExportQuery};
