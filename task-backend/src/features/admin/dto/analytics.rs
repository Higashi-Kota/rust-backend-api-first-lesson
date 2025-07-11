// task-backend/src/api/dto/analytics_dto.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::core::subscription_tier::SubscriptionTier;
use crate::shared::types::pagination::PaginationMeta;

// --- Request DTOs ---

/// アナリティクス期間指定リクエスト
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct AnalyticsTimeRangeRequest {
    #[validate(range(min = 1, max = 365, message = "Days must be between 1 and 365"))]
    pub days: Option<u32>,

    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,

    #[validate(length(
        min = 1,
        max = 50,
        message = "Metric type must be between 1 and 50 characters"
    ))]
    pub metric_type: Option<String>,
}

/// タスク統計リクエスト
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct TaskAnalyticsRequest {
    pub user_id: Option<Uuid>,
    pub include_details: Option<bool>,
    pub group_by: Option<TaskGroupBy>,
}

/// ユーザー統計リクエスト
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UserAnalyticsRequest {
    pub include_inactive: Option<bool>,
    pub subscription_tier: Option<SubscriptionTier>,
    pub registration_period_days: Option<u32>,
}

// --- Response DTOs ---

/// システム統計レスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemStatsResponse {
    pub overview: SystemOverview,
    pub user_metrics: UserMetrics,
    pub task_metrics: TaskMetrics,
    pub subscription_metrics: SubscriptionMetrics,
    pub security_metrics: SecurityMetrics,
    pub generated_at: DateTime<Utc>,
}

impl SystemStatsResponse {
    pub fn new() -> Self {
        Self {
            overview: SystemOverview {
                total_users: 0,
                active_users_last_30_days: 0,
                total_tasks: 0,
                completed_tasks: 0,
                system_uptime_days: 0,
                database_size_mb: 0.0,
            },
            user_metrics: UserMetrics {
                new_registrations_today: 0,
                new_registrations_this_week: 0,
                new_registrations_this_month: 0,
                active_users_today: 0,
                daily_active_users: 0,
                weekly_active_users: 0,
                retention_rate_30_days: 0.0,
                average_session_duration_minutes: 0.0,
                user_distribution_by_tier: vec![],
            },
            task_metrics: TaskMetrics {
                tasks_created_today: 0,
                tasks_completed_today: 0,
                tasks_created_this_week: 0,
                tasks_completed_this_week: 0,
                average_completion_time_hours: 0.0,
                completion_rate_percentage: 0.0,
                top_task_categories: vec![],
            },
            subscription_metrics: SubscriptionMetrics::default(),
            security_metrics: SecurityMetrics::default(),
            generated_at: Utc::now(),
        }
    }
}

/// システム概要
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemOverview {
    pub total_users: u64,
    pub active_users_last_30_days: u64,
    pub total_tasks: u64,
    pub completed_tasks: u64,
    pub system_uptime_days: u64,
    pub database_size_mb: f64,
}

/// ユーザーメトリクス
#[derive(Debug, Serialize, Deserialize)]
pub struct UserMetrics {
    pub new_registrations_today: u64,
    pub new_registrations_this_week: u64,
    pub new_registrations_this_month: u64,
    pub active_users_today: u64,
    pub daily_active_users: u64,
    pub weekly_active_users: u64,
    pub retention_rate_30_days: f64,
    pub average_session_duration_minutes: f64,
    pub user_distribution_by_tier: Vec<TierDistribution>,
}

/// タスクメトリクス
#[derive(Debug, Serialize, Deserialize)]
pub struct TaskMetrics {
    pub tasks_created_today: u64,
    pub tasks_completed_today: u64,
    pub tasks_created_this_week: u64,
    pub tasks_completed_this_week: u64,
    pub average_completion_time_hours: f64,
    pub completion_rate_percentage: f64,
    pub top_task_categories: Vec<TaskCategoryStats>,
}

