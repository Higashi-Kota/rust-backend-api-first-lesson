use crate::domain::bulk_operation_history_model::{
    self, BulkOperationErrorDetails, BulkOperationType, Entity as BulkOperationHistory,
};
use crate::error::{AppError, AppResult};
use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter,
    QueryOrder, QuerySelect, Set,
};
use uuid::Uuid;

pub struct BulkOperationHistoryRepository {
    db: DatabaseConnection,
}

impl BulkOperationHistoryRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn create(
        &self,
        operation_type: BulkOperationType,
        performed_by: Uuid,
        affected_count: i32,
    ) -> AppResult<bulk_operation_history_model::Model> {
        let model =
            bulk_operation_history_model::Model::new(operation_type, performed_by, affected_count);

        let active_model = bulk_operation_history_model::ActiveModel {
            id: Set(model.id),
            operation_type: Set(model.operation_type.clone()),
            performed_by: Set(model.performed_by),
            affected_count: Set(model.affected_count),
            status: Set(model.status.clone()),
            error_details: Set(model.error_details.clone()),
            created_at: Set(model.created_at),
            completed_at: Set(model.completed_at),
        };

        let result = active_model.insert(&self.db).await?;
        Ok(result)
    }

    pub async fn start_operation(
        &self,
        operation_id: Uuid,
    ) -> AppResult<bulk_operation_history_model::Model> {
        let mut operation = self
            .get_by_id(operation_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Bulk operation not found".to_string()))?;

        operation.start();

        let mut active_model = operation.clone().into_active_model();
        active_model.status = Set(operation.status.clone());

        let result = active_model.update(&self.db).await?;
        Ok(result)
    }

    pub async fn complete_operation(
        &self,
        operation_id: Uuid,
    ) -> AppResult<bulk_operation_history_model::Model> {
        let mut operation = self
            .get_by_id(operation_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Bulk operation not found".to_string()))?;

        operation.complete();

        let mut active_model = operation.clone().into_active_model();
        active_model.status = Set(operation.status.clone());
        active_model.completed_at = Set(operation.completed_at);

        let result = active_model.update(&self.db).await?;
        Ok(result)
    }

    pub async fn fail_operation(
        &self,
        operation_id: Uuid,
        error_details: Option<BulkOperationErrorDetails>,
    ) -> AppResult<bulk_operation_history_model::Model> {
        let mut operation = self
            .get_by_id(operation_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Bulk operation not found".to_string()))?;

        operation.fail(error_details);

        let mut active_model = operation.clone().into_active_model();
        active_model.status = Set(operation.status.clone());
        active_model.completed_at = Set(operation.completed_at);
        active_model.error_details = Set(operation.error_details.clone());

        let result = active_model.update(&self.db).await?;
        Ok(result)
    }

    pub async fn partially_complete_operation(
        &self,
        operation_id: Uuid,
        error_details: BulkOperationErrorDetails,
    ) -> AppResult<bulk_operation_history_model::Model> {
        let mut operation = self
            .get_by_id(operation_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Bulk operation not found".to_string()))?;

        operation.partially_complete(error_details);

        let mut active_model = operation.clone().into_active_model();
        active_model.status = Set(operation.status.clone());
        active_model.completed_at = Set(operation.completed_at);
        active_model.error_details = Set(operation.error_details.clone());

        let result = active_model.update(&self.db).await?;
        Ok(result)
    }

    pub async fn get_by_id(
        &self,
        id: Uuid,
    ) -> AppResult<Option<bulk_operation_history_model::Model>> {
        let operation = BulkOperationHistory::find_by_id(id).one(&self.db).await?;

        Ok(operation)
    }

    pub async fn get_by_user(
        &self,
        user_id: Uuid,
        limit: u64,
    ) -> AppResult<Vec<bulk_operation_history_model::Model>> {
        let operations = BulkOperationHistory::find()
            .filter(bulk_operation_history_model::Column::PerformedBy.eq(user_id))
            .order_by_desc(bulk_operation_history_model::Column::CreatedAt)
            .limit(limit)
            .all(&self.db)
            .await?;

        Ok(operations)
    }

    pub async fn get_recent(
        &self,
        limit: u64,
    ) -> AppResult<Vec<bulk_operation_history_model::Model>> {
        let operations = BulkOperationHistory::find()
            .order_by_desc(bulk_operation_history_model::Column::CreatedAt)
            .limit(limit)
            .all(&self.db)
            .await?;

        Ok(operations)
    }

    pub async fn get_by_date_range(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> AppResult<Vec<bulk_operation_history_model::Model>> {
        let operations = BulkOperationHistory::find()
            .filter(bulk_operation_history_model::Column::CreatedAt.gte(start_date))
            .filter(bulk_operation_history_model::Column::CreatedAt.lt(end_date))
            .order_by_desc(bulk_operation_history_model::Column::CreatedAt)
            .all(&self.db)
            .await?;

        Ok(operations)
    }

    pub async fn delete_old_histories(&self, before_date: DateTime<Utc>) -> AppResult<u64> {
        let result = BulkOperationHistory::delete_many()
            .filter(bulk_operation_history_model::Column::CreatedAt.lt(before_date))
            .exec(&self.db)
            .await?;

        Ok(result.rows_affected)
    }
}
