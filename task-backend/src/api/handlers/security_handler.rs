// task-backend/src/api/handlers/security_handler.rs

use crate::api::dto::security_dto::*;
use crate::api::AppState;
use crate::error::{AppError, AppResult};
use crate::middleware::auth::AuthenticatedUserWithRole;
use crate::types::ApiResponse;
use axum::{
    extract::{Json, State},
    routing::{get, post},
    Router,
};
use tracing::info;
use validator::Validate;

/// トークン利用統計取得（管理者用）
pub async fn get_token_stats_handler(
    State(_app_state): State<AppState>,
    admin_user: AuthenticatedUserWithRole,
) -> AppResult<ApiResponse<TokenStatsResponse>> {
    info!(
        admin_id = %admin_user.user_id(),
        "Token stats request"
    );

    // セキュリティサービスから実際の統計を取得
    let refresh_token_stats = _app_state
        .security_service
        .get_refresh_token_stats()
        .await?;
    let password_reset_stats = _app_state
        .security_service
        .get_password_reset_stats()
        .await?;

    let response = TokenStatsResponse {
        refresh_tokens: refresh_token_stats,
        password_reset_tokens: password_reset_stats,
        message: "Token statistics retrieved successfully".to_string(),
    };

    Ok(ApiResponse::success(response))
}

/// リフレッシュトークン監視（管理者用）
pub async fn get_refresh_tokens_handler(
    State(_app_state): State<AppState>,
    _admin_user: AuthenticatedUserWithRole,
) -> AppResult<ApiResponse<RefreshTokenMonitorResponse>> {
    info!(
        admin_id = %_admin_user.user_id(),
        "Refresh token monitoring request"
    );

    // アクティブなトークン概要はシンプルなメッセージに変更（実装省略）
    let active_tokens: Vec<crate::api::dto::security_dto::ActiveTokenSummary> = vec![];

    let response = RefreshTokenMonitorResponse {
        active_tokens,
        message: "Refresh token monitoring data retrieved successfully".to_string(),
    };

    Ok(ApiResponse::success(response))
}

/// 期限切れトークン自動削除（管理者用）
pub async fn cleanup_tokens_handler(
    State(_app_state): State<AppState>,
    _admin_user: AuthenticatedUserWithRole,
    Json(payload): Json<CleanupTokensRequest>,
) -> AppResult<ApiResponse<CleanupTokensResponse>> {
    info!(
        admin_id = %_admin_user.user_id(),
        cleanup_type = ?payload.cleanup_type,
        "Token cleanup request"
    );

    let cleanup_result = match payload.cleanup_type.as_str() {
        "refresh_tokens" => {
            _app_state
                .security_service
                .cleanup_expired_refresh_tokens()
                .await?
        }
        "password_reset_tokens" => {
            _app_state
                .security_service
                .cleanup_expired_password_reset_tokens()
                .await?
        }
        "all" => {
            let refresh_result = _app_state
                .security_service
                .cleanup_expired_refresh_tokens()
                .await?;
            let password_result = _app_state
                .security_service
                .cleanup_expired_password_reset_tokens()
                .await?;
            CleanupResult {
                deleted_count: refresh_result.deleted_count + password_result.deleted_count,
                cleanup_type: "all".to_string(),
            }
        }
        _ => {
            return Err(AppError::BadRequest(
                "Invalid cleanup type. Use 'refresh_tokens', 'password_reset_tokens', or 'all'"
                    .to_string(),
            ));
        }
    };

    let response = CleanupTokensResponse {
        result: cleanup_result,
        message: "Token cleanup completed successfully".to_string(),
    };

    Ok(ApiResponse::success(response))
}

/// パスワードリセット監視（管理者用）
pub async fn get_password_resets_handler(
    State(_app_state): State<AppState>,
    _admin_user: AuthenticatedUserWithRole,
) -> AppResult<ApiResponse<PasswordResetMonitorResponse>> {
    info!(
        admin_id = %_admin_user.user_id(),
        "Password reset monitoring request"
    );

    // セキュリティサービスから実際のパスワードリセット活動を取得
    let recent_activity = _app_state
        .security_service
        .get_recent_password_reset_activity()
        .await?;

    let response = PasswordResetMonitorResponse {
        recent_activity,
        message: "Password reset monitoring data retrieved successfully".to_string(),
    };

    Ok(ApiResponse::success(response))
}