/// サブスクリプションメトリクス
#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionMetrics {
    pub conversion_rate_percentage: f64,
    pub churn_rate_percentage: f64,
    pub average_revenue_per_user: f64,
    pub monthly_recurring_revenue: f64,
    pub upgrade_rate_percentage: f64,
    pub downgrade_rate_percentage: f64,
}

/// セキュリティメトリクス
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SecurityMetrics {
    pub suspicious_ips: Vec<SuspiciousIpInfo>,
    pub failed_login_attempts_today: u64,
    pub failed_login_attempts_this_week: u64,
    pub security_incidents_this_month: u64,
}

/// 不審なIPアドレス情報
#[derive(Debug, Serialize, Deserialize)]
pub struct SuspiciousIpInfo {
    pub ip_address: String,
    pub failed_attempts: u64,
    pub last_attempt: DateTime<Utc>,
}

/// 階層分布
#[derive(Debug, Serialize, Deserialize)]
pub struct TierDistribution {
    pub tier: SubscriptionTier,
    pub count: u64,
    pub percentage: f64,
}

/// タスクカテゴリ統計
#[derive(Debug, Serialize, Deserialize)]
pub struct TaskCategoryStats {
    pub category: String,
    pub total_tasks: u64,
    pub completed_tasks: u64,
    pub completion_rate: f64,
}

/// ユーザーアクティビティレスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct UserActivityResponse {
    pub user_id: Uuid,
    pub daily_activities: Vec<DailyActivity>,
    pub summary: ActivitySummary,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
}

/// 日次アクティビティ
#[derive(Debug, Serialize, Deserialize)]
pub struct DailyActivity {
    pub date: DateTime<Utc>,
    pub tasks_created: u32,
    pub tasks_completed: u32,
    pub login_count: u32,
    pub session_duration_minutes: u32,
}

/// アクティビティサマリー
#[derive(Debug, Serialize, Deserialize)]
pub struct ActivitySummary {
    pub total_tasks_created: u64,
    pub total_tasks_completed: u64,
    pub total_login_days: u64,
    pub average_daily_tasks: f64,
    pub completion_rate_percentage: f64,
    pub most_active_day: String,
    pub productivity_score: f64,
}

/// タスク統計詳細レスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct TaskStatsDetailResponse {
    pub overview: TaskStatsOverview,
    pub status_distribution: Vec<TaskStatusDistribution>,
    pub priority_distribution: Vec<TaskPriorityDistribution>,
    pub trends: TaskTrends,
    pub user_performance: Option<Vec<UserTaskPerformance>>,
    pub pagination: Option<PaginationMeta>,
}

/// タスク統計概要
#[derive(Debug, Serialize, Deserialize)]
pub struct TaskStatsOverview {
    pub total_tasks: u64,
    pub completed_tasks: u64,
    pub pending_tasks: u64,
    pub overdue_tasks: u64,
    pub average_completion_days: f64,
    pub completion_rate: f64,
}

/// タスクステータス分布
#[derive(Debug, Serialize, Deserialize)]
pub struct TaskStatusDistribution {
    pub status: String,
    pub count: u64,
    pub percentage: f64,
}

/// タスク優先度分布
#[derive(Debug, Serialize, Deserialize)]
pub struct TaskPriorityDistribution {
    pub priority: String,
    pub count: u64,
    pub percentage: f64,
    pub average_completion_days: f64,
}

/// タスクトレンド
#[derive(Debug, Serialize, Deserialize)]
pub struct TaskTrends {
    pub weekly_creation: Vec<WeeklyTrend>,
    pub weekly_completion: Vec<WeeklyTrend>,
    pub completion_velocity: f64,
    pub productivity_trend: ProductivityTrend,
}

/// 週次トレンド
#[derive(Debug, Serialize, Deserialize)]
pub struct WeeklyTrend {
    pub week_start: DateTime<Utc>,
    pub count: u64,
    pub change_from_previous_week: f64,
}

/// 生産性トレンド
#[derive(Debug, Serialize, Deserialize)]
pub struct ProductivityTrend {
    pub direction: String, // "increasing", "decreasing", "stable"
    pub change_percentage: f64,
    pub prediction_next_week: f64,
}

