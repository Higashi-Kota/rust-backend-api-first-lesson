// task-backend/src/features/subscription/repositories/history.rs
#![allow(dead_code)] // Repository methods for subscription history

use crate::db::DbPool;
use crate::error::AppResult;
use sea_orm::*;
use uuid::Uuid;

use super::super::models::history::{Column, Entity, Model, SubscriptionChangeInfo};

pub struct SubscriptionHistoryRepository {
    db: DbPool,
}

impl SubscriptionHistoryRepository {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    /// サブスクリプション変更履歴を作成
    pub async fn create(
        &self,
        user_id: Uuid,
        previous_tier: Option<String>,
        new_tier: String,
        changed_by: Option<Uuid>,
        reason: Option<String>,
    ) -> AppResult<Model> {
        let history = Model::new(user_id, previous_tier, new_tier, changed_by, reason);

        // ActiveModelを挿入してModelを取得
        let created_history = history.insert(&self.db).await?;

        Ok(created_history)
    }

    /// すべてのサブスクリプション履歴を取得
    pub async fn find_all(&self) -> AppResult<Vec<Model>> {
        let histories = Entity::find()
            .order_by_desc(Column::ChangedAt)
            .all(&self.db)
            .await?;
        Ok(histories)
    }

    /// ユーザーのサブスクリプション履歴を時系列で取得
    pub async fn find_by_user_id(&self, user_id: Uuid) -> AppResult<Vec<Model>> {
        let histories = Entity::find()
            .filter(Column::UserId.eq(user_id))
            .order_by_desc(Column::ChangedAt)
            .all(&self.db)
            .await?;

        Ok(histories)
    }

    /// ユーザーのサブスクリプション履歴を取得（ページネーション付き）
    pub async fn find_by_user_id_paginated(
        &self,
        user_id: Uuid,
        page: u64,
        page_size: u64,
    ) -> AppResult<(Vec<Model>, u64)> {
        let offset = (page - 1) * page_size;

        let histories = Entity::find()
            .filter(Column::UserId.eq(user_id))
            .order_by_desc(Column::ChangedAt)
            .offset(offset)
            .limit(page_size)
            .all(&self.db)
            .await?;

        let total_count = Entity::find()
            .filter(Column::UserId.eq(user_id))
            .count(&self.db)
            .await?;

        Ok((histories, total_count))
    }

    /// 最新のサブスクリプション変更履歴を取得
    pub async fn find_latest_by_user_id(&self, user_id: Uuid) -> AppResult<Option<Model>> {
        let latest = Entity::find()
            .filter(Column::UserId.eq(user_id))
            .order_by_desc(Column::ChangedAt)
            .one(&self.db)
            .await?;

        Ok(latest)
    }

    /// 特定期間のサブスクリプション変更履歴を取得
    pub async fn find_by_date_range(
        &self,
        start_date: chrono::DateTime<chrono::Utc>,
        end_date: chrono::DateTime<chrono::Utc>,
    ) -> AppResult<Vec<Model>> {
        let histories = Entity::find()
            .filter(Column::ChangedAt.between(start_date, end_date))
            .order_by_desc(Column::ChangedAt)
            .all(&self.db)
            .await?;

        Ok(histories)
    }

    /// 特定階層への変更履歴を取得
    pub async fn find_by_tier(&self, tier: &str) -> AppResult<Vec<Model>> {
        let histories = Entity::find()
            .filter(Column::NewTier.eq(tier))
            .order_by_desc(Column::ChangedAt)
            .all(&self.db)
            .await?;

        Ok(histories)
    }

    /// アップグレード履歴のみを取得
    pub async fn find_upgrades(&self) -> AppResult<Vec<SubscriptionChangeInfo>> {
        let histories = Entity::find()
            .order_by_desc(Column::ChangedAt)
            .all(&self.db)
            .await?;

        let upgrades: Vec<SubscriptionChangeInfo> = histories
            .into_iter()
            .filter(|h| h.is_upgrade())
            .map(SubscriptionChangeInfo::from)
            .collect();

        Ok(upgrades)
    }

