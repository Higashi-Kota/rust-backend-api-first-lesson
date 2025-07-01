// task-backend/src/api/handlers/gdpr_handler.rs

use crate::api::dto::common::ApiResponse;
use crate::api::dto::gdpr_dto::{
    ComplianceStatusResponse, DataDeletionRequest, DataDeletionResponse, DataExportRequest,
    DataExportResponse,
};
use crate::api::AppState;
use crate::error::AppResult;
use crate::middleware::auth::{AuthenticatedUser, AuthenticatedUserWithRole};
use crate::service::gdpr_service::GdprService;
use axum::{
    extract::{Path, State},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

/// Export user data (user can export their own data, admin can export any user's data)
pub async fn export_user_data_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Path(user_id): Path<Uuid>,
    Json(request): Json<DataExportRequest>,
) -> AppResult<Json<ApiResponse<DataExportResponse>>> {
    // Check if user is accessing their own data or if they're an admin
    if user.user_id() != user_id && !user.is_admin() {
        return Err(crate::error::AppError::Forbidden(
            "You can only export your own data".to_string(),
        ));
    }

    let gdpr_service = Arc::new(GdprService::new((*app_state.db).clone()));
    let export_data = gdpr_service.export_user_data(user_id, request).await?;

    Ok(Json(ApiResponse::success(
        "User data exported successfully",
        export_data,
    )))
}

/// Delete user data (user can delete their own data, admin can delete any user's data)
pub async fn delete_user_data_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Path(user_id): Path<Uuid>,
    Json(request): Json<DataDeletionRequest>,
) -> AppResult<Json<ApiResponse<DataDeletionResponse>>> {
    // Check if user is deleting their own data or if they're an admin
    if user.user_id() != user_id && !user.is_admin() {
        return Err(crate::error::AppError::Forbidden(
            "You can only delete your own data".to_string(),
        ));
    }

    let gdpr_service = Arc::new(GdprService::new((*app_state.db).clone()));
    let deletion_result = gdpr_service.delete_user_data(user_id, request).await?;

    Ok(Json(ApiResponse::success(
        "User data deleted successfully",
        deletion_result,
    )))
}

/// Get GDPR compliance status (user can check their own status, admin can check any user's status)
pub async fn get_compliance_status_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Path(user_id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<ComplianceStatusResponse>>> {
    // Check if user is checking their own status or if they're an admin
    if user.user_id() != user_id && !user.is_admin() {
        return Err(crate::error::AppError::Forbidden(
            "You can only check your own compliance status".to_string(),
        ));
    }

    let gdpr_service = Arc::new(GdprService::new((*app_state.db).clone()));
    let status = gdpr_service.get_compliance_status(user_id).await?;

    Ok(Json(ApiResponse::success(
        "Compliance status retrieved successfully",
        status,
    )))
}

/// Admin endpoint to export any user's data
pub async fn admin_export_user_data_handler(
    State(app_state): State<AppState>,
    _admin: AuthenticatedUserWithRole,
    Path(user_id): Path<Uuid>,
    Json(request): Json<DataExportRequest>,
) -> AppResult<Json<ApiResponse<DataExportResponse>>> {
    let gdpr_service = Arc::new(GdprService::new((*app_state.db).clone()));
    let export_data = gdpr_service.export_user_data(user_id, request).await?;

    Ok(Json(ApiResponse::success(
        "User data exported successfully",
        export_data,
    )))
}

/// Admin endpoint to delete any user's data
pub async fn admin_delete_user_data_handler(
    State(app_state): State<AppState>,
    _admin: AuthenticatedUserWithRole,
    Path(user_id): Path<Uuid>,
    Json(request): Json<DataDeletionRequest>,
) -> AppResult<Json<ApiResponse<DataDeletionResponse>>> {
    let gdpr_service = Arc::new(GdprService::new((*app_state.db).clone()));
    let deletion_result = gdpr_service.delete_user_data(user_id, request).await?;

    Ok(Json(ApiResponse::success(
        "User data deleted successfully",
        deletion_result,
    )))
}

// --- Router ---

use axum::{
    routing::{delete, get, post},
    Router,
};

/// GDPR router
pub fn gdpr_router(app_state: AppState) -> Router {
    Router::new()
        // User-accessible endpoints
        .route("/gdpr/users/{id}/export", post(export_user_data_handler))
        .route("/gdpr/users/{id}/delete", delete(delete_user_data_handler))
        .route(
            "/gdpr/users/{id}/status",
            get(get_compliance_status_handler),
        )
        // Admin endpoints
        .route(
            "/admin/gdpr/users/{id}/export",
            post(admin_export_user_data_handler),
        )
        .route(
            "/admin/gdpr/users/{id}/delete",
            delete(admin_delete_user_data_handler),
        )
        .with_state(app_state)
}

/// GDPR router with state
pub fn gdpr_router_with_state(app_state: AppState) -> Router {
    gdpr_router(app_state)
}
