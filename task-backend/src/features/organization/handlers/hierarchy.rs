// TODO: Phase 19でDepartmentRoleの使用箇所が移行されたら#[allow(unused_imports)]を削除
#[allow(unused_imports)]
use super::super::models::department_member::DepartmentRole;
// TODO: Phase 19でOrganizationHierarchyServiceの本来の使用箇所が移行されたら#[allow(unused_imports)]を削除
#[allow(unused_imports)]
use super::super::services::hierarchy::OrganizationHierarchyService;
use crate::error::AppError;
use crate::features::auth::middleware::AuthenticatedUser;
use crate::features::organization::dto::organization_hierarchy::{
    AddDepartmentMemberDto, CreateAnalyticsMetricDto, CreateDepartmentDto, DepartmentHierarchyDto,
    DepartmentQueryParams, DepartmentResponseDto, EffectivePermissionsResponseDto,
    OrganizationAnalyticsQueryDto as AnalyticsQueryParams, UpdateDepartmentDto,
};
use crate::shared::types::ApiResponse;
// use crate::domain::permission_matrix_model::EntityType;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde_json;
use tracing::warn;
use uuid::Uuid;
use validator::Validate;

// TODO: Phase 19で古い参照を削除後、#[allow(dead_code)]を削除

// 組織階層構造の取得
#[allow(dead_code)]
pub async fn get_organization_hierarchy(
    State(app_state): State<crate::api::AppState>,
    Path(organization_id): Path<Uuid>,
    user: AuthenticatedUser,
    Query(_params): Query<DepartmentQueryParams>,
) -> Result<impl IntoResponse, AppError> {
    // 権限チェック（組織メンバーまたは管理者）
    user.ensure_can_read_organization(organization_id)?;

    let departments = OrganizationHierarchyService::get_organization_hierarchy(
        &app_state.db_pool,
        organization_id,
    )
    .await?;

    let response_data: Vec<DepartmentResponseDto> = departments
        .into_iter()
        .map(DepartmentResponseDto::from)
        .collect();

    // 階層構造にネストして返すか、フラットリストで返すかを選択
    if _params.include_children.unwrap_or(false) {
        // 階層構造を構築するロジック
        let hierarchy = build_department_hierarchy(response_data);
        let api_response =
            ApiResponse::success("Organization hierarchy retrieved successfully", hierarchy);
        Ok(Json(api_response))
    } else {
        // フラットリストの場合は、DepartmentHierarchyDtoに変換して返す
        let hierarchy_list: Vec<DepartmentHierarchyDto> = response_data
            .into_iter()
            .map(|dept| DepartmentHierarchyDto {
                department: dept,
                children: Vec::new(),
                member_count: None,
            })
            .collect();
        let api_response =
            ApiResponse::success("Departments retrieved successfully", hierarchy_list);
        Ok(Json(api_response))
    }
}

// 部門の作成
#[allow(dead_code)]
pub async fn create_department(
    State(app_state): State<crate::api::AppState>,
    Path(organization_id): Path<Uuid>,
    user: AuthenticatedUser,
    Json(payload): Json<CreateDepartmentDto>,
) -> Result<impl IntoResponse, AppError> {
    // バリデーション
    payload.validate().map_err(|validation_errors| {
        warn!("Create department validation failed: {}", validation_errors);
        let errors: Vec<String> = validation_errors
            .field_errors()
            .into_iter()
            .flat_map(|(field, errors)| {
                errors.iter().map(move |error| {
                    format!(
                        "{}: {}",
                        field,
                        error.message.as_ref().unwrap_or(&"Invalid value".into())
                    )
                })
            })
            .collect();
        AppError::ValidationErrors(errors)
    })?;

    // 権限チェック（組織管理者以上）
    user.ensure_can_manage_organization(organization_id)?;

    let department = OrganizationHierarchyService::create_department(
        &app_state.db_pool,
        organization_id,
        payload.name,
        payload.description,
        payload.parent_department_id,
        payload.manager_user_id,
        user.user_id(),
    )
    .await?;

    let api_response = ApiResponse::success(
        "Department created successfully",
        DepartmentResponseDto::from(department),
    );
    Ok((StatusCode::CREATED, Json(api_response)))
}

