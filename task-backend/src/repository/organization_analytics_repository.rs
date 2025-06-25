use crate::domain::organization_analytics_model::{
    self, AnalyticsType, Entity as OrganizationAnalytics, Period,
};
use crate::error::AppError;
use chrono::{DateTime, Utc};
use sea_orm::prelude::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use uuid::Uuid;

pub struct OrganizationAnalyticsRepository;

#[allow(dead_code)]
impl OrganizationAnalyticsRepository {
    pub async fn create(
        db: &DatabaseConnection,
        analytics: organization_analytics_model::ActiveModel,
    ) -> Result<organization_analytics_model::Model, AppError> {
        let result = analytics.insert(db).await?;
        Ok(result)
    }

    pub async fn find_by_id(
        db: &DatabaseConnection,
        id: Uuid,
    ) -> Result<Option<organization_analytics_model::Model>, AppError> {
        let result = OrganizationAnalytics::find_by_id(id).one(db).await?;
        Ok(result)
    }

    pub async fn find_by_organization_id(
        db: &DatabaseConnection,
        organization_id: Uuid,
        limit: Option<u64>,
    ) -> Result<Vec<organization_analytics_model::Model>, AppError> {
        let mut query = OrganizationAnalytics::find()
            .filter(organization_analytics_model::Column::OrganizationId.eq(organization_id))
            .order_by_desc(organization_analytics_model::Column::PeriodEnd);

        if let Some(limit_val) = limit {
            query = query.limit(limit_val);
        }

        let result = query.all(db).await?;
        Ok(result)
    }

    pub async fn find_by_department_id(
        db: &DatabaseConnection,
        department_id: Uuid,
        limit: Option<u64>,
    ) -> Result<Vec<organization_analytics_model::Model>, AppError> {
        let mut query = OrganizationAnalytics::find()
            .filter(organization_analytics_model::Column::DepartmentId.eq(department_id))
            .order_by_desc(organization_analytics_model::Column::PeriodEnd);

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
    ) -> Result<Vec<organization_analytics_model::Model>, AppError> {
        let mut query = OrganizationAnalytics::find()
            .filter(organization_analytics_model::Column::OrganizationId.eq(organization_id))
            .filter(
                organization_analytics_model::Column::AnalyticsType.eq(analytics_type.to_string()),
            )
            .order_by_desc(organization_analytics_model::Column::PeriodEnd);

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
    ) -> Result<Vec<organization_analytics_model::Model>, AppError> {
        let result = OrganizationAnalytics::find()
            .filter(organization_analytics_model::Column::OrganizationId.eq(organization_id))
            .filter(organization_analytics_model::Column::Period.eq(period.to_string()))
            .filter(organization_analytics_model::Column::PeriodStart.gte(start_date))
            .filter(organization_analytics_model::Column::PeriodEnd.lte(end_date))
            .order_by_desc(organization_analytics_model::Column::PeriodEnd)
            .all(db)
            .await?;
        Ok(result)
    }

    pub async fn find_by_department_and_type(
        db: &DatabaseConnection,
        department_id: Uuid,
        analytics_type: AnalyticsType,
        limit: Option<u64>,
    ) -> Result<Vec<organization_analytics_model::Model>, AppError> {
        let mut query = OrganizationAnalytics::find()
            .filter(organization_analytics_model::Column::DepartmentId.eq(department_id))
            .filter(
                organization_analytics_model::Column::AnalyticsType.eq(analytics_type.to_string()),
            )
            .order_by_desc(organization_analytics_model::Column::PeriodEnd);

        if let Some(limit_val) = limit {
            query = query.limit(limit_val);
        }

        let result = query.all(db).await?;
        Ok(result)
    }

    pub async fn find_by_metric_name(
        db: &DatabaseConnection,
        organization_id: Uuid,
        metric_name: &str,
        limit: Option<u64>,
    ) -> Result<Vec<organization_analytics_model::Model>, AppError> {
        let mut query = OrganizationAnalytics::find()
            .filter(organization_analytics_model::Column::OrganizationId.eq(organization_id))
            .filter(organization_analytics_model::Column::MetricName.eq(metric_name))
            .order_by_desc(organization_analytics_model::Column::PeriodEnd);

        if let Some(limit_val) = limit {
            query = query.limit(limit_val);
        }

        let result = query.all(db).await?;
        Ok(result)
    }

