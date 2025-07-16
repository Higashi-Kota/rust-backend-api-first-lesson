// task-backend/src/api/handlers/user_handler.rs
use crate::api::dto::user_dto::{
    AccountStatusUpdateResponse, BulkOperationResponse, BulkUserOperationsRequest,
    EmailVerificationHistoryResponse, EmailVerificationResponse, ResendVerificationEmailRequest,
    SubscriptionAnalyticsResponse, SubscriptionQuery, UpdateAccountStatusRequest,
    UpdateEmailRequest, UpdateProfileRequest, UpdateUserSettingsRequest, UpdateUsernameRequest,
    UserAdditionalInfo, UserAnalyticsResponse, UserListResponse, UserPermissionsResponse,
    UserProfileResponse, UserSearchQuery, UserSettingsResponse, UserStatsResponse, UserSummary,
    VerifyEmailRequest,
};
use crate::api::AppState;
use crate::domain::subscription_tier::SubscriptionTier;
use crate::domain::user_model::SafeUser;
use crate::error::{AppError, AppResult};
use crate::middleware::auth::AuthenticatedUser;
use crate::middleware::auth::AuthenticatedUserWithRole;
use crate::types::{ApiResponse, Timestamp};
use crate::utils::error_helper::convert_validation_errors;
use crate::utils::permission::PermissionChecker;
use axum::{
    extract::{FromRequestParts, Json, Path, Query, State},
    http::{request::Parts, StatusCode},
    routing::{delete, get, patch, post},
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
            .map_err(|_| AppError::BadRequest("Invalid path parameter".to_string()))?;

        // UUIDをパースして検証エラー形式で返す
        let uuid = Uuid::parse_str(&path_str)
            .map_err(|_| AppError::BadRequest(format!("Invalid UUID format: '{}'", path_str)))?;

        Ok(UuidPath(uuid))
    }
}

// --- ユーザープロフィール管理 ---

/// ユーザープロフィール取得
pub async fn get_profile_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
) -> AppResult<ApiResponse<UserProfileResponse>> {
    let user_profile = app_state
        .user_service
        .get_user_profile(user.claims.user_id)
        .await?;

    info!(user_id = %user.claims.user_id, "User profile retrieved");

    // 機能使用状況を追跡
    crate::track_feature!(
        app_state.clone(),
        user.claims.user_id,
        "User Profile",
        "view"
    );

    Ok(ApiResponse::success(UserProfileResponse {
        user: user_profile,
    }))
}

/// ユーザー名更新
pub async fn update_username_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<UpdateUsernameRequest>,
) -> AppResult<ApiResponse<SafeUser>> {
    // バリデーション
    payload
        .validate()
        .map_err(|e| convert_validation_errors(e, "user_handler::update_username"))?;

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

    Ok(ApiResponse::success(updated_user))
}

/// メールアドレス更新
pub async fn update_email_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<UpdateEmailRequest>,
) -> AppResult<ApiResponse<SafeUser>> {
    // バリデーション
    payload
        .validate()
        .map_err(|e| convert_validation_errors(e, "user_handler::update_email"))?;

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

    Ok(ApiResponse::success(updated_user))
}

/// プロフィール一括更新
pub async fn update_profile_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<UpdateProfileRequest>,
) -> AppResult<ApiResponse<SafeUser>> {
    // バリデーション
    payload
        .validate()
        .map_err(|e| convert_validation_errors(e, "user_handler::update_profile"))?;

    // カスタムバリデーション
    payload.validate_update().map_err(|e| {
        warn!("Profile update custom validation failed: {}", e);
        AppError::BadRequest(e)
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

    Ok(ApiResponse::success(updated_user))
}

/// ユーザー統計情報取得
pub async fn get_user_stats_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
) -> AppResult<ApiResponse<UserStatsResponse>> {
    let stats = app_state
        .user_service
        .get_user_stats(user.claims.user_id)
        .await?;

    let additional_info = UserAdditionalInfo::from_user_stats(&stats);

    info!(user_id = %user.claims.user_id, "User stats retrieved");

    Ok(ApiResponse::success(UserStatsResponse {
        stats,
        additional_info,
    }))
}

/// メール認証実行
pub async fn verify_email_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<VerifyEmailRequest>,
) -> AppResult<ApiResponse<EmailVerificationResponse>> {
    // バリデーション
    payload
        .validate()
        .map_err(|e| convert_validation_errors(e, "user_handler::admin_verify_email"))?;

    info!(user_id = %user.claims.user_id, "Email verification attempt");

    // トークン検証ロジックを実装
    // 実際の実装では、メール認証トークンの検証を行う
    let token_verification_result = app_state
        .user_service
        .verify_email_token(user.claims.user_id, &payload.token)
        .await;

    let verified_user = match token_verification_result {
        Ok(_verified_user) => {
            // トークンが有効な場合、ユーザーの email_verified フラグを更新
            app_state
                .user_service
                .verify_email(user.claims.user_id)
                .await?
        }
        Err(_) => {
            // トークンが無効な場合はエラーを返す
            return Err(AppError::BadRequest(
                "Invalid or expired verification token".to_string(),
            ));
        }
    };

    info!(user_id = %user.claims.user_id, "Email verified successfully");

    Ok(ApiResponse::success(EmailVerificationResponse {
        message: "Email verified successfully".to_string(),
        verified: true,
        user: Some(verified_user),
    }))
}