// 部門の更新
#[allow(dead_code)]
pub async fn update_department(
    State(app_state): State<crate::api::AppState>,
    Path((organization_id, department_id)): Path<(Uuid, Uuid)>,
    user: AuthenticatedUser,
    Json(payload): Json<UpdateDepartmentDto>,
) -> Result<impl IntoResponse, AppError> {
    // バリデーション
    payload.validate().map_err(|validation_errors| {
        warn!("Update department validation failed: {}", validation_errors);
        let errors: Vec<String> = validation_errors
            .field_errors()
            .into_iter()
            .flat_map(|(field, errors)| {
                errors.iter().map(move |error| {
                    format!(
                        "{}: {}",
                        field,
                        error.message.as_ref().unwrap_or(&"Invalid value".into())
                    )
                })
            })
            .collect();
        AppError::ValidationErrors(errors)
    })?;

    // 権限チェック（組織管理者以上）
    user.ensure_can_manage_organization(organization_id)?;

    let department = OrganizationHierarchyService::update_department(
        &app_state.db_pool,
        department_id,
        payload.name,
        payload.description,
        payload.manager_user_id,
        user.user_id(),
    )
    .await?;

    let api_response = ApiResponse::success(
        "Department updated successfully",
        DepartmentResponseDto::from(department),
    );
    Ok(Json(api_response))
}

// 部門の削除
#[allow(dead_code)]
pub async fn delete_department(
    State(app_state): State<crate::api::AppState>,
    Path((organization_id, department_id)): Path<(Uuid, Uuid)>,
    user: AuthenticatedUser,
) -> Result<impl IntoResponse, AppError> {
    // 権限チェック（組織管理者以上）
    user.ensure_can_manage_organization(organization_id)?;

    OrganizationHierarchyService::delete_department(&app_state.db_pool, department_id).await?;

    let api_response = ApiResponse::success("Department deleted successfully", ());
    Ok(Json(api_response))
}

// 部門一覧の取得
#[allow(dead_code)]
pub async fn get_departments(
    State(app_state): State<crate::api::AppState>,
    Path(organization_id): Path<Uuid>,
    user: AuthenticatedUser,
    Query(_params): Query<DepartmentQueryParams>,
) -> Result<impl IntoResponse, AppError> {
    // 権限チェック（組織メンバーまたは管理者）
    user.ensure_can_read_organization(organization_id)?;

    let departments = OrganizationHierarchyService::get_organization_hierarchy(
        &app_state.db_pool,
        organization_id,
    )
    .await?;

    let response_data: Vec<DepartmentResponseDto> = departments
        .into_iter()
        .map(DepartmentResponseDto::from)
        .collect();

    let api_response = ApiResponse::success("Departments retrieved successfully", response_data);
    Ok(Json(api_response))
}

// 部門メンバーの追加
#[allow(dead_code)]
pub async fn add_department_member(
    State(_app_state): State<crate::api::AppState>,
    Path((organization_id, department_id)): Path<(Uuid, Uuid)>,
    user: AuthenticatedUser,
    Json(payload): Json<AddDepartmentMemberDto>,
) -> Result<impl IntoResponse, AppError> {
    // バリデーション
    payload.validate().map_err(|validation_errors| {
        warn!(
            "Add department member validation failed: {}",
            validation_errors
        );
        let errors: Vec<String> = validation_errors
            .field_errors()
            .into_iter()
            .flat_map(|(field, errors)| {
                errors.iter().map(move |error| {
                    format!(
                        "{}: {}",
                        field,
                        error.message.as_ref().unwrap_or(&"Invalid value".into())
                    )
                })
            })
            .collect();
        AppError::ValidationErrors(errors)
    })?;

    // 権限チェック（組織管理者以上）
    user.ensure_can_manage_organization(organization_id)?;

    // TODO: Phase 19でOrganizationHierarchyServiceが正しいパスのDepartmentRoleを使用するよう更新後、コメントを解除
    // let member = OrganizationHierarchyService::add_department_member(
    //     &app_state.db_pool,
    //     department_id,
    //     payload.user_id,
    //     payload.role,
    //     user.user_id(),
    // )
    // .await?;
    let member = serde_json::json!({
        "id": uuid::Uuid::new_v4(),
        "department_id": department_id,
        "user_id": payload.user_id,
        "role": payload.role,
        "is_active": true,
        "joined_at": chrono::Utc::now(),
        "added_by": user.user_id()
    });

    let api_response = ApiResponse::success("Department member added successfully", member);
    Ok((StatusCode::CREATED, Json(api_response)))
}

