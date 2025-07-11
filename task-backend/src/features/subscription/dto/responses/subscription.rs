// task-backend/src/features/subscription/dto/responses/subscription.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::subscription_tier::SubscriptionTier;
use crate::domain::user_model::SafeUser;
use crate::features::auth::repository::user_repository::SubscriptionTierStats;
use crate::features::subscription::models::history::SubscriptionChangeInfo;
use crate::features::subscription::repositories::history::UserSubscriptionStats;
use crate::shared::types::common::{ApiResponse, OperationResult};
use crate::shared::types::pagination::PaginationMeta;

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
    pub pagination: PaginationMeta,
}

/// ユーザーのサブスクリプション統計レスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct UserSubscriptionStatsResponse {
    pub user_id: Uuid,
    pub total_changes: u64,
    pub upgrade_count: u64,
    pub downgrade_count: u64,
    pub current_tier: Option<String>,
    pub first_subscription_date: Option<DateTime<Utc>>,
    pub time_on_current_tier_days: Option<u64>,
}

/// サブスクリプション階層統計レスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionTierStatsResponse {
    pub tier: String,
    pub tier_display_name: String,
    pub user_count: u64,
    pub percentage: f64,
    pub monthly_revenue: f64,
}

/// サブスクリプション制限情報
#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionLimits {
    pub max_organizations: Option<u64>,
    pub max_teams_per_org: Option<u64>,
    pub max_team_members: Option<u64>,
    pub max_tasks: Option<u64>,
    pub max_file_storage_gb: Option<u64>,
    pub api_rate_limit_per_hour: Option<u64>,
}

/// サブスクリプション階層情報
#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionTierInfo {
    pub tier: String,
    pub display_name: String,
    pub level: u8,
    pub monthly_price: f64,
    pub annual_price: f64,
    pub features: Vec<String>,
    pub limits: SubscriptionLimits,
}

/// サブスクリプション概要レスポンス（管理者用）
#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionOverviewResponse {
    pub total_users: u64,
    pub distribution: Vec<TierDistribution>,
    pub tier_changes: Vec<TierChangeStats>,
    pub recent_upgrades_count: u64,
    pub recent_downgrades_count: u64,
    pub period_days: u32,
    pub tier_stats: Vec<SubscriptionTierStatsResponse>,
    pub revenue_stats: Option<RevenueStats>,
}

/// 階層別ユーザー分布
#[derive(Debug, Serialize, Deserialize)]
pub struct TierDistribution {
    pub tier: String,
    pub user_count: u64,
}

/// 階層変更統計
#[derive(Debug, Serialize, Deserialize)]
pub struct TierChangeStats {
    pub tier: String,
    pub change_count: u64,
}

/// 収益統計
#[derive(Debug, Serialize, Deserialize)]
pub struct RevenueStats {
    pub monthly_recurring_revenue: f64,
    pub annual_recurring_revenue: f64,
    pub average_revenue_per_user: f64,
}

// Response type aliases for handlers
#[allow(dead_code)] // Type alias for API responses
pub type UpgradeSubscriptionResponse = ApiResponse<OperationResult<SubscriptionChangeResponse>>;
#[allow(dead_code)] // Type alias for API responses
pub type DowngradeSubscriptionResponse = ApiResponse<OperationResult<SubscriptionChangeResponse>>;

// Implementation of response builders
impl CurrentSubscriptionResponse {
    #[allow(dead_code)] // DTO constructor method
    pub fn new(user_id: Uuid, tier: String, subscribed_at: DateTime<Utc>) -> Self {
        let tier_info = Self::get_tier_info(&tier);
        let current_tier_obj = SubscriptionTier::from_str(&tier);

        Self {
            user_id,
            current_tier: tier.clone(),
            tier_display_name: tier_info.display_name.clone(),
            tier_level: tier_info.level,
            subscribed_at,
            features: tier_info.features,
            limits: tier_info.limits,
            next_available_tiers: SubscriptionTier::all()
                .into_iter()
                .filter(|t| {
                    if let Some(current) = &current_tier_obj {
                        t.level() > current.level()
                    } else {
                        true
                    }
                })
                .map(|t| Self::get_tier_info(t.as_str()))
                .collect(),
        }
    }

