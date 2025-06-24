// task-backend/src/api/handlers/user_handler.rs
use crate::api::dto::user_dto::*;
use crate::api::dto::{ApiResponse, OperationResult};
use crate::api::AppState;
use crate::error::{AppError, AppResult};
use crate::middleware::auth::AuthenticatedUser;
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
        .get_user_profile(user.claims.user_id)
        .await?;

    info!(user_id = %user.claims.user_id, "User profile retrieved");

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
        user_id = %user.claims.user_id,
        new_username = %payload.username,
        "Username update attempt"
    );

    let updated_user = app_state
        .user_service
        .update_username(user.claims.user_id, &payload.username)
        .await?;

    info!(
        user_id = %user.claims.user_id,
        new_username = %payload.username,
        "Username updated successfully"
    );

    Ok(Json(ApiResponse::success(
        "Username updated successfully",
        OperationResult::updated(updated_user, vec!["username".to_string()]),
    )))
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
        user_id = %user.claims.user_id,
        new_email = %payload.email,
        "Email update attempt"
    );

    let updated_user = app_state
        .user_service
        .update_email(user.claims.user_id, &payload.email)
        .await?;

    info!(
        user_id = %user.claims.user_id,
        new_email = %payload.email,
        "Email updated successfully"
    );

    Ok(Json(ApiResponse::success(
        "Email updated successfully. Please verify your new email address",
        OperationResult::updated(updated_user, vec!["email".to_string()]),
    )))
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

    info!(user_id = %user.claims.user_id, "Profile update attempt");

    let changes = payload.get_updated_fields();
    let mut updated_user = app_state
        .user_service
        .get_user_profile(user.claims.user_id)
        .await?;

    // ユーザー名の更新
    if let Some(new_username) = &payload.username {
        updated_user = app_state
            .user_service
            .update_username(user.claims.user_id, new_username)
            .await?;
    }

    // メールアドレスの更新
    if let Some(new_email) = &payload.email {
        updated_user = app_state
            .user_service
            .update_email(user.claims.user_id, new_email)
            .await?;
    }

    info!(
        user_id = %user.claims.user_id,
        changes = ?changes,
        "Profile updated successfully"
    );

    Ok(Json(ApiResponse::success(
        "Profile updated successfully",
        OperationResult::updated(updated_user, changes),
    )))
}

