// task-backend/src/api/dto/subscription_dto.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::api::dto::common::{ApiResponse, OperationResult, PaginationMeta};
use crate::domain::subscription_history_model::SubscriptionChangeInfo;
use crate::domain::subscription_tier::SubscriptionTier;
use crate::domain::user_model::SafeUser;
use crate::repository::subscription_history_repository::UserSubscriptionStats;
use crate::repository::user_repository::SubscriptionTierStats;

// --- Request DTOs ---

/// サブスクリプション階層変更リクエスト
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct ChangeSubscriptionRequest {
    #[validate(length(
        min = 3,
        max = 20,
        message = "Tier must be between 3 and 20 characters"
    ))]
    pub new_tier: String,

    #[validate(length(max = 500, message = "Reason must be 500 characters or less"))]
    pub reason: Option<String>,
}

/// サブスクリプション階層アップグレードリクエスト  
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpgradeSubscriptionRequest {
    #[validate(custom(function = "validate_upgrade_tier"))]
    pub target_tier: SubscriptionTier,

    #[validate(length(max = 500, message = "Reason must be 500 characters or less"))]
    pub reason: Option<String>,
}

/// サブスクリプション階層ダウングレードリクエスト
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct DowngradeSubscriptionRequest {
    #[validate(custom(function = "validate_downgrade_tier"))]
    pub target_tier: SubscriptionTier,

    #[validate(length(max = 500, message = "Reason must be 500 characters or less"))]
    pub reason: Option<String>,
}

/// 管理者用サブスクリプション変更リクエスト
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct AdminChangeSubscriptionRequest {
    #[validate(length(
        min = 3,
        max = 20,
        message = "Tier must be between 3 and 20 characters"
    ))]
    pub new_tier: String,

    #[validate(length(max = 500, message = "Reason must be 500 characters or less"))]
    pub reason: Option<String>,

    pub force_change: Option<bool>,
}

// --- Response DTOs ---

/// 現在のサブスクリプション情報レスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct CurrentSubscriptionResponse {
    pub user_id: Uuid,
    pub current_tier: String,
    pub tier_display_name: String,
    pub tier_level: u8,
    pub subscribed_at: DateTime<Utc>,
    pub features: Vec<String>,
    pub limits: SubscriptionLimits,
    pub next_available_tiers: Vec<SubscriptionTierInfo>,
}

/// サブスクリプション変更レスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionChangeResponse {
    pub user: SafeUser,
    pub previous_tier: String,
    pub new_tier: String,
    pub change_type: String, // "upgrade", "downgrade", "admin_change"
    pub reason: Option<String>,
    pub changed_at: DateTime<Utc>,
    pub changed_by: Option<Uuid>,
    pub message: String,
}

/// サブスクリプション履歴レスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionHistoryResponse {
    pub user_id: Uuid,
    pub history: Vec<SubscriptionChangeInfo>,
    pub pagination: Option<PaginationMeta>,
    pub stats: UserSubscriptionStats,
}

/// サブスクリプション統計レスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionStatsResponse {
    pub total_users: u64,
    pub tier_distribution: Vec<SubscriptionTierStats>,
    pub recent_changes: RecentChanges,
    pub revenue_info: RevenueInfo,
}

/// サブスクリプション階層情報
#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionTierInfo {
    pub tier: String,
    pub display_name: String,
    pub level: u8,
    pub features: Vec<String>,
    pub limits: SubscriptionLimits,
    pub monthly_price: Option<f64>,
    pub yearly_price: Option<f64>,
}

/// サブスクリプション制限
#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionLimits {
    pub max_tasks: Option<u32>,
    pub max_projects: Option<u32>,
    pub max_team_members: Option<u32>,
    pub max_storage_mb: Option<u32>,
    pub api_requests_per_hour: Option<u32>,
    pub advanced_features_enabled: bool,
    pub priority_support: bool,
}

/// 最近の変更
#[derive(Debug, Serialize, Deserialize)]
pub struct RecentChanges {
    pub upgrades_last_7_days: u64,
    pub downgrades_last_7_days: u64,
    pub upgrades_last_30_days: u64,
    pub downgrades_last_30_days: u64,
}

/// 収益情報
#[derive(Debug, Serialize, Deserialize)]
pub struct RevenueInfo {
    pub monthly_recurring_revenue: f64,
    pub annual_recurring_revenue: f64,
    pub average_revenue_per_user: f64,
    pub churn_rate: f64,
}

