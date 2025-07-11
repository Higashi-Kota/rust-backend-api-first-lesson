#![allow(dead_code)] // Repository methods for permission matrix

use crate::domain::permission_matrix_model::{self, Entity as PermissionMatrix, EntityType};
use crate::error::AppError;
use sea_orm::prelude::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use uuid::Uuid;

#[allow(dead_code)] // Will be used for permission matrix features
pub struct PermissionMatrixRepository;

impl PermissionMatrixRepository {
    pub async fn find_by_entity(
        db: &DatabaseConnection,
        entity_type: EntityType,
        entity_id: Uuid,
    ) -> Result<Option<permission_matrix_model::Model>, AppError> {
        let result = PermissionMatrix::find()
            .filter(permission_matrix_model::Column::EntityType.eq(entity_type.to_string()))
            .filter(permission_matrix_model::Column::EntityId.eq(entity_id))
            .filter(permission_matrix_model::Column::IsActive.eq(true))
            .one(db)
            .await?;
        Ok(result)
    }

    pub async fn find_department_matrices(
        db: &DatabaseConnection,
        department_ids: Vec<Uuid>,
    ) -> Result<Vec<permission_matrix_model::Model>, AppError> {
        let result = PermissionMatrix::find()
            .filter(
                permission_matrix_model::Column::EntityType.eq(EntityType::Department.to_string()),
            )
            .filter(permission_matrix_model::Column::EntityId.is_in(department_ids))
            .filter(permission_matrix_model::Column::IsActive.eq(true))
            .order_by_desc(permission_matrix_model::Column::UpdatedAt)
            .all(db)
            .await?;
        Ok(result)
    }

    pub async fn update_by_entity(
        db: &DatabaseConnection,
        entity_type: EntityType,
        entity_id: Uuid,
        matrix: permission_matrix_model::ActiveModel,
    ) -> Result<permission_matrix_model::Model, AppError> {
        // First deactivate any existing matrix for this entity
        if let Some(existing) = Self::find_by_entity(db, entity_type.clone(), entity_id).await? {
            let mut deactivate_model: permission_matrix_model::ActiveModel = existing.into();
            deactivate_model.is_active = sea_orm::Set(false);
            deactivate_model.updated_at = sea_orm::Set(chrono::Utc::now());
            deactivate_model.update(db).await?;
        }

        // Create new matrix
        let result = matrix.insert(db).await?;
        Ok(result)
    }
}
