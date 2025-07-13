use super::super::models::department::{self, Entity as OrganizationDepartment};
use crate::error::AppError;
use sea_orm::prelude::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use uuid::Uuid;

pub struct DepartmentRepository;

impl DepartmentRepository {
    pub async fn create(
        db: &DatabaseConnection,
        department: department::ActiveModel,
    ) -> Result<department::Model, AppError> {
        let result = department.insert(db).await?;
        Ok(result)
    }

    pub async fn find_by_id(
        db: &DatabaseConnection,
        id: Uuid,
    ) -> Result<Option<department::Model>, AppError> {
        let result = OrganizationDepartment::find_by_id(id).one(db).await?;
        Ok(result)
    }

    pub async fn find_hierarchy_by_organization_id(
        db: &DatabaseConnection,
        organization_id: Uuid,
    ) -> Result<Vec<department::Model>, AppError> {
        let result = OrganizationDepartment::find()
            .filter(department::Column::OrganizationId.eq(organization_id))
            .filter(department::Column::IsActive.eq(true))
            .order_by_asc(department::Column::HierarchyLevel)
            .order_by_asc(department::Column::HierarchyPath)
            .all(db)
            .await?;
        Ok(result)
    }

    pub async fn find_children_by_parent_id(
        db: &DatabaseConnection,
        parent_id: Uuid,
    ) -> Result<Vec<department::Model>, AppError> {
        let result = OrganizationDepartment::find()
            .filter(department::Column::ParentDepartmentId.eq(parent_id))
            .filter(department::Column::IsActive.eq(true))
            .order_by_asc(department::Column::Name)
            .all(db)
            .await?;
        Ok(result)
    }

    pub async fn find_by_hierarchy_path_prefix(
        db: &DatabaseConnection,
        organization_id: Uuid,
        hierarchy_path_prefix: &str,
    ) -> Result<Vec<department::Model>, AppError> {
        let result = OrganizationDepartment::find()
            .filter(department::Column::OrganizationId.eq(organization_id))
            .filter(department::Column::HierarchyPath.starts_with(hierarchy_path_prefix))
            .filter(department::Column::IsActive.eq(true))
            .order_by_asc(department::Column::HierarchyLevel)
            .order_by_asc(department::Column::HierarchyPath)
            .all(db)
            .await?;
        Ok(result)
    }

    pub async fn update_by_model(
        db: &DatabaseConnection,
        department: department::ActiveModel,
    ) -> Result<department::Model, AppError> {
        let result = department.update(db).await?;
        Ok(result)
    }

    pub async fn delete_by_id(db: &DatabaseConnection, id: Uuid) -> Result<(), AppError> {
        let department = Self::find_by_id(db, id)
            .await?
            .ok_or_else(|| AppError::NotFound("Department not found".to_string()))?;

        let mut active_model: department::ActiveModel = department.into();
        active_model.is_active = sea_orm::Set(false);
        active_model.updated_at = sea_orm::Set(chrono::Utc::now());
        active_model.update(db).await?;

        Ok(())
    }

    pub async fn find_by_name_and_organization(
        db: &DatabaseConnection,
        name: &str,
        organization_id: Uuid,
        parent_department_id: Option<Uuid>,
    ) -> Result<Option<department::Model>, AppError> {
        let mut query = OrganizationDepartment::find()
            .filter(department::Column::Name.eq(name))
            .filter(department::Column::OrganizationId.eq(organization_id))
            .filter(department::Column::IsActive.eq(true));

        if let Some(parent_id) = parent_department_id {
            query = query.filter(department::Column::ParentDepartmentId.eq(parent_id));
        } else {
            query = query.filter(department::Column::ParentDepartmentId.is_null());
        }

        let result = query.one(db).await?;
        Ok(result)
    }

    pub async fn update_hierarchy_path_recursive(
        db: &DatabaseConnection,
        department_id: Uuid,
        new_parent_path: Option<&str>,
    ) -> Result<(), AppError> {
        // Get the department
        let department = Self::find_by_id(db, department_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Department not found".to_string()))?;

        // Build new hierarchy path
        let new_path = match new_parent_path {
            Some(parent_path) => format!("{}/{}", parent_path, department_id),
            None => department_id.to_string(),
        };

        // Update the department's path
        let mut active_model: department::ActiveModel = department.clone().into();
        active_model.hierarchy_path = sea_orm::Set(new_path.clone());
        active_model.updated_at = sea_orm::Set(chrono::Utc::now());
        active_model.update(db).await?;

        // Update all children recursively
        let children = Self::find_children_by_parent_id(db, department_id).await?;
        for child in children {
            Box::pin(Self::update_hierarchy_path_recursive(
                db,
                child.id,
                Some(&new_path),
            ))
            .await?;
        }

        Ok(())
    }

    pub async fn count_by_organization(
        db: &DatabaseConnection,
        organization_id: Uuid,
    ) -> Result<u64, AppError> {
        let count = OrganizationDepartment::find()
            .filter(department::Column::OrganizationId.eq(organization_id))
            .filter(department::Column::IsActive.eq(true))
            .count(db)
            .await?;
        Ok(count)
    }

    pub async fn find_root_departments(
        db: &DatabaseConnection,
        organization_id: Uuid,
    ) -> Result<Vec<department::Model>, AppError> {
        let result = OrganizationDepartment::find()
            .filter(department::Column::OrganizationId.eq(organization_id))
            .filter(department::Column::ParentDepartmentId.is_null())
            .filter(department::Column::IsActive.eq(true))
            .order_by_asc(department::Column::Name)
            .all(db)
            .await?;
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