/// サブスクリプション履歴クエリパラメータ
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct SubscriptionHistoryQuery {
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    #[serde(rename = "type")]
    pub change_type: Option<SubscriptionChangeType>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

/// サブスクリプション変更タイプ
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum SubscriptionChangeType {
    Upgrade,
    Downgrade,
    All,
}

/// 管理者向けサブスクリプション履歴レスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct AdminSubscriptionHistoryResponse {
    pub changes: Vec<SubscriptionChangeInfo>,
    pub pagination: PaginationMeta,
    pub summary: SubscriptionHistorySummary,
}

/// サブスクリプション履歴サマリー
#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionHistorySummary {
    pub total_changes: u64,
    pub upgrades_count: u64,
    pub downgrades_count: u64,
    pub date_range: DateRange,
}

/// 日付範囲
#[derive(Debug, Serialize, Deserialize)]
pub struct DateRange {
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
}

/// サブスクリプション統計レスポンス（Phase 5.2用）
#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionStatsResponseV2 {
    pub tier_change_stats: Vec<TierChangeStat>,
    pub trend_analysis: TrendAnalysis,
    pub revenue_impact: RevenueImpact,
}

/// 階層変更統計
#[derive(Debug, Serialize, Deserialize)]
pub struct TierChangeStat {
    pub tier: String,
    pub change_count: u64,
    pub percentage: f64,
}

/// トレンド分析
#[derive(Debug, Serialize, Deserialize)]
pub struct TrendAnalysis {
    pub growth_rate: f64,
    pub churn_rate: f64,
    pub net_movement: i64,
    pub period: String,
}

/// 収益影響
#[derive(Debug, Serialize, Deserialize)]
pub struct RevenueImpact {
    pub revenue_change: f64,
    pub revenue_change_percentage: f64,
    pub upgrades_revenue: f64,
    pub downgrades_revenue_loss: f64,
}

/// サブスクリプション履歴詳細レスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionHistoryDetailResponse {
    pub history_id: Uuid,
    pub user_id: Uuid,
    pub previous_tier: Option<String>,
    pub new_tier: String,
    pub change_type: String,
    pub reason: Option<String>,
    pub changed_at: DateTime<Utc>,
    pub changed_by: Option<Uuid>,
    pub changed_by_user: Option<ChangedByUserInfo>,
    pub tier_comparison: TierComparison,
}

/// 変更実行者情報
#[derive(Debug, Serialize, Deserialize)]
pub struct ChangedByUserInfo {
    pub user_id: Uuid,
    pub username: String,
    pub email: String,
    pub role: String,
}

/// 階層比較情報
#[derive(Debug, Serialize, Deserialize)]
pub struct TierComparison {
    pub previous_tier_info: Option<SubscriptionTierInfo>,
    pub new_tier_info: SubscriptionTierInfo,
    pub features_added: Vec<String>,
    pub features_removed: Vec<String>,
    pub limits_changed: LimitsChanged,
}

/// 制限の変更
#[derive(Debug, Serialize, Deserialize)]
pub struct LimitsChanged {
    pub max_tasks_change: Option<i64>,
    pub max_projects_change: Option<i64>,
    pub max_team_members_change: Option<i64>,
    pub max_storage_mb_change: Option<i64>,
    pub api_requests_per_hour_change: Option<i64>,
}

/// 階層別ユーザーリストレスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct TierUsersResponse {
    pub tier: String,
    pub users: Vec<TierUserInfo>,
    pub total_count: u64,
    pub pagination: Option<PaginationMeta>,
}

/// 階層別ユーザー情報
#[derive(Debug, Serialize, Deserialize)]
pub struct TierUserInfo {
    pub user_id: Uuid,
    pub username: String,
    pub email: String,
    pub subscribed_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub is_active: bool,
}

// --- Type Aliases for API Responses ---

pub type ChangeSubscriptionResponse = ApiResponse<OperationResult<SubscriptionChangeResponse>>;
pub type UpgradeSubscriptionResponse = ApiResponse<OperationResult<SubscriptionChangeResponse>>;
pub type DowngradeSubscriptionResponse = ApiResponse<OperationResult<SubscriptionChangeResponse>>;

// --- Helper Implementations ---

