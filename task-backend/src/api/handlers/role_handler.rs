// task-backend/src/api/handlers/role_handler.rs
use crate::api::dto::common::{PaginatedResponse, PaginationQuery};
use crate::api::dto::role_dto::*;
use crate::api::dto::OperationResult;
use crate::api::AppState;
use crate::error::{AppError, AppResult};
use crate::middleware::auth::AuthenticatedUserWithRole;
use crate::types::{ApiResponse, SortQuery};
use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use serde_json;
use tracing::{info, warn};
use uuid::Uuid;
use validator::Validate;

// --- クエリパラメータ ---

/// ロール検索用クエリパラメータ
#[derive(Debug, Deserialize, Validate)]
pub struct RoleSearchQuery {
    #[serde(flatten)]
    pub pagination: PaginationQuery,
    #[serde(flatten)]
    pub sort: SortQuery,
    pub active_only: Option<bool>,
}

// --- カスタム抽出器 ---

/// UUID パス抽出器
pub struct UuidPath(pub Uuid);

impl<S> axum::extract::FromRequestParts<S> for UuidPath
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let Path(path_str) = Path::<String>::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::BadRequest("Invalid path parameter".to_string()))?;

        let uuid = Uuid::parse_str(&path_str)
            .map_err(|_| AppError::BadRequest(format!("Invalid UUID format: '{}'", path_str)))?;

        Ok(UuidPath(uuid))
    }
}

// --- ロール管理ハンドラー ---

/// ロール一覧取得
pub async fn list_roles_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUserWithRole,
    Query(query): Query<RoleSearchQuery>,
) -> AppResult<ApiResponse<PaginatedResponse<RoleResponse>>> {
    // 権限チェックはミドルウェアで実施済み

    let (page, per_page) = query.pagination.get_pagination();

    info!(
        admin_id = %user.user_id(),
        active_only = ?query.active_only,
        page = %page,
        per_page = %per_page,
        sort_by = ?query.sort.sort_by,
        sort_order = ?query.sort.sort_order,
        "Fetching roles list"
    );

    // ロール一覧を取得（ページネーション対応）
    let (roles, total_count) = if query.active_only.unwrap_or(false) {
        app_state
            .role_service
            .list_active_roles_paginated(page, per_page)
            .await?
    } else {
        app_state
            .role_service
            .list_all_roles_paginated(page, per_page)
            .await?
    };

    // RoleResponseに変換
    let role_responses: Vec<RoleResponse> = roles.into_iter().map(RoleResponse::from).collect();

    info!(
        admin_id = %user.user_id(),
        roles_count = %role_responses.len(),
        total_count = %total_count,
        "Roles list retrieved successfully"
    );

    let response = PaginatedResponse::new(role_responses, page, per_page, total_count as i64);
    Ok(ApiResponse::success(response))
}

/// 特定ロール取得
pub async fn get_role_handler(
    State(app_state): State<AppState>,
    UuidPath(role_id): UuidPath,
    user: AuthenticatedUserWithRole,
) -> AppResult<ApiResponse<RoleResponse>> {
    // RoleWithPermissionsのcan_view_resourceメソッドを活用
    if let Some(role) = user.role() {
        if !role.can_view_resource("role", Some(role_id), user.user_id()) {
            warn!(
                user_id = %user.user_id(),
                role_id = %role_id,
                "Insufficient permissions to view role"
            );
            return Err(AppError::Forbidden("Cannot view this role".to_string()));
        }
    } else {
        return Err(AppError::Forbidden("No role assigned".to_string()));
    }

    info!(
        admin_id = %user.user_id(),
        role_id = %role_id,
        "Fetching role details"
    );

    let role = app_state.role_service.get_role_by_id(role_id).await?;

    info!(
        admin_id = %user.user_id(),
        role_id = %role_id,
        role_name = %role.name,
        "Role details retrieved successfully"
    );

    Ok(ApiResponse::success(RoleResponse::from(role)))
}

/// ロール作成
pub async fn create_role_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUserWithRole,
    Json(mut payload): Json<CreateRoleRequest>,
) -> AppResult<impl IntoResponse> {
    // 権限チェックはミドルウェアで実施済み

    // バリデーションとサニタイズ
    payload
        .validate_and_sanitize()
        .map_err(|validation_errors| {
            warn!("Role creation validation failed: {}", validation_errors);
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
            AppError::BadRequest(errors.join(", "))
        })?;

    info!(
        admin_id = %user.user_id(),
        role_name = %payload.name,
        "Creating new role"
    );

    // ロール作成
    let created_role = app_state
        .role_service
        .create_role(&user.claims, payload.into_service_input())
        .await?;

    info!(
        admin_id = %user.user_id(),
        role_id = %created_role.id,
        role_name = %created_role.name,
        "Role created successfully"
    );

    Ok((
        StatusCode::CREATED,
        ApiResponse::success(CreateRoleResponse::build(created_role)),
    ))
}

