// task-backend/src/api/handlers/gdpr_handler.rs

use crate::api::dto::gdpr_dto::{
    ComplianceStatusResponse, ConsentHistoryResponse, ConsentStatusResponse, ConsentUpdateRequest,
    DataDeletionRequest, DataDeletionResponse, DataExportRequest, DataExportResponse,
    SingleConsentUpdateRequest,
};
use crate::api::AppState;
use crate::error::AppResult;
use crate::middleware::auth::{AuthenticatedUser, AuthenticatedUserWithRole};
use crate::middleware::authorization::{resources, Action};
use crate::require_permission;
use crate::service::gdpr_service::GdprService;
use crate::types::ApiResponse;
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
) -> AppResult<ApiResponse<DataExportResponse>> {
    // Check if user is accessing their own data or if they're an admin
    if user.user_id() != user_id && !user.is_admin() {
        return Err(crate::error::AppError::Forbidden(
            "You can only export your own data".to_string(),
        ));
    }

    let gdpr_service = Arc::new(GdprService::new((*app_state.db).clone()));
    let export_data = gdpr_service.export_user_data(user_id, request).await?;

    Ok(ApiResponse::success(export_data))
}

/// Delete user data (user can delete their own data, admin can delete any user's data)
pub async fn delete_user_data_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Path(user_id): Path<Uuid>,
    Json(request): Json<DataDeletionRequest>,
) -> AppResult<ApiResponse<DataDeletionResponse>> {
    // Check if user is deleting their own data or if they're an admin
    if user.user_id() != user_id && !user.is_admin() {
        return Err(crate::error::AppError::Forbidden(
            "You can only delete your own data".to_string(),
        ));
    }

    let gdpr_service = Arc::new(GdprService::new((*app_state.db).clone()));
    let deletion_result = gdpr_service.delete_user_data(user_id, request).await?;

    Ok(ApiResponse::success(deletion_result))
}

/// Get GDPR compliance status (user can check their own status, admin can check any user's status)
pub async fn get_compliance_status_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Path(user_id): Path<Uuid>,
) -> AppResult<ApiResponse<ComplianceStatusResponse>> {
    // Check if user is checking their own status or if they're an admin
    if user.user_id() != user_id && !user.is_admin() {
        return Err(crate::error::AppError::Forbidden(
            "You can only check your own compliance status".to_string(),
        ));
    }

    let gdpr_service = Arc::new(GdprService::new((*app_state.db).clone()));
    let status = gdpr_service.get_compliance_status(user_id).await?;

    Ok(ApiResponse::success(status))
}

/// Admin endpoint to export any user's data
pub async fn admin_export_user_data_handler(
    State(app_state): State<AppState>,
    _admin: AuthenticatedUserWithRole,
    Path(user_id): Path<Uuid>,
    Json(request): Json<DataExportRequest>,
) -> AppResult<ApiResponse<DataExportResponse>> {
    let gdpr_service = Arc::new(GdprService::new((*app_state.db).clone()));
    let export_data = gdpr_service.export_user_data(user_id, request).await?;

    Ok(ApiResponse::success(export_data))
}

/// Admin endpoint to delete any user's data
pub async fn admin_delete_user_data_handler(
    State(app_state): State<AppState>,
    _admin: AuthenticatedUserWithRole,
    Path(user_id): Path<Uuid>,
    Json(request): Json<DataDeletionRequest>,
) -> AppResult<ApiResponse<DataDeletionResponse>> {
    let gdpr_service = Arc::new(GdprService::new((*app_state.db).clone()));
    let deletion_result = gdpr_service.delete_user_data(user_id, request).await?;

    Ok(ApiResponse::success(deletion_result))
}

// --- Router ---

use axum::{
    routing::{delete, get, patch, post},
    Router,
};

/// Get user consent status
pub async fn get_consent_status_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Path(user_id): Path<Uuid>,
) -> AppResult<ApiResponse<ConsentStatusResponse>> {
    // Check if user is accessing their own data or if they're an admin
    if user.user_id() != user_id && !user.is_admin() {
        return Err(crate::error::AppError::Forbidden(
            "You can only view your own consent status".to_string(),
        ));
    }

    let gdpr_service = Arc::new(GdprService::new((*app_state.db).clone()));
    let status = gdpr_service.get_consent_status(user_id).await?;

    Ok(ApiResponse::success(status))
}

/// Update user consents
pub async fn update_consents_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Path(user_id): Path<Uuid>,
    Json(request): Json<ConsentUpdateRequest>,
) -> AppResult<ApiResponse<ConsentStatusResponse>> {
    // Users can only update their own consents
    if user.user_id() != user_id {
        return Err(crate::error::AppError::Forbidden(
            "You can only update your own consents".to_string(),
        ));
    }

    let gdpr_service = Arc::new(GdprService::new((*app_state.db).clone()));
    let status = gdpr_service
        .update_consents(user_id, request, None, None)
        .await?;

    Ok(ApiResponse::success(status))
}

/// Update single consent
pub async fn update_single_consent_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Path(user_id): Path<Uuid>,
    Json(request): Json<SingleConsentUpdateRequest>,
) -> AppResult<ApiResponse<ConsentStatusResponse>> {
    // Users can only update their own consents
    if user.user_id() != user_id {
        return Err(crate::error::AppError::Forbidden(
            "You can only update your own consents".to_string(),
        ));
    }

    let gdpr_service = Arc::new(GdprService::new((*app_state.db).clone()));
    let status = gdpr_service
        .update_single_consent(user_id, request, None, None)
        .await?;

    Ok(ApiResponse::success(status))
}

/// Get consent history
pub async fn get_consent_history_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Path(user_id): Path<Uuid>,
) -> AppResult<ApiResponse<ConsentHistoryResponse>> {
    // Check if user is accessing their own data or if they're an admin
    if user.user_id() != user_id && !user.is_admin() {
        return Err(crate::error::AppError::Forbidden(
            "You can only view your own consent history".to_string(),
        ));
    }

    let gdpr_service = Arc::new(GdprService::new((*app_state.db).clone()));
    let history = gdpr_service.get_consent_history(user_id, Some(100)).await?;

    Ok(ApiResponse::success(history))
}

/// GDPR router
pub fn gdpr_router(app_state: AppState) -> Router {
    Router::new()
        // User-accessible endpoints
        .route(
            "/gdpr/users/{user_id}/export",
            post(export_user_data_handler),
        )
        .route(
            "/gdpr/users/{user_id}/delete",
            delete(delete_user_data_handler),
        )
        .route(
            "/gdpr/users/{user_id}/status",
            get(get_compliance_status_handler),
        )
        // Consent endpoints
        .route(
            "/gdpr/users/{user_id}/consents",
            get(get_consent_status_handler).put(update_consents_handler),
        )
        .route(
            "/gdpr/users/{user_id}/consents/single",
            patch(update_single_consent_handler),
        )
        .route(
            "/gdpr/users/{user_id}/consents/history",
            get(get_consent_history_handler),
        )
        // Admin endpoints
        .route(
            "/admin/gdpr/users/{user_id}/export",
            post(admin_export_user_data_handler)
                .route_layer(require_permission!(resources::GDPR, Action::Admin)),
        )
        .route(
            "/admin/gdpr/users/{user_id}/delete",
            delete(admin_delete_user_data_handler)
                .route_layer(require_permission!(resources::GDPR, Action::Admin)),
        )
        .with_state(app_state)
}

/// GDPR router with state
pub fn gdpr_router_with_state(app_state: AppState) -> Router {
    gdpr_router(app_state)
}