/// メール認証再送信
pub async fn resend_verification_email_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<ResendVerificationEmailRequest>,
) -> AppResult<ApiResponse<EmailVerificationResponse>> {
    // バリデーション
    payload.validate().map_err(|e| {
        convert_validation_errors(e, "user_handler::admin_resend_verification_email")
    })?;

    info!(
        user_id = %user.claims.user_id,
        email = %payload.email,
        "Verification email resend attempt"
    );

    // メール送信ロジックを実装
    let verification_result = app_state
        .user_service
        .resend_verification_email(user.claims.user_id, &payload.email)
        .await;

    match verification_result {
        Ok(_) => {
            info!(
                user_id = %user.claims.user_id,
                email = %payload.email,
                "Verification email sent successfully"
            );
            Ok(ApiResponse::success(EmailVerificationResponse {
                message: "Verification email sent successfully".to_string(),
                verified: false,
                user: None,
            }))
        }
        Err(e) => {
            warn!(
                user_id = %user.claims.user_id,
                email = %payload.email,
                error = %e,
                "Failed to send verification email"
            );
            Err(AppError::InternalServerError(
                "Failed to send verification email".to_string(),
            ))
        }
    }
}

/// ユーザー設定取得
pub async fn get_user_settings_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
) -> AppResult<ApiResponse<UserSettingsResponse>> {
    // 実際の設定をデータベースから取得
    let user_settings = app_state
        .user_service
        .get_user_settings_legacy(user.claims.user_id)
        .await?;

    info!(user_id = %user.claims.user_id, "User settings retrieved");

    Ok(ApiResponse::success(user_settings))
}

/// ユーザー設定更新
pub async fn update_user_settings_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<UpdateUserSettingsRequest>,
) -> AppResult<ApiResponse<()>> {
    // タイムゾーンのバリデーション
    if let Some(ref tz) = payload.timezone {
        // 基本的なタイムゾーン検証（より詳細な検証も可能）
        let valid_timezones = [
            "UTC",
            "Asia/Tokyo",
            "America/New_York",
            "Europe/London",
            "America/Los_Angeles",
            "Europe/Paris",
            "Asia/Shanghai",
            "America/Chicago",
            "Asia/Singapore",
            "Australia/Sydney",
        ];

        if !valid_timezones.contains(&tz.as_str()) && !tz.starts_with("UTC") {
            return Err(AppError::BadRequest(format!("Invalid timezone: {}", tz)));
        }
    }

    // DTOをドメインモデルに変換
    let input = crate::domain::user_settings_model::UserSettingsInput {
        language: payload.language,
        timezone: payload.timezone,
        notifications_enabled: payload.notifications_enabled,
        email_notifications: payload
            .email_notifications
            .and_then(|v| serde_json::from_value(v).ok()),
        ui_preferences: payload
            .ui_preferences
            .and_then(|v| serde_json::from_value(v).ok()),
    };

    // 設定を更新
    app_state
        .user_service
        .update_user_settings(user.claims.user_id, input)
        .await?;

    info!(user_id = %user.claims.user_id, "User settings updated");

    Ok(ApiResponse::success(()))
}

/// ユーザー設定削除（デフォルトに戻す）
pub async fn delete_user_settings_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
) -> AppResult<(StatusCode, ApiResponse<()>)> {
    // 設定をデフォルトに戻す
    app_state
        .user_service
        .reset_user_settings_to_default(user.claims.user_id)
        .await?;

    info!(user_id = %user.claims.user_id, "User settings reset to default");

    Ok((StatusCode::NO_CONTENT, ApiResponse::success(())))
}