impl CurrentSubscriptionResponse {
    pub fn new(user_id: Uuid, current_tier: String, subscribed_at: DateTime<Utc>) -> Self {
        let tier_info = Self::get_tier_info(&current_tier);
        Self {
            user_id,
            current_tier: current_tier.clone(),
            tier_display_name: tier_info.display_name,
            tier_level: tier_info.level,
            subscribed_at,
            features: tier_info.features,
            limits: tier_info.limits,
            next_available_tiers: Self::get_next_available_tiers(&current_tier),
        }
    }

    pub fn get_tier_info(tier: &str) -> SubscriptionTierInfo {
        match tier.to_lowercase().as_str() {
            "free" => SubscriptionTierInfo {
                tier: "free".to_string(),
                display_name: "Free".to_string(),
                level: 1,
                features: vec![
                    "Basic task management".to_string(),
                    "Up to 100 tasks".to_string(),
                    "Basic reporting".to_string(),
                ],
                limits: SubscriptionLimits {
                    max_tasks: Some(100),
                    max_projects: Some(3),
                    max_team_members: Some(1),
                    max_storage_mb: Some(100),
                    api_requests_per_hour: Some(100),
                    advanced_features_enabled: false,
                    priority_support: false,
                },
                monthly_price: Some(0.0),
                yearly_price: Some(0.0),
            },
            "pro" => SubscriptionTierInfo {
                tier: "pro".to_string(),
                display_name: "Pro".to_string(),
                level: 2,
                features: vec![
                    "Advanced task management".to_string(),
                    "Up to 10,000 tasks".to_string(),
                    "Advanced reporting".to_string(),
                    "Team collaboration".to_string(),
                    "Export functionality".to_string(),
                ],
                limits: SubscriptionLimits {
                    max_tasks: Some(10_000),
                    max_projects: Some(50),
                    max_team_members: Some(10),
                    max_storage_mb: Some(5_000),
                    api_requests_per_hour: Some(1_000),
                    advanced_features_enabled: true,
                    priority_support: true,
                },
                monthly_price: Some(19.99),
                yearly_price: Some(199.99),
            },
            "enterprise" => SubscriptionTierInfo {
                tier: "enterprise".to_string(),
                display_name: "Enterprise".to_string(),
                level: 3,
                features: vec![
                    "Unlimited task management".to_string(),
                    "Unlimited tasks".to_string(),
                    "Enterprise reporting".to_string(),
                    "Advanced team collaboration".to_string(),
                    "Bulk operations".to_string(),
                    "API access".to_string(),
                    "Custom integrations".to_string(),
                    "Priority support".to_string(),
                ],
                limits: SubscriptionLimits {
                    max_tasks: None,
                    max_projects: None,
                    max_team_members: None,
                    max_storage_mb: None,
                    api_requests_per_hour: None,
                    advanced_features_enabled: true,
                    priority_support: true,
                },
                monthly_price: Some(99.99),
                yearly_price: Some(999.99),
            },
            _ => SubscriptionTierInfo {
                tier: "unknown".to_string(),
                display_name: "Unknown".to_string(),
                level: 0,
                features: vec![],
                limits: SubscriptionLimits {
                    max_tasks: Some(0),
                    max_projects: Some(0),
                    max_team_members: Some(0),
                    max_storage_mb: Some(0),
                    api_requests_per_hour: Some(0),
                    advanced_features_enabled: false,
                    priority_support: false,
                },
                monthly_price: None,
                yearly_price: None,
            },
        }
    }

    fn get_next_available_tiers(current_tier: &str) -> Vec<SubscriptionTierInfo> {
        let current_level = Self::get_tier_info(current_tier).level;
        let all_tiers = ["free", "pro", "enterprise"];

        all_tiers
            .iter()
            .map(|tier| Self::get_tier_info(tier))
            .filter(|tier| tier.level > current_level)
            .collect()
    }
}

