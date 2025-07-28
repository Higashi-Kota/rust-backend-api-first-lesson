use crate::api::dto::organization_hierarchy_dto::DepartmentSearchQuery;
use crate::domain::organization_department_model::{self, Entity as OrganizationDepartment};
use crate::error::AppError;
use crate::types::{SortOrder, SortQuery};
use sea_orm::prelude::*;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
};
use uuid::Uuid;

pub struct OrganizationDepartmentRepository;

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

    /// 部門を検索（ページネーション・ソート機能付き）
    pub async fn search_departments(
        db: &DatabaseConnection,
        query: &DepartmentSearchQuery,
        organization_id: Uuid,
        page: i32,
        per_page: i32,
    ) -> Result<(Vec<organization_department_model::Model>, u64), AppError> {
        let mut condition = Condition::all()
            .add(organization_department_model::Column::OrganizationId.eq(organization_id));

        // 検索条件の適用
        if let Some(search_term) = &query.search {
            let search_pattern = format!("%{}%", search_term);
            condition = condition.add(
                Condition::any()
                    .add(organization_department_model::Column::Name.like(&search_pattern))
                    .add(organization_department_model::Column::Description.like(&search_pattern)),
            );
        }

        // アクティブのみフィルタ
        if query.active_only.unwrap_or(true) {
            condition = condition.add(organization_department_model::Column::IsActive.eq(true));
        }

        let mut db_query = OrganizationDepartment::find().filter(condition);

        // ソートの適用
        db_query = Self::apply_sorting(db_query, &query.sort);

        // ページネーション
        let page_size = per_page as u64;
        let offset = ((page - 1) * per_page) as u64;

        let paginator = db_query.paginate(db, page_size);
        let total_count = paginator.num_items().await?;
        let departments = paginator.fetch_page(offset / page_size).await?;

        Ok((departments, total_count))
    }

    /// ソート適用ヘルパー
    fn apply_sorting(
        mut query: sea_orm::Select<organization_department_model::Entity>,
        sort: &SortQuery,
    ) -> sea_orm::Select<organization_department_model::Entity> {
        if let Some(sort_by) = &sort.sort_by {
            let allowed_fields = DepartmentSearchQuery::allowed_sort_fields();

            if allowed_fields.contains(&sort_by.as_str()) {
                match sort_by.as_str() {
                    "name" => {
                        query = match sort.sort_order {
                            SortOrder::Asc => {
                                query.order_by_asc(organization_department_model::Column::Name)
                            }
                            SortOrder::Desc => {
                                query.order_by_desc(organization_department_model::Column::Name)
                            }
                        };
                    }
                    "created_at" => {
                        query = match sort.sort_order {
                            SortOrder::Asc => {
                                query.order_by_asc(organization_department_model::Column::CreatedAt)
                            }
                            SortOrder::Desc => query
                                .order_by_desc(organization_department_model::Column::CreatedAt),
                        };
                    }
                    "updated_at" => {
                        query = match sort.sort_order {
                            SortOrder::Asc => {
                                query.order_by_asc(organization_department_model::Column::UpdatedAt)
                            }
                            SortOrder::Desc => query
                                .order_by_desc(organization_department_model::Column::UpdatedAt),
                        };
                    }
                    "path" => {
                        query = match sort.sort_order {
                            SortOrder::Asc => query
                                .order_by_asc(organization_department_model::Column::HierarchyPath),
                            SortOrder::Desc => query.order_by_desc(
                                organization_department_model::Column::HierarchyPath,
                            ),
                        };
                    }
                    _ => {
                        // デフォルトは作成日時の降順
                        query =
                            query.order_by_desc(organization_department_model::Column::CreatedAt);
                    }
                }
            } else {
                // 許可されていないフィールドの場合はデフォルト
                query = query.order_by_desc(organization_department_model::Column::CreatedAt);
            }
        } else {
            // sort_byが指定されていない場合はデフォルト
            query = query.order_by_desc(organization_department_model::Column::CreatedAt);
        }

        query
    }
}
