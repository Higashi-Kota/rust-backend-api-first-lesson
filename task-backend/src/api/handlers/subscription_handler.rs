// task-backend/src/api/handlers/subscription_handler.rs

use crate::api::dto::subscription_dto::*;
use crate::api::dto::{ApiResponse, OperationResult};
use crate::api::AppState;
use crate::domain::subscription_tier::SubscriptionTier;
use crate::error::{AppError, AppResult};
use crate::middleware::auth::{AuthenticatedUser, AuthenticatedUserWithRole};
use axum::{
    extract::{FromRequestParts, Json, Path, Query, State},
    http::request::Parts,
    routing::{get, patch, post},
    Router,
};
use serde::Deserialize;
use tracing::{info, warn};
use uuid::Uuid;
use validator::Validate;

// --- Query Parameters ---

/// サブスクリプション履歴検索パラメータ
#[derive(Debug, Deserialize, Validate)]
pub struct SubscriptionHistoryQuery {
    #[validate(range(min = 1, max = 100, message = "Page must be between 1 and 100"))]
    pub page: Option<u64>,

    #[validate(range(min = 1, max = 100, message = "Page size must be between 1 and 100"))]
    pub page_size: Option<u64>,
}

/// 管理者用統計クエリパラメータ
#[derive(Debug, Deserialize)]
pub struct AdminStatsQuery {
    pub include_revenue: Option<bool>,
    pub days: Option<u32>,
}

// --- カスタム抽出器 ---

/// UUID パス抽出器
pub struct UuidPath(pub Uuid);

impl<S> FromRequestParts<S> for UuidPath
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Path(path_str) = Path::<String>::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::ValidationErrors(vec!["Invalid path parameter".to_string()]))?;

        let uuid = Uuid::parse_str(&path_str).map_err(|_| {
            AppError::ValidationErrors(vec![format!("Invalid UUID format: '{}'", path_str)])
        })?;

        Ok(UuidPath(uuid))
    }
}

// --- Handler Functions ---

/// 現在のサブスクリプション情報取得
pub async fn get_current_subscription_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
) -> AppResult<Json<CurrentSubscriptionResponse>> {
    let user_profile = app_state
        .user_service
        .get_user_profile(user.claims.user_id)
        .await?;

    let response = CurrentSubscriptionResponse::new(
        user.claims.user_id,
        user_profile.subscription_tier,
        user_profile.created_at,
    );

    info!(
        user_id = %user.claims.user_id,
        tier = %response.current_tier,
        "Current subscription retrieved"
    );

    Ok(Json(response))
}