    /// ダウングレード履歴のみを取得
    pub async fn find_downgrades(&self) -> AppResult<Vec<SubscriptionChangeInfo>> {
        let histories = Entity::find()
            .order_by_desc(Column::ChangedAt)
            .all(&self.db)
            .await?;

        let downgrades: Vec<SubscriptionChangeInfo> = histories
            .into_iter()
            .filter(|h| h.is_downgrade())
            .map(SubscriptionChangeInfo::from)
            .collect();

        Ok(downgrades)
    }

    /// 統計情報: 階層別の変更回数
    pub async fn get_tier_change_stats(&self) -> AppResult<Vec<(String, u64)>> {
        use sea_orm::QuerySelect;

        let stats = Entity::find()
            .select_only()
            .column(Column::NewTier)
            .column_as(Column::Id.count(), "count")
            .group_by(Column::NewTier)
            .order_by_desc(Column::Id.count())
            .into_tuple::<(String, i64)>()
            .all(&self.db)
            .await?;

        let stats: Vec<(String, u64)> = stats
            .into_iter()
            .map(|(tier, count)| (tier, count as u64))
            .collect();

        Ok(stats)
    }

    /// 特定ユーザーの統計情報
    pub async fn get_user_change_stats(&self, user_id: Uuid) -> AppResult<UserSubscriptionStats> {
        let total_changes = Entity::find()
            .filter(Column::UserId.eq(user_id))
            .count(&self.db)
            .await?;

        let all_changes = Entity::find()
            .filter(Column::UserId.eq(user_id))
            .all(&self.db)
            .await?;

        let upgrades = all_changes.iter().filter(|h| h.is_upgrade()).count();

        let downgrades = all_changes.iter().filter(|h| h.is_downgrade()).count();

        let latest = self.find_latest_by_user_id(user_id).await?;

        // Calculate days at current tier
        let days_at_current_tier = if let Some(ref latest_change) = latest {
            let now = chrono::Utc::now();
            let duration = now - latest_change.changed_at;
            duration.num_days() as u64
        } else {
            0
        };

        // Check if user has ever upgraded
        let has_ever_upgraded = upgrades > 0;

        Ok(UserSubscriptionStats {
            user_id,
            total_changes,
            upgrade_count: upgrades as u64,
            downgrade_count: downgrades as u64,
            current_tier: latest.map(|h| h.new_tier),
            first_subscription_date: Entity::find()
                .filter(Column::UserId.eq(user_id))
                .order_by_asc(Column::ChangedAt)
                .one(&self.db)
                .await?
                .map(|h| h.changed_at),
            days_at_current_tier,
            has_ever_upgraded,
        })
    }

    /// ID で履歴を取得
    pub async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Model>> {
        let history = Entity::find_by_id(id).one(&self.db).await?;
        Ok(history)
    }

    /// 履歴を削除（通常は行わないが、GDPR対応など）
    pub async fn delete_by_id(&self, id: Uuid) -> AppResult<bool> {
        let result = Entity::delete_by_id(id).exec(&self.db).await?;
        Ok(result.rows_affected > 0)
    }

    /// ユーザーの全履歴を削除（ユーザー削除時など）
    pub async fn delete_by_user_id(&self, user_id: Uuid) -> AppResult<u64> {
        let result = Entity::delete_many()
            .filter(Column::UserId.eq(user_id))
            .exec(&self.db)
            .await?;

        Ok(result.rows_affected)
    }

    /// 特定期間内のコンバージョン数（無料から有料への変更）をカウント
    pub async fn count_conversions_in_period(
        &self,
        start_date: chrono::DateTime<chrono::Utc>,
        end_date: chrono::DateTime<chrono::Utc>,
    ) -> AppResult<u64> {
        let histories = Entity::find()
            .filter(Column::ChangedAt.between(start_date, end_date))
            .filter(Column::PreviousTier.eq("free"))
            .filter(Column::NewTier.ne("free"))
            .count(&self.db)
            .await?;

        Ok(histories)
    }
}

/// ユーザーのサブスクリプション統計情報
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserSubscriptionStats {
    pub user_id: Uuid,
    pub total_changes: u64,
    pub upgrade_count: u64,
    pub downgrade_count: u64,
    pub current_tier: Option<String>,
    pub first_subscription_date: Option<chrono::DateTime<chrono::Utc>>,
    pub days_at_current_tier: u64,
    pub has_ever_upgraded: bool,
}
