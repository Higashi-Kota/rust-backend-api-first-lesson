// task-backend/src/features/subscription/services/subscription.rs

use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::features::user::models::user::Model as User;
use crate::features::user::repositories::user::{SubscriptionTierStats, UserRepository};
use crate::infrastructure::email::EmailService;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use uuid::Uuid;

use super::super::models::history::{Model as SubscriptionHistory, SubscriptionChangeInfo};
use super::super::repositories::history::{SubscriptionHistoryRepository, UserSubscriptionStats};

#[derive(Clone)]
pub struct SubscriptionService {
    subscription_history_repo: Arc<SubscriptionHistoryRepository>,
    user_repo: Arc<UserRepository>,
    email_service: Arc<EmailService>,
}

impl SubscriptionService {
    pub fn new(db: DbPool, email_service: Arc<EmailService>) -> Self {
        let subscription_history_repo = Arc::new(SubscriptionHistoryRepository::new(db.clone()));
        let user_repo = Arc::new(UserRepository::new(db.clone()));

        Self {
            subscription_history_repo,
            user_repo,
            email_service,
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
            .create(user_id, previous_tier, new_tier.clone(), changed_by, reason)
            .await?;

        // サブスクリプション変更メールを送信
        if let Err(e) = self
            .email_service
            .send_subscription_change_email(
                &updated_user.email,
                &updated_user.username,
                &current_user.subscription_tier,
                &new_tier,
            )
            .await
        {
            // メール送信失敗はログに記録するが、処理は継続
            tracing::warn!("Failed to send subscription change email: {}", e);
        }

        Ok((updated_user, history))
    }

    /// ユーザーのサブスクリプション履歴を取得（ページング付き）
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

    /// 期間内のサブスクリプション履歴を取得
    pub async fn get_subscription_history_by_date_range(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> AppResult<Vec<SubscriptionChangeInfo>> {
        let histories = self
            .subscription_history_repo
            .find_by_date_range(start_date, end_date)
            .await?;

        Ok(histories
            .into_iter()
            .map(SubscriptionChangeInfo::from)
            .collect())
    }

    /// 階層変更統計を取得
    pub async fn get_tier_change_statistics(&self) -> AppResult<Vec<(String, u64)>> {
        self.subscription_history_repo.get_tier_change_stats().await
    }

    /// アップグレード履歴を取得
    pub async fn get_upgrade_history(&self) -> AppResult<Vec<SubscriptionChangeInfo>> {
        self.subscription_history_repo.find_upgrades().await
    }

    /// ダウングレード履歴を取得
    pub async fn get_downgrade_history(&self) -> AppResult<Vec<SubscriptionChangeInfo>> {
        self.subscription_history_repo.find_downgrades().await
    }

    /// サブスクリプション分布を取得
    pub async fn get_subscription_distribution(&self) -> AppResult<Vec<(String, u64)>> {
        // サブスクリプション階層別のユーザー数を取得
        let tier_stats = self.user_repo.get_subscription_tier_stats().await?;

        Ok(tier_stats
            .into_iter()
            .map(|stat| (stat.tier, stat.user_count))
            .collect())
    }
}