/// アカウント状態更新（管理者用）
pub async fn update_account_status_handler(
    State(app_state): State<AppState>,
    UuidPath(user_id): UuidPath,
    admin_user: AuthenticatedUserWithRole,
    Json(payload): Json<UpdateAccountStatusRequest>,
) -> AppResult<ApiResponse<AccountStatusUpdateResponse>> {
    // UserClaimsの権限チェックメソッドを活用 - ユーザーリソースの更新権限を確認
    if !admin_user.claims.can_update_resource("user", Some(user_id)) {
        warn!(
            user_id = %admin_user.user_id(),
            role = ?admin_user.role().map(|r| &r.name),
            target_user_id = %user_id,
            "Access denied: Cannot update user account status"
        );
        return Err(AppError::Forbidden(
            "Cannot update user account status".to_string(),
        ));
    }

    // バリデーション
    payload
        .validate()
        .map_err(|e| convert_validation_errors(e, "user_handler::admin_update_account_status"))?;

    // 自分自身のアカウント状態変更を防ぐ
    if admin_user.user_id() == user_id {
        warn!(
            admin_id = %admin_user.user_id(),
            "Admin attempting to change own account status"
        );
        return Err(AppError::BadRequest(
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

    Ok(ApiResponse::success(AccountStatusUpdateResponse {
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
) -> AppResult<ApiResponse<UserListResponse>> {
    // ユーザー一覧表示権限チェック（PermissionServiceを使用）
    if let Err(e) = app_state
        .permission_service
        .check_list_users_permission(admin_user.user_id())
        .await
    {
        warn!(
            user_id = %admin_user.user_id(),
            role = ?admin_user.role().map(|r| &r.name),
            "Access denied: Permission required for advanced user search"
        );
        return Err(e);
    }

    let query_with_defaults = query.with_defaults();

    let (page, per_page) = query_with_defaults.pagination.get_pagination();

    info!(
        admin_id = %admin_user.user_id(),
        page = page,
        per_page = per_page,
        query = ?query_with_defaults.q,
        "Advanced user search request"
    );

    // 詳細検索機能付きでユーザー一覧を取得
    let (users_with_roles, total_count) = app_state
        .user_service
        .list_users_with_roles_paginated(page, per_page)
        .await?;

    // SafeUserWithRoleをUserSummaryに変換（タスク数を含む）
    let mut user_summaries: Vec<UserSummary> = Vec::new();

    for user_with_role in users_with_roles {
        // 各ユーザーのタスク数を取得
        let task_count = app_state
            .task_service
            .count_tasks_for_user(user_with_role.id)
            .await
            .unwrap_or(0);

        user_summaries.push(UserSummary {
            id: user_with_role.id,
            username: user_with_role.username,
            email: user_with_role.email,
            is_active: user_with_role.is_active,
            email_verified: user_with_role.email_verified,
            created_at: user_with_role.created_at,
            last_login_at: user_with_role.last_login_at,
            task_count: task_count.try_into().unwrap_or(0),
        });
    }

    Ok(ApiResponse::success(UserListResponse::new(
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
) -> AppResult<ApiResponse<UserAnalyticsResponse>> {
    // 管理者権限チェック（PermissionServiceを使用）
    app_state
        .permission_service
        .check_admin_permission(admin_user.user_id())
        .await?;

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

    Ok(ApiResponse::success(UserAnalyticsResponse {
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
) -> AppResult<ApiResponse<UserListResponse>> {
    // ユーザー一覧表示権限チェック（PermissionServiceを使用）
    if let Err(e) = app_state
        .permission_service
        .check_list_users_permission(admin_user.user_id())
        .await
    {
        warn!(
            user_id = %admin_user.user_id(),
            role = ?admin_user.role().map(|r| &r.name),
            "Access denied: Permission required for role-based user search"
        );
        return Err(e);
    }

    let query_with_defaults = query.with_defaults();
    let (page, per_page) = query_with_defaults.pagination.get_pagination();

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

    // SafeUserWithRoleをUserSummaryに変換（タスク数を含む）
    let mut user_summaries: Vec<UserSummary> = Vec::new();

    for user_with_role in paginated_users {
        // 各ユーザーのタスク数を取得
        let task_count = app_state
            .task_service
            .count_tasks_for_user(user_with_role.id)
            .await
            .unwrap_or(0);

        user_summaries.push(UserSummary {
            id: user_with_role.id,
            username: user_with_role.username,
            email: user_with_role.email,
            is_active: user_with_role.is_active,
            email_verified: user_with_role.email_verified,
            created_at: user_with_role.created_at,
            last_login_at: user_with_role.last_login_at,
            task_count: task_count.try_into().unwrap_or(0),
        });
    }

    Ok(ApiResponse::success(UserListResponse::new(
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
) -> AppResult<ApiResponse<SubscriptionAnalyticsResponse>> {
    // 管理者権限チェック（PermissionServiceを使用）
    app_state
        .permission_service
        .check_admin_permission(admin_user.user_id())
        .await?;

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

    Ok(ApiResponse::success(SubscriptionAnalyticsResponse {
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
) -> AppResult<ApiResponse<BulkOperationResponse>> {
    // 管理者権限チェック（PermissionServiceを使用）
    app_state
        .permission_service
        .check_admin_permission(admin_user.user_id())
        .await?;

    // バリデーション
    payload
        .validate()
        .map_err(|e| convert_validation_errors(e, "user_handler::admin_bulk_user_operations"))?;

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
            admin_user.user_id(),
        )
        .await?;

    let execution_time = start_time.elapsed().as_millis() as u64;
    let operation_id = uuid::Uuid::new_v4().to_string();

    Ok(ApiResponse::success(BulkOperationResponse {
        operation_id,
        operation: payload.operation.to_string(),
        total_users: payload.user_ids.len(),
        successful_operations: result.successful,
        failed_operations: result.failed,
        errors: result.errors,
        message: format!("Bulk operation '{}' completed", payload.operation),
        results: result.results,
        execution_time_ms: execution_time,
        executed_at: Timestamp::now(),
    }))
}

/// ユーザー一覧取得（管理者用）
pub async fn list_users_handler(
    State(app_state): State<AppState>,
    admin_user: AuthenticatedUserWithRole,
    Query(query): Query<UserSearchQuery>,
) -> AppResult<ApiResponse<UserListResponse>> {
    // ユーザー一覧表示権限チェック（PermissionServiceを使用）
    if let Err(e) = app_state
        .permission_service
        .check_list_users_permission(admin_user.user_id())
        .await
    {
        warn!(
            user_id = %admin_user.user_id(),
            role = ?admin_user.role().map(|r| &r.name),
            "Access denied: Permission required for user list"
        );
        return Err(e);
    }

    // バリデーション
    query
        .validate()
        .map_err(|e| convert_validation_errors(e, "user_handler::search_users"))?;

    let query_with_defaults = query.with_defaults();
    let (page, per_page) = query_with_defaults.pagination.get_pagination();

    info!(
        admin_id = %admin_user.user_id(),
        page = page,
        per_page = per_page,
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
                page,
                per_page,
            )
            .await?
    } else {
        // 通常の一覧取得
        app_state
            .user_service
            .list_users_with_roles_paginated(page, per_page)
            .await?
    };

    // SafeUserWithRoleをUserSummaryに変換（タスク数を含む）
    let mut user_summaries: Vec<UserSummary> = Vec::new();

    for user_with_role in users_with_roles {
        // 各ユーザーのタスク数を取得
        let task_count = app_state
            .task_service
            .count_tasks_for_user(user_with_role.id)
            .await
            .unwrap_or(0);

        user_summaries.push(UserSummary {
            id: user_with_role.id,
            username: user_with_role.username,
            email: user_with_role.email,
            is_active: user_with_role.is_active,
            email_verified: user_with_role.email_verified,
            created_at: user_with_role.created_at,
            last_login_at: user_with_role.last_login_at,
            task_count: task_count.try_into().unwrap_or(0),
        });
    }

    info!(
        admin_id = %admin_user.user_id(),
        users_count = %user_summaries.len(),
        total_count = %total_count,
        "Admin user search retrieved successfully"
    );

    Ok(ApiResponse::success(UserListResponse::new(
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
) -> AppResult<ApiResponse<UserProfileResponse>> {
    // ユーザーアクセス権限チェック（PermissionServiceを使用）
    if let Err(e) = app_state
        .permission_service
        .check_user_access(admin_user.user_id(), user_id)
        .await
    {
        warn!(
            user_id = %admin_user.user_id(),
            role = ?admin_user.role().map(|r| &r.name),
            target_user_id = %user_id,
            "Access denied: Cannot access user profile"
        );
        return Err(e);
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

    Ok(ApiResponse::success(UserProfileResponse {
        user: user_profile,
    }))
}

/// 最終ログイン時刻更新
pub async fn update_last_login_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
) -> AppResult<ApiResponse<()>> {
    app_state
        .user_service
        .update_last_login(user.claims.user_id)
        .await?;

    info!(user_id = %user.claims.user_id, "Last login time updated");

    // ApiResponse::success_messageを活用
    Ok(ApiResponse::success(()))
}

/// ユーザー権限チェック
pub async fn check_user_permissions_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUserWithRole,
) -> AppResult<ApiResponse<UserPermissionsResponse>> {
    // ユーザー情報を取得
    let user_profile = app_state
        .user_service
        .get_user_profile(user.user_id())
        .await?;

    // 権限情報を生成
    let is_member = user.role().is_some_and(PermissionChecker::is_member);
    let is_admin = user.is_admin();
    let is_active = user_profile.is_active;
    let email_verified = user_profile.email_verified;

    let permissions = UserPermissionsResponse {
        user_id: user.user_id(),
        is_member,
        is_admin,
        is_active,
        email_verified,
        subscription_tier: user_profile.subscription_tier.clone(),
        can_create_teams: is_member
            && (user_profile.subscription_tier == "Pro"
                || user_profile.subscription_tier == "enterprise"),
        can_access_analytics: is_admin || user_profile.subscription_tier == "enterprise",
    };

    Ok(ApiResponse::success(permissions))
}

// --- ヘルスチェック ---

/// メール認証履歴を取得
pub async fn get_email_verification_history_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
) -> AppResult<ApiResponse<EmailVerificationHistoryResponse>> {
    info!(
        user_id = %user.user_id(),
        "Getting email verification history"
    );

    // メール認証履歴を取得
    let history_response = app_state
        .auth_service
        .get_email_verification_history(user.user_id())
        .await?;

    info!(
        user_id = %user.user_id(),
        total_verifications = %history_response.total_verifications,
        "Email verification history retrieved"
    );

    Ok(ApiResponse::success(history_response))
}

/// ユーザーサービスのヘルスチェック
async fn user_health_check_handler() -> &'static str {
    "User service OK"
}

/// ユーザーのサブスクリプションをアップグレード
pub async fn upgrade_user_subscription_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Path(user_id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> AppResult<ApiResponse<serde_json::Value>> {
    // 権限チェック：自分自身または管理者のみ
    if user.user_id() != user_id && !user.is_admin() {
        return Err(AppError::Forbidden(
            "You can only upgrade your own subscription".to_string(),
        ));
    }

    // Extract new_tier from payload
    let new_tier_str = payload
        .get("new_tier")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("new_tier is required".to_string()))?;

    let new_tier = SubscriptionTier::from_str(new_tier_str).ok_or_else(|| {
        AppError::BadRequest(format!("Invalid subscription tier: {}", new_tier_str))
    })?;

    // Get current user info
    let current_user = app_state.user_service.get_user_profile(user_id).await?;

    let current_tier = SubscriptionTier::from_str(&current_user.subscription_tier)
        .ok_or_else(|| AppError::BadRequest("Invalid current subscription tier".to_string()))?;

    // Check if it's actually an upgrade
    if new_tier.level() <= current_tier.level() {
        return Err(AppError::BadRequest(
            "Cannot upgrade to the same or lower tier".to_string(),
        ));
    }

    // Execute subscription change
    let reason = payload
        .get("reason")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let (updated_user, _history) = app_state
        .subscription_service
        .change_subscription_tier(
            user_id,
            new_tier.as_str().to_string(),
            Some(user.user_id()),
            reason,
        )
        .await?;

    // Create the response in the expected format
    let response = serde_json::json!({
        "user_id": updated_user.id,
        "previous_tier": current_tier.as_str(),
        "new_tier": new_tier.as_str(),
        "upgraded_at": chrono::Utc::now(),
    });

    Ok(ApiResponse::success(response))
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
        .route("/users/settings", patch(update_user_settings_handler))
        .route("/users/settings", delete(delete_user_settings_handler))
        // メール認証
        .route("/users/verify-email", post(verify_email_handler))
        .route(
            "/users/resend-verification",
            post(resend_verification_email_handler),
        )
        .route(
            "/users/email-verification-history",
            get(get_email_verification_history_handler),
        )
        // ユーティリティ
        .route("/users/update-last-login", post(update_last_login_handler))
        .route("/users/permissions", get(check_user_permissions_handler))
        // サブスクリプション
        .route(
            "/users/{id}/subscription/upgrade",
            post(upgrade_user_subscription_handler),
        )
        // 管理者用エンドポイント
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