/// ユーザータスクパフォーマンス
#[derive(Debug, Serialize, Deserialize)]
pub struct UserTaskPerformance {
    pub user_id: Uuid,
    pub username: String,
    pub tasks_created: u64,
    pub tasks_completed: u64,
    pub completion_rate: f64,
    pub average_completion_time_hours: f64,
    pub productivity_score: f64,
}

// --- Enums ---

/// タスクグループ化方法
#[derive(Debug, Serialize, Deserialize)]
pub enum TaskGroupBy {
    Status,
    Priority,
    User,
    CreatedDate,
    CompletedDate,
}

// --- Helper Implementations ---

impl Default for SystemStatsResponse {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for SystemOverview {
    fn default() -> Self {
        Self {
            total_users: 0,
            active_users_last_30_days: 0,
            total_tasks: 0,
            completed_tasks: 0,
            system_uptime_days: 0,
            database_size_mb: 0.0,
        }
    }
}

impl Default for UserMetrics {
    fn default() -> Self {
        Self {
            new_registrations_today: 0,
            new_registrations_this_week: 0,
            new_registrations_this_month: 0,
            active_users_today: 0,
            daily_active_users: 0,
            weekly_active_users: 0,
            retention_rate_30_days: 0.0,
            average_session_duration_minutes: 0.0,
            user_distribution_by_tier: Vec::new(),
        }
    }
}

impl Default for TaskMetrics {
    fn default() -> Self {
        Self {
            tasks_created_today: 0,
            tasks_completed_today: 0,
            tasks_created_this_week: 0,
            tasks_completed_this_week: 0,
            average_completion_time_hours: 0.0,
            completion_rate_percentage: 0.0,
            top_task_categories: Vec::new(),
        }
    }
}

impl Default for SubscriptionMetrics {
    fn default() -> Self {
        Self {
            conversion_rate_percentage: 0.0,
            churn_rate_percentage: 0.0,
            average_revenue_per_user: 0.0,
            monthly_recurring_revenue: 0.0,
            upgrade_rate_percentage: 0.0,
            downgrade_rate_percentage: 0.0,
        }
    }
}

impl ActivitySummary {
    #[allow(dead_code)] // Utility method for analytics calculations
    pub fn calculate_productivity_score(
        completion_rate: f64,
        average_daily_tasks: f64,
        login_consistency: f64,
    ) -> f64 {
        (completion_rate * 0.5) + (average_daily_tasks.min(10.0) * 0.3) + (login_consistency * 0.2)
    }
}

impl TaskStatsOverview {
    #[allow(dead_code)] // Utility method for task statistics
    pub fn calculate_completion_rate(completed: u64, total: u64) -> f64 {
        if total == 0 {
            0.0
        } else {
            (completed as f64 / total as f64) * 100.0
        }
    }
}

// --- User Analytics & Management API DTOs ---

/// ユーザー行動分析リクエスト
#[derive(Debug, Serialize, Deserialize)]
pub struct UserBehaviorAnalyticsQuery {
    pub user_id: Option<Uuid>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
    pub metrics: Option<Vec<String>>, // ["login_frequency", "task_activity", "feature_usage"]
    pub include_comparisons: Option<bool>,
}

/// 高度なエクスポートリクエスト
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct AdvancedExportRequest {
    pub export_type: ExportType,
    pub filters: Option<ExportFilters>,
    pub format: ExportFormat,
    #[validate(range(
        min = 1,
        max = 100000,
        message = "Max records must be between 1 and 100000"
    ))]
    pub max_records: Option<u32>,
    pub include_metadata: Option<bool>,
    pub custom_fields: Option<Vec<String>>,
}

/// ユーザー行動分析レスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct UserBehaviorAnalyticsResponse {
    pub user_id: Uuid,
    pub analysis_period: AnalysisPeriod,
    pub behavioral_metrics: BehavioralMetrics,
    pub activity_patterns: ActivityPatterns,
    pub feature_usage: FeatureUsageMetrics,
    pub performance_indicators: PerformanceIndicators,
    pub comparisons: Option<UserComparisons>,
    pub recommendations: Vec<UserRecommendation>,
    pub generated_at: DateTime<Utc>,
}

