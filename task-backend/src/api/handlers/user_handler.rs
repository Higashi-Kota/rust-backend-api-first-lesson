// task-backend/src/api/handlers/user_handler.rs
use crate::api::dto::user_dto::*;
use crate::api::handlers::auth_handler::AuthenticatedUser;
use crate::api::AppState;
use crate::domain::user_model::UserClaims;
use crate::error::{AppError, AppResult};
use crate::middleware::auth::AuthenticatedUserWithRole;
use axum::{
    extract::{FromRequestParts, Json, Path, Query, State},
    http::{request::Parts, StatusCode},
    response::IntoResponse,
    routing::{get, patch, post},
    Router,
};
use tracing::{info, warn};
use uuid::Uuid;
use validator::Validate;

// カスタムUUID抽出器
pub struct UuidPath(pub Uuid);

impl<S> FromRequestParts<S> for UuidPath
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // パスパラメータを文字列として最初に抽出
        let Path(path_str) = Path::<String>::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::ValidationErrors(vec!["Invalid path parameter".to_string()]))?;

        // UUIDをパースして検証エラー形式で返す
        let uuid = Uuid::parse_str(&path_str).map_err(|_| {
            AppError::ValidationErrors(vec![format!("Invalid UUID format: '{}'", path_str)])
        })?;

        Ok(UuidPath(uuid))
    }
}

// --- ユーザープロフィール管理 ---

/// ユーザープロフィール取得
pub async fn get_profile_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
) -> AppResult<Json<UserProfileResponse>> {
    let user_profile = app_state
        .user_service
        .get_user_profile(user.0.user_id)
        .await?;

    info!(user_id = %user.0.user_id, "User profile retrieved");

    Ok(Json(UserProfileResponse { user: user_profile }))
}

