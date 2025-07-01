// task-backend/src/api/dto/subscription_history_dto.rs

use crate::domain::subscription_history_model::{
    Model as SubscriptionHistory, SubscriptionChangeInfo,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
    pub pagination: Option<crate::api::dto::common::PaginationMeta>,
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