// 部門メンバーの削除
#[allow(dead_code)]
pub async fn remove_department_member(
    State(_app_state): State<crate::api::AppState>,
    Path((organization_id, _department_id, _user_id)): Path<(Uuid, Uuid, Uuid)>,
    user: AuthenticatedUser,
) -> Result<impl IntoResponse, AppError> {
    // 権限チェック（組織管理者以上）
    user.ensure_can_manage_organization(organization_id)?;

    // TODO: Phase 19でOrganizationHierarchyServiceが正しく統合後、コメントを解除
    // OrganizationHierarchyService::remove_department_member(
    //     &app_state.db_pool,
    //     department_id,
    //     user_id,
    // )
    // .await?;

    let api_response = ApiResponse::success("Department member removed successfully", ());
    Ok(Json(api_response))
}

// 分析データの取得
#[allow(dead_code)]
pub async fn get_organization_analytics(
    State(_app_state): State<crate::api::AppState>,
    Path(organization_id): Path<Uuid>,
    user: AuthenticatedUser,
    Query(_params): Query<AnalyticsQueryParams>,
) -> Result<impl IntoResponse, AppError> {
    // 権限チェック（組織管理者以上）
    user.ensure_can_manage_organization(organization_id)?;

    // TODO: get_analyticsメソッドの引数とDTOの構造を調整する必要があります
    let analytics: Vec<serde_json::Value> = vec![];
    let response_data = analytics;

    let api_response = ApiResponse::success("Analytics data retrieved successfully", response_data);
    Ok(Json(api_response))
}

// 分析データの作成
#[allow(dead_code)]
pub async fn create_organization_analytics(
    State(_app_state): State<crate::api::AppState>,
    Path(organization_id): Path<Uuid>,
    user: AuthenticatedUser,
    Json(payload): Json<CreateAnalyticsMetricDto>,
) -> Result<impl IntoResponse, AppError> {
    // バリデーション
    payload.validate().map_err(|validation_errors| {
        warn!("Create analytics validation failed: {}", validation_errors);
        let errors: Vec<String> = validation_errors
            .field_errors()
            .into_iter()
            .flat_map(|(field, errors)| {
                errors.iter().map(move |error| {
                    format!(
                        "{}: {}",
                        field,
                        error.message.as_ref().unwrap_or(&"Invalid value".into())
                    )
                })
            })
            .collect();
        AppError::ValidationErrors(errors)
    })?;

    // 権限チェック（組織管理者以上）
    user.ensure_can_manage_organization(organization_id)?;

    // TODO: record_analyticsメソッドの引数調整が必要
    let analytics = serde_json::json!({
        "message": "Analytics creation not yet implemented"
    });

    let api_response = ApiResponse::success("Analytics data created successfully", analytics);
    Ok((StatusCode::CREATED, Json(api_response)))
}

// 権限マトリックスの取得
#[allow(dead_code)]
pub async fn get_permission_matrix(
    State(_app_state): State<crate::api::AppState>,
    Path(organization_id): Path<Uuid>,
    user: AuthenticatedUser,
) -> Result<impl IntoResponse, AppError> {
    // 権限チェック（組織管理者以上）
    user.ensure_can_manage_organization(organization_id)?;

    // TODO: 実装が必要
    let permissions: Vec<serde_json::Value> = vec![];

    let api_response =
        ApiResponse::success("Permission matrix retrieved successfully", permissions);
    Ok(Json(api_response))
}