impl SubscriptionChangeResponse {
    pub fn new(
        user: SafeUser,
        previous_tier: String,
        new_tier: String,
        reason: Option<String>,
        changed_by: Option<Uuid>,
    ) -> Self {
        let change_type = if let (Ok(prev_tier), Ok(new_tier_enum)) = (
            SubscriptionTier::from_str(&previous_tier).ok_or("invalid"),
            SubscriptionTier::from_str(&new_tier).ok_or("invalid"),
        ) {
            match new_tier_enum.level().cmp(&prev_tier.level()) {
                std::cmp::Ordering::Greater => "upgrade",
                std::cmp::Ordering::Less => "downgrade",
                std::cmp::Ordering::Equal => "no_change",
            }
        } else {
            "admin_change"
        };

        let message = match change_type {
            "upgrade" => format!(
                "Successfully upgraded from {} to {}",
                previous_tier, new_tier
            ),
            "downgrade" => format!(
                "Successfully downgraded from {} to {}",
                previous_tier, new_tier
            ),
            "admin_change" => format!(
                "Subscription changed from {} to {} by administrator",
                previous_tier, new_tier
            ),
            _ => format!(
                "Subscription tier changed from {} to {}",
                previous_tier, new_tier
            ),
        };

        Self {
            user,
            previous_tier,
            new_tier,
            change_type: change_type.to_string(),
            reason,
            changed_at: Utc::now(),
            changed_by,
            message,
        }
    }
}

// --- Validation Functions ---

fn validate_upgrade_tier(tier: &SubscriptionTier) -> Result<(), validator::ValidationError> {
    match tier {
        SubscriptionTier::Free => {
            let mut error = validator::ValidationError::new("invalid_upgrade");
            error.message = Some("Cannot upgrade to Free tier".into());
            Err(error)
        }
        _ => Ok(()),
    }
}

fn validate_downgrade_tier(tier: &SubscriptionTier) -> Result<(), validator::ValidationError> {
    match tier {
        SubscriptionTier::Enterprise => {
            let mut error = validator::ValidationError::new("invalid_downgrade");
            error.message = Some("Cannot downgrade to Enterprise tier".into());
            Err(error)
        }
        _ => Ok(()),
    }
}

// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_subscription_response_creation() {
        let user_id = Uuid::new_v4();
        let response = CurrentSubscriptionResponse::new(user_id, "pro".to_string(), Utc::now());

        assert_eq!(response.user_id, user_id);
        assert_eq!(response.current_tier, "pro");
        assert_eq!(response.tier_display_name, "Pro");
        assert_eq!(response.tier_level, 2);
        assert!(!response.features.is_empty());
        assert!(response.limits.advanced_features_enabled);
    }

    #[test]
    fn test_subscription_change_response_upgrade() {
        let user = SafeUser {
            id: Uuid::new_v4(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            is_active: true,
            email_verified: true,
            role_id: Uuid::new_v4(),
            subscription_tier: "free".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_login_at: None,
        };

        let response = SubscriptionChangeResponse::new(
            user,
            "free".to_string(),
            "pro".to_string(),
            Some("Upgrade for more features".to_string()),
            None,
        );

        assert_eq!(response.change_type, "upgrade");
        assert!(response.message.contains("upgraded"));
        assert_eq!(response.previous_tier, "free");
        assert_eq!(response.new_tier, "pro");
    }

    #[test]
    fn test_tier_info_retrieval() {
        let free_info = CurrentSubscriptionResponse::get_tier_info("free");
        assert_eq!(free_info.level, 1);
        assert_eq!(free_info.limits.max_tasks, Some(100));

        let pro_info = CurrentSubscriptionResponse::get_tier_info("pro");
        assert_eq!(pro_info.level, 2);
        assert_eq!(pro_info.limits.max_tasks, Some(10_000));

        let enterprise_info = CurrentSubscriptionResponse::get_tier_info("enterprise");
        assert_eq!(enterprise_info.level, 3);
        assert_eq!(enterprise_info.limits.max_tasks, None);
    }

    #[test]
    fn test_next_available_tiers() {
        let next_tiers = CurrentSubscriptionResponse::get_next_available_tiers("free");
        assert_eq!(next_tiers.len(), 2);
        assert!(next_tiers.iter().any(|t| t.tier == "pro"));
        assert!(next_tiers.iter().any(|t| t.tier == "enterprise"));

        let next_tiers_pro = CurrentSubscriptionResponse::get_next_available_tiers("pro");
        assert_eq!(next_tiers_pro.len(), 1);
        assert!(next_tiers_pro.iter().any(|t| t.tier == "enterprise"));

        let next_tiers_enterprise =
            CurrentSubscriptionResponse::get_next_available_tiers("enterprise");
        assert_eq!(next_tiers_enterprise.len(), 0);
    }
}
