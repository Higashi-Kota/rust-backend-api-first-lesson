use crate::domain::feature_usage_metrics_model::{FeatureUsageInput, Model as FeatureUsageMetric};
use crate::error::AppResult;
use crate::log_with_context;
use crate::repository::feature_usage_metrics_repository::FeatureUsageMetricsRepository;
use std::sync::Arc;
use uuid::Uuid;

/// 機能使用状況追跡サービス
pub struct FeatureTrackingService {
    feature_usage_repo: Arc<FeatureUsageMetricsRepository>,
}

impl FeatureTrackingService {
    pub fn new(feature_usage_repo: Arc<FeatureUsageMetricsRepository>) -> Self {
        Self { feature_usage_repo }
    }

    /// 機能使用状況を記録
    pub async fn track_feature_usage(
        &self,
        user_id: Uuid,
        feature_name: &str,
        action_type: &str,
        metadata: Option<serde_json::Value>,
    ) -> AppResult<()> {
        log_with_context!(
            tracing::Level::DEBUG,
            "Tracking feature usage",
            "user_id" => user_id,
            "feature_name" => feature_name,
            "action_type" => action_type
        );
        let input = FeatureUsageInput {
            feature_name: feature_name.to_string(),
            action_type: action_type.to_string(),
            metadata,
        };

        self.feature_usage_repo.create(user_id, input).await?;

        log_with_context!(
            tracing::Level::INFO,
            "Feature usage tracked successfully",
            "user_id" => user_id,
            "feature_name" => feature_name,
            "action_type" => action_type
        );

        Ok(())
    }

    /// ユーザーの機能使用状況を取得
    pub async fn get_user_feature_usage(
        &self,
        user_id: Uuid,
        days: i64,
    ) -> AppResult<Vec<FeatureUsageMetric>> {
        log_with_context!(
            tracing::Level::DEBUG,
            "Getting user feature usage",
            "user_id" => user_id,
            "days" => days
        );
        let start_date = chrono::Utc::now() - chrono::Duration::days(days);
        let end_date = chrono::Utc::now();

        self.feature_usage_repo
            .get_by_user_and_date_range(user_id, start_date, end_date)
            .await
    }
}