/// 高度なエクスポート結果レスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct AdvancedExportResponse {
    pub export_id: Uuid,
    pub export_type: ExportType,
    pub format: ExportFormat,
    pub total_records: u32,
    pub file_size_bytes: u64,
    pub download_url: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub metadata: ExportMetadata,
    pub processing_status: ExportStatus,
    pub created_at: DateTime<Utc>,
}

// --- Supporting Enums and Structures ---

/// エクスポート種別
#[derive(Debug, Serialize, Deserialize)]
pub enum ExportType {
    Users,
    Tasks,
    Analytics,
    AuditLogs,
    SystemMetrics,
    UserBehavior,
    SubscriptionHistory,
}

/// エクスポート形式
#[derive(Debug, Serialize, Deserialize)]
pub enum ExportFormat {
    Csv,
    Json,
    Excel,
    Pdf,
    Xml,
}

/// エクスポートステータス
#[derive(Debug, Serialize, Deserialize)]
pub enum ExportStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Expired,
}

/// 分析期間
#[derive(Debug, Serialize, Deserialize)]
pub struct AnalysisPeriod {
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub duration_days: u32,
    pub granularity: MetricGranularity,
}

/// メトリクス粒度
#[derive(Debug, Serialize, Deserialize)]
pub enum MetricGranularity {
    Hourly,
    Daily,
    Weekly,
    Monthly,
}

/// 行動メトリクス
#[derive(Debug, Serialize, Deserialize)]
pub struct BehavioralMetrics {
    pub login_frequency: LoginFrequency,
    pub session_duration: SessionDuration,
    pub activity_score: f64,
    pub engagement_level: EngagementLevel,
    pub feature_adoption_rate: f64,
    pub consistency_score: f64,
}

/// 活動パターン
#[derive(Debug, Serialize, Deserialize)]
pub struct ActivityPatterns {
    pub peak_activity_hours: Vec<u8>,  // 0-23
    pub most_active_days: Vec<String>, // Monday, Tuesday, etc.
    pub activity_distribution: ActivityDistribution,
    pub workflow_patterns: Vec<WorkflowPattern>,
    pub seasonal_trends: Vec<SeasonalTrend>,
}

/// 機能使用メトリクス
#[derive(Debug, Serialize, Deserialize)]
pub struct FeatureUsageMetrics {
    pub most_used_features: Vec<FeatureUsage>,
    pub least_used_features: Vec<FeatureUsage>,
    pub feature_progression: Vec<FeatureProgression>,
    pub subscription_utilization: SubscriptionUtilization,
}

/// パフォーマンス指標
#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceIndicators {
    pub task_completion_rate: f64,
    pub average_task_duration: f64,
    pub productivity_score: f64,
    pub error_rate: f64,
    pub satisfaction_indicators: SatisfactionIndicators,
}

/// ユーザー比較
#[derive(Debug, Serialize, Deserialize)]
pub struct UserComparisons {
    pub peer_comparison: PeerComparison,
    pub historical_comparison: HistoricalComparison,
    pub tier_comparison: TierComparison,
}

/// ユーザー推奨事項
#[derive(Debug, Serialize, Deserialize)]
pub struct UserRecommendation {
    pub recommendation_type: RecommendationType,
    pub title: String,
    pub description: String,
    pub priority: RecommendationPriority,
    pub expected_impact: String,
    pub action_url: Option<String>,
}

/// エクスポートフィルター
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ExportFilters {
    pub date_range: Option<DateRange>,
    pub user_filter: Option<UserFilter>,
    pub status_filter: Option<Vec<String>>,
    pub subscription_filter: Option<Vec<SubscriptionTier>>,
    pub custom_filters: Option<serde_json::Value>,
}

/// エクスポートメタデータ
#[derive(Debug, Serialize, Deserialize)]
pub struct ExportMetadata {
    pub filters_applied: ExportFilters,
    pub columns_included: Vec<String>,
    pub data_version: String,
    pub export_source: String,
    pub checksum: String,
    pub compression: Option<String>,
}

