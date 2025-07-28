use crate::{
    api::dto::{common::OperationResult, organization_hierarchy_dto::*},
    domain::permission_matrix_model::EntityType,
    error::AppError,
    extractors::{deserialize_uuid, ValidatedMultiPath, ValidatedUuid},
    middleware::auth::AuthenticatedUser,
    service::organization_hierarchy_service::OrganizationHierarchyService,
    shared::types::PaginatedResponse,
    types::{ApiResponse, Timestamp},
    utils::error_helper::convert_validation_errors,
};
use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

// 複数パラメータ用のPath構造体
#[derive(Deserialize)]
pub struct OrganizationDepartmentPath {
    #[serde(deserialize_with = "deserialize_uuid")]
    pub organization_id: Uuid,
    #[serde(deserialize_with = "deserialize_uuid")]
    pub department_id: Uuid,
}

#[derive(Deserialize)]
pub struct OrganizationDepartmentUserPath {
    #[serde(deserialize_with = "deserialize_uuid")]
    pub organization_id: Uuid,
    #[serde(deserialize_with = "deserialize_uuid")]
    pub department_id: Uuid,
    #[serde(deserialize_with = "deserialize_uuid")]
    pub user_id: Uuid,
}

// 組織階層構造の取得
pub async fn get_organization_hierarchy(
    State(app_state): State<crate::api::AppState>,
    ValidatedUuid(organization_id): ValidatedUuid,
    user: AuthenticatedUser,
    Query(params): Query<DepartmentSearchQuery>,
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
    if params.include_children.unwrap_or(false) {
        // 階層構造を構築するロジック
        let hierarchy = build_department_hierarchy(response_data);
        let api_response = ApiResponse::success(hierarchy);
        Ok(api_response)
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
        let api_response = ApiResponse::success(hierarchy_list);
        Ok(api_response)
    }
}