/// 緊急時全トークン無効化（管理者用）
pub async fn revoke_all_tokens_handler(
    State(_app_state): State<AppState>,
    admin_user: AuthenticatedUserWithRole,
    Json(payload): Json<RevokeAllTokensRequest>,
) -> AppResult<ApiResponse<RevokeAllTokensResponse>> {
    info!(
        admin_id = %admin_user.user_id(),
        target_user_id = ?payload.user_id,
        reason = %payload.reason,
        exclude_current = payload.exclude_current_user,
        "Token revocation request"
    );

    // リクエストを検証
    if let Err(validation_errors) = payload.validate() {
        return Err(AppError::BadRequest(format!(
            "Invalid revoke request: {:?}",
            validation_errors
        )));
    }

    // セキュリティサービスで全トークン無効化を実行
    let current_user_id = if payload.exclude_current_user {
        Some(admin_user.user_id())
    } else {
        None
    };

    let revoke_result = _app_state
        .security_service
        .revoke_all_tokens(&payload, current_user_id)
        .await?;

    let response = RevokeAllTokensResponse {
        result: revoke_result,
        message: "Token revocation completed successfully".to_string(),
    };

    Ok(ApiResponse::success(response))
}

/// セッション分析（管理者用）
pub async fn get_session_analytics_handler(
    State(_app_state): State<AppState>,
    _admin_user: AuthenticatedUserWithRole,
) -> AppResult<ApiResponse<SessionAnalyticsResponse>> {
    info!(
        admin_id = %_admin_user.user_id(),
        "Session analytics request"
    );

    // セキュリティサービスからセッション分析を取得
    let analytics = _app_state.security_service.get_session_analytics().await?;

    let response = SessionAnalyticsResponse {
        analytics,
        message: "Session analytics retrieved successfully".to_string(),
    };

    Ok(ApiResponse::success(response))
}

/// セキュリティ監査レポート生成（管理者用）
pub async fn generate_audit_report_handler(
    State(_app_state): State<AppState>,
    admin_user: AuthenticatedUserWithRole,
    Json(payload): Json<AuditReportRequest>,
) -> AppResult<ApiResponse<AuditReportResponse>> {
    info!(
        admin_id = %admin_user.user_id(),
        report_type = %payload.report_type,
        "Audit report generation request"
    );

    // リクエストを検証
    if let Err(validation_errors) = payload.validate() {
        return Err(AppError::BadRequest(format!(
            "Invalid audit report request: {:?}",
            validation_errors
        )));
    }

    // セキュリティサービスで監査レポートを生成
    let audit_report = _app_state
        .security_service
        .generate_audit_report(&payload, admin_user.user_id())
        .await?;

    let response = AuditReportResponse {
        report: audit_report,
        message: "Audit report generated successfully".to_string(),
    };

    Ok(ApiResponse::success(response))
}

/// セキュリティ管理ルーターを作成
pub fn security_router(app_state: AppState) -> Router {
    use crate::middleware::authorization::{resources, Action};
    use crate::require_permission;

    Router::new()
        // Phase 1.2 セキュリティ・トークン管理 API
        .route(
            "/admin/security/token-stats",
            get(get_token_stats_handler)
                .route_layer(require_permission!(resources::SECURITY, Action::Admin)),
        )
        .route(
            "/admin/security/refresh-tokens",
            get(get_refresh_tokens_handler)
                .route_layer(require_permission!(resources::SECURITY, Action::Admin)),
        )
        .route(
            "/admin/security/cleanup-tokens",
            post(cleanup_tokens_handler)
                .route_layer(require_permission!(resources::SECURITY, Action::Admin)),
        )
        .route(
            "/admin/security/password-resets",
            get(get_password_resets_handler)
                .route_layer(require_permission!(resources::SECURITY, Action::Admin)),
        )
        // 新規実装: Phase 1.2 残り3エンドポイント
        .route(
            "/admin/security/revoke-all-tokens",
            post(revoke_all_tokens_handler)
                .route_layer(require_permission!(resources::SECURITY, Action::Admin)),
        )
        .route(
            "/admin/security/session-analytics",
            get(get_session_analytics_handler)
                .route_layer(require_permission!(resources::SECURITY, Action::Admin)),
        )
        .route(
            "/admin/security/audit-report",
            post(generate_audit_report_handler)
                .route_layer(require_permission!(resources::SECURITY, Action::Admin)),
        )
        .with_state(app_state)
}
