use crate::error::AppResult;
use crate::repository::{
    bulk_operation_history_repository::BulkOperationHistoryRepository,
    daily_activity_summary_repository::DailyActivitySummaryRepository,
    feature_usage_metrics_repository::FeatureUsageMetricsRepository,
};
use chrono::{Duration, Utc};
use sea_orm::DatabaseConnection;
use uuid::Uuid;

pub struct AdminService {
    bulk_operation_repository: BulkOperationHistoryRepository,
    daily_summary_repository: DailyActivitySummaryRepository,
    feature_usage_repository: FeatureUsageMetricsRepository,
}

impl AdminService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            bulk_operation_repository: BulkOperationHistoryRepository::new(db.clone()),
            daily_summary_repository: DailyActivitySummaryRepository::new(db.clone()),
            feature_usage_repository: FeatureUsageMetricsRepository::new(db),
        }
    }

    /// Clean up daily activity summaries older than specified days
    pub async fn cleanup_daily_summaries(&self, days: i32) -> AppResult<i32> {
        let cutoff_date = Utc::now().date_naive() - Duration::days(days as i64);
        let deleted = self
            .daily_summary_repository
            .delete_old_summaries(cutoff_date)
            .await?;
        Ok(deleted as i32)
    }

    /// Clean up bulk operation history older than specified days
    pub async fn cleanup_bulk_operations(&self, days: i32) -> AppResult<i32> {
        let cutoff_date = Utc::now() - Duration::days(days as i64);
        let deleted = self
            .bulk_operation_repository
            .delete_old_histories(cutoff_date)
            .await?;
        Ok(deleted as i32)
    }

    /// Clean up feature usage metrics older than specified days
    pub async fn cleanup_feature_usage_metrics(&self, days: i32) -> AppResult<i32> {
        let cutoff_date = Utc::now() - Duration::days(days as i64);
        let deleted = self
            .feature_usage_repository
            .delete_old_metrics(cutoff_date)
            .await?;
        Ok(deleted as i32)
    }

    /// Alias for cleanup_feature_usage_metrics
    pub async fn cleanup_feature_metrics(&self, days: i32) -> AppResult<i32> {
        self.cleanup_feature_usage_metrics(days).await
    }

    /// Clean up all expired data
    pub async fn cleanup_all(
        &self,
        daily_summaries_days: i32,
        bulk_operations_days: i32,
        feature_metrics_days: i32,
    ) -> AppResult<(i32, i32, i32)> {
        let daily_count = self.cleanup_daily_summaries(daily_summaries_days).await?;
        let bulk_count = self.cleanup_bulk_operations(bulk_operations_days).await?;
        let metrics_count = self
            .cleanup_feature_usage_metrics(feature_metrics_days)
            .await?;
        Ok((daily_count, bulk_count, metrics_count))
    }

    /// List bulk operations
    pub async fn list_bulk_operations(
        &self,
    ) -> AppResult<Vec<crate::domain::bulk_operation_history_model::Model>> {
        // Get recent bulk operations (last 100)
        self.bulk_operation_repository.get_recent(100).await
    }

    /// Get user feature metrics
    pub async fn get_user_feature_metrics(
        &self,
        user_id: Uuid,
        days: i32,
    ) -> AppResult<Vec<crate::domain::feature_usage_metrics_model::Model>> {
        let since = Utc::now() - Duration::days(days as i64);
        self.feature_usage_repository
            .get_user_metrics(user_id, since)
            .await
    }
}
