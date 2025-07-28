use crate::api::dto::organization_hierarchy_dto::DepartmentSearchQuery;
use crate::domain::{
    department_member_model::{self, DepartmentRole},
    organization_analytics_model::{self, AnalyticsType, MetricValue, Period},
    organization_department_model,
    permission_matrix_model::{
        self, ComplianceSettings, EntityType, InheritanceSettings, PermissionMatrix,
    },
};
use crate::error::AppError;
use crate::log_with_context;
use crate::repository::{
    department_member_repository::DepartmentMemberRepository,
    organization_analytics_repository::OrganizationAnalyticsRepository,
    organization_department_repository::OrganizationDepartmentRepository,
    permission_matrix_repository::PermissionMatrixRepository,
};
use crate::utils::error_helper::internal_server_error;
use chrono::{DateTime, Utc};
use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};
use uuid::Uuid;

pub struct OrganizationHierarchyService;

impl OrganizationHierarchyService {
    // 組織階層構造の取得
    pub async fn get_organization_hierarchy(
        db: &DatabaseConnection,
        organization_id: Uuid,
    ) -> Result<Vec<organization_department_model::Model>, AppError> {
        OrganizationDepartmentRepository::find_hierarchy_by_organization_id(db, organization_id)
            .await
    }

    // 統一クエリパラメータを使用した部門検索
    pub async fn search_departments(
        db: &DatabaseConnection,
        query: &DepartmentSearchQuery,
        organization_id: Uuid,
    ) -> Result<(Vec<organization_department_model::Model>, u64), AppError> {
        log_with_context!(
            tracing::Level::DEBUG,
            "Searching departments",
            "organization_id" => organization_id,
            "search_term" => query.search.as_deref().unwrap_or("")
        );

        // ページネーション値の取得
        let (page, per_page) = query.pagination.get_pagination();

        // リポジトリのsearch_departmentsメソッドを呼び出し
        let (departments, total) = OrganizationDepartmentRepository::search_departments(
            db,
            query,
            organization_id,
            page,
            per_page,
        )
        .await
        .map_err(|e| {
            internal_server_error(
                e,
                "organization_hierarchy_service::search_departments",
                "Failed to search departments",
            )
        })?;

        log_with_context!(
            tracing::Level::INFO,
            "Departments search completed",
            "organization_id" => organization_id,
            "total_found" => total,
            "page" => page,
            "per_page" => per_page
        );

        Ok((departments, total))
    }

    // 部門の作成
    pub async fn create_department(
        db: &DatabaseConnection,
        organization_id: Uuid,
        name: String,
        description: Option<String>,
        parent_department_id: Option<Uuid>,
        manager_user_id: Option<Uuid>,
        created_by: Uuid,
    ) -> Result<organization_department_model::Model, AppError> {
        log_with_context!(
            tracing::Level::DEBUG,
            "Creating department",
            "organization_id" => organization_id,
            "name" => &name,
            "parent_department_id" => parent_department_id,
            "created_by" => created_by
        );
        // 同名部門のチェック（同一親部門内で）
        if let Some(_existing) = OrganizationDepartmentRepository::find_by_name_and_organization(
            db,
            &name,
            organization_id,
            parent_department_id,
        )
        .await?
        {
            return Err(AppError::Conflict(
                "Department with same name already exists in this parent".to_string(),
            ));
        }

        // 循環参照のチェック
        if let Some(parent_id) = parent_department_id {
            if OrganizationDepartmentRepository::exists_circular_dependency(
                db, parent_id, parent_id,
            )
            .await?
            {
                return Err(AppError::BadRequest(
                    "Circular dependency detected".to_string(),
                ));
            }
        }

        // 階層レベルとパスの計算
        let (hierarchy_level, hierarchy_path) = if let Some(parent_id) = parent_department_id {
            let parent = OrganizationDepartmentRepository::find_by_id(db, parent_id)
                .await?
                .ok_or_else(|| AppError::NotFound("Parent department not found".to_string()))?;
            (
                parent.hierarchy_level + 1,
                format!("{}/{}", parent.hierarchy_path, Uuid::new_v4()),
            )
        } else {
            (0, format!("/{}", Uuid::new_v4()))
        };

        let new_department = organization_department_model::ActiveModel {
            id: Set(Uuid::new_v4()),
            name: Set(name),
            description: Set(description),
            organization_id: Set(organization_id),
            parent_department_id: Set(parent_department_id),
            hierarchy_level: Set(hierarchy_level),
            hierarchy_path: Set(hierarchy_path),
            manager_user_id: Set(manager_user_id),
            is_active: Set(true),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
        };

        let department = OrganizationDepartmentRepository::create(db, new_department).await?;

        log_with_context!(
            tracing::Level::INFO,
            "Department created successfully",
            "organization_id" => organization_id,
            "department_id" => department.id,
            "name" => &department.name
        );

        // 作成者を部門マネージャーとして追加
        if let Some(manager_id) = manager_user_id {
            let member = department_member_model::ActiveModel {
                id: Set(Uuid::new_v4()),
                department_id: Set(department.id),
                user_id: Set(manager_id),
                role: Set(DepartmentRole::Manager.to_string()),
                is_active: Set(true),
                joined_at: Set(Utc::now()),
                added_by: Set(created_by),
                created_at: Set(Utc::now()),
                updated_at: Set(Utc::now()),
            };
            DepartmentMemberRepository::create(db, member).await?;
        }

        Ok(department)
    }