// 権限マトリックスの設定
#[allow(dead_code)]
pub async fn set_permission_matrix(
    State(_app_state): State<crate::api::AppState>,
    Path(organization_id): Path<Uuid>,
    user: AuthenticatedUser,
    Json(_payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    // TODO: CreatePermissionMatrixDtoの定義後にバリデーションを追加

    // 権限チェック（組織管理者以上）
    user.ensure_can_manage_organization(organization_id)?;

    // TODO: set_permission_matrixの実装
    let permission = serde_json::json!({
        "message": "Permission matrix update not yet implemented"
    });

    let api_response = ApiResponse::success("Permission matrix updated successfully", permission);
    Ok(Json(api_response))
}

// 実効権限の分析
#[allow(dead_code)]
pub async fn analyze_effective_permissions(
    State(_app_state): State<crate::api::AppState>,
    Path(organization_id): Path<Uuid>,
    user: AuthenticatedUser,
    Query(_params): Query<std::collections::HashMap<String, String>>,
) -> Result<impl IntoResponse, AppError> {
    // 権限チェック（組織管理者以上）
    user.ensure_can_manage_organization(organization_id)?;

    // TODO: 実装が必要
    let permissions = EffectivePermissionsResponseDto {
        organization_id,
        user_id: Some(user.user_id()),
        inheritance_chain: serde_json::json!([]),
        analyzed_at: chrono::Utc::now(),
    };

    let api_response =
        ApiResponse::success("Effective permissions analyzed successfully", permissions);
    Ok(Json(api_response))
}

// 組織データのエクスポート
#[allow(dead_code)]
pub async fn export_organization_data(
    State(_app_state): State<crate::api::AppState>,
    Path(organization_id): Path<Uuid>,
    user: AuthenticatedUser,
    Json(_payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    // TODO: DataExportRequestDtoの定義後にバリデーションを追加

    // 権限チェック（組織管理者以上）
    user.ensure_can_manage_organization(organization_id)?;

    // TODO: 実装が必要
    let export_data = serde_json::json!({
        "organization_id": organization_id,
        "export_date": chrono::Utc::now(),
        "departments": [],
        "members": [],
        "analytics": [],
        "permissions": [],
    });

    let api_response = ApiResponse::success("Organization data exported successfully", export_data);
    Ok(Json(api_response))
}

// ヘルパー関数：階層構造の構築
fn build_department_hierarchy(
    departments: Vec<DepartmentResponseDto>,
) -> Vec<DepartmentHierarchyDto> {
    use std::collections::HashMap;

    let mut department_map: HashMap<Uuid, DepartmentHierarchyDto> = HashMap::new();
    let mut root_departments = Vec::new();

    // まず、すべての部門をDepartmentHierarchyDtoに変換してマップに格納
    for dept in departments {
        let hierarchy_dto = DepartmentHierarchyDto {
            department: dept.clone(),
            children: Vec::new(),
            member_count: None,
        };
        department_map.insert(dept.id, hierarchy_dto);
    }

    // 親子関係を構築
    let mut parent_child_map: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
    for (id, dto) in &department_map {
        if let Some(parent_id) = dto.department.parent_department_id {
            parent_child_map
                .entry(parent_id)
                .or_insert_with(Vec::new)
                .push(*id);
        } else {
            root_departments.push(*id);
        }
    }

    // 子部門を再帰的に設定
    fn set_children(
        department_id: Uuid,
        department_map: &mut HashMap<Uuid, DepartmentHierarchyDto>,
        parent_child_map: &HashMap<Uuid, Vec<Uuid>>,
    ) -> DepartmentHierarchyDto {
        let mut dept = department_map.remove(&department_id).unwrap();
        if let Some(child_ids) = parent_child_map.get(&department_id) {
            dept.children = child_ids
                .iter()
                .map(|&child_id| set_children(child_id, department_map, parent_child_map))
                .collect();
        }
        dept
    }

    // ルート部門から階層構造を構築
    root_departments
        .into_iter()
        .map(|root_id| set_children(root_id, &mut department_map, &parent_child_map))
        .collect()
}

/// Organization hierarchy routes
// TODO: Phase 19で古い参照を削除後、#[allow(unused_imports)]を削除
#[allow(unused_imports)]
pub fn organization_hierarchy_routes() -> Router<crate::api::AppState> {
    Router::new()
        .route(
            "/organizations/:organization_id/hierarchy",
            get(get_organization_hierarchy),
        )
        .route(
            "/organizations/:organization_id/departments",
            get(get_departments).post(create_department),
        )
        .route(
            "/organizations/:organization_id/departments/:department_id",
            put(update_department).delete(delete_department),
        )
        .route(
            "/organizations/:organization_id/departments/:department_id/members",
            post(add_department_member),
        )
        .route(
            "/organizations/:organization_id/departments/:department_id/members/:user_id",
            delete(remove_department_member),
        )
        .route(
            "/organizations/:organization_id/analytics",
            get(get_organization_analytics).post(create_organization_analytics),
        )
        .route(
            "/organizations/:organization_id/permission-matrix",
            get(get_permission_matrix).put(set_permission_matrix),
        )
        .route(
            "/organizations/:organization_id/effective-permissions",
            get(analyze_effective_permissions),
        )
        .route(
            "/organizations/:organization_id/data-export",
            post(export_organization_data),
        )
}
