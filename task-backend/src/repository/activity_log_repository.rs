// task-backend/src/repository/activity_log_repository.rs

use crate::db::DbPool;
use crate::domain::activity_log_model::{ActiveModel, Column, Entity, Model};
use crate::error::AppResult;
use chrono::{DateTime, Utc};
use sea_orm::*;
use uuid::Uuid;

/// アクティビティログ検索用フィルタ
#[derive(Debug, Clone)]
pub struct ActivityLogFilter {
    pub user_id: Option<Uuid>,
    pub resource_type: Option<String>,
    pub action: Option<String>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub page: u64,
    pub per_page: u64,
}

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

    /// アクティビティログをクエリで検索
    pub async fn find_with_query(&self, filter: ActivityLogFilter) -> AppResult<(Vec<Model>, u64)> {
        let mut query = Entity::find();

        // フィルタ条件を適用
        if let Some(user_id) = filter.user_id {
            query = query.filter(Column::UserId.eq(user_id));
        }
        if let Some(resource_type) = filter.resource_type {
            query = query.filter(Column::ResourceType.eq(resource_type));
        }
        if let Some(action) = filter.action {
            query = query.filter(Column::Action.eq(action));
        }
        if let Some(from) = filter.from {
            query = query.filter(Column::CreatedAt.gte(from));
        }
        if let Some(to) = filter.to {
            query = query.filter(Column::CreatedAt.lte(to));
        }

        // 総件数を取得
        let total = query.clone().count(&self.db).await?;

        // ページネーション
        let offset = (filter.page - 1) * filter.per_page;
        let logs = query
            .order_by_desc(Column::CreatedAt)
            .limit(filter.per_page)
            .offset(offset)
            .all(&self.db)
            .await?;

        Ok((logs, total))
    }
}