    // 部門情報の更新
    pub async fn update_department(
        db: &DatabaseConnection,
        department_id: Uuid,
        name: Option<String>,
        description: Option<String>,
        manager_user_id: Option<Uuid>,
        new_parent_id: Option<Uuid>,
    ) -> Result<organization_department_model::Model, AppError> {
        log_with_context!(
            tracing::Level::DEBUG,
            "Updating department",
            "department_id" => department_id
        );
        let department = OrganizationDepartmentRepository::find_by_id(db, department_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Department not found".to_string()))?;

        // 親部門変更時の循環参照チェック
        if let Some(parent_id) = new_parent_id {
            if OrganizationDepartmentRepository::exists_circular_dependency(
                db,
                department_id,
                parent_id,
            )
            .await?
            {
                return Err(AppError::BadRequest(
                    "Circular dependency detected".to_string(),
                ));
            }
        }

        let mut active_model: organization_department_model::ActiveModel = department.into();

        if let Some(new_name) = name {
            active_model.name = Set(new_name);
        }
        if let Some(new_description) = description {
            active_model.description = Set(Some(new_description));
        }
        if let Some(new_manager) = manager_user_id {
            active_model.manager_user_id = Set(Some(new_manager));
        }

        // 親部門変更時の階層パス更新
        if new_parent_id != *active_model.parent_department_id.as_ref() {
            active_model.parent_department_id = Set(new_parent_id);

            let (new_level, new_path) = if let Some(parent_id) = new_parent_id {
                let parent = OrganizationDepartmentRepository::find_by_id(db, parent_id)
                    .await?
                    .ok_or_else(|| AppError::NotFound("Parent department not found".to_string()))?;
                (
                    parent.hierarchy_level + 1,
                    format!("{}/{}", parent.hierarchy_path, department_id),
                )
            } else {
                (0, format!("/{}", department_id))
            };

            active_model.hierarchy_level = Set(new_level);
            active_model.hierarchy_path = Set(new_path.clone());

            // 子部門の階層パス更新
            let children =
                OrganizationDepartmentRepository::find_children_by_parent_id(db, department_id)
                    .await?;
            Self::update_children_hierarchy_paths(db, &children, &new_path, new_level).await?;
        }

        active_model.updated_at = Set(Utc::now());
        let updated_department =
            OrganizationDepartmentRepository::update_by_id(db, department_id, active_model).await?;

        log_with_context!(
            tracing::Level::INFO,
            "Department updated successfully",
            "department_id" => department_id
        );

        Ok(updated_department)
    }

    // 部門の削除（子部門を親部門に移動）
    pub async fn delete_department(
        db: &DatabaseConnection,
        department_id: Uuid,
    ) -> Result<(), AppError> {
        log_with_context!(
            tracing::Level::DEBUG,
            "Deleting department",
            "department_id" => department_id
        );
        let department = OrganizationDepartmentRepository::find_by_id(db, department_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Department not found".to_string()))?;

        // 子部門を親部門に移動
        let children =
            OrganizationDepartmentRepository::find_children_by_parent_id(db, department_id).await?;
        for child in children {
            let mut child_active: organization_department_model::ActiveModel = child.into();
            child_active.parent_department_id = Set(department.parent_department_id);

            // 階層レベルとパスの更新
            let (new_level, new_path) = if let Some(grandparent_id) =
                department.parent_department_id
            {
                let grandparent = OrganizationDepartmentRepository::find_by_id(db, grandparent_id)
                    .await?
                    .ok_or_else(|| {
                        AppError::NotFound("Grandparent department not found".to_string())
                    })?;
                (
                    grandparent.hierarchy_level + 1,
                    format!(
                        "{}/{}",
                        grandparent.hierarchy_path,
                        child_active.id.as_ref()
                    ),
                )
            } else {
                (0, format!("/{}", child_active.id.as_ref()))
            };

            child_active.hierarchy_level = Set(new_level);
            child_active.hierarchy_path = Set(new_path);
            child_active.updated_at = Set(Utc::now());
            child_active.update(db).await?;
        }

        // 部門メンバーの非アクティブ化
        let members = DepartmentMemberRepository::find_by_department_id(db, department_id).await?;
        for member in members {
            DepartmentMemberRepository::deactivate_by_id(db, member.id).await?;
        }

        // 部門の論理削除
        OrganizationDepartmentRepository::delete_by_id(db, department_id).await?;

        log_with_context!(
            tracing::Level::INFO,
            "Department deleted successfully",
            "department_id" => department_id
        );

        Ok(())
    }

    // 組織分析ダッシュボードの取得
    pub async fn get_organization_analytics(
        db: &DatabaseConnection,
        organization_id: Uuid,
        period: Option<Period>,
        analytics_type: Option<AnalyticsType>,
        limit: Option<u64>,
    ) -> Result<Vec<organization_analytics_model::Model>, AppError> {
        match (period, analytics_type) {
            (Some(p), Some(_t)) => {
                let end_date = Utc::now();
                let start_date = match p {
                    Period::Daily => end_date - chrono::Duration::days(30),
                    Period::Weekly => end_date - chrono::Duration::weeks(12),
                    Period::Monthly => end_date - chrono::Duration::days(365),
                    Period::Quarterly => end_date - chrono::Duration::days(365 * 2),
                    Period::Yearly => end_date - chrono::Duration::days(365 * 5),
                };
                OrganizationAnalyticsRepository::find_by_organization_and_period(
                    db,
                    organization_id,
                    p,
                    start_date,
                    end_date,
                )
                .await
            }
            (None, Some(t)) => {
                OrganizationAnalyticsRepository::find_by_organization_and_type(
                    db,
                    organization_id,
                    t,
                    limit,
                )
                .await
            }
            _ => {
                OrganizationAnalyticsRepository::find_by_organization_id(db, organization_id, limit)
                    .await
            }
        }
    }

    // 権限マトリックスの設定
    pub async fn set_permission_matrix(
        db: &DatabaseConnection,
        entity_type: EntityType,
        entity_id: Uuid,
        matrix_data: PermissionMatrix,
        inheritance_settings: Option<InheritanceSettings>,
        compliance_settings: Option<ComplianceSettings>,
        updated_by: Uuid,
    ) -> Result<permission_matrix_model::Model, AppError> {
        let entity_type_str = entity_type.to_string();

        log_with_context!(
            tracing::Level::DEBUG,
            "Setting permission matrix",
            "entity_type" => &entity_type_str,
            "entity_id" => entity_id,
            "updated_by" => updated_by
        );
        let new_matrix = permission_matrix_model::ActiveModel {
            id: Set(Uuid::new_v4()),
            entity_type: Set(entity_type_str.clone()),
            entity_id: Set(entity_id),
            matrix_version: Set("v1.0".to_string()),
            matrix_data: Set(serde_json::to_value(matrix_data).unwrap_or_default()),
            inheritance_settings: Set(
                inheritance_settings.map(|s| serde_json::to_value(s).unwrap_or_default())
            ),
            compliance_settings: Set(
                compliance_settings.map(|s| serde_json::to_value(s).unwrap_or_default())
            ),
            updated_by: Set(updated_by),
            is_active: Set(true),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
        };

        let result =
            PermissionMatrixRepository::update_by_entity(db, entity_type, entity_id, new_matrix)
                .await?;

        log_with_context!(
            tracing::Level::INFO,
            "Permission matrix set successfully",
            "entity_type" => &entity_type_str,
            "entity_id" => entity_id
        );

        Ok(result)
    }

    // 権限マトリックスの取得
    pub async fn get_permission_matrix(
        db: &DatabaseConnection,
        entity_type: EntityType,
        entity_id: Uuid,
    ) -> Result<Option<permission_matrix_model::Model>, AppError> {
        PermissionMatrixRepository::find_by_entity(db, entity_type, entity_id).await
    }

    // 実効権限の分析
    pub async fn analyze_effective_permissions(
        db: &DatabaseConnection,
        organization_id: Uuid,
        user_id: Option<Uuid>,
    ) -> Result<serde_json::Value, AppError> {
        let mut permissions_chain = Vec::new();

        // 組織レベルの権限マトリックス
        if let Some(org_matrix) = PermissionMatrixRepository::find_by_entity(
            db,
            EntityType::Organization,
            organization_id,
        )
        .await?
        {
            permissions_chain.push(serde_json::json!({
                "level": "organization",
                "source": "organization_matrix",
                "matrix": org_matrix.matrix_data,
                "inheritance_settings": org_matrix.inheritance_settings
            }));
        }

        // ユーザーが指定された場合、そのユーザーの部門権限も取得
        if let Some(uid) = user_id {
            let user_departments = DepartmentMemberRepository::find_by_user_id(db, uid).await?;
            for membership in user_departments {
                if let Some(dept_matrix) = PermissionMatrixRepository::find_by_entity(
                    db,
                    EntityType::Department,
                    membership.department_id,
                )
                .await?
                {
                    permissions_chain.push(serde_json::json!({
                        "level": "department",
                        "source": format!("dept_{}", membership.department_id),
                        "role": membership.role,
                        "matrix": dept_matrix.matrix_data,
                        "inheritance_settings": dept_matrix.inheritance_settings
                    }));
                }
            }
        }

        Ok(serde_json::json!({
            "organization_id": organization_id,
            "user_id": user_id,
            "inheritance_chain": permissions_chain,
            "analyzed_at": Utc::now()
        }))
    }

    // 組織データのエクスポート
    pub async fn export_organization_data(
        db: &DatabaseConnection,
        organization_id: Uuid,
        include_analytics: bool,
        include_permissions: bool,
    ) -> Result<serde_json::Value, AppError> {
        // 組織基本情報
        let departments = Self::get_organization_hierarchy(db, organization_id).await?;

        let mut export_data = serde_json::json!({
            "organization_id": organization_id,
            "departments": departments,
            "exported_at": Utc::now()
        });

        // 分析データの追加
        if include_analytics {
            let analytics = OrganizationAnalyticsRepository::find_by_organization_id(
                db,
                organization_id,
                Some(100),
            )
            .await?;
            export_data["analytics"] = serde_json::to_value(analytics).map_err(|e| {
                internal_server_error(
                    e,
                    "organization_hierarchy_service::export_organization_data",
                    "Serialization error",
                )
            })?;
        }

        // 権限データの追加
        if include_permissions {
            let org_matrix = PermissionMatrixRepository::find_by_entity(
                db,
                EntityType::Organization,
                organization_id,
            )
            .await?;
            export_data["organization_permissions"] =
                serde_json::to_value(org_matrix).map_err(|e| {
                    internal_server_error(
                        e,
                        "organization_hierarchy_service::export_organization_data",
                        "Serialization error",
                    )
                })?;

            let dept_ids: Vec<Uuid> = departments.iter().map(|d| d.id).collect();
            let dept_matrices =
                PermissionMatrixRepository::find_department_matrices(db, dept_ids).await?;
            export_data["department_permissions"] =
                serde_json::to_value(dept_matrices).map_err(|e| {
                    internal_server_error(
                        e,
                        "organization_hierarchy_service::export_organization_data",
                        "Serialization error",
                    )
                })?;
        }

        Ok(export_data)
    }

    // 部門メンバーの追加
    pub async fn add_department_member(
        db: &DatabaseConnection,
        department_id: Uuid,
        user_id: Uuid,
        role: DepartmentRole,
        added_by: Uuid,
    ) -> Result<department_member_model::Model, AppError> {
        log_with_context!(
            tracing::Level::DEBUG,
            "Adding department member",
            "department_id" => department_id,
            "user_id" => user_id,
            "role" => &role.to_string(),
            "added_by" => added_by
        );
        // 既存メンバーシップのチェック
        if DepartmentMemberRepository::is_member_of_department(db, user_id, department_id).await? {
            return Err(AppError::Conflict(
                "User is already a member of this department".to_string(),
            ));
        }

        let new_member = department_member_model::ActiveModel {
            id: Set(Uuid::new_v4()),
            department_id: Set(department_id),
            user_id: Set(user_id),
            role: Set(role.to_string()),
            is_active: Set(true),
            joined_at: Set(Utc::now()),
            added_by: Set(added_by),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
        };

        let member = DepartmentMemberRepository::create(db, new_member).await?;

        log_with_context!(
            tracing::Level::INFO,
            "Department member added successfully",
            "department_id" => department_id,
            "user_id" => user_id,
            "role" => &role.to_string()
        );

        Ok(member)
    }

    // 部門メンバーの削除
    pub async fn remove_department_member(
        db: &DatabaseConnection,
        department_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        log_with_context!(
            tracing::Level::DEBUG,
            "Removing department member",
            "department_id" => department_id,
            "user_id" => user_id
        );
        DepartmentMemberRepository::deactivate_by_department_and_user(db, department_id, user_id)
            .await?;

        log_with_context!(
            tracing::Level::INFO,
            "Department member removed successfully",
            "department_id" => department_id,
            "user_id" => user_id
        );

        Ok(())
    }

    // ヘルパー関数：子部門の階層パス更新（非再帰版）
    async fn update_children_hierarchy_paths(
        db: &DatabaseConnection,
        children: &[organization_department_model::Model],
        parent_path: &str,
        parent_level: i32,
    ) -> Result<(), AppError> {
        let mut queue = Vec::new();

        // 初期子要素をキューに追加
        for child in children {
            let new_path = format!("{}/{}", parent_path, child.id);
            let new_level = parent_level + 1;
            queue.push((child.clone(), new_path, new_level));
        }

        // 幅優先探索で階層パスを更新
        while let Some((child, new_path, new_level)) = queue.pop() {
            let mut child_active: organization_department_model::ActiveModel = child.clone().into();
            child_active.hierarchy_path = Set(new_path.clone());
            child_active.hierarchy_level = Set(new_level);
            child_active.updated_at = Set(Utc::now());
            child_active.update(db).await?;

            // 子部門の子要素をキューに追加
            let grandchildren =
                OrganizationDepartmentRepository::find_children_by_parent_id(db, child.id).await?;
            for grandchild in grandchildren {
                let grandchild_path = format!("{}/{}", new_path, grandchild.id);
                let grandchild_level = new_level + 1;
                queue.push((grandchild, grandchild_path, grandchild_level));
            }
        }
        Ok(())
    }

    // 子部門の取得
    pub async fn get_child_departments(
        db: &DatabaseConnection,
        department_id: Uuid,
    ) -> Result<Vec<organization_department_model::Model>, AppError> {
        OrganizationDepartmentRepository::find_children_by_parent_id(db, department_id).await
    }

    // 組織分析メトリクスの作成
    #[allow(clippy::too_many_arguments)]
    pub async fn create_analytics_metric(
        db: &DatabaseConnection,
        organization_id: Uuid,
        department_id: Option<Uuid>,
        analytics_type: AnalyticsType,
        metric_name: String,
        metric_value: MetricValue,
        period: Period,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
        calculated_by: Uuid,
    ) -> Result<organization_analytics_model::Model, AppError> {
        log_with_context!(
            tracing::Level::DEBUG,
            "Creating analytics metric",
            "organization_id" => organization_id,
            "department_id" => department_id,
            "analytics_type" => &analytics_type.to_string(),
            "metric_name" => &metric_name,
            "period" => &period.to_string()
        );
        // 重複チェック
        if OrganizationAnalyticsRepository::exists_analytics_for_period(
            db,
            organization_id,
            department_id,
            analytics_type.clone(),
            &metric_name,
            period_start,
            period_end,
        )
        .await?
        {
            return Err(AppError::Conflict(
                "Analytics metric already exists for this period".to_string(),
            ));
        }

        let new_analytics = organization_analytics_model::ActiveModel {
            id: Set(Uuid::new_v4()),
            organization_id: Set(organization_id),
            department_id: Set(department_id),
            analytics_type: Set(analytics_type.to_string()),
            metric_name: Set(metric_name),
            metric_value: Set(serde_json::to_value(metric_value).unwrap_or_default()),
            period: Set(period.to_string()),
            period_start: Set(period_start),
            period_end: Set(period_end),
            calculated_by: Set(calculated_by),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
        };

        let metric = OrganizationAnalyticsRepository::create(db, new_analytics).await?;

        log_with_context!(
            tracing::Level::INFO,
            "Analytics metric created successfully",
            "organization_id" => organization_id,
            "metric_id" => metric.id,
            "metric_name" => &metric.metric_name
        );

        Ok(metric)
    }
}
