use super::super::models::analytics::{
    self, AnalyticsType, Entity as OrganizationAnalytics, Period,
};
use crate::error::AppError;
use chrono::{DateTime, Utc};
use sea_orm::prelude::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use uuid::Uuid;

// TODO: Phase 19で本来の使用箇所が移行されたら#[allow(dead_code)]を削除
#[allow(dead_code)]
pub struct AnalyticsRepository;

impl AnalyticsRepository {
    pub async fn create(
        db: &DatabaseConnection,
        analytics: analytics::ActiveModel,
    ) -> Result<analytics::Model, AppError> {
        let result = analytics.insert(db).await?;
        Ok(result)
    }

    pub async fn find_by_organization_id(
        db: &DatabaseConnection,
        organization_id: Uuid,
        limit: Option<u64>,
    ) -> Result<Vec<analytics::Model>, AppError> {
        let mut query = OrganizationAnalytics::find()
            .filter(analytics::Column::OrganizationId.eq(organization_id))
            .order_by_desc(analytics::Column::PeriodEnd);

        if let Some(limit_val) = limit {
            query = query.limit(limit_val);
        }

        let result = query.all(db).await?;
        Ok(result)
    }

    pub async fn find_by_organization_and_type(
        db: &DatabaseConnection,
        organization_id: Uuid,
        analytics_type: AnalyticsType,
        limit: Option<u64>,
    ) -> Result<Vec<analytics::Model>, AppError> {
        let mut query = OrganizationAnalytics::find()
            .filter(analytics::Column::OrganizationId.eq(organization_id))
            .filter(analytics::Column::AnalyticsType.eq(analytics_type.to_string()))
            .order_by_desc(analytics::Column::PeriodEnd);

        if let Some(limit_val) = limit {
            query = query.limit(limit_val);
        }

        let result = query.all(db).await?;
        Ok(result)
    }

    pub async fn find_by_organization_and_period(
        db: &DatabaseConnection,
        organization_id: Uuid,
        period: Period,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<analytics::Model>, AppError> {
        let result = OrganizationAnalytics::find()
            .filter(analytics::Column::OrganizationId.eq(organization_id))
            .filter(analytics::Column::Period.eq(period.to_string()))
            .filter(analytics::Column::PeriodStart.gte(start_date))
            .filter(analytics::Column::PeriodEnd.lte(end_date))
            .order_by_desc(analytics::Column::PeriodEnd)
            .all(db)
            .await?;
        Ok(result)
    }

    pub async fn exists_analytics_for_period(
        db: &DatabaseConnection,
        organization_id: Uuid,
        department_id: Option<Uuid>,
        analytics_type: AnalyticsType,
        metric_name: &str,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> Result<bool, AppError> {
        let mut query = OrganizationAnalytics::find()
            .filter(analytics::Column::OrganizationId.eq(organization_id))
            .filter(analytics::Column::AnalyticsType.eq(analytics_type.to_string()))
            .filter(analytics::Column::MetricName.eq(metric_name))
            .filter(analytics::Column::PeriodStart.eq(period_start))
            .filter(analytics::Column::PeriodEnd.eq(period_end));

        if let Some(dept_id) = department_id {
            query = query.filter(analytics::Column::DepartmentId.eq(dept_id));
        } else {
            query = query.filter(analytics::Column::DepartmentId.is_null());
        }

        let count = query.count(db).await?;
        Ok(count > 0)
    }

    pub async fn find_latest_by_metric(
        db: &DatabaseConnection,
        organization_id: Uuid,
        department_id: Option<Uuid>,
        analytics_type: AnalyticsType,
        metric_name: &str,
    ) -> Result<Option<analytics::Model>, AppError> {
        let mut query = OrganizationAnalytics::find()
            .filter(analytics::Column::OrganizationId.eq(organization_id))
            .filter(analytics::Column::AnalyticsType.eq(analytics_type.to_string()))
            .filter(analytics::Column::MetricName.eq(metric_name));

        if let Some(dept_id) = department_id {
            query = query.filter(analytics::Column::DepartmentId.eq(dept_id));
        } else {
            query = query.filter(analytics::Column::DepartmentId.is_null());
        }

        let result = query
            .order_by_desc(analytics::Column::PeriodEnd)
            .one(db)
            .await?;
        Ok(result)
    }

    pub async fn find_by_department_id(
        db: &DatabaseConnection,
        department_id: Uuid,
        limit: Option<u64>,
    ) -> Result<Vec<analytics::Model>, AppError> {
        let mut query = OrganizationAnalytics::find()
            .filter(analytics::Column::DepartmentId.eq(department_id))
            .order_by_desc(analytics::Column::PeriodEnd);

        if let Some(limit_val) = limit {
            query = query.limit(limit_val);
        }

        let result = query.all(db).await?;
        Ok(result)
    }

    pub async fn find_aggregated_metrics(
        db: &DatabaseConnection,
        organization_id: Uuid,
        analytics_type: AnalyticsType,
        metric_name: &str,
        period: Period,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<analytics::Model>, AppError> {
        let result = OrganizationAnalytics::find()
            .filter(analytics::Column::OrganizationId.eq(organization_id))
            .filter(analytics::Column::AnalyticsType.eq(analytics_type.to_string()))
            .filter(analytics::Column::MetricName.eq(metric_name))
            .filter(analytics::Column::Period.eq(period.to_string()))
            .filter(analytics::Column::PeriodStart.gte(start_date))
            .filter(analytics::Column::PeriodEnd.lte(end_date))
            .order_by_asc(analytics::Column::PeriodStart)
            .all(db)
            .await?;
        Ok(result)
    }

    pub async fn delete_old_analytics(
        db: &DatabaseConnection,
        organization_id: Uuid,
        before_date: DateTime<Utc>,
    ) -> Result<u64, AppError> {
        let result = OrganizationAnalytics::delete_many()
            .filter(analytics::Column::OrganizationId.eq(organization_id))
            .filter(analytics::Column::PeriodEnd.lt(before_date))
            .exec(db)
            .await?;
        Ok(result.rows_affected)
    }
}