// 部門の作成
pub async fn create_department(
    State(app_state): State<crate::api::AppState>,
    ValidatedUuid(organization_id): ValidatedUuid,
    user: AuthenticatedUser,
    Json(payload): Json<CreateDepartmentDto>,
) -> Result<impl IntoResponse, AppError> {
    // バリデーション
    payload.validate().map_err(|e| {
        convert_validation_errors(e, "organization_hierarchy_handler::create_department")
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

    let response_data = DepartmentResponseDto::from(department);
    let api_response = ApiResponse::success(response_data);
    Ok(api_response)
}

// 部門一覧の取得
pub async fn get_departments(
    State(app_state): State<crate::api::AppState>,
    ValidatedUuid(organization_id): ValidatedUuid,
    user: AuthenticatedUser,
    Query(params): Query<DepartmentSearchQuery>,
) -> Result<impl IntoResponse, AppError> {
    // 権限チェック
    user.ensure_can_read_organization(organization_id)?;

    // 統一されたsearch_departmentsメソッドを使用
    let (departments, total) = OrganizationHierarchyService::search_departments(
        &app_state.db_pool,
        &params,
        organization_id,
    )
    .await?;

    let mut response_data: Vec<DepartmentResponseDto> = departments
        .into_iter()
        .map(DepartmentResponseDto::from)
        .collect();

    // アクティブな部門のみフィルタ
    if params.active_only.unwrap_or(true) {
        response_data.retain(|dept| dept.is_active);
    }

    let (page, per_page) = params.pagination.get_pagination();
    let response = PaginatedResponse::new(response_data, page, per_page, total as i64);

    let api_response = ApiResponse::success(response);
    Ok(api_response)
}

// 部門の更新
pub async fn update_department(
    State(app_state): State<crate::api::AppState>,
    ValidatedMultiPath(params): ValidatedMultiPath<OrganizationDepartmentPath>,
    user: AuthenticatedUser,
    Json(payload): Json<UpdateDepartmentDto>,
) -> Result<impl IntoResponse, AppError> {
    // バリデーション
    payload.validate().map_err(|e| {
        convert_validation_errors(e, "organization_hierarchy_handler::update_department")
    })?;

    // 権限チェック（組織管理者またはその部門のマネージャー）
    user.ensure_can_manage_organization_or_department(
        params.organization_id,
        params.department_id,
    )?;

    let updated_department = OrganizationHierarchyService::update_department(
        &app_state.db_pool,
        params.department_id,
        payload.name,
        payload.description,
        payload.manager_user_id,
        payload.new_parent_id,
    )
    .await?;

    let response_data = DepartmentResponseDto::from(updated_department);
    let api_response = ApiResponse::success(response_data);
    Ok(api_response)
}

// 部門の削除
pub async fn delete_department(
    State(app_state): State<crate::api::AppState>,
    ValidatedMultiPath(params): ValidatedMultiPath<OrganizationDepartmentPath>,
    user: AuthenticatedUser,
) -> Result<impl IntoResponse, AppError> {
    // 権限チェック（組織管理者以上）
    user.ensure_can_manage_organization(params.organization_id)?;

    // 削除前に影響を受ける子部門のIDを取得
    let affected_children = OrganizationHierarchyService::get_child_departments(
        &app_state.db_pool,
        params.department_id,
    )
    .await?
    .into_iter()
    .map(|dept| dept.id)
    .collect();

    OrganizationHierarchyService::delete_department(&app_state.db_pool, params.department_id)
        .await?;

    let response_data = DepartmentOperationResponseDto {
        success: true,
        message: "Department deleted successfully".to_string(),
        department_id: Some(params.department_id),
        affected_children: Some(affected_children),
    };

    // OperationResult::deletedを使用して削除操作の結果を表現
    let deletion_result = OperationResult::deleted(response_data);

    let api_response = ApiResponse::success(deletion_result.item);
    Ok(api_response)
}

// 組織分析ダッシュボードの取得
pub async fn get_organization_analytics(
    State(app_state): State<crate::api::AppState>,
    ValidatedUuid(organization_id): ValidatedUuid,
    user: AuthenticatedUser,
    Query(query): Query<OrganizationAnalyticsQueryDto>,
) -> Result<impl IntoResponse, AppError> {
    // バリデーション
    query.validate().map_err(|e| {
        convert_validation_errors(
            e,
            "organization_hierarchy_handler::get_department_analytics",
        )
    })?;

    // 権限チェック（組織メンバー以上）
    user.ensure_can_read_organization(organization_id)?;

    let analytics = OrganizationHierarchyService::get_organization_analytics(
        &app_state.db_pool,
        organization_id,
        query.period,
        query.analytics_type,
        query.limit,
    )
    .await?;

    let response_data: Vec<OrganizationAnalyticsResponseDto> = analytics
        .into_iter()
        .map(OrganizationAnalyticsResponseDto::from)
        .collect();

    let api_response = ApiResponse::success(response_data);
    Ok(api_response)
}

// 権限マトリックスの設定
pub async fn set_organization_permission_matrix(
    State(app_state): State<crate::api::AppState>,
    ValidatedUuid(organization_id): ValidatedUuid,
    user: AuthenticatedUser,
    Json(payload): Json<SetPermissionMatrixDto>,
) -> Result<impl IntoResponse, AppError> {
    // バリデーション
    payload.validate().map_err(|e| {
        convert_validation_errors(
            e,
            "organization_hierarchy_handler::update_permission_matrix",
        )
    })?;

    // 権限チェック（組織管理者以上）
    user.ensure_can_manage_organization(organization_id)?;

    let matrix = OrganizationHierarchyService::set_permission_matrix(
        &app_state.db_pool,
        EntityType::Organization,
        organization_id,
        payload.matrix_data,
        payload.inheritance_settings,
        payload.compliance_settings,
        user.user_id(),
    )
    .await?;

    let response_data = PermissionMatrixResponseDto::from(matrix);
    let api_response = ApiResponse::success(response_data);
    Ok(api_response)
}

// 権限マトリックスの取得
pub async fn get_organization_permission_matrix(
    State(app_state): State<crate::api::AppState>,
    ValidatedUuid(organization_id): ValidatedUuid,
    user: AuthenticatedUser,
) -> Result<impl IntoResponse, AppError> {
    // 権限チェック（組織メンバー以上）
    user.ensure_can_read_organization(organization_id)?;

    let matrix = OrganizationHierarchyService::get_permission_matrix(
        &app_state.db_pool,
        EntityType::Organization,
        organization_id,
    )
    .await?;

    match matrix {
        Some(matrix) => {
            let response_data = PermissionMatrixResponseDto::from(matrix);
            let api_response = ApiResponse::success(response_data);
            Ok(api_response)
        }
        None => Err(AppError::NotFound(
            "Permission matrix not found".to_string(),
        )),
    }
}

// 実効権限の分析
pub async fn get_effective_permissions(
    State(app_state): State<crate::api::AppState>,
    ValidatedUuid(organization_id): ValidatedUuid,
    user: AuthenticatedUser,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<impl IntoResponse, AppError> {
    // 権限チェック（組織管理者または自分の権限を確認）
    let target_user = params
        .get("user_id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .unwrap_or(user.user_id());
    if target_user != user.user_id() {
        user.ensure_can_manage_organization(organization_id)?;
    } else {
        user.ensure_can_read_organization(organization_id)?;
    }

    let permissions = OrganizationHierarchyService::analyze_effective_permissions(
        &app_state.db_pool,
        organization_id,
        Some(target_user),
    )
    .await?;

    let response_data = EffectivePermissionsResponseDto {
        organization_id,
        user_id: Some(target_user),
        inheritance_chain: permissions,
        analyzed_at: Timestamp::now(),
    };

    let api_response = ApiResponse::success(response_data);
    Ok(api_response)
}

// 組織データのエクスポート
pub async fn export_organization_data(
    State(app_state): State<crate::api::AppState>,
    ValidatedUuid(organization_id): ValidatedUuid,
    user: AuthenticatedUser,
    Query(export_options): Query<ExportOrganizationDataDto>,
) -> Result<impl IntoResponse, AppError> {
    // バリデーション
    export_options.validate().map_err(|e| {
        convert_validation_errors(
            e,
            "organization_hierarchy_handler::export_organization_data",
        )
    })?;

    // 権限チェック（組織管理者以上）
    user.ensure_can_manage_organization(organization_id)?;

    let export_data = OrganizationHierarchyService::export_organization_data(
        &app_state.db_pool,
        organization_id,
        export_options.include_analytics,
        export_options.include_permissions,
    )
    .await?;

    let api_response = ApiResponse::success(export_data);
    Ok(api_response)
}

// 部門メンバーの追加
pub async fn add_department_member(
    State(app_state): State<crate::api::AppState>,
    ValidatedMultiPath(params): ValidatedMultiPath<OrganizationDepartmentPath>,
    user: AuthenticatedUser,
    Json(payload): Json<AddDepartmentMemberDto>,
) -> Result<impl IntoResponse, AppError> {
    // バリデーション
    payload.validate().map_err(|e| {
        convert_validation_errors(e, "organization_hierarchy_handler::add_department_member")
    })?;

    // 権限チェック（組織管理者またはその部門のマネージャー）
    user.ensure_can_manage_organization_or_department(
        params.organization_id,
        params.department_id,
    )?;

    let member = OrganizationHierarchyService::add_department_member(
        &app_state.db_pool,
        params.department_id,
        payload.user_id,
        payload.role,
        user.user_id(),
    )
    .await?;

    let response_data = DepartmentMemberResponseDto::from(member);
    let api_response = ApiResponse::success(response_data);
    Ok(api_response)
}

// 部門メンバーの削除
pub async fn remove_department_member(
    State(app_state): State<crate::api::AppState>,
    ValidatedMultiPath(params): ValidatedMultiPath<OrganizationDepartmentUserPath>,
    user: AuthenticatedUser,
) -> Result<impl IntoResponse, AppError> {
    // 権限チェック（組織管理者またはその部門のマネージャー）
    user.ensure_can_manage_organization_or_department(
        params.organization_id,
        params.department_id,
    )?;

    OrganizationHierarchyService::remove_department_member(
        &app_state.db_pool,
        params.department_id,
        params.user_id,
    )
    .await?;

    let response_data = OperationResult::new(
        format!(
            "Department member {} removed from department {}",
            params.user_id, params.department_id
        ),
        vec!["Department member removed".to_string()],
    );

    let api_response = ApiResponse::success(response_data);
    Ok(api_response)
}

// 分析メトリクスの作成
pub async fn create_analytics_metric(
    State(app_state): State<crate::api::AppState>,
    ValidatedUuid(organization_id): ValidatedUuid,
    user: AuthenticatedUser,
    Json(payload): Json<CreateAnalyticsMetricDto>,
) -> Result<impl IntoResponse, AppError> {
    // バリデーション
    payload.validate().map_err(|e| {
        convert_validation_errors(e, "organization_hierarchy_handler::create_analytics_metric")
    })?;

    // 権限チェック（組織管理者以上）
    user.ensure_can_manage_organization(organization_id)?;

    let metric = OrganizationHierarchyService::create_analytics_metric(
        &app_state.db_pool,
        organization_id,
        payload.department_id,
        payload.analytics_type,
        payload.metric_name,
        payload.metric_value,
        payload.period,
        payload.period_start.into(),
        payload.period_end.into(),
        user.user_id(),
    )
    .await?;

    let response_data = OrganizationAnalyticsResponseDto::from(metric);
    let api_response = ApiResponse::success(response_data);
    Ok(api_response)
}

// NOTE: ルーターの定義はorganization_handler.rsに統合されました
// 組織階層関連のルートは/organizations/{id}/*の競合を避けるため
// organization_router_with_state()で定義されています

/// 部門のフラットリストから階層構造を構築するヘルパー関数
fn build_department_hierarchy(
    departments: Vec<DepartmentResponseDto>,
) -> Vec<DepartmentHierarchyDto> {
    use std::collections::HashMap;

    // 部門IDから部門データへのマップを作成
    let mut dept_map: HashMap<Uuid, DepartmentResponseDto> = departments
        .into_iter()
        .map(|dept| (dept.id, dept))
        .collect();

    // 階層構造を構築
    let mut hierarchy: Vec<DepartmentHierarchyDto> = Vec::new();
    let mut children_map: HashMap<Option<Uuid>, Vec<DepartmentHierarchyDto>> = HashMap::new();

    // 最初にすべての部門を親IDでグループ化
    for (_, dept) in dept_map.drain() {
        let hierarchy_dto = DepartmentHierarchyDto {
            department: dept.clone(),
            children: Vec::new(),
            member_count: None, // 実装時にメンバー数を取得する場合は設定
        };

        children_map
            .entry(dept.parent_department_id)
            .or_default()
            .push(hierarchy_dto);
    }

    // ルート部門（parent_department_id が None）を取得
    if let Some(root_depts) = children_map.remove(&None) {
        hierarchy = root_depts;
    }

    // 再帰的に子部門を追加
    fn add_children(
        dept: &mut DepartmentHierarchyDto,
        children_map: &mut HashMap<Option<Uuid>, Vec<DepartmentHierarchyDto>>,
    ) {
        if let Some(mut children) = children_map.remove(&Some(dept.department.id)) {
            for child in &mut children {
                add_children(child, children_map);
            }
            dept.children = children;
        }
    }

    // 各ルート部門に子部門を追加
    for dept in &mut hierarchy {
        add_children(dept, &mut children_map);
    }

    hierarchy
}