/// サブスクリプションアップグレード
pub async fn upgrade_subscription_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<UpgradeSubscriptionRequest>,
) -> AppResult<Json<UpgradeSubscriptionResponse>> {
    // バリデーション
    payload.validate().map_err(|validation_errors| {
        warn!(
            "Subscription upgrade validation failed: {}",
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

    // 現在のサブスクリプション階層を取得
    let current_user = app_state
        .user_service
        .get_user_profile(user.claims.user_id)
        .await?;

    let current_tier =
        SubscriptionTier::from_str(&current_user.subscription_tier).ok_or_else(|| {
            AppError::ValidationError("Invalid current subscription tier".to_string())
        })?;

    // アップグレード可能かチェック
    if payload.target_tier.level() <= current_tier.level() {
        return Err(AppError::ValidationError(
            "Cannot upgrade to the same or lower tier".to_string(),
        ));
    }

    info!(
        user_id = %user.claims.user_id,
        from_tier = %current_tier.as_str(),
        to_tier = %payload.target_tier.as_str(),
        "Subscription upgrade attempt"
    );

    // サブスクリプション変更を実行
    let (updated_user, _history) = app_state
        .subscription_service
        .change_subscription_tier(
            user.claims.user_id,
            payload.target_tier.as_str().to_string(),
            Some(user.claims.user_id),
            payload.reason,
        )
        .await?;

    let change_response = SubscriptionChangeResponse::new(
        updated_user.into(),
        current_tier.as_str().to_string(),
        payload.target_tier.as_str().to_string(),
        None,
        Some(user.claims.user_id),
    );

    info!(
        user_id = %user.claims.user_id,
        new_tier = %payload.target_tier.as_str(),
        "Subscription upgraded successfully"
    );

    Ok(Json(ApiResponse::success(
        "Subscription upgraded successfully",
        OperationResult::updated(change_response, vec!["subscription_tier".to_string()]),
    )))
}

/// サブスクリプションダウングレード
pub async fn downgrade_subscription_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<DowngradeSubscriptionRequest>,
) -> AppResult<Json<DowngradeSubscriptionResponse>> {
    // バリデーション
    payload.validate().map_err(|validation_errors| {
        warn!(
            "Subscription downgrade validation failed: {}",
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

    // 現在のサブスクリプション階層を取得
    let current_user = app_state
        .user_service
        .get_user_profile(user.claims.user_id)
        .await?;

    let current_tier =
        SubscriptionTier::from_str(&current_user.subscription_tier).ok_or_else(|| {
            AppError::ValidationError("Invalid current subscription tier".to_string())
        })?;

    // ダウングレード可能かチェック
    if payload.target_tier.level() >= current_tier.level() {
        return Err(AppError::ValidationError(
            "Cannot downgrade to the same or higher tier".to_string(),
        ));
    }

    info!(
        user_id = %user.claims.user_id,
        from_tier = %current_tier.as_str(),
        to_tier = %payload.target_tier.as_str(),
        "Subscription downgrade attempt"
    );

    // サブスクリプション変更を実行
    let (updated_user, _history) = app_state
        .subscription_service
        .change_subscription_tier(
            user.claims.user_id,
            payload.target_tier.as_str().to_string(),
            Some(user.claims.user_id),
            payload.reason,
        )
        .await?;

    let change_response = SubscriptionChangeResponse::new(
        updated_user.into(),
        current_tier.as_str().to_string(),
        payload.target_tier.as_str().to_string(),
        None,
        Some(user.claims.user_id),
    );

    info!(
        user_id = %user.claims.user_id,
        new_tier = %payload.target_tier.as_str(),
        "Subscription downgraded successfully"
    );

    Ok(Json(ApiResponse::success(
        "Subscription downgraded successfully",
        OperationResult::updated(change_response, vec!["subscription_tier".to_string()]),
    )))
}

/// サブスクリプション履歴取得
pub async fn get_subscription_history_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Query(query): Query<SubscriptionHistoryQuery>,
) -> AppResult<Json<SubscriptionHistoryResponse>> {
    // バリデーション
    query.validate().map_err(|validation_errors| {
        warn!(
            "Subscription history query validation failed: {}",
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

    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);

    info!(
        user_id = %user.claims.user_id,
        page = %page,
        page_size = %page_size,
        "Subscription history request"
    );

    // サブスクリプション履歴を取得
    let (history, total_count) = app_state
        .subscription_service
        .get_user_subscription_history(user.claims.user_id, page, page_size)
        .await?;

    let stats = app_state
        .subscription_service
        .get_user_subscription_stats(user.claims.user_id)
        .await?;

    let pagination =
        crate::api::dto::PaginationMeta::new(page as i32, page_size as i32, total_count as i64);

    let response = SubscriptionHistoryResponse {
        user_id: user.claims.user_id,
        history,
        pagination: Some(pagination),
        stats,
    };

    info!(
        user_id = %user.claims.user_id,
        history_count = %response.history.len(),
        "Subscription history retrieved"
    );

    Ok(Json(response))
}

/// 管理者用サブスクリプション統計取得
pub async fn get_subscription_stats_handler(
    State(app_state): State<AppState>,
    admin_user: AuthenticatedUserWithRole,
    Query(query): Query<AdminStatsQuery>,
) -> AppResult<Json<SubscriptionStatsResponse>> {
    // 管理者権限チェック
    if !admin_user.is_admin() {
        warn!(
            user_id = %admin_user.user_id(),
            role = ?admin_user.role().map(|r| &r.name),
            "Access denied: Admin permission required for subscription stats"
        );
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    info!(
        admin_id = %admin_user.user_id(),
        include_revenue = ?query.include_revenue,
        days = ?query.days,
        "Admin subscription stats request"
    );

    // サブスクリプション統計を取得
    let tier_distribution = app_state
        .subscription_service
        .get_subscription_tier_stats()
        .await?;

    let total_users = tier_distribution.iter().map(|t| t.total_users).sum();

    // モックデータ（実際の実装では適切なサービスから取得）
    let recent_changes = RecentChanges {
        upgrades_last_7_days: 15,
        downgrades_last_7_days: 3,
        upgrades_last_30_days: 45,
        downgrades_last_30_days: 8,
    };

    let revenue_info = if query.include_revenue.unwrap_or(false) {
        RevenueInfo {
            monthly_recurring_revenue: 12500.0,
            annual_recurring_revenue: 150000.0,
            average_revenue_per_user: 25.5,
            churn_rate: 2.3,
        }
    } else {
        RevenueInfo {
            monthly_recurring_revenue: 0.0,
            annual_recurring_revenue: 0.0,
            average_revenue_per_user: 0.0,
            churn_rate: 0.0,
        }
    };

    let response = SubscriptionStatsResponse {
        total_users,
        tier_distribution,
        recent_changes,
        revenue_info,
    };

    info!(
        admin_id = %admin_user.user_id(),
        total_users = %response.total_users,
        "Subscription stats retrieved"
    );

    Ok(Json(response))
}

/// 管理者用ユーザーのサブスクリプション変更
pub async fn admin_change_subscription_handler(
    State(app_state): State<AppState>,
    UuidPath(user_id): UuidPath,
    admin_user: AuthenticatedUserWithRole,
    Json(payload): Json<AdminChangeSubscriptionRequest>,
) -> AppResult<Json<ChangeSubscriptionResponse>> {
    // 管理者権限チェック
    if !admin_user.is_admin() {
        warn!(
            user_id = %admin_user.user_id(),
            role = ?admin_user.role().map(|r| &r.name),
            target_user_id = %user_id,
            "Access denied: Admin permission required for subscription change"
        );
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    // バリデーション
    payload.validate().map_err(|validation_errors| {
        warn!(
            "Admin subscription change validation failed: {}",
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

    // 対象ユーザーの現在の情報を取得
    let current_user = app_state.user_service.get_user_profile(user_id).await?;

    info!(
        admin_id = %admin_user.user_id(),
        target_user_id = %user_id,
        target_username = %current_user.username,
        from_tier = %current_user.subscription_tier,
        to_tier = %payload.new_tier,
        force_change = ?payload.force_change,
        "Admin subscription change attempt"
    );

    // サブスクリプション変更を実行
    let (updated_user, _history) = app_state
        .subscription_service
        .change_subscription_tier(
            user_id,
            payload.new_tier.clone(),
            Some(admin_user.user_id()),
            payload.reason,
        )
        .await?;

    let change_response = SubscriptionChangeResponse::new(
        updated_user.into(),
        current_user.subscription_tier,
        payload.new_tier,
        None,
        Some(admin_user.user_id()),
    );

    info!(
        admin_id = %admin_user.user_id(),
        target_user_id = %user_id,
        new_tier = %change_response.new_tier,
        "Admin subscription change completed"
    );

    Ok(Json(ApiResponse::success(
        "Subscription changed successfully by administrator",
        OperationResult::updated(change_response, vec!["subscription_tier".to_string()]),
    )))
}

// --- ルーター ---

/// サブスクリプションルーターを作成
pub fn subscription_router(app_state: AppState) -> Router {
    Router::new()
        // 一般ユーザー向けエンドポイント
        .route(
            "/subscriptions/current",
            get(get_current_subscription_handler),
        )
        .route("/subscriptions/upgrade", post(upgrade_subscription_handler))
        .route(
            "/subscriptions/downgrade",
            post(downgrade_subscription_handler),
        )
        .route(
            "/subscriptions/history",
            get(get_subscription_history_handler),
        )
        // 管理者向けエンドポイント
        .route(
            "/admin/subscriptions/stats",
            get(get_subscription_stats_handler),
        )
        .route(
            "/admin/users/{id}/subscription",
            patch(admin_change_subscription_handler),
        )
        .with_state(app_state)
}

/// サブスクリプションルーターをAppStateから作成
pub fn subscription_router_with_state(app_state: AppState) -> Router {
    subscription_router(app_state)
}
