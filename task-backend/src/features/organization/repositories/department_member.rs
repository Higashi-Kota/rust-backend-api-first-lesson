#![allow(dead_code)] // Repository methods for department members

use super::super::models::department_member::{self, Entity as DepartmentMember};
use crate::error::AppError;
use sea_orm::prelude::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use uuid::Uuid;

// TODO: Phase 19で本来の使用箇所が移行されたら#[allow(dead_code)]を削除
#[allow(dead_code)]
pub struct DepartmentMemberRepository;

impl DepartmentMemberRepository {
    pub async fn create(
        db: &DatabaseConnection,
        member: department_member::ActiveModel,
    ) -> Result<department_member::Model, AppError> {
        let result = member.insert(db).await?;
        Ok(result)
    }

    pub async fn find_by_id(
        db: &DatabaseConnection,
        id: Uuid,
    ) -> Result<Option<department_member::Model>, AppError> {
        let result = DepartmentMember::find_by_id(id).one(db).await?;
        Ok(result)
    }

    pub async fn find_by_department_id(
        db: &DatabaseConnection,
        department_id: Uuid,
    ) -> Result<Vec<department_member::Model>, AppError> {
        let result = DepartmentMember::find()
            .filter(department_member::Column::DepartmentId.eq(department_id))
            .filter(department_member::Column::IsActive.eq(true))
            .order_by_asc(department_member::Column::Role)
            .order_by_asc(department_member::Column::JoinedAt)
            .all(db)
            .await?;
        Ok(result)
    }

    pub async fn find_by_user_id(
        db: &DatabaseConnection,
        user_id: Uuid,
    ) -> Result<Vec<department_member::Model>, AppError> {
        let result = DepartmentMember::find()
            .filter(department_member::Column::UserId.eq(user_id))
            .filter(department_member::Column::IsActive.eq(true))
            .order_by_desc(department_member::Column::JoinedAt)
            .all(db)
            .await?;
        Ok(result)
    }

    pub async fn find_by_department_and_user(
        db: &DatabaseConnection,
        department_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<department_member::Model>, AppError> {
        let result = DepartmentMember::find()
            .filter(department_member::Column::DepartmentId.eq(department_id))
            .filter(department_member::Column::UserId.eq(user_id))
            .filter(department_member::Column::IsActive.eq(true))
            .one(db)
            .await?;
        Ok(result)
    }

    pub async fn deactivate_by_id(db: &DatabaseConnection, id: Uuid) -> Result<(), AppError> {
        let member = Self::find_by_id(db, id)
            .await?
            .ok_or_else(|| AppError::NotFound("Department member not found".to_string()))?;

        let mut active_model: department_member::ActiveModel = member.into();
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
            let mut active_model: department_member::ActiveModel = member.into();
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
            .filter(department_member::Column::UserId.eq(user_id))
            .filter(department_member::Column::DepartmentId.eq(department_id))
            .filter(department_member::Column::IsActive.eq(true))
            .count(db)
            .await?;
        Ok(count > 0)
    }

    pub async fn count_by_department(
        db: &DatabaseConnection,
        department_id: Uuid,
    ) -> Result<u64, AppError> {
        let count = DepartmentMember::find()
            .filter(department_member::Column::DepartmentId.eq(department_id))
            .filter(department_member::Column::IsActive.eq(true))
            .count(db)
            .await?;
        Ok(count)
    }

    pub async fn update_role(
        db: &DatabaseConnection,
        id: Uuid,
        new_role: &str,
    ) -> Result<department_member::Model, AppError> {
        let member = Self::find_by_id(db, id)
            .await?
            .ok_or_else(|| AppError::NotFound("Department member not found".to_string()))?;

        let mut active_model: department_member::ActiveModel = member.into();
        active_model.role = sea_orm::Set(new_role.to_string());
        active_model.updated_at = sea_orm::Set(chrono::Utc::now());
        let result = active_model.update(db).await?;
        Ok(result)
    }

    pub async fn find_managers_by_department(
        db: &DatabaseConnection,
        department_id: Uuid,
    ) -> Result<Vec<department_member::Model>, AppError> {
        let result = DepartmentMember::find()
            .filter(department_member::Column::DepartmentId.eq(department_id))
            .filter(department_member::Column::Role.eq("manager"))
            .filter(department_member::Column::IsActive.eq(true))
            .all(db)
            .await?;
        Ok(result)
    }

    pub async fn find_users_departments_with_role(
        db: &DatabaseConnection,
        user_id: Uuid,
        roles: Vec<&str>,
    ) -> Result<Vec<department_member::Model>, AppError> {
        let result = DepartmentMember::find()
            .filter(department_member::Column::UserId.eq(user_id))
            .filter(department_member::Column::Role.is_in(roles))
            .filter(department_member::Column::IsActive.eq(true))
            .all(db)
            .await?;
        Ok(result)
    }
}