/// ログイン頻度
#[derive(Debug, Serialize, Deserialize)]
pub struct LoginFrequency {
    pub daily_average: f64,
    pub weekly_average: f64,
    pub monthly_average: f64,
    pub consistency_score: f64,
    pub longest_streak_days: u32,
    pub current_streak_days: u32,
}

/// セッション継続時間
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionDuration {
    pub average_minutes: f64,
    pub median_minutes: f64,
    pub longest_session_minutes: f64,
    pub shortest_session_minutes: f64,
    pub session_count: u32,
}

/// エンゲージメントレベル
#[derive(Debug, Serialize, Deserialize)]
pub enum EngagementLevel {
    Low,
    Medium,
    High,
    VeryHigh,
}

/// 活動分布
#[derive(Debug, Serialize, Deserialize)]
pub struct ActivityDistribution {
    pub morning: f64,   // 0-12
    pub afternoon: f64, // 12-18
    pub evening: f64,   // 18-24
    pub weekday: f64,
    pub weekend: f64,
}

/// ワークフローパターン
#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowPattern {
    pub pattern_name: String,
    pub frequency: u32,
    pub efficiency_score: f64,
    pub steps: Vec<WorkflowStep>,
    pub average_duration_minutes: f64,
}

/// 季節トレンド
#[derive(Debug, Serialize, Deserialize)]
pub struct SeasonalTrend {
    pub season: String,
    pub activity_multiplier: f64,
    pub peak_months: Vec<String>,
    pub trend_strength: f64,
}

/// 機能使用状況
#[derive(Debug, Serialize, Deserialize)]
pub struct FeatureUsage {
    pub feature_name: String,
    pub usage_count: u32,
    pub usage_percentage: f64,
    pub last_used: DateTime<Utc>,
    pub proficiency_level: ProficiencyLevel,
}

/// 機能進捗
#[derive(Debug, Serialize, Deserialize)]
pub struct FeatureProgression {
    pub feature_name: String,
    pub adoption_date: DateTime<Utc>,
    pub proficiency_level: ProficiencyLevel,
    pub usage_trend: TrendDirection,
    pub mastery_percentage: f64,
}

/// サブスクリプション利用状況
#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionUtilization {
    pub current_tier: SubscriptionTier,
    pub tier_utilization_percentage: f64,
    pub underutilized_features: Vec<String>,
    pub upgrade_recommendations: Vec<UpgradeRecommendation>,
    pub cost_efficiency_score: f64,
}

/// 満足度指標
#[derive(Debug, Serialize, Deserialize)]
pub struct SatisfactionIndicators {
    pub feature_satisfaction_score: f64,
    pub performance_satisfaction_score: f64,
    pub overall_satisfaction_score: f64,
    pub nps_score: Option<f64>,
    pub feedback_sentiment: SentimentScore,
}

/// 同僚比較
#[derive(Debug, Serialize, Deserialize)]
pub struct PeerComparison {
    pub percentile_rank: f64,
    pub above_average_metrics: Vec<String>,
    pub below_average_metrics: Vec<String>,
    pub peer_group_size: u32,
    pub benchmark_score: f64,
}

/// 履歴比較
#[derive(Debug, Serialize, Deserialize)]
pub struct HistoricalComparison {
    pub improvement_areas: Vec<ImprovementArea>,
    pub declining_areas: Vec<DecliningArea>,
    pub consistency_score: f64,
    pub growth_rate: f64,
    pub trend_analysis: TrendAnalysis,
}

/// 階層比較
#[derive(Debug, Serialize, Deserialize)]
pub struct TierComparison {
    pub current_tier: SubscriptionTier,
    pub tier_average_metrics: TierMetrics,
    pub tier_percentile: f64,
    pub upgrade_impact_prediction: Option<UpgradeImpact>,
}

