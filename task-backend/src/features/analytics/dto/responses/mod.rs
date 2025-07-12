use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// Simple response structures for backwards compatibility
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemStatsResponse {
    pub total_users: u64,
    pub active_users: u64,
    pub total_tasks: u64,
    pub completed_tasks: u64,
    pub task_completion_rate: f64,
    pub total_organizations: u64,
    pub active_teams: u64,
    pub average_tasks_per_user: f64,
    pub subscription_distribution: Vec<SubscriptionTierDistribution>,
    pub suspicious_ips: Vec<String>,
    pub daily_active_users: u64,
    pub weekly_active_users: u64,
}

impl SystemStatsResponse {
    pub fn new() -> Self {
        Self {
            total_users: 0,
            active_users: 0,
            total_tasks: 0,
            completed_tasks: 0,
            task_completion_rate: 0.0,
            total_organizations: 0,
            active_teams: 0,
            average_tasks_per_user: 0.0,
            subscription_distribution: vec![],
            suspicious_ips: vec![],
            daily_active_users: 0,
            weekly_active_users: 0,
        }
    }
}

impl Default for SystemStatsResponse {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionTierDistribution {
    pub tier: String,
    pub count: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeatureUsage {
    pub feature_name: String,
    pub usage_count: u32,
    pub usage_percentage: f64,
    pub last_used: DateTime<Utc>,
    pub proficiency_level: ProficiencyLevel,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProficiencyLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserBehaviorAnalyticsResponse {
    pub user_id: Uuid,
    pub analysis_period: AnalysisPeriod,
    pub behavioral_metrics: BehavioralMetrics,
    pub activity_patterns: ActivityPatterns,
    pub feature_usage: FeatureUsageMetrics,
    pub performance_indicators: PerformanceIndicators,
    pub comparisons: Option<crate::features::admin::dto::analytics::UserComparisons>,
    pub recommendations: Vec<crate::features::admin::dto::analytics::UserRecommendation>,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalysisPeriod {
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub duration_days: u32,
    pub granularity: MetricGranularity,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MetricGranularity {
    Hourly,
    Daily,
    Weekly,
    Monthly,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BehavioralMetrics {
    pub login_frequency: LoginFrequency,
    pub session_duration: SessionDuration,
    pub activity_score: f64,
    pub engagement_level: EngagementLevel,
    pub feature_adoption_rate: f64,
    pub consistency_score: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginFrequency {
    pub daily_average: f64,
    pub weekly_average: f64,
    pub monthly_average: f64,
    pub consistency_score: f64,
    pub longest_streak_days: u32,
    pub current_streak_days: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionDuration {
    pub average_minutes: f64,
    pub median_minutes: f64,
    pub longest_session_minutes: f64,
    pub shortest_session_minutes: f64,
    pub session_count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EngagementLevel {
    Low,
    Medium,
    High,
    VeryHigh,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActivityPatterns {
    pub peak_activity_hours: Vec<u32>,
    pub most_active_days: Vec<String>,
    pub activity_distribution: ActivityDistribution,
    pub workflow_patterns: Vec<crate::features::admin::dto::analytics::WorkflowPattern>,
    pub seasonal_trends: Vec<SeasonalTrend>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActivityDistribution {
    pub morning: f64,
    pub afternoon: f64,
    pub evening: f64,
    pub weekday: f64,
    pub weekend: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SeasonalTrend {
    pub period: String,
    pub activity_level: f64,
    pub trend_direction: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeatureUsageMetrics {
    pub most_used_features: Vec<FeatureUsage>,
    pub least_used_features: Vec<FeatureUsage>,
    pub feature_progression: Vec<FeatureProgression>,
    pub subscription_utilization: SubscriptionUtilization,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeatureProgression {
    pub feature_name: String,
    pub progression_stage: String,
    pub time_spent_hours: f64,
    pub mastery_percentage: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionUtilization {
    pub current_tier: crate::core::subscription_tier::SubscriptionTier,
    pub tier_utilization_percentage: f64,
    pub underutilized_features: Vec<String>,
    pub upgrade_recommendations: Vec<String>,
    pub cost_efficiency_score: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceIndicators {
    pub task_completion_rate: f64,
    pub average_task_duration: f64,
    pub productivity_score: f64,
    pub error_rate: f64,
    pub satisfaction_indicators: SatisfactionIndicators,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SatisfactionIndicators {
    pub feature_satisfaction_score: f64,
    pub performance_satisfaction_score: f64,
    pub overall_satisfaction_score: f64,
    pub nps_score: Option<f64>,
    pub feedback_sentiment: SentimentScore,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SentimentScore {
    pub positive_percentage: f64,
    pub neutral_percentage: f64,
    pub negative_percentage: f64,
    pub overall_sentiment: SentimentCategory,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SentimentCategory {
    Negative,
    Neutral,
    Positive,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RecommendationType {
    FeatureAdoption,
    WorkflowOptimization,
    PerformanceImprovement,
    SubscriptionUpgrade,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RecommendationPriority {
    Low,
    Medium,
    High,
    Critical,
}

// Task statistics response
#[derive(Debug, Serialize, Deserialize)]
pub struct TaskStatsDetailResponse {
    pub overview: TaskStatsOverview,
    pub status_distribution: Vec<StatusDistribution>,
    pub priority_distribution: Vec<PriorityDistribution>,
    pub trends: TaskTrends,
    pub user_performance: Option<HashMap<Uuid, UserTaskPerformance>>,
    pub pagination: Option<PaginationInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskStatsOverview {
    pub total_tasks: u32,
    pub completed_tasks: u32,
    pub pending_tasks: u32,
    pub overdue_tasks: u32,
    pub average_completion_days: f64,
    pub completion_rate: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusDistribution {
    pub status: String,
    pub count: u32,
    pub percentage: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PriorityDistribution {
    pub priority: String,
    pub count: u32,
    pub percentage: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskTrends {
    pub weekly_creation: Vec<u32>,
    pub weekly_completion: Vec<u32>,
    pub completion_velocity: f64,
    pub productivity_trend: ProductivityTrend,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductivityTrend {
    pub direction: String,
    pub change_percentage: f64,
    pub prediction_next_week: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserTaskPerformance {
    pub user_id: Uuid,
    pub tasks_created: u32,
    pub tasks_completed: u32,
    pub average_completion_time: f64,
    pub overdue_rate: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationInfo {
    pub current_page: u32,
    pub total_pages: u32,
    pub per_page: u32,
    pub total_items: u32,
}

// Export response
#[derive(Debug, Serialize, Deserialize)]
pub struct AdvancedExportResponse {
    pub export_id: Uuid,
    pub export_type: String,
    pub format: crate::features::analytics::dto::requests::ExportFormat,
    pub total_records: u64,
    pub file_size_bytes: u64,
    pub download_url: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub metadata: ExportMetadata,
    pub processing_status: ExportStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportMetadata {
    pub filters_applied: HashMap<String, serde_json::Value>,
    pub columns_included: Vec<String>,
    pub data_version: String,
    pub export_source: String,
    pub checksum: String,
    pub compression: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Expired,
}

// Feature usage count response
#[derive(Debug, Serialize, Deserialize)]
pub struct FeatureUsageCountsResponse {
    pub period_days: u32,
    pub feature_counts: Vec<FeatureUsageCount>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeatureUsageCount {
    pub feature_name: String,
    pub total_usage: u64,
    pub unique_users: u64,
}

// Simple feature usage statistics response
#[derive(Debug, Serialize, Deserialize)]
pub struct FeatureUsageStatsResponse {
    pub period_days: i32,
}

// User feature usage response
#[derive(Debug, Serialize, Deserialize)]
pub struct UserFeatureUsageResponse {
    pub user_id: Uuid,
}

// Re-export analytics DTOs from admin module (excluding conflicting types)
