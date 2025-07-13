use crate::error::AppResult;
use crate::features::analytics::models::daily_activity_summary::{DailyActivityInput, Model};
use crate::features::analytics::repositories::daily_activity_summary::DailyActivitySummaryRepository;
use chrono::{Duration, NaiveDate, Utc};
use sea_orm::DatabaseConnection;
use std::sync::Arc;

pub struct ActivitySummaryService {
    repository: Arc<DailyActivitySummaryRepository>,
}

impl ActivitySummaryService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            repository: Arc::new(DailyActivitySummaryRepository::new(db)),
        }
    }

    pub async fn get_daily_summary(&self, date: NaiveDate) -> AppResult<Option<Model>> {
        self.repository.get_by_date(date).await
    }

    pub async fn get_summaries_range(
        &self,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> AppResult<Vec<Model>> {
        self.repository.get_date_range(start_date, end_date).await
    }

    pub async fn get_recent_summaries(&self, days: i64) -> AppResult<Vec<Model>> {
        self.repository.get_latest(days).await
    }

    pub async fn calculate_growth_rate(&self, days: i64) -> AppResult<f64> {
        self.repository.calculate_growth_rate(days).await
    }

    pub async fn update_daily_summary(
        &self,
        date: NaiveDate,
        input: DailyActivityInput,
    ) -> AppResult<Model> {
        self.repository.upsert(date, input).await
    }

    pub async fn cleanup_old_summaries(&self, days_to_keep: i64) -> AppResult<u64> {
        let cutoff_date = Utc::now().date_naive() - Duration::days(days_to_keep);
        self.repository.delete_old_summaries(cutoff_date).await
    }
}
