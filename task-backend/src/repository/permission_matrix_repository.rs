use crate::domain::permission_matrix_model::{self, Entity as PermissionMatrix, EntityType};
use crate::error::AppError;
use sea_orm::prelude::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use uuid::Uuid;

pub struct PermissionMatrixRepository;

#[allow(dead_code)]
impl PermissionMatrixRepository {
    pub async fn create(
        db: &DatabaseConnection,
        matrix: permission_matrix_model::ActiveModel,
    ) -> Result<permission_matrix_model::Model, AppError> {
        let result = matrix.insert(db).await?;
        Ok(result)
    }

    pub async fn find_by_id(
        db: &DatabaseConnection,
        id: Uuid,
    ) -> Result<Option<permission_matrix_model::Model>, AppError> {
        let result = PermissionMatrix::find_by_id(id).one(db).await?;
        Ok(result)
    }

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

    pub async fn find_by_entity_type(
        db: &DatabaseConnection,
        entity_type: EntityType,
    ) -> Result<Vec<permission_matrix_model::Model>, AppError> {
        let result = PermissionMatrix::find()
            .filter(permission_matrix_model::Column::EntityType.eq(entity_type.to_string()))
            .filter(permission_matrix_model::Column::IsActive.eq(true))
            .order_by_desc(permission_matrix_model::Column::UpdatedAt)
            .all(db)
            .await?;
        Ok(result)
    }

    pub async fn find_by_updated_by(
        db: &DatabaseConnection,
        updated_by: Uuid,
    ) -> Result<Vec<permission_matrix_model::Model>, AppError> {
        let result = PermissionMatrix::find()
            .filter(permission_matrix_model::Column::UpdatedBy.eq(updated_by))
            .filter(permission_matrix_model::Column::IsActive.eq(true))
            .order_by_desc(permission_matrix_model::Column::UpdatedAt)
            .all(db)
            .await?;
        Ok(result)
    }

    pub async fn find_organization_matrices(
        db: &DatabaseConnection,
        organization_ids: Vec<Uuid>,
    ) -> Result<Vec<permission_matrix_model::Model>, AppError> {
        let result = PermissionMatrix::find()
            .filter(
                permission_matrix_model::Column::EntityType
                    .eq(EntityType::Organization.to_string()),
            )
            .filter(permission_matrix_model::Column::EntityId.is_in(organization_ids))
            .filter(permission_matrix_model::Column::IsActive.eq(true))
            .order_by_desc(permission_matrix_model::Column::UpdatedAt)
            .all(db)
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

    pub async fn find_team_matrices(
        db: &DatabaseConnection,
        team_ids: Vec<Uuid>,
    ) -> Result<Vec<permission_matrix_model::Model>, AppError> {
        let result = PermissionMatrix::find()
            .filter(permission_matrix_model::Column::EntityType.eq(EntityType::Team.to_string()))
            .filter(permission_matrix_model::Column::EntityId.is_in(team_ids))
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

    pub async fn update_by_id(
        db: &DatabaseConnection,
        id: Uuid,
        matrix: permission_matrix_model::ActiveModel,
    ) -> Result<permission_matrix_model::Model, AppError> {
        let mut active_model = matrix;
        active_model.id = sea_orm::Set(id);
        active_model.updated_at = sea_orm::Set(chrono::Utc::now());
        let result = active_model.update(db).await?;
        Ok(result)
    }

    pub async fn deactivate_by_entity(
        db: &DatabaseConnection,
        entity_type: EntityType,
        entity_id: Uuid,
    ) -> Result<(), AppError> {
        if let Some(matrix) = Self::find_by_entity(db, entity_type, entity_id).await? {
            let mut active_model: permission_matrix_model::ActiveModel = matrix.into();
            active_model.is_active = sea_orm::Set(false);
            active_model.updated_at = sea_orm::Set(chrono::Utc::now());
            active_model.update(db).await?;
        }
        Ok(())
    }

    pub async fn delete_by_id(db: &DatabaseConnection, id: Uuid) -> Result<(), AppError> {
        let matrix = Self::find_by_id(db, id)
            .await?
            .ok_or_else(|| AppError::NotFound("Permission matrix not found".to_string()))?;

        let mut active_model: permission_matrix_model::ActiveModel = matrix.into();
        active_model.is_active = sea_orm::Set(false);
        active_model.updated_at = sea_orm::Set(chrono::Utc::now());
        active_model.update(db).await?;

        Ok(())
    }

    pub async fn count_by_entity_type(
        db: &DatabaseConnection,
        entity_type: EntityType,
    ) -> Result<u64, AppError> {
        let count = PermissionMatrix::find()
            .filter(permission_matrix_model::Column::EntityType.eq(entity_type.to_string()))
            .filter(permission_matrix_model::Column::IsActive.eq(true))
            .count(db)
            .await?;
        Ok(count)
    }

    pub async fn find_latest_by_entity_type(
        db: &DatabaseConnection,
        entity_type: EntityType,
        limit: u64,
    ) -> Result<Vec<permission_matrix_model::Model>, AppError> {
        let result = PermissionMatrix::find()
            .filter(permission_matrix_model::Column::EntityType.eq(entity_type.to_string()))
            .filter(permission_matrix_model::Column::IsActive.eq(true))
            .order_by_desc(permission_matrix_model::Column::UpdatedAt)
            .limit(limit)
            .all(db)
            .await?;
        Ok(result)
    }

    pub async fn find_matrices_by_version(
        db: &DatabaseConnection,
        version: &str,
    ) -> Result<Vec<permission_matrix_model::Model>, AppError> {
        let result = PermissionMatrix::find()
            .filter(permission_matrix_model::Column::MatrixVersion.eq(version))
            .filter(permission_matrix_model::Column::IsActive.eq(true))
            .order_by_desc(permission_matrix_model::Column::UpdatedAt)
            .all(db)
            .await?;
        Ok(result)
    }

    pub async fn entity_has_matrix(
        db: &DatabaseConnection,
        entity_type: EntityType,
        entity_id: Uuid,
    ) -> Result<bool, AppError> {
        let count = PermissionMatrix::find()
            .filter(permission_matrix_model::Column::EntityType.eq(entity_type.to_string()))
            .filter(permission_matrix_model::Column::EntityId.eq(entity_id))
            .filter(permission_matrix_model::Column::IsActive.eq(true))
            .count(db)
            .await?;
        Ok(count > 0)
    }
}