/// ロール更新
pub async fn update_role_handler(
    State(app_state): State<AppState>,
    UuidPath(role_id): UuidPath,
    user: AuthenticatedUserWithRole,
    Json(mut payload): Json<UpdateRoleRequest>,
) -> AppResult<ApiResponse<OperationResult<RoleResponse>>> {
    // 権限チェックはミドルウェアで実施済み

    // 更新内容があるかチェック
    if !payload.has_updates() {
        return Err(AppError::BadRequest(
            "At least one field must be provided for update".to_string(),
        ));
    }

    // バリデーションとサニタイズ
    payload
        .validate_and_sanitize()
        .map_err(|validation_errors| {
            warn!("Role update validation failed: {}", validation_errors);
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
            AppError::BadRequest(errors.join(", "))
        })?;

    info!(
        admin_id = %user.user_id(),
        role_id = %role_id,
        "Updating role"
    );

    // 変更内容を記録
    let mut changes = Vec::new();
    if payload.name.is_some() {
        changes.push("name".to_string());
    }
    if payload.display_name.is_some() {
        changes.push("display_name".to_string());
    }
    if payload.description.is_some() {
        changes.push("description".to_string());
    }
    if payload.is_active.is_some() {
        changes.push("is_active".to_string());
    }

    // ロール更新
    let updated_role = app_state
        .role_service
        .update_role(&user.claims, role_id, payload.into_service_input())
        .await?;

    info!(
        admin_id = %user.user_id(),
        role_id = %role_id,
        role_name = %updated_role.name,
        changes = ?changes,
        "Role updated successfully"
    );

    Ok(ApiResponse::success(OperationResult::updated(
        RoleResponse::from(updated_role),
        changes,
    )))
}

/// ロール削除
pub async fn delete_role_handler(
    State(app_state): State<AppState>,
    UuidPath(role_id): UuidPath,
    user: AuthenticatedUserWithRole,
) -> AppResult<ApiResponse<serde_json::Value>> {
    // 権限チェックはミドルウェアで実施済み

    info!(
        admin_id = %user.user_id(),
        role_id = %role_id,
        "Deleting role"
    );

    // ロール削除
    app_state
        .role_service
        .delete_role(&user.claims, role_id)
        .await?;

    info!(
        admin_id = %user.user_id(),
        role_id = %role_id,
        "Role deleted successfully"
    );

    Ok(ApiResponse::success(
        serde_json::json!({ "deleted_role_id": role_id }),
    ))
}

// --- ユーザーロール管理ハンドラー ---

/// ユーザーにロール割り当て
pub async fn assign_role_to_user_handler(
    State(app_state): State<AppState>,
    Path(user_id): Path<Uuid>,
    user: AuthenticatedUserWithRole,
    Json(payload): Json<AssignRoleRequest>,
) -> AppResult<ApiResponse<AssignRoleResponse>> {
    // 権限チェックはミドルウェアで実施済み

    info!(
        admin_id = %user.user_id(),
        target_user_id = %user_id,
        role_id = %payload.role_id,
        "Assigning role to user"
    );

    // ロール割り当て
    let user_with_role = app_state
        .role_service
        .assign_role_to_user(&user.claims, user_id, payload.role_id)
        .await?;

    info!(
        admin_id = %user.user_id(),
        target_user_id = %user_id,
        role_id = %payload.role_id,
        role_name = %user_with_role.role.name,
        "Role assigned to user successfully"
    );

    Ok(ApiResponse::success(AssignRoleResponse {
        message: "Role assigned successfully".to_string(),
        user_id,
        role: RoleResponse::from(user_with_role.role),
    }))
}

// --- 統計情報ハンドラー ---

// --- ヘルスチェック ---

// --- 追加エンドポイント ---

// --- ルーター ---

/// ロールルーターを作成
pub fn role_router(app_state: AppState) -> Router {
    use crate::middleware::authorization::{resources, Action};
    use crate::require_permission;

    let router = Router::new()
        // ロール管理
        .route(
            "/admin/roles",
            get(list_roles_handler)
                .route_layer(require_permission!(resources::ROLE, Action::View))
                .post(create_role_handler)
                .route_layer(require_permission!(resources::ROLE, Action::Create)),
        )
        .route(
            "/admin/roles/{id}",
            get(get_role_handler)
                .route_layer(require_permission!(resources::ROLE, Action::View))
                .patch(update_role_handler)
                .route_layer(require_permission!(resources::ROLE, Action::Update))
                .delete(delete_role_handler)
                .route_layer(require_permission!(resources::ROLE, Action::Delete)),
        )
        // ユーザーロール管理
        .route(
            "/admin/users/{id}/role",
            post(assign_role_to_user_handler)
                .route_layer(require_permission!(resources::ROLE, Action::Admin)),
        );

    router.with_state(app_state)
}

/// ロールルーターをAppStateから作成
pub fn role_router_with_state(app_state: AppState) -> Router {
    role_router(app_state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_search_query() {
        // 正常なクエリパラメータ
        let query = RoleSearchQuery {
            pagination: PaginationQuery::default(),
            sort: SortQuery::default(),
            active_only: Some(true),
        };

        assert!(query.validate().is_ok());
        assert_eq!(query.active_only, Some(true));
    }

    #[test]
    fn test_uuid_path_parsing() {
        let valid_uuid = "550e8400-e29b-41d4-a716-446655440000";
        let uuid = Uuid::parse_str(valid_uuid).unwrap();
        let uuid_path = UuidPath(uuid);

        assert_eq!(uuid_path.0, uuid);
    }

    #[test]
    fn test_assign_role_request() {
        let request = AssignRoleRequest {
            role_id: Uuid::new_v4(),
        };

        // AssignRoleRequestには現在バリデーションはないが、
        // 将来的にバリデーションが追加される可能性がある
        assert!(!request.role_id.to_string().is_empty());
    }
}