/// ユーザー統計情報取得
pub async fn get_user_stats_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
) -> AppResult<Json<UserStatsResponse>> {
    let stats = app_state
        .user_service
        .get_user_stats(user.claims.user_id)
        .await?;

    let additional_info = UserAdditionalInfo::from_user_stats(&stats);

    info!(user_id = %user.claims.user_id, "User stats retrieved");

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

    info!(user_id = %user.claims.user_id, "Email verification attempt");

    // TODO: トークン検証ロジックを実装
    // 現在はプレースホルダー

    let verified_user = app_state
        .user_service
        .verify_email(user.claims.user_id)
        .await?;

    info!(user_id = %user.claims.user_id, "Email verified successfully");

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
        user_id = %user.claims.user_id,
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

    info!(user_id = %user.claims.user_id, "User settings retrieved");

    Json(UserSettingsResponse {
        user_id: user.claims.user_id,
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
    if !admin_user.is_admin() {
        warn!(
            user_id = %admin_user.user_id(),
            role = ?admin_user.role().map(|r| &r.name),
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

/// 高度なユーザー検索（管理者用）
pub async fn advanced_search_users_handler(
    State(app_state): State<AppState>,
    admin_user: AuthenticatedUserWithRole,
    Query(query): Query<UserSearchQuery>,
) -> AppResult<Json<UserListResponse>> {
    // 管理者権限チェック
    if !admin_user.is_admin() {
        warn!(
            user_id = %admin_user.user_id(),
            role = ?admin_user.role().map(|r| &r.name),
            "Access denied: Admin permission required for advanced user search"
        );
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    let query_with_defaults = query.with_defaults();

    info!(
        admin_id = %admin_user.user_id(),
        page = ?query_with_defaults.page,
        per_page = ?query_with_defaults.per_page,
        query = ?query_with_defaults.q,
        "Advanced user search request"
    );

    // 詳細検索機能付きでユーザー一覧を取得
    let (users_with_roles, total_count) = app_state
        .user_service
        .list_users_with_roles_paginated(
            query_with_defaults.page.unwrap_or(1),
            query_with_defaults.per_page.unwrap_or(20),
        )
        .await?;

    let page = query_with_defaults.page.unwrap_or(1);
    let per_page = query_with_defaults.per_page.unwrap_or(20);

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

    Ok(Json(UserListResponse::new(
        user_summaries,
        page,
        per_page,
        total_count as i64,
    )))
}

/// ユーザー分析ダッシュボード（管理者用）
pub async fn get_user_analytics_handler(
    State(app_state): State<AppState>,
    admin_user: AuthenticatedUserWithRole,
) -> AppResult<Json<UserAnalyticsResponse>> {
    // 管理者権限チェック
    if !admin_user.is_admin() {
        warn!(
            user_id = %admin_user.user_id(),
            role = ?admin_user.role().map(|r| &r.name),
            "Access denied: Admin permission required for user analytics"
        );
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    info!(
        admin_id = %admin_user.user_id(),
        "User analytics request"
    );

    // ユーザー統計を取得
    let stats = app_state
        .user_service
        .get_user_stats_for_analytics()
        .await?;

    // ロール別統計を取得
    let role_stats = app_state.user_service.get_user_stats_by_role().await?;

    Ok(Json(UserAnalyticsResponse {
        stats,
        role_stats,
        message: "User analytics retrieved successfully".to_string(),
    }))
}

/// ロール別ユーザー取得（管理者用）
pub async fn get_users_by_role_handler(
    State(app_state): State<AppState>,
    Path(role): Path<String>,
    admin_user: AuthenticatedUserWithRole,
    Query(query): Query<UserSearchQuery>,
) -> AppResult<Json<UserListResponse>> {
    // 管理者権限チェック
    if !admin_user.is_admin() {
        warn!(
            user_id = %admin_user.user_id(),
            role = ?admin_user.role().map(|r| &r.name),
            "Access denied: Admin permission required for role-based user search"
        );
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    let query_with_defaults = query.with_defaults();
    let page = query_with_defaults.page.unwrap_or(1);
    let per_page = query_with_defaults.per_page.unwrap_or(20);

    info!(
        admin_id = %admin_user.user_id(),
        role = %role,
        "Role-based user search request"
    );

    // ロール別ユーザーを取得
    let users = app_state.user_service.find_by_role_name(&role).await?;
    let total_count = users.len() as u64;

    // ページネーション適用
    let start = ((page - 1) * per_page) as usize;
    let end = (start + per_page as usize).min(users.len());
    let paginated_users = users
        .into_iter()
        .skip(start)
        .take(end - start)
        .collect::<Vec<_>>();

    // SafeUserWithRoleをUserSummaryに変換
    let user_summaries: Vec<UserSummary> = paginated_users
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

    Ok(Json(UserListResponse::new(
        user_summaries,
        page,
        per_page,
        total_count as i64,
    )))
}

/// サブスクリプション別ユーザー分析（管理者用）
pub async fn get_users_by_subscription_handler(
    State(app_state): State<AppState>,
    admin_user: AuthenticatedUserWithRole,
    Query(query): Query<SubscriptionQuery>,
) -> AppResult<Json<SubscriptionAnalyticsResponse>> {
    // 管理者権限チェック
    if !admin_user.is_admin() {
        warn!(
            user_id = %admin_user.user_id(),
            role = ?admin_user.role().map(|r| &r.name),
            "Access denied: Admin permission required for subscription analytics"
        );
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    info!(
        admin_id = %admin_user.user_id(),
        "Subscription analytics request"
    );

    // サブスクリプション階層別の分析を実装
    let tier = query.tier.unwrap_or_else(|| "all".to_string());

    let analytics = if tier == "all" {
        app_state.user_service.get_subscription_analytics().await?
    } else {
        app_state
            .user_service
            .get_subscription_analytics_by_tier(&tier)
            .await?
    };

    Ok(Json(SubscriptionAnalyticsResponse {
        tier,
        analytics,
        message: "Subscription analytics retrieved successfully".to_string(),
    }))
}

/// 一括ユーザー操作（管理者用）
pub async fn bulk_user_operations_handler(
    State(app_state): State<AppState>,
    admin_user: AuthenticatedUserWithRole,
    Json(payload): Json<BulkUserOperationsRequest>,
) -> AppResult<Json<BulkOperationResponse>> {
    // 管理者権限チェック
    if !admin_user.is_admin() {
        warn!(
            user_id = %admin_user.user_id(),
            role = ?admin_user.role().map(|r| &r.name),
            "Access denied: Admin permission required for bulk operations"
        );
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    // バリデーション
    payload.validate().map_err(|validation_errors| {
        warn!("Bulk operations validation failed: {}", validation_errors);
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
        admin_id = %admin_user.user_id(),
        operation = %payload.operation,
        user_ids_count = %payload.user_ids.len(),
        notify_users = %payload.notify_users.unwrap_or(false),
        "Bulk user operations request"
    );

    let start_time = std::time::Instant::now();

    // 一括操作を実行（拡張版）
    let result = app_state
        .user_service
        .bulk_user_operations_extended(
            &payload.operation,
            &payload.user_ids,
            payload.parameters.as_ref(),
            payload.notify_users.unwrap_or(false),
        )
        .await?;

    let execution_time = start_time.elapsed().as_millis() as u64;
    let operation_id = uuid::Uuid::new_v4().to_string();

    Ok(Json(BulkOperationResponse {
        operation_id,
        operation: payload.operation.to_string(),
        total_users: payload.user_ids.len(),
        successful_operations: result.successful,
        failed_operations: result.failed,
        errors: result.errors,
        message: format!("Bulk operation '{}' completed", payload.operation),
        results: result.results,
        execution_time_ms: execution_time,
        executed_at: chrono::Utc::now().to_rfc3339(),
    }))
}

/// ユーザー一覧取得（管理者用）
pub async fn list_users_handler(
    State(app_state): State<AppState>,
    admin_user: AuthenticatedUserWithRole,
    Query(query): Query<UserSearchQuery>,
) -> AppResult<Json<UserListResponse>> {
    // 管理者権限チェック
    if !admin_user.is_admin() {
        warn!(
            user_id = %admin_user.user_id(),
            role = ?admin_user.role().map(|r| &r.name),
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
        query = ?query_with_defaults.q,
        "Admin user search request"
    );

    // 検索機能付きでユーザー一覧を取得
    let (users_with_roles, total_count) = if query_with_defaults.q.is_some()
        || query_with_defaults.is_active.is_some()
        || query_with_defaults.email_verified.is_some()
    {
        // 検索・フィルター機能を使用
        app_state
            .user_service
            .search_users(
                query_with_defaults.q,
                query_with_defaults.is_active,
                query_with_defaults.email_verified,
                query_with_defaults.page.unwrap_or(1),
                query_with_defaults.per_page.unwrap_or(20),
            )
            .await?
    } else {
        // 通常の一覧取得
        app_state
            .user_service
            .list_users_with_roles_paginated(
                query_with_defaults.page.unwrap_or(1),
                query_with_defaults.per_page.unwrap_or(20),
            )
            .await?
    };

    let page = query_with_defaults.page.unwrap_or(1);
    let per_page = query_with_defaults.per_page.unwrap_or(20);

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
        "Admin user search retrieved successfully"
    );

    Ok(Json(UserListResponse::new(
        user_summaries,
        page,
        per_page,
        total_count as i64,
    )))
}

/// 特定ユーザー情報取得（管理者用）
pub async fn get_user_by_id_handler(
    State(app_state): State<AppState>,
    UuidPath(user_id): UuidPath,
    admin_user: AuthenticatedUserWithRole,
) -> AppResult<Json<UserProfileResponse>> {
    // 管理者権限チェック
    if !admin_user.is_admin() {
        warn!(
            user_id = %admin_user.user_id(),
            role = ?admin_user.role().map(|r| &r.name),
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
        .update_last_login(user.claims.user_id)
        .await?;

    info!(user_id = %user.claims.user_id, "Last login time updated");

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
        // 管理者用エンドポイント - Phase 1.1 高度なユーザー管理 API
        .route("/admin/users", get(list_users_handler))
        .route(
            "/admin/users/advanced-search",
            get(advanced_search_users_handler),
        )
        .route("/admin/users/analytics", get(get_user_analytics_handler))
        .route(
            "/admin/users/by-role/{role}",
            get(get_users_by_role_handler),
        )
        .route(
            "/admin/users/by-subscription",
            get(get_users_by_subscription_handler),
        )
        .route(
            "/admin/users/bulk-operations",
            post(bulk_user_operations_handler),
        )
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
