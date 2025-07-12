#![allow(dead_code)] // Repository methods for activity summaries

use crate::error::AppResult;
use crate::features::analytics::models::daily_activity_summary::Entity as DailyActivitySummary;
use chrono::{NaiveDate, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter,
    QueryOrder, Set,
};

pub struct DailyActivitySummaryRepository {
    db: DatabaseConnection,
}

impl DailyActivitySummaryRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn upsert(
        &self,
        date: NaiveDate,
        input: crate::features::analytics::models::daily_activity_summary::DailyActivityInput,
    ) -> AppResult<crate::features::analytics::models::daily_activity_summary::Model> {
        // 既存のサマリーを検索
        let existing = DailyActivitySummary::find()
            .filter(
                crate::features::analytics::models::daily_activity_summary::Column::Date.eq(date),
            )
            .one(&self.db)
            .await?;

        match existing {
            Some(mut model) => {
                // 既存のレコードを更新
                model.update(input);
                let mut active_model = model.clone().into_active_model();
                active_model.total_users = Set(model.total_users);
                active_model.active_users = Set(model.active_users);
                active_model.new_users = Set(model.new_users);
                active_model.tasks_created = Set(model.tasks_created);
                active_model.tasks_completed = Set(model.tasks_completed);
                active_model.updated_at = Set(model.updated_at);

                let result = active_model.update(&self.db).await?;
                Ok(result)
            }
            None => {
                // 新規レコードを作成
                let model = crate::features::analytics::models::daily_activity_summary::Model::new(
                    date, input,
                );
                let active_model =
                    crate::features::analytics::models::daily_activity_summary::ActiveModel {
                        date: Set(model.date),
                        total_users: Set(model.total_users),
                        active_users: Set(model.active_users),
                        new_users: Set(model.new_users),
                        tasks_created: Set(model.tasks_created),
                        tasks_completed: Set(model.tasks_completed),
                        created_at: Set(model.created_at),
                        updated_at: Set(model.updated_at),
                    };

                let result = active_model.insert(&self.db).await?;
                Ok(result)
            }
        }
    }

    pub async fn get_by_date(
        &self,
        date: NaiveDate,
    ) -> AppResult<Option<crate::features::analytics::models::daily_activity_summary::Model>> {
        let summary = DailyActivitySummary::find()
            .filter(
                crate::features::analytics::models::daily_activity_summary::Column::Date.eq(date),
            )
            .one(&self.db)
            .await?;

        Ok(summary)
    }

    pub async fn get_date_range(
        &self,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> AppResult<Vec<crate::features::analytics::models::daily_activity_summary::Model>> {
        let summaries = DailyActivitySummary::find()
            .filter(
                crate::features::analytics::models::daily_activity_summary::Column::Date
                    .gte(start_date),
            )
            .filter(
                crate::features::analytics::models::daily_activity_summary::Column::Date
                    .lte(end_date),
            )
            .order_by_desc(crate::features::analytics::models::daily_activity_summary::Column::Date)
            .all(&self.db)
            .await?;

        Ok(summaries)
    }

    pub async fn get_latest(
        &self,
        days: i64,
    ) -> AppResult<Vec<crate::features::analytics::models::daily_activity_summary::Model>> {
        let start_date = Utc::now().date_naive() - chrono::Duration::days(days - 1);
        let end_date = Utc::now().date_naive();

        self.get_date_range(start_date, end_date).await
    }

    pub async fn calculate_growth_rate(&self, days: i64) -> AppResult<f64> {
        let summaries = self.get_latest(days).await?;

        if summaries.is_empty() || summaries.len() < 2 {
            return Ok(0.0);
        }

        // 最新と最古のデータを取得
        let latest = &summaries[0];
        let oldest = &summaries[summaries.len() - 1];

        if oldest.total_users == 0 {
            return Ok(0.0);
        }

        // 成長率を計算: ((最新 - 最古) / 最古) * 100
        let growth_rate =
            ((latest.total_users - oldest.total_users) as f64 / oldest.total_users as f64) * 100.0;

        Ok(growth_rate)
    }

    pub async fn delete_old_summaries(&self, before_date: NaiveDate) -> AppResult<u64> {
        let result = DailyActivitySummary::delete_many()
            .filter(
                crate::features::analytics::models::daily_activity_summary::Column::Date
                    .lt(before_date),
            )
            .exec(&self.db)
            .await?;

        Ok(result.rows_affected)
    }
}
