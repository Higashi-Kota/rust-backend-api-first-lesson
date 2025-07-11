// task-backend/src/api/dto/subscription_history_dto.rs

use crate::features::subscription::models::history::{
    Model as SubscriptionHistory, SubscriptionChangeInfo,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// サブスクリプション履歴アイテムレスポンス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionHistoryItemResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub previous_tier: Option<String>,
    pub new_tier: String,
    pub changed_at: DateTime<Utc>,
    pub changed_by: Option<Uuid>,
    pub reason: Option<String>,
    pub is_upgrade: bool,
    pub is_downgrade: bool,
}

impl From<SubscriptionHistory> for SubscriptionHistoryItemResponse {
    fn from(history: SubscriptionHistory) -> Self {
        let (is_upgrade, is_downgrade) = match &history.previous_tier {
            None => (false, false), // 初回登録
            Some(prev) => {
                let prev_level = get_tier_level(prev);
                let new_level = get_tier_level(&history.new_tier);
                (new_level > prev_level, new_level < prev_level)
            }
        };

        Self {
            id: history.id,
            user_id: history.user_id,
            previous_tier: history.previous_tier,
            new_tier: history.new_tier,
            changed_at: history.changed_at,
            changed_by: history.changed_by,
            reason: history.reason,
            is_upgrade,
            is_downgrade,
        }
    }
}

fn get_tier_level(tier: &str) -> i32 {
    match tier.to_lowercase().as_str() {
        "free" => 1,
        "pro" => 2,
        "enterprise" => 3,
        _ => 0,
    }
}

/// サブスクリプション履歴レスポンス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionHistoryResponse {
    pub user_id: Uuid,
    pub history: Vec<SubscriptionHistoryItemResponse>,
    pub pagination: Option<crate::shared::types::pagination::PaginationMeta>,
    pub stats: SubscriptionStatsResponse,
}

/// サブスクリプション統計レスポンス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionStatsResponse {
    pub total_changes: u64,
    pub upgrade_count: u64,
    pub downgrade_count: u64,
    pub current_tier: Option<String>,
    pub subscription_duration_days: Option<i64>,
}

/// サブスクリプション変更情報レスポンス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionChangeInfoResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub previous_tier: Option<String>,
    pub new_tier: String,
    pub changed_at: DateTime<Utc>,
    pub change_type: String,
    pub is_upgrade: bool,
    pub is_downgrade: bool,
}

impl From<SubscriptionChangeInfo> for SubscriptionChangeInfoResponse {
    fn from(info: SubscriptionChangeInfo) -> Self {
        let change_type = if info.previous_tier.is_none() {
            "initial".to_string()
        } else if info.is_upgrade {
            "upgrade".to_string()
        } else if info.is_downgrade {
            "downgrade".to_string()
        } else {
            "lateral".to_string()
        };

        Self {
            id: info.id,
            user_id: info.user_id,
            previous_tier: info.previous_tier,
            new_tier: info.new_tier,
            changed_at: info.changed_at,
            change_type,
            is_upgrade: info.is_upgrade,
            is_downgrade: info.is_downgrade,
        }
    }
}

/// サブスクリプション履歴検索クエリ（ページネーション付き）
#[derive(Debug, Deserialize, Validate)]
#[allow(dead_code)] // Query parameters used in deserialization
pub struct SubscriptionHistorySearchQuery {
    pub tier: Option<String>,
    pub user_id: Option<Uuid>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    #[serde(default = "default_page")]
    pub page: i32,
    #[serde(default = "default_per_page")]
    pub per_page: i32,
}

fn default_page() -> i32 {
    1
}

fn default_per_page() -> i32 {
    10
}

/// サブスクリプション分析レスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionAnalyticsResponse {
    pub total_upgrades: u64,
    pub total_downgrades: u64,
    pub tier_distribution: Vec<SubscriptionTierDistribution>,
    pub recent_upgrades: Vec<SubscriptionChangeInfo>,
    pub recent_downgrades: Vec<SubscriptionChangeInfo>,
    pub monthly_trend: Vec<MonthlyTrend>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionTierDistribution {
    pub tier: String,
    pub count: u64,
    pub percentage: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MonthlyTrend {
    pub month: String,
    pub upgrades: u64,
    pub downgrades: u64,
    pub net_change: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteHistoryResponse {
    pub user_id: Uuid,
    pub deleted_count: u64,
    pub deleted_at: DateTime<Utc>,
}