    pub fn get_tier_info(tier: &str) -> SubscriptionTierInfo {
        match tier {
            "free" => SubscriptionTierInfo {
                tier: "free".to_string(),
                display_name: "Free".to_string(),
                level: 1,
                monthly_price: 0.0,
                annual_price: 0.0,
                features: vec![
                    "Basic task management".to_string(),
                    "Up to 3 teams".to_string(),
                    "Community support".to_string(),
                ],
                limits: SubscriptionLimits {
                    max_organizations: Some(1),
                    max_teams_per_org: Some(3),
                    max_team_members: Some(5),
                    max_tasks: Some(100),
                    max_file_storage_gb: Some(1),
                    api_rate_limit_per_hour: Some(100),
                },
            },
            "pro" => SubscriptionTierInfo {
                tier: "pro".to_string(),
                display_name: "Professional".to_string(),
                level: 2,
                monthly_price: 9.99,
                annual_price: 99.99,
                features: vec![
                    "Advanced task management".to_string(),
                    "Unlimited teams".to_string(),
                    "Priority support".to_string(),
                    "API access".to_string(),
                    "Advanced analytics".to_string(),
                ],
                limits: SubscriptionLimits {
                    max_organizations: Some(5),
                    max_teams_per_org: None,
                    max_team_members: Some(20),
                    max_tasks: Some(10000),
                    max_file_storage_gb: Some(50),
                    api_rate_limit_per_hour: Some(1000),
                },
            },
            "enterprise" => SubscriptionTierInfo {
                tier: "enterprise".to_string(),
                display_name: "Enterprise".to_string(),
                level: 3,
                monthly_price: 49.99,
                annual_price: 499.99,
                features: vec![
                    "Enterprise task management".to_string(),
                    "Unlimited everything".to_string(),
                    "Dedicated support".to_string(),
                    "Advanced API access".to_string(),
                    "Custom integrations".to_string(),
                    "Advanced security features".to_string(),
                ],
                limits: SubscriptionLimits {
                    max_organizations: None,
                    max_teams_per_org: None,
                    max_team_members: None,
                    max_tasks: None,
                    max_file_storage_gb: None,
                    api_rate_limit_per_hour: None,
                },
            },
            _ => SubscriptionTierInfo {
                tier: tier.to_string(),
                display_name: tier.to_string(),
                level: 0,
                monthly_price: 0.0,
                annual_price: 0.0,
                features: vec![],
                limits: SubscriptionLimits {
                    max_organizations: Some(0),
                    max_teams_per_org: Some(0),
                    max_team_members: Some(0),
                    max_tasks: Some(0),
                    max_file_storage_gb: Some(0),
                    api_rate_limit_per_hour: Some(0),
                },
            },
        }
    }
}

impl SubscriptionChangeResponse {
    #[allow(dead_code)] // DTO constructor method
    pub fn new(
        user: SafeUser,
        previous_tier: String,
        new_tier: String,
        reason: Option<String>,
        changed_by: Option<Uuid>,
    ) -> Self {
        let change_type = if changed_by.is_some() && changed_by != Some(user.id) {
            "admin_change".to_string()
        } else {
            match (
                SubscriptionTier::from_str(&previous_tier),
                SubscriptionTier::from_str(&new_tier),
            ) {
                (Some(prev), Some(new)) => {
                    if new.level() > prev.level() {
                        "upgrade".to_string()
                    } else {
                        "downgrade".to_string()
                    }
                }
                _ => "change".to_string(),
            }
        };

        let message = match change_type.as_str() {
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
                "Subscription changed from {} to {}",
                previous_tier, new_tier
            ),
        };

        Self {
            user,
            previous_tier,
            new_tier,
            change_type,
            reason,
            changed_at: Utc::now(),
            changed_by,
            message,
        }
    }
}

impl UserSubscriptionStatsResponse {
    pub fn from_stats(stats: UserSubscriptionStats) -> Self {
        let time_on_current_tier_days = stats.first_subscription_date.map(|date| {
            let duration = Utc::now() - date;
            duration.num_days() as u64
        });

        Self {
            user_id: stats.user_id,
            total_changes: stats.total_changes,
            upgrade_count: stats.upgrade_count,
            downgrade_count: stats.downgrade_count,
            current_tier: stats.current_tier,
            first_subscription_date: stats.first_subscription_date,
            time_on_current_tier_days,
        }
    }
}

impl SubscriptionTierStatsResponse {
    pub fn from_stats(stats: SubscriptionTierStats) -> Self {
        let tier_info = CurrentSubscriptionResponse::get_tier_info(&stats.tier);
        let monthly_revenue = tier_info.monthly_price * stats.user_count as f64;

        Self {
            tier: stats.tier,
            tier_display_name: tier_info.display_name,
            user_count: stats.user_count,
            percentage: (stats.user_count as f64 / stats.total_users as f64) * 100.0,
            monthly_revenue,
        }
    }
}
