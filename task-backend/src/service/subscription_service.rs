// task-backend/src/service/subscription_service.rs

use crate::db::DbPool;
use crate::domain::subscription_history_model::{
    Model as SubscriptionHistory, SubscriptionChangeInfo,
};
use crate::domain::user_model::Model as User;
use crate::error::{AppError, AppResult};
use crate::repository::subscription_history_repository::{
    SubscriptionHistoryRepository, UserSubscriptionStats,
};
use crate::repository::user_repository::{SubscriptionTierStats, UserRepository};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct SubscriptionService {
    subscription_history_repo: Arc<SubscriptionHistoryRepository>,
    user_repo: Arc<UserRepository>,
}

impl SubscriptionService {
    pub fn new(db: DbPool) -> Self {
        let subscription_history_repo = Arc::new(SubscriptionHistoryRepository::new(db.clone()));
        let user_repo = Arc::new(UserRepository::new(db.clone()));

        Self {
            subscription_history_repo,
            user_repo,
        }
    }

    /// ユーザーのサブスクリプション階層を変更
    pub async fn change_subscription_tier(
        &self,
        user_id: Uuid,
        new_tier: String,
        changed_by: Option<Uuid>,
        reason: Option<String>,
    ) -> AppResult<(User, SubscriptionHistory)> {
        // 現在のユーザー情報を取得
        let current_user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        let previous_tier = Some(current_user.subscription_tier.clone());

        // 同じ階層の場合は変更しない
        if current_user.subscription_tier == new_tier {
            return Err(AppError::ValidationError(
                "New subscription tier is the same as current tier".to_string(),
            ));
        }

        // サブスクリプション階層を検証
        self.validate_subscription_tier(&new_tier)?;

        // ユーザーのサブスクリプション階層を更新
        let updated_user = self
            .user_repo
            .update_subscription_tier(user_id, new_tier.clone())
            .await?
            .ok_or_else(|| AppError::InternalServerError("Failed to update user".to_string()))?;

        // 履歴を記録
        let history = self
            .subscription_history_repo
            .create(user_id, previous_tier, new_tier, changed_by, reason)
            .await?;

        Ok((updated_user, history))
    }

    /// ユーザーのサブスクリプション履歴を取得
    pub async fn get_user_subscription_history(
        &self,
        user_id: Uuid,
        page: u64,
        page_size: u64,
    ) -> AppResult<(Vec<SubscriptionChangeInfo>, u64)> {
        let (histories, total) = self
            .subscription_history_repo
            .find_by_user_id_paginated(user_id, page, page_size)
            .await?;

        let history_info: Vec<SubscriptionChangeInfo> = histories
            .into_iter()
            .map(SubscriptionChangeInfo::from)
            .collect();

        Ok((history_info, total))
    }

    /// ユーザーのサブスクリプション統計を取得
    pub async fn get_user_subscription_stats(
        &self,
        user_id: Uuid,
    ) -> AppResult<UserSubscriptionStats> {
        self.subscription_history_repo
            .get_user_change_stats(user_id)
            .await
    }

    /// サブスクリプション階層別統計を取得
    pub async fn get_subscription_tier_stats(&self) -> AppResult<Vec<SubscriptionTierStats>> {
        self.user_repo
            .get_subscription_tier_stats()
            .await
            .map_err(AppError::DbErr)
    }

    /// サブスクリプション階層の妥当性を検証
    fn validate_subscription_tier(&self, tier: &str) -> AppResult<()> {
        match tier.to_lowercase().as_str() {
            "free" | "pro" | "enterprise" => Ok(()),
            _ => Err(AppError::ValidationError(format!(
                "Invalid subscription tier: {}. Valid tiers are: free, pro, enterprise",
                tier
            ))),
        }
    }
}