/// 推奨事項タイプ
#[derive(Debug, Serialize, Deserialize)]
pub enum RecommendationType {
    FeatureAdoption,
    ProductivityImprovement,
    SubscriptionUpgrade,
    WorkflowOptimization,
    SecurityEnhancement,
    TrainingRecommendation,
}

/// 推奨事項優先度
#[derive(Debug, Serialize, Deserialize)]
pub enum RecommendationPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// 日付範囲
#[derive(Debug, Serialize, Deserialize)]
pub struct DateRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

/// ユーザーフィルター
#[derive(Debug, Serialize, Deserialize)]
pub struct UserFilter {
    pub user_ids: Option<Vec<Uuid>>,
    pub roles: Option<Vec<String>>,
    pub subscription_tiers: Option<Vec<SubscriptionTier>>,
    pub active_only: Option<bool>,
    pub registration_period: Option<DateRange>,
}

/// ワークフローステップ
#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub step_name: String,
    pub order: u32,
    pub duration_minutes: f64,
    pub success_rate: f64,
    pub common_errors: Vec<String>,
}

/// 熟練度レベル
#[derive(Debug, Serialize, Deserialize)]
pub enum ProficiencyLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

/// ユーザー分析エクスポートクエリ
#[derive(Debug, Serialize, Deserialize)]
pub struct UserAnalyticsExportQuery {
    pub user_ids: Option<Vec<Uuid>>,
}

/// ユーザー分析エクスポート
#[derive(Debug, Serialize, Deserialize)]
pub struct UserAnalyticsExport {
    pub user_id: Uuid,
    pub username: String,
    pub email: String,
    pub registration_date: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
    pub total_tasks: i32,
    pub completed_tasks: i32,
    pub feature_usage_summary: Vec<FeatureUsageSummary>,
}

/// 機能使用概要
#[derive(Debug, Serialize, Deserialize)]
pub struct FeatureUsageSummary {
    pub feature_name: String,
    pub usage_count: u32,
    pub last_used: DateTime<Utc>,
}

/// トレンド方向
#[derive(Debug, Serialize, Deserialize)]
pub enum TrendDirection {
    Increasing,
    Decreasing,
    Stable,
    Volatile,
}

/// アップグレード推奨
#[derive(Debug, Serialize, Deserialize)]
pub struct UpgradeRecommendation {
    pub recommended_tier: SubscriptionTier,
    pub reason: String,
    pub expected_benefits: Vec<String>,
    pub confidence_score: f64,
    pub estimated_roi: f64,
}

/// 改善エリア
#[derive(Debug, Serialize, Deserialize)]
pub struct ImprovementArea {
    pub metric_name: String,
    pub improvement_percentage: f64,
    pub timeframe: String,
    pub recommended_actions: Vec<String>,
}

/// 衰退エリア
#[derive(Debug, Serialize, Deserialize)]
pub struct DecliningArea {
    pub metric_name: String,
    pub decline_percentage: f64,
    pub timeframe: String,
    pub intervention_suggestions: Vec<String>,
}

/// 階層メトリクス
#[derive(Debug, Serialize, Deserialize)]
pub struct TierMetrics {
    pub average_activity_score: f64,
    pub average_feature_usage: f64,
    pub average_productivity_score: f64,
    pub tier_satisfaction_score: f64,
}

/// アップグレード影響
#[derive(Debug, Serialize, Deserialize)]
pub struct UpgradeImpact {
    pub productivity_increase: f64,
    pub feature_access_improvement: f64,
    pub estimated_roi: f64,
    pub payback_period_months: u32,
}

/// 感情スコア
#[derive(Debug, Serialize, Deserialize)]
pub struct SentimentScore {
    pub positive_percentage: f64,
    pub negative_percentage: f64,
    pub neutral_percentage: f64,
    pub overall_sentiment: SentimentCategory,
}

/// 感情カテゴリ
#[derive(Debug, Serialize, Deserialize)]
pub enum SentimentCategory {
    VeryPositive,
    Positive,
    Neutral,
    Negative,
    VeryNegative,
}

