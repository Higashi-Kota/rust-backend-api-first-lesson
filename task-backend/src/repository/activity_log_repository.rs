// task-backend/src/repository/activity_log_repository.rs

use crate::db::DbPool;
use crate::domain::activity_log_model::{ActiveModel, Column, Entity, Model};
use crate::error::AppResult;
use chrono::{DateTime, Utc};
use sea_orm::*;

#[derive(Clone)]
pub struct ActivityLogRepository {
    db: DbPool,
}

impl ActivityLogRepository {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    /// アクティビティログを作成
    pub async fn create(&self, log: &Model) -> AppResult<Model> {
        let active_model = ActiveModel {
            id: Set(log.id),
            user_id: Set(log.user_id),
            action: Set(log.action.clone()),
            resource_type: Set(log.resource_type.clone()),
            resource_id: Set(log.resource_id),
            ip_address: Set(log.ip_address.clone()),
            user_agent: Set(log.user_agent.clone()),
            details: Set(log.details.clone()),
            created_at: Set(log.created_at),
        };

        let result = active_model.insert(&self.db).await?;
        Ok(result)
    }

    /// 今日のユニークユーザー数を取得
    pub async fn count_unique_users_today(&self) -> AppResult<u64> {
        let today = Utc::now().date_naive().and_hms_opt(0, 0, 0).unwrap();
        let today_utc = DateTime::<Utc>::from_naive_utc_and_offset(today, Utc);

        let count = Entity::find()
            .filter(Column::CreatedAt.gte(today_utc))
            .select_only()
            .column(Column::UserId)
            .distinct()
            .count(&self.db)
            .await?;

        Ok(count)
    }

    /// 今週のユニークユーザー数を取得
    pub async fn count_unique_users_this_week(&self) -> AppResult<u64> {
        let week_ago = Utc::now() - chrono::Duration::days(7);

        let count = Entity::find()
            .filter(Column::CreatedAt.gte(week_ago))
            .select_only()
            .column(Column::UserId)
            .distinct()
            .count(&self.db)
            .await?;

        Ok(count)
    }

    /// 指定日数内のユニークユーザー数を取得
    pub async fn count_unique_users_in_days(&self, days: i64) -> AppResult<u64> {
        let days_ago = Utc::now() - chrono::Duration::days(days);

        let count = Entity::find()
            .filter(Column::CreatedAt.gte(days_ago))
            .select_only()
            .column(Column::UserId)
            .distinct()
            .count(&self.db)
            .await?;

        Ok(count)
    }
}
