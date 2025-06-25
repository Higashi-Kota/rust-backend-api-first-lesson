use crate::domain::organization_department_model::{self, Entity as OrganizationDepartment};
use crate::error::AppError;
use sea_orm::prelude::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use uuid::Uuid;

pub struct OrganizationDepartmentRepository;

#[allow(dead_code)]
impl OrganizationDepartmentRepository {
    pub async fn create(
        db: &DatabaseConnection,
        department: organization_department_model::ActiveModel,
    ) -> Result<organization_department_model::Model, AppError> {
        let result = department.insert(db).await?;
        Ok(result)
    }

    pub async fn find_by_id(
        db: &DatabaseConnection,
        id: Uuid,
    ) -> Result<Option<organization_department_model::Model>, AppError> {
        let result = OrganizationDepartment::find_by_id(id).one(db).await?;
        Ok(result)
    }

    pub async fn find_by_organization_id(
        db: &DatabaseConnection,
        organization_id: Uuid,
    ) -> Result<Vec<organization_department_model::Model>, AppError> {
        let result = OrganizationDepartment::find()
            .filter(organization_department_model::Column::OrganizationId.eq(organization_id))
            .filter(organization_department_model::Column::IsActive.eq(true))
            .order_by_asc(organization_department_model::Column::HierarchyLevel)
            .order_by_asc(organization_department_model::Column::Name)
            .all(db)
            .await?;
        Ok(result)
    }

    pub async fn find_hierarchy_by_organization_id(
        db: &DatabaseConnection,
        organization_id: Uuid,
    ) -> Result<Vec<organization_department_model::Model>, AppError> {
        let result = OrganizationDepartment::find()
            .filter(organization_department_model::Column::OrganizationId.eq(organization_id))
            .filter(organization_department_model::Column::IsActive.eq(true))
            .order_by_asc(organization_department_model::Column::HierarchyLevel)
            .order_by_asc(organization_department_model::Column::HierarchyPath)
            .all(db)
            .await?;
        Ok(result)
    }

    pub async fn find_children_by_parent_id(
        db: &DatabaseConnection,
        parent_id: Uuid,
    ) -> Result<Vec<organization_department_model::Model>, AppError> {
        let result = OrganizationDepartment::find()
            .filter(organization_department_model::Column::ParentDepartmentId.eq(parent_id))
            .filter(organization_department_model::Column::IsActive.eq(true))
            .order_by_asc(organization_department_model::Column::Name)
            .all(db)
            .await?;
        Ok(result)
    }

    pub async fn find_root_departments_by_organization_id(
        db: &DatabaseConnection,
        organization_id: Uuid,
    ) -> Result<Vec<organization_department_model::Model>, AppError> {
        let result = OrganizationDepartment::find()
            .filter(organization_department_model::Column::OrganizationId.eq(organization_id))
            .filter(organization_department_model::Column::ParentDepartmentId.is_null())
            .filter(organization_department_model::Column::IsActive.eq(true))
            .order_by_asc(organization_department_model::Column::Name)
            .all(db)
            .await?;
        Ok(result)
    }

    pub async fn find_by_manager_id(
        db: &DatabaseConnection,
        manager_id: Uuid,
    ) -> Result<Vec<organization_department_model::Model>, AppError> {
        let result = OrganizationDepartment::find()
            .filter(organization_department_model::Column::ManagerUserId.eq(manager_id))
            .filter(organization_department_model::Column::IsActive.eq(true))
            .order_by_asc(organization_department_model::Column::HierarchyLevel)
            .order_by_asc(organization_department_model::Column::Name)
            .all(db)
            .await?;
        Ok(result)
    }

    pub async fn find_by_hierarchy_path_prefix(
        db: &DatabaseConnection,
        organization_id: Uuid,
        hierarchy_path_prefix: &str,
    ) -> Result<Vec<organization_department_model::Model>, AppError> {
        let result = OrganizationDepartment::find()
            .filter(organization_department_model::Column::OrganizationId.eq(organization_id))
            .filter(
                organization_department_model::Column::HierarchyPath
                    .starts_with(hierarchy_path_prefix),
            )
            .filter(organization_department_model::Column::IsActive.eq(true))
            .order_by_asc(organization_department_model::Column::HierarchyLevel)
            .order_by_asc(organization_department_model::Column::HierarchyPath)
            .all(db)
            .await?;
        Ok(result)
    }

    pub async fn update_by_id(
        db: &DatabaseConnection,
        id: Uuid,
        department: organization_department_model::ActiveModel,
    ) -> Result<organization_department_model::Model, AppError> {
        let mut active_model = department;
        active_model.id = sea_orm::Set(id);
        let result = active_model.update(db).await?;
        Ok(result)
    }

    pub async fn delete_by_id(db: &DatabaseConnection, id: Uuid) -> Result<(), AppError> {
        let department = Self::find_by_id(db, id)
            .await?
            .ok_or_else(|| AppError::NotFound("Department not found".to_string()))?;

        let mut active_model: organization_department_model::ActiveModel = department.into();
        active_model.is_active = sea_orm::Set(false);
        active_model.updated_at = sea_orm::Set(chrono::Utc::now());
        active_model.update(db).await?;

        Ok(())
    }

    pub async fn count_by_organization_id(
        db: &DatabaseConnection,
        organization_id: Uuid,
    ) -> Result<u64, AppError> {
        let count = OrganizationDepartment::find()
            .filter(organization_department_model::Column::OrganizationId.eq(organization_id))
            .filter(organization_department_model::Column::IsActive.eq(true))
            .count(db)
            .await?;
        Ok(count)
    }

    pub async fn update_hierarchy_paths_batch(
        db: &DatabaseConnection,
        updates: Vec<(Uuid, String, i32)>,
    ) -> Result<(), AppError> {
        for (id, new_path, new_level) in updates {
            let mut active_model = organization_department_model::ActiveModel::new();
            active_model.id = sea_orm::Set(id);
            active_model.hierarchy_path = sea_orm::Set(new_path);
            active_model.hierarchy_level = sea_orm::Set(new_level);
            active_model.updated_at = sea_orm::Set(chrono::Utc::now());
            active_model.update(db).await?;
        }
        Ok(())
    }

    pub async fn find_by_name_and_organization(
        db: &DatabaseConnection,
        name: &str,
        organization_id: Uuid,
        parent_department_id: Option<Uuid>,
    ) -> Result<Option<organization_department_model::Model>, AppError> {
        let mut query = OrganizationDepartment::find()
            .filter(organization_department_model::Column::Name.eq(name))
            .filter(organization_department_model::Column::OrganizationId.eq(organization_id))
            .filter(organization_department_model::Column::IsActive.eq(true));

        match parent_department_id {
            Some(parent_id) => {
                query = query.filter(
                    organization_department_model::Column::ParentDepartmentId.eq(parent_id),
                );
            }
            None => {
                query = query
                    .filter(organization_department_model::Column::ParentDepartmentId.is_null());
            }
        }

        let result = query.one(db).await?;
        Ok(result)
    }

    pub async fn exists_circular_dependency(
        db: &DatabaseConnection,
        department_id: Uuid,
        potential_parent_id: Uuid,
    ) -> Result<bool, AppError> {
        // Check if potential_parent_id is a descendant of department_id
        let department = Self::find_by_id(db, department_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Department not found".to_string()))?;

        let descendants = Self::find_by_hierarchy_path_prefix(
            db,
            department.organization_id,
            &department.hierarchy_path,
        )
        .await?;

        for descendant in descendants {
            if descendant.id == potential_parent_id {
                return Ok(true);
            }
        }

        Ok(false)
    }
}