/// トレンド分析
#[derive(Debug, Serialize, Deserialize)]
pub struct TrendAnalysis {
    pub trend_direction: TrendDirection,
    pub trend_strength: f64,
    pub seasonality_detected: bool,
    pub forecast_accuracy: f64,
}

// --- Helper Implementations ---

impl Default for BehavioralMetrics {
    fn default() -> Self {
        Self {
            login_frequency: LoginFrequency::default(),
            session_duration: SessionDuration::default(),
            activity_score: 0.0,
            engagement_level: EngagementLevel::Medium,
            feature_adoption_rate: 0.0,
            consistency_score: 0.0,
        }
    }
}

impl Default for LoginFrequency {
    fn default() -> Self {
        Self {
            daily_average: 0.0,
            weekly_average: 0.0,
            monthly_average: 0.0,
            consistency_score: 0.0,
            longest_streak_days: 0,
            current_streak_days: 0,
        }
    }
}

impl Default for SessionDuration {
    fn default() -> Self {
        Self {
            average_minutes: 0.0,
            median_minutes: 0.0,
            longest_session_minutes: 0.0,
            shortest_session_minutes: 0.0,
            session_count: 0,
        }
    }
}

impl BehavioralMetrics {
    #[allow(dead_code)] // Utility method for engagement analysis
    pub fn calculate_engagement_level(activity_score: f64) -> EngagementLevel {
        match activity_score {
            score if score >= 80.0 => EngagementLevel::VeryHigh,
            score if score >= 60.0 => EngagementLevel::High,
            score if score >= 40.0 => EngagementLevel::Medium,
            _ => EngagementLevel::Low,
        }
    }
}

// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_stats_response_creation() {
        let response = SystemStatsResponse::new();
        assert_eq!(response.overview.total_users, 0);
        assert_eq!(response.user_metrics.new_registrations_today, 0);
        assert_eq!(response.task_metrics.tasks_created_today, 0);
        assert_eq!(
            response.subscription_metrics.conversion_rate_percentage,
            0.0
        );
    }

    #[test]
    fn test_completion_rate_calculation() {
        assert_eq!(TaskStatsOverview::calculate_completion_rate(50, 100), 50.0);
        assert_eq!(TaskStatsOverview::calculate_completion_rate(0, 100), 0.0);
        assert_eq!(
            TaskStatsOverview::calculate_completion_rate(100, 100),
            100.0
        );
        assert_eq!(TaskStatsOverview::calculate_completion_rate(10, 0), 0.0);
    }

    #[test]
    fn test_productivity_score_calculation() {
        let score = ActivitySummary::calculate_productivity_score(0.8, 5.0, 0.9);
        // 0.8 * 0.5 + 5.0 * 0.3 + 0.9 * 0.2 = 0.4 + 1.5 + 0.18 = 2.08
        assert!((score - 2.08).abs() < 0.1);

        let max_score = ActivitySummary::calculate_productivity_score(1.0, 15.0, 1.0);
        // 1.0 * 0.5 + min(15.0, 10.0) * 0.3 + 1.0 * 0.2 = 0.5 + 3.0 + 0.2 = 3.7
        assert!((max_score - 3.7).abs() < 0.1);
    }

    #[test]
    fn test_analytics_time_range_validation() {
        let valid_request = AnalyticsTimeRangeRequest {
            days: Some(30),
            start_date: None,
            end_date: None,
            metric_type: Some("user_activity".to_string()),
        };
        assert!(valid_request.validate().is_ok());

        let invalid_request = AnalyticsTimeRangeRequest {
            days: Some(400), // Invalid: > 365
            start_date: None,
            end_date: None,
            metric_type: Some("x".repeat(100)), // Invalid: > 50 chars
        };
        assert!(invalid_request.validate().is_err());
    }

    #[test]
    fn test_tier_distribution() {
        let distribution = TierDistribution {
            tier: SubscriptionTier::Pro,
            count: 150,
            percentage: 30.0,
        };
        assert_eq!(distribution.tier, SubscriptionTier::Pro);
        assert_eq!(distribution.count, 150);
        assert_eq!(distribution.percentage, 30.0);
    }
}