    pub async fn find_latest_by_organization(
        db: &DatabaseConnection,
        organization_id: Uuid,
        limit: u64,
    ) -> Result<Vec<organization_analytics_model::Model>, AppError> {
        let result = OrganizationAnalytics::find()
            .filter(organization_analytics_model::Column::OrganizationId.eq(organization_id))
            .order_by_desc(organization_analytics_model::Column::PeriodEnd)
            .limit(limit)
            .all(db)
            .await?;
        Ok(result)
    }

    pub async fn update_by_id(
        db: &DatabaseConnection,
        id: Uuid,
        analytics: organization_analytics_model::ActiveModel,
    ) -> Result<organization_analytics_model::Model, AppError> {
        let mut active_model = analytics;
        active_model.id = sea_orm::Set(id);
        active_model.updated_at = sea_orm::Set(Utc::now());
        let result = active_model.update(db).await?;
        Ok(result)
    }

    pub async fn delete_by_id(db: &DatabaseConnection, id: Uuid) -> Result<(), AppError> {
        OrganizationAnalytics::delete_by_id(id).exec(db).await?;
        Ok(())
    }

    pub async fn delete_by_organization_id(
        db: &DatabaseConnection,
        organization_id: Uuid,
    ) -> Result<u64, AppError> {
        let result = OrganizationAnalytics::delete_many()
            .filter(organization_analytics_model::Column::OrganizationId.eq(organization_id))
            .exec(db)
            .await?;
        Ok(result.rows_affected)
    }

    pub async fn delete_by_department_id(
        db: &DatabaseConnection,
        department_id: Uuid,
    ) -> Result<u64, AppError> {
        let result = OrganizationAnalytics::delete_many()
            .filter(organization_analytics_model::Column::DepartmentId.eq(department_id))
            .exec(db)
            .await?;
        Ok(result.rows_affected)
    }

    pub async fn delete_old_analytics(
        db: &DatabaseConnection,
        before_date: DateTime<Utc>,
    ) -> Result<u64, AppError> {
        let result = OrganizationAnalytics::delete_many()
            .filter(organization_analytics_model::Column::PeriodEnd.lt(before_date))
            .exec(db)
            .await?;
        Ok(result.rows_affected)
    }

    pub async fn count_by_organization_id(
        db: &DatabaseConnection,
        organization_id: Uuid,
    ) -> Result<u64, AppError> {
        let count = OrganizationAnalytics::find()
            .filter(organization_analytics_model::Column::OrganizationId.eq(organization_id))
            .count(db)
            .await?;
        Ok(count)
    }

    pub async fn count_by_department_id(
        db: &DatabaseConnection,
        department_id: Uuid,
    ) -> Result<u64, AppError> {
        let count = OrganizationAnalytics::find()
            .filter(organization_analytics_model::Column::DepartmentId.eq(department_id))
            .count(db)
            .await?;
        Ok(count)
    }

    pub async fn find_analytics_summary(
        db: &DatabaseConnection,
        organization_id: Uuid,
        period: Period,
        limit: u64,
    ) -> Result<Vec<organization_analytics_model::Model>, AppError> {
        let result = OrganizationAnalytics::find()
            .filter(organization_analytics_model::Column::OrganizationId.eq(organization_id))
            .filter(organization_analytics_model::Column::Period.eq(period.to_string()))
            .order_by_desc(organization_analytics_model::Column::PeriodEnd)
            .order_by_asc(organization_analytics_model::Column::AnalyticsType)
            .limit(limit)
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
            .filter(organization_analytics_model::Column::OrganizationId.eq(organization_id))
            .filter(
                organization_analytics_model::Column::AnalyticsType.eq(analytics_type.to_string()),
            )
            .filter(organization_analytics_model::Column::MetricName.eq(metric_name))
            .filter(organization_analytics_model::Column::PeriodStart.eq(period_start))
            .filter(organization_analytics_model::Column::PeriodEnd.eq(period_end));

        if let Some(dept_id) = department_id {
            query = query.filter(organization_analytics_model::Column::DepartmentId.eq(dept_id));
        } else {
            query = query.filter(organization_analytics_model::Column::DepartmentId.is_null());
        }

        let count = query.count(db).await?;
        Ok(count > 0)
    }
}
