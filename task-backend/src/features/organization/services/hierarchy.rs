use super::super::models::{
    analytics::{self, AnalyticsType, MetricValue, Period},
    department,
    department_member::{self, DepartmentRole},
};
use super::super::repositories::{
    AnalyticsRepository, DepartmentMemberRepository, DepartmentRepository,
};
use crate::domain::permission_matrix_model::{
    self, ComplianceSettings, EntityType, InheritanceSettings,
};
use crate::error::AppError;
// TODO: Phase 19でPermissionMatrixRepositoryを使用するようになったら#[allow(unused_imports)]を削除
#[allow(unused_imports)]
use crate::repository::permission_matrix_repository::PermissionMatrixRepository;
use chrono::{DateTime, Utc};
use sea_orm::{DatabaseConnection, Set};
use uuid::Uuid;

// TODO: Phase 19で本来の使用箇所が移行されたら#[allow(dead_code)]を削除
#[allow(dead_code)]
pub struct OrganizationHierarchyService;

impl OrganizationHierarchyService {
    // 組織階層構造の取得
    pub async fn get_organization_hierarchy(
        db: &DatabaseConnection,
        organization_id: Uuid,
    ) -> Result<Vec<department::Model>, AppError> {
        DepartmentRepository::find_hierarchy_by_organization_id(db, organization_id).await
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
    ) -> Result<department::Model, AppError> {
        // 同名部門のチェック（同一親部門内で）
        if let Some(_existing) = DepartmentRepository::find_by_name_and_organization(
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
            if Self::exists_circular_dependency(db, parent_id, parent_id).await? {
                return Err(AppError::BadRequest(
                    "Circular dependency detected".to_string(),
                ));
            }
        }

        // 階層レベルとパスの計算
        let (hierarchy_level, hierarchy_path) = if let Some(parent_id) = parent_department_id {
            let parent = DepartmentRepository::find_by_id(db, parent_id)
                .await?
                .ok_or_else(|| AppError::NotFound("Parent department not found".to_string()))?;
            (
                parent.hierarchy_level + 1,
                format!("{}/{}", parent.hierarchy_path, Uuid::new_v4()),
            )
        } else {
            (0, format!("/{}", Uuid::new_v4()))
        };

        let new_department = department::ActiveModel {
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

        let department = DepartmentRepository::create(db, new_department).await?;

        // 作成者を部門マネージャーとして追加
        if let Some(manager_id) = manager_user_id {
            let member = department_member::ActiveModel {
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

    // 部門の更新
    pub async fn update_department(
        db: &DatabaseConnection,
        department_id: Uuid,
        name: Option<String>,
        description: Option<String>,
        manager_user_id: Option<Uuid>,
        updated_by: Uuid,
    ) -> Result<department::Model, AppError> {
        let department = DepartmentRepository::find_by_id(db, department_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Department not found".to_string()))?;

        // 名前変更時の重複チェック
        if let Some(ref new_name) = name {
            if let Some(_existing) = DepartmentRepository::find_by_name_and_organization(
                db,
                new_name,
                department.organization_id,
                department.parent_department_id,
            )
            .await?
            {
                if _existing.id != department_id {
                    return Err(AppError::Conflict(
                        "Department with same name already exists in this parent".to_string(),
                    ));
                }
            }
        }

        let mut active_model: department::ActiveModel = department.clone().into();

        if let Some(n) = name {
            active_model.name = Set(n);
        }
        if description.is_some() {
            active_model.description = Set(description);
        }
        if manager_user_id.is_some() {
            active_model.manager_user_id = Set(manager_user_id);
        }
        active_model.updated_at = Set(Utc::now());

        let updated = DepartmentRepository::update_by_id(db, department_id, active_model).await?;

        // マネージャー変更時の処理
        if manager_user_id.is_some() && manager_user_id != department.manager_user_id {
            // 既存のマネージャーを一般メンバーに降格
            if let Some(old_manager_id) = department.manager_user_id {
                if let Some(old_manager) = DepartmentMemberRepository::find_by_department_and_user(
                    db,
                    department_id,
                    old_manager_id,
                )
                .await?
                {
                    DepartmentMemberRepository::update_role(
                        db,
                        old_manager.id,
                        &DepartmentRole::Member.to_string(),
                    )
                    .await?;
                }
            }

            // 新しいマネージャーを設定
            if let Some(new_manager_id) = manager_user_id {
                if let Some(new_manager) = DepartmentMemberRepository::find_by_department_and_user(
                    db,
                    department_id,
                    new_manager_id,
                )
                .await?
                {
                    DepartmentMemberRepository::update_role(
                        db,
                        new_manager.id,
                        &DepartmentRole::Manager.to_string(),
                    )
                    .await?;
                } else {
                    // メンバーでない場合は追加
                    let member = department_member::ActiveModel {
                        id: Set(Uuid::new_v4()),
                        department_id: Set(department_id),
                        user_id: Set(new_manager_id),
                        role: Set(DepartmentRole::Manager.to_string()),
                        is_active: Set(true),
                        joined_at: Set(Utc::now()),
                        added_by: Set(updated_by),
                        created_at: Set(Utc::now()),
                        updated_at: Set(Utc::now()),
                    };
                    DepartmentMemberRepository::create(db, member).await?;
                }
            }
        }

        Ok(updated)
    }

    // 部門の削除（論理削除）
    pub async fn delete_department(
        db: &DatabaseConnection,
        department_id: Uuid,
    ) -> Result<(), AppError> {
        // 子部門の存在チェック
        let children = DepartmentRepository::find_children_by_parent_id(db, department_id).await?;
        if !children.is_empty() {
            return Err(AppError::BadRequest(
                "Cannot delete department with child departments".to_string(),
            ));
        }

        // 部門メンバーの非アクティブ化
        let members = DepartmentMemberRepository::find_by_department_id(db, department_id).await?;
        for member in members {
            DepartmentMemberRepository::deactivate_by_id(db, member.id).await?;
        }

        // 部門の論理削除
        DepartmentRepository::delete_by_id(db, department_id).await?;

        Ok(())
    }

    // 部門メンバーの追加
    pub async fn add_department_member(
        db: &DatabaseConnection,
        department_id: Uuid,
        user_id: Uuid,
        role: DepartmentRole,
        added_by: Uuid,
    ) -> Result<department_member::Model, AppError> {
        // 既存メンバーチェック
        if let Some(_existing) =
            DepartmentMemberRepository::find_by_department_and_user(db, department_id, user_id)
                .await?
        {
            return Err(AppError::Conflict(
                "User is already a member of this department".to_string(),
            ));
        }

        let member = department_member::ActiveModel {
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

        DepartmentMemberRepository::create(db, member).await
    }

    // 部門メンバーの削除
    pub async fn remove_department_member(
        db: &DatabaseConnection,
        department_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        DepartmentMemberRepository::deactivate_by_department_and_user(db, department_id, user_id)
            .await
    }

    // 権限マトリックスの設定
    // TODO: Phase 19でPermissionMatrixModelの構造を修正後、実装を復活
    #[allow(dead_code)]
    pub async fn set_permission_matrix(
        _db: &DatabaseConnection,
        _entity_type: EntityType,
        _entity_id: Uuid,
        _resource: String,
        _action: String,
        _allowed: bool,
        _conditions: Option<serde_json::Value>,
        _inheritance_settings: InheritanceSettings,
        _compliance_settings: ComplianceSettings,
        _updated_by: Uuid,
    ) -> Result<permission_matrix_model::Model, AppError> {
        // 一時的にダミー実装
        Err(AppError::InternalServerError(
            "Permission matrix not yet implemented".to_string(),
        ))
        
        // TODO: Phase 19で以下のコメントアウトを解除
        // let active_model = permission_matrix_model::ActiveModel {
        //     id: Set(Uuid::new_v4()),
        //     entity_type: Set(entity_type.to_string()),
        //     entity_id: Set(entity_id),
        //     resource: Set(resource),
        //     action: Set(action),
        //     allowed: Set(allowed),
        //     conditions: Set(conditions),
        //     inheritance_settings: Set(serde_json::to_value(&inheritance_settings)?),
        //     compliance_settings: Set(serde_json::to_value(&compliance_settings)?),
        //     priority: Set(0),
        //     effective_from: Set(None),
        //     effective_until: Set(None),
        //     created_by: Set(updated_by),
        //     updated_by: Set(updated_by),
        //     created_at: Set(Utc::now()),
        //     updated_at: Set(Utc::now()),
        // };
        //
        // PermissionMatrixRepository::create(db, active_model).await
    }

    // 分析データの記録
    pub async fn record_analytics(
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
    ) -> Result<analytics::Model, AppError> {
        // 既存データの確認
        if AnalyticsRepository::exists_analytics_for_period(
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
                "Analytics data already exists for this period".to_string(),
            ));
        }

        let analytics = analytics::ActiveModel {
            id: Set(Uuid::new_v4()),
            organization_id: Set(organization_id),
            department_id: Set(department_id),
            analytics_type: Set(analytics_type.to_string()),
            metric_name: Set(metric_name),
            metric_value: Set(serde_json::to_value(&metric_value).map_err(|e| {
                AppError::InternalServerError(format!("Failed to serialize metric value: {}", e))
            })?),
            period: Set(period.to_string()),
            period_start: Set(period_start),
            period_end: Set(period_end),
            calculated_by: Set(calculated_by),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
        };

        AnalyticsRepository::create(db, analytics).await
    }

    // 分析データの取得
    pub async fn get_analytics(
        db: &DatabaseConnection,
        organization_id: Uuid,
        department_id: Option<Uuid>,
        analytics_type: Option<AnalyticsType>,
        period: Option<Period>,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
        limit: Option<u64>,
    ) -> Result<Vec<analytics::Model>, AppError> {
        if let Some(dept_id) = department_id {
            AnalyticsRepository::find_by_department_id(db, dept_id, limit).await
        } else if let Some(a_type) = analytics_type {
            AnalyticsRepository::find_by_organization_and_type(db, organization_id, a_type, limit)
                .await
        } else if let (Some(p), Some(start), Some(end)) = (period, start_date, end_date) {
            AnalyticsRepository::find_by_organization_and_period(db, organization_id, p, start, end)
                .await
        } else {
            AnalyticsRepository::find_by_organization_id(db, organization_id, limit).await
        }
    }

    // ヘルパーメソッド：循環依存のチェック
    async fn exists_circular_dependency(
        db: &DatabaseConnection,
        department_id: Uuid,
        target_parent_id: Uuid,
    ) -> Result<bool, AppError> {
        if department_id == target_parent_id {
            return Ok(true);
        }

        let mut current_id = Some(target_parent_id);
        while let Some(id) = current_id {
            if id == department_id {
                return Ok(true);
            }
            let dept = DepartmentRepository::find_by_id(db, id).await?;
            current_id = dept.and_then(|d| d.parent_department_id);
        }

        Ok(false)
    }
}
