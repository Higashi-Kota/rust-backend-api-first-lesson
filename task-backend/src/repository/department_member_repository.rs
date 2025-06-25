use crate::domain::department_member_model::{self, DepartmentRole, Entity as DepartmentMember};
use crate::error::AppError;
use sea_orm::prelude::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use uuid::Uuid;

pub struct DepartmentMemberRepository;

#[allow(dead_code)]
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

    pub async fn find_by_department_and_role(
        db: &DatabaseConnection,
        department_id: Uuid,
        role: DepartmentRole,
    ) -> Result<Vec<department_member_model::Model>, AppError> {
        let result = DepartmentMember::find()
            .filter(department_member_model::Column::DepartmentId.eq(department_id))
            .filter(department_member_model::Column::Role.eq(role.to_string()))
            .filter(department_member_model::Column::IsActive.eq(true))
            .order_by_asc(department_member_model::Column::JoinedAt)
            .all(db)
            .await?;
        Ok(result)
    }

    pub async fn find_managers_by_department_ids(
        db: &DatabaseConnection,
        department_ids: Vec<Uuid>,
    ) -> Result<Vec<department_member_model::Model>, AppError> {
        let result = DepartmentMember::find()
            .filter(department_member_model::Column::DepartmentId.is_in(department_ids))
            .filter(department_member_model::Column::Role.eq(DepartmentRole::Manager.to_string()))
            .filter(department_member_model::Column::IsActive.eq(true))
            .order_by_asc(department_member_model::Column::DepartmentId)
            .all(db)
            .await?;
        Ok(result)
    }

    pub async fn find_by_added_by(
        db: &DatabaseConnection,
        added_by: Uuid,
        limit: Option<u64>,
    ) -> Result<Vec<department_member_model::Model>, AppError> {
        let mut query = DepartmentMember::find()
            .filter(department_member_model::Column::AddedBy.eq(added_by))
            .filter(department_member_model::Column::IsActive.eq(true))
            .order_by_desc(department_member_model::Column::JoinedAt);

        if let Some(limit_val) = limit {
            query = query.limit(limit_val);
        }

        let result = query.all(db).await?;
        Ok(result)
    }

    pub async fn update_by_id(
        db: &DatabaseConnection,
        id: Uuid,
        member: department_member_model::ActiveModel,
    ) -> Result<department_member_model::Model, AppError> {
        let mut active_model = member;
        active_model.id = sea_orm::Set(id);
        active_model.updated_at = sea_orm::Set(chrono::Utc::now());
        let result = active_model.update(db).await?;
        Ok(result)
    }

    pub async fn update_role_by_department_and_user(
        db: &DatabaseConnection,
        department_id: Uuid,
        user_id: Uuid,
        new_role: DepartmentRole,
    ) -> Result<department_member_model::Model, AppError> {
        let member = Self::find_by_department_and_user(db, department_id, user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Department member not found".to_string()))?;

        let mut active_model: department_member_model::ActiveModel = member.into();
        active_model.role = sea_orm::Set(new_role.to_string());
        active_model.updated_at = sea_orm::Set(chrono::Utc::now());
        let result = active_model.update(db).await?;
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

    pub async fn activate_by_department_and_user(
        db: &DatabaseConnection,
        department_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<department_member_model::Model>, AppError> {
        // Find both active and inactive memberships
        let member = DepartmentMember::find()
            .filter(department_member_model::Column::DepartmentId.eq(department_id))
            .filter(department_member_model::Column::UserId.eq(user_id))
            .one(db)
            .await?;

        if let Some(member) = member {
            let mut active_model: department_member_model::ActiveModel = member.into();
            active_model.is_active = sea_orm::Set(true);
            active_model.updated_at = sea_orm::Set(chrono::Utc::now());
            let result = active_model.update(db).await?;
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }

    pub async fn count_by_department_id(
        db: &DatabaseConnection,
        department_id: Uuid,
    ) -> Result<u64, AppError> {
        let count = DepartmentMember::find()
            .filter(department_member_model::Column::DepartmentId.eq(department_id))
            .filter(department_member_model::Column::IsActive.eq(true))
            .count(db)
            .await?;
        Ok(count)
    }

    pub async fn count_by_user_id(db: &DatabaseConnection, user_id: Uuid) -> Result<u64, AppError> {
        let count = DepartmentMember::find()
            .filter(department_member_model::Column::UserId.eq(user_id))
            .filter(department_member_model::Column::IsActive.eq(true))
            .count(db)
            .await?;
        Ok(count)
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

    pub async fn user_has_role_in_department(
        db: &DatabaseConnection,
        user_id: Uuid,
        department_id: Uuid,
        role: DepartmentRole,
    ) -> Result<bool, AppError> {
        let count = DepartmentMember::find()
            .filter(department_member_model::Column::UserId.eq(user_id))
            .filter(department_member_model::Column::DepartmentId.eq(department_id))
            .filter(department_member_model::Column::Role.eq(role.to_string()))
            .filter(department_member_model::Column::IsActive.eq(true))
            .count(db)
            .await?;
        Ok(count > 0)
    }

    pub async fn get_user_role_in_department(
        db: &DatabaseConnection,
        user_id: Uuid,
        department_id: Uuid,
    ) -> Result<Option<DepartmentRole>, AppError> {
        let member = Self::find_by_department_and_user(db, department_id, user_id).await?;
        Ok(member.map(|m| m.get_role()))
    }

    pub async fn find_all_user_departments_with_roles(
        db: &DatabaseConnection,
        user_id: Uuid,
    ) -> Result<Vec<(Uuid, DepartmentRole)>, AppError> {
        let members = Self::find_by_user_id(db, user_id).await?;
        let department_roles: Vec<(Uuid, DepartmentRole)> = members
            .into_iter()
            .map(|m| (m.department_id, m.get_role()))
            .collect();
        Ok(department_roles)
    }
}
