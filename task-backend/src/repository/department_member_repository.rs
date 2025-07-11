use crate::domain::department_member_model::{self, Entity as DepartmentMember};
use crate::error::AppError;
use sea_orm::prelude::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use uuid::Uuid;

#[allow(dead_code)] // Will be used for organization hierarchy features
pub struct DepartmentMemberRepository;

#[allow(dead_code)] // TODO: Will be used when department member features are integrated
impl DepartmentMemberRepository {
    pub async fn create(
        db: &DatabaseConnection,
        member: department_member_model::ActiveModel,
    ) -> Result<department_member_model::Model, AppError> {
        let result = member.insert(db).await?;
        Ok(result)
    }

    pub async fn find_by_id(
        db: &DatabaseConnection,
        id: Uuid,
    ) -> Result<Option<department_member_model::Model>, AppError> {
        let result = DepartmentMember::find_by_id(id).one(db).await?;
        Ok(result)
    }

    pub async fn find_by_department_id(
        db: &DatabaseConnection,
        department_id: Uuid,
    ) -> Result<Vec<department_member_model::Model>, AppError> {
        let result = DepartmentMember::find()
            .filter(department_member_model::Column::DepartmentId.eq(department_id))
            .filter(department_member_model::Column::IsActive.eq(true))
            .order_by_asc(department_member_model::Column::Role)
            .order_by_asc(department_member_model::Column::JoinedAt)
            .all(db)
            .await?;
        Ok(result)
    }

    pub async fn find_by_user_id(
        db: &DatabaseConnection,
        user_id: Uuid,
    ) -> Result<Vec<department_member_model::Model>, AppError> {
        let result = DepartmentMember::find()
            .filter(department_member_model::Column::UserId.eq(user_id))
            .filter(department_member_model::Column::IsActive.eq(true))
            .order_by_desc(department_member_model::Column::JoinedAt)
            .all(db)
            .await?;
        Ok(result)
    }

    pub async fn find_by_department_and_user(
        db: &DatabaseConnection,
        department_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<department_member_model::Model>, AppError> {
        let result = DepartmentMember::find()
            .filter(department_member_model::Column::DepartmentId.eq(department_id))
            .filter(department_member_model::Column::UserId.eq(user_id))
            .filter(department_member_model::Column::IsActive.eq(true))
            .one(db)
            .await?;
        Ok(result)
    }

    pub async fn deactivate_by_id(db: &DatabaseConnection, id: Uuid) -> Result<(), AppError> {
        let member = Self::find_by_id(db, id)
            .await?
            .ok_or_else(|| AppError::NotFound("Department member not found".to_string()))?;

        let mut active_model: department_member_model::ActiveModel = member.into();
        active_model.is_active = sea_orm::Set(false);
        active_model.updated_at = sea_orm::Set(chrono::Utc::now());
        active_model.update(db).await?;

        Ok(())
    }

    pub async fn deactivate_by_department_and_user(
        db: &DatabaseConnection,
        department_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        if let Some(member) = Self::find_by_department_and_user(db, department_id, user_id).await? {
            let mut active_model: department_member_model::ActiveModel = member.into();
            active_model.is_active = sea_orm::Set(false);
            active_model.updated_at = sea_orm::Set(chrono::Utc::now());
            active_model.update(db).await?;
        }
        Ok(())
    }

    pub async fn is_member_of_department(
        db: &DatabaseConnection,
        user_id: Uuid,
        department_id: Uuid,
    ) -> Result<bool, AppError> {
        let count = DepartmentMember::find()
            .filter(department_member_model::Column::UserId.eq(user_id))
            .filter(department_member_model::Column::DepartmentId.eq(department_id))
            .filter(department_member_model::Column::IsActive.eq(true))
            .count(db)
            .await?;
        Ok(count > 0)
    }
}
