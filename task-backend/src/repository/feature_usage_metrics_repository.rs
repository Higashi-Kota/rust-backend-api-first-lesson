use crate::domain::feature_usage_metrics_model::{self, Entity as FeatureUsageMetrics};
use crate::error::AppResult;
use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};
use std::collections::HashMap;
use uuid::Uuid;

pub struct FeatureUsageMetricsRepository {
    db: DatabaseConnection,
}

impl FeatureUsageMetricsRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn create(
        &self,
        user_id: Uuid,
        input: feature_usage_metrics_model::FeatureUsageInput,
    ) -> AppResult<feature_usage_metrics_model::Model> {
        let model = feature_usage_metrics_model::Model::new(user_id, input);

        let active_model = feature_usage_metrics_model::ActiveModel {
            id: Set(model.id),
            user_id: Set(model.user_id),
            feature_name: Set(model.feature_name.clone()),
            action_type: Set(model.action_type.clone()),
            metadata: Set(model.metadata.clone()),
            created_at: Set(model.created_at),
        };

        let result = active_model.insert(&self.db).await?;
        Ok(result)
    }

    pub async fn get_by_user_and_date_range(
        &self,
        user_id: Uuid,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> AppResult<Vec<feature_usage_metrics_model::Model>> {
        let metrics = FeatureUsageMetrics::find()
            .filter(feature_usage_metrics_model::Column::UserId.eq(user_id))
            .filter(feature_usage_metrics_model::Column::CreatedAt.gte(start_date))
            .filter(feature_usage_metrics_model::Column::CreatedAt.lt(end_date))
            .order_by_desc(feature_usage_metrics_model::Column::CreatedAt)
            .all(&self.db)
            .await?;

        Ok(metrics)
    }

    pub async fn get_feature_usage_counts(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> AppResult<HashMap<String, i64>> {
        use sea_orm::sea_query::Expr;
        use sea_orm::{FromQueryResult, QuerySelect};

        #[derive(Debug, FromQueryResult)]
        struct FeatureUsageCount {
            feature_name: String,
            count: i64,
        }

        let results = FeatureUsageMetrics::find()
            .select_only()
            .column(feature_usage_metrics_model::Column::FeatureName)
            .column_as(
                Expr::col(feature_usage_metrics_model::Column::Id).count(),
                "count",
            )
            .filter(feature_usage_metrics_model::Column::CreatedAt.gte(start_date))
            .filter(feature_usage_metrics_model::Column::CreatedAt.lt(end_date))
            .group_by(feature_usage_metrics_model::Column::FeatureName)
            .into_model::<FeatureUsageCount>()
            .all(&self.db)
            .await?;

        let mut counts = HashMap::new();
        for result in results {
            counts.insert(result.feature_name, result.count);
        }

        Ok(counts)
    }

    pub async fn get_user_action_counts(
        &self,
        user_id: Uuid,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> AppResult<HashMap<String, i64>> {
        use sea_orm::sea_query::Expr;
        use sea_orm::{FromQueryResult, QuerySelect};

        #[derive(Debug, FromQueryResult)]
        struct ActionCount {
            feature_name: String,
            action_type: String,
            count: i64,
        }

        let results = FeatureUsageMetrics::find()
            .select_only()
            .column(feature_usage_metrics_model::Column::FeatureName)
            .column(feature_usage_metrics_model::Column::ActionType)
            .column_as(
                Expr::col(feature_usage_metrics_model::Column::Id).count(),
                "count",
            )
            .filter(feature_usage_metrics_model::Column::UserId.eq(user_id))
            .filter(feature_usage_metrics_model::Column::CreatedAt.gte(start_date))
            .filter(feature_usage_metrics_model::Column::CreatedAt.lt(end_date))
            .group_by(feature_usage_metrics_model::Column::FeatureName)
            .group_by(feature_usage_metrics_model::Column::ActionType)
            .into_model::<ActionCount>()
            .all(&self.db)
            .await?;

        let mut counts = HashMap::new();
        for result in results {
            let key = format!("{}_{}", result.feature_name, result.action_type);
            counts.insert(key, result.count);
        }

        Ok(counts)
    }

    pub async fn delete_old_metrics(&self, before_date: DateTime<Utc>) -> AppResult<u64> {
        let result = FeatureUsageMetrics::delete_many()
            .filter(feature_usage_metrics_model::Column::CreatedAt.lt(before_date))
            .exec(&self.db)
            .await?;

        Ok(result.rows_affected)
    }
}