/// ユーザー名更新
pub async fn update_username_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<UpdateUsernameRequest>,
) -> AppResult<Json<ProfileUpdateResponse>> {
    // バリデーション
    payload.validate().map_err(|validation_errors| {
        warn!("Username update validation failed: {}", validation_errors);
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

    info!(
        user_id = %user.0.user_id,
        new_username = %payload.username,
        "Username update attempt"
    );

    let updated_user = app_state
        .user_service
        .update_username(user.0.user_id, &payload.username)
        .await?;

    info!(
        user_id = %user.0.user_id,
        new_username = %payload.username,
        "Username updated successfully"
    );

    Ok(Json(ProfileUpdateResponse {
        user: updated_user,
        message: "Username updated successfully".to_string(),
        changes: vec!["username".to_string()],
    }))
}

/// メールアドレス更新
pub async fn update_email_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<UpdateEmailRequest>,
) -> AppResult<Json<ProfileUpdateResponse>> {
    // バリデーション
    payload.validate().map_err(|validation_errors| {
        warn!("Email update validation failed: {}", validation_errors);
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

    info!(
        user_id = %user.0.user_id,
        new_email = %payload.email,
        "Email update attempt"
    );

    let updated_user = app_state
        .user_service
        .update_email(user.0.user_id, &payload.email)
        .await?;

    info!(
        user_id = %user.0.user_id,
        new_email = %payload.email,
        "Email updated successfully"
    );

    Ok(Json(ProfileUpdateResponse {
        user: updated_user,
        message: "Email updated successfully. Please verify your new email address".to_string(),
        changes: vec!["email".to_string()],
    }))
}

/// プロフィール一括更新
pub async fn update_profile_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<UpdateProfileRequest>,
) -> AppResult<Json<ProfileUpdateResponse>> {
    // バリデーション
    payload.validate().map_err(|validation_errors| {
        warn!("Profile update validation failed: {}", validation_errors);
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

    // カスタムバリデーション
    payload.validate_update().map_err(|e| {
        warn!("Profile update custom validation failed: {}", e);
        AppError::ValidationErrors(vec![e])
    })?;

    info!(user_id = %user.0.user_id, "Profile update attempt");

    let changes = payload.get_updated_fields();
    let mut updated_user = app_state
        .user_service
        .get_user_profile(user.0.user_id)
        .await?;

    // ユーザー名の更新
    if let Some(new_username) = &payload.username {
        updated_user = app_state
            .user_service
            .update_username(user.0.user_id, new_username)
            .await?;
    }

    // メールアドレスの更新
    if let Some(new_email) = &payload.email {
        updated_user = app_state
            .user_service
            .update_email(user.0.user_id, new_email)
            .await?;
    }

    info!(
        user_id = %user.0.user_id,
        changes = ?changes,
        "Profile updated successfully"
    );

    Ok(Json(ProfileUpdateResponse {
        user: updated_user,
        message: "Profile updated successfully".to_string(),
        changes,
    }))
}

/// ユーザー統計情報取得
pub async fn get_user_stats_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
) -> AppResult<Json<UserStatsResponse>> {
    let stats = app_state
        .user_service
        .get_user_stats(user.0.user_id)
        .await?;

    let additional_info = UserAdditionalInfo::from_user_stats(&stats);

    info!(user_id = %user.0.user_id, "User stats retrieved");

    Ok(Json(UserStatsResponse {
        stats,
        additional_info,
    }))
}

/// メール認証実行
pub async fn verify_email_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<VerifyEmailRequest>,
) -> AppResult<Json<EmailVerificationResponse>> {
    // バリデーション
    payload.validate().map_err(|validation_errors| {
        warn!(
            "Email verification validation failed: {}",
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

    info!(user_id = %user.0.user_id, "Email verification attempt");

    // TODO: トークン検証ロジックを実装
    // 現在はプレースホルダー

    let verified_user = app_state.user_service.verify_email(user.0.user_id).await?;

    info!(user_id = %user.0.user_id, "Email verified successfully");

    Ok(Json(EmailVerificationResponse {
        message: "Email verified successfully".to_string(),
        verified: true,
        user: Some(verified_user),
    }))
}

/// メール認証再送信
pub async fn resend_verification_email_handler(
    State(_app_state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<ResendVerificationEmailRequest>,
) -> AppResult<Json<EmailVerificationResponse>> {
    // バリデーション
    payload.validate().map_err(|validation_errors| {
        warn!(
            "Resend verification email validation failed: {}",
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

    info!(
        user_id = %user.0.user_id,
        email = %payload.email,
        "Verification email resend attempt"
    );

    // TODO: メール送信ロジックを実装

    Ok(Json(EmailVerificationResponse {
        message: "Verification email sent successfully".to_string(),
        verified: false,
        user: None,
    }))
}

/// ユーザー設定取得
pub async fn get_user_settings_handler(user: AuthenticatedUser) -> Json<UserSettingsResponse> {
    // TODO: 実際の設定をデータベースから取得
    // 現在はデフォルト値を返す

    info!(user_id = %user.0.user_id, "User settings retrieved");

    Json(UserSettingsResponse {
        user_id: user.0.user_id,
        preferences: UserPreferences::default(),
        security: SecuritySettings::default(),
        notifications: NotificationSettings::default(),
    })
}

/// アカウント状態更新（管理者用）
pub async fn update_account_status_handler(
    State(app_state): State<AppState>,
    UuidPath(user_id): UuidPath,
    admin_user: AuthenticatedUserWithRole,
    Json(payload): Json<UpdateAccountStatusRequest>,
) -> AppResult<Json<AccountStatusUpdateResponse>> {
    // 管理者権限チェック
    if !admin_user.role().is_admin() {
        warn!(
            user_id = %admin_user.user_id(),
            role = %admin_user.role().name,
            target_user_id = %user_id,
            "Access denied: Admin permission required for account status update"
        );
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    // バリデーション
    payload.validate().map_err(|validation_errors| {
        warn!(
            "Account status update validation failed: {}",
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

    // 自分自身のアカウント状態変更を防ぐ
    if admin_user.user_id() == user_id {
        warn!(
            admin_id = %admin_user.user_id(),
            "Admin attempting to change own account status"
        );
        return Err(AppError::ValidationError(
            "Cannot change your own account status".to_string(),
        ));
    }

    info!(
        admin_id = %admin_user.user_id(),
        target_user_id = %user_id,
        new_status = %payload.is_active,
        "Account status update attempt"
    );

    // 現在の状態を取得
    let current_user = app_state.user_service.get_user_profile(user_id).await?;
    let previous_status = current_user.is_active;

    // 状態を更新
    let updated_user = app_state
        .user_service
        .toggle_account_status(user_id, payload.is_active)
        .await?;

    info!(
        admin_id = %admin_user.user_id(),
        target_user_id = %user_id,
        target_user = %current_user.username,
        previous_status = %previous_status,
        new_status = %payload.is_active,
        "Account status updated successfully"
    );

    Ok(Json(AccountStatusUpdateResponse {
        user: updated_user,
        message: format!(
            "Account {} successfully",
            if payload.is_active {
                "activated"
            } else {
                "deactivated"
            }
        ),
        previous_status,
        new_status: payload.is_active,
    }))
}

// --- 管理者用ユーザー管理 ---

/// ユーザー一覧取得（管理者用）
pub async fn list_users_handler(
    State(app_state): State<AppState>,
    admin_user: AuthenticatedUserWithRole,
    Query(query): Query<UserSearchQuery>,
) -> AppResult<Json<UserListResponse>> {
    // 管理者権限チェック
    if !admin_user.role().is_admin() {
        warn!(
            user_id = %admin_user.user_id(),
            role = %admin_user.role().name,
            "Access denied: Admin permission required for user list"
        );
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    // バリデーション
    query.validate().map_err(|validation_errors| {
        warn!("User search validation failed: {}", validation_errors);
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

    let query_with_defaults = query.with_defaults();

    info!(
        admin_id = %admin_user.user_id(),
        page = ?query_with_defaults.page,
        per_page = ?query_with_defaults.per_page,
        "Admin user list request"
    );

    // ロール情報付きでユーザー一覧を取得
    let (users_with_roles, total_count) = app_state
        .user_service
        .list_users_with_roles_paginated(
            query_with_defaults.page.unwrap_or(1),
            query_with_defaults.per_page.unwrap_or(20),
        )
        .await?;

    let page = query_with_defaults.page.unwrap_or(1);
    let per_page = query_with_defaults.per_page.unwrap_or(20);

    let pagination = PaginationInfo::new(page, per_page, total_count as i64);

    // SafeUserWithRoleをUserSummaryに変換
    let user_summaries: Vec<UserSummary> = users_with_roles
        .into_iter()
        .map(|user_with_role| UserSummary {
            id: user_with_role.id,
            username: user_with_role.username,
            email: user_with_role.email,
            is_active: user_with_role.is_active,
            email_verified: user_with_role.email_verified,
            created_at: user_with_role.created_at,
            last_login_at: user_with_role.last_login_at,
            task_count: 0, // TODO: タスク数を取得
        })
        .collect();

    info!(
        admin_id = %admin_user.user_id(),
        users_count = %user_summaries.len(),
        total_count = %total_count,
        "Admin user list retrieved successfully"
    );

    Ok(Json(UserListResponse {
        users: user_summaries,
        pagination,
        total_count: total_count as i64,
    }))
}

/// 特定ユーザー情報取得（管理者用）
pub async fn get_user_by_id_handler(
    State(app_state): State<AppState>,
    UuidPath(user_id): UuidPath,
    admin_user: AuthenticatedUserWithRole,
) -> AppResult<Json<UserProfileResponse>> {
    // 管理者権限チェック
    if !admin_user.role().is_admin() {
        warn!(
            user_id = %admin_user.user_id(),
            role = %admin_user.role().name,
            target_user_id = %user_id,
            "Access denied: Admin permission required for user profile access"
        );
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    info!(
        admin_id = %admin_user.user_id(),
        target_user_id = %user_id,
        "Admin user profile request"
    );

    // ユーザー情報を取得
    let user_profile = app_state.user_service.get_user_profile(user_id).await?;

    info!(
        admin_id = %admin_user.user_id(),
        target_user_id = %user_id,
        "Admin user profile retrieved successfully"
    );

    Ok(Json(UserProfileResponse { user: user_profile }))
}

/// 最終ログイン時刻更新
pub async fn update_last_login_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
) -> AppResult<impl IntoResponse> {
    app_state
        .user_service
        .update_last_login(user.0.user_id)
        .await?;

    info!(user_id = %user.0.user_id, "Last login time updated");

    Ok(StatusCode::NO_CONTENT)
}

// --- ヘルスチェック ---

/// ユーザーサービスのヘルスチェック
async fn user_health_check_handler() -> &'static str {
    "User service OK"
}

// --- ルーター ---

/// ユーザールーターを作成
pub fn user_router(app_state: AppState) -> Router {
    Router::new()
        // プロフィール管理
        .route("/users/profile", get(get_profile_handler))
        .route("/users/profile/username", patch(update_username_handler))
        .route("/users/profile/email", patch(update_email_handler))
        .route("/users/profile", patch(update_profile_handler))
        .route("/users/stats", get(get_user_stats_handler))
        .route("/users/settings", get(get_user_settings_handler))
        // メール認証
        .route("/users/verify-email", post(verify_email_handler))
        .route(
            "/users/resend-verification",
            post(resend_verification_email_handler),
        )
        // ユーティリティ
        .route("/users/update-last-login", post(update_last_login_handler))
        // 管理者用エンドポイント
        .route("/admin/users", get(list_users_handler))
        .route("/admin/users/{id}", get(get_user_by_id_handler))
        .route(
            "/admin/users/{id}/status",
            patch(update_account_status_handler),
        )
        // ヘルスチェック
        .route("/users/health", get(user_health_check_handler))
        .with_state(app_state)
}

/// ユーザールーターをAppStateから作成
pub fn user_router_with_state(app_state: AppState) -> Router {
    user_router(app_state)
}

// --- ヘルパー関数 ---

/// ユーザー権限チェック（将来の拡張用）
#[allow(dead_code)]
fn check_admin_permission(_user: &UserClaims) -> AppResult<()> {
    // TODO: 管理者権限チェックロジックを実装
    // if !user.is_admin {
    //     return Err(AppError::Forbidden("Admin access required".to_string()));
    // }
    Ok(())
}

/// ユーザーアクセス権限チェック（自分自身または管理者）
#[allow(dead_code)]
fn check_user_access_permission(
    requesting_user: &UserClaims,
    target_user_id: Uuid,
) -> AppResult<()> {
    if requesting_user.user_id != target_user_id {
        // TODO: 管理者権限チェック
        // if !requesting_user.is_admin {
        //     return Err(AppError::Forbidden("Access denied".to_string()));
        // }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_user_router_creation() {
        // ユーザールーターの作成テスト
        // 実際のテストでは mock を使用
    }
}
