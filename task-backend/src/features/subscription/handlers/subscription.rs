// task-backend/src/features/subscription/handlers/subscription.rs

use crate::api::AppState;
use crate::core::subscription_tier::SubscriptionTier;
use crate::error::{AppError, AppResult};
use crate::features::auth::middleware::{AuthenticatedUser, AuthenticatedUserWithRole};
use crate::shared::types::pagination::{PaginationMeta, PaginationQuery};
use crate::shared::types::{ApiResponse, OperationResult};
use axum::{
    extract::{FromRequestParts, Json, Path, Query, State},
    http::request::Parts,
    routing::{get, patch, post},
    Router,
};
use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use serde_json::json;
use tracing::{info, warn};
use uuid::Uuid;
use validator::Validate;

use super::super::dto::subscription::*;

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

    let current_tier = SubscriptionTier::from_str(&current_user.subscription_tier)
        .ok_or_else(|| AppError::BadRequest("Invalid current subscription tier".to_string()))?;

    // アップグレード可能かチェック
    if payload.target_tier.level() <= current_tier.level() {
        return Err(AppError::BadRequest(
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

/// 利用可能なサブスクリプション階層一覧を取得
pub async fn get_available_tiers_handler(
    _user: AuthenticatedUser,
) -> AppResult<Json<ApiResponse<Vec<SubscriptionTierInfo>>>> {
    let tiers = SubscriptionTier::all();
    let tier_infos: Vec<SubscriptionTierInfo> = tiers
        .into_iter()
        .map(|tier| CurrentSubscriptionResponse::get_tier_info(tier.as_str()))
        .collect();

    Ok(Json(ApiResponse::success(
        "Available subscription tiers retrieved successfully",
        tier_infos,
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

    let current_tier = SubscriptionTier::from_str(&current_user.subscription_tier)
        .ok_or_else(|| AppError::BadRequest("Invalid current subscription tier".to_string()))?;

    // ダウングレード可能かチェック
    if payload.target_tier.level() >= current_tier.level() {
        return Err(AppError::BadRequest(
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

/// サブスクリプション階層を手動で変更（管理者用）
pub async fn change_user_subscription_handler(
    State(app_state): State<AppState>,
    admin: AuthenticatedUserWithRole,
    UuidPath(user_id): UuidPath,
    Json(payload): Json<AdminChangeSubscriptionRequest>,
) -> AppResult<Json<ApiResponse<SubscriptionChangeResponse>>> {
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

    // 現在のサブスクリプション階層を取得
    let current_user = app_state.user_service.get_user_profile(user_id).await?;

    // 強制変更でない場合、同じ階層への変更はエラー
    if !payload.force_change.unwrap_or(false) && current_user.subscription_tier == payload.new_tier
    {
        return Err(AppError::BadRequest(
            "User already has this subscription tier. Use force_change=true to force the change."
                .to_string(),
        ));
    }

    info!(
        admin_id = %admin.user.user_id,
        target_user_id = %user_id,
        from_tier = %current_user.subscription_tier,
        to_tier = %payload.new_tier,
        "Admin subscription change attempt"
    );

    // サブスクリプション変更を実行
    let (updated_user, _history) = app_state
        .subscription_service
        .change_subscription_tier(
            user_id,
            payload.new_tier.clone(),
            Some(admin.user.user_id),
            payload.reason.clone(),
        )
        .await?;

    let change_response = SubscriptionChangeResponse::new(
        updated_user.into(),
        current_user.subscription_tier,
        payload.new_tier.clone(),
        payload.reason,
        Some(admin.user.user_id),
    );

    info!(
        admin_id = %admin.user.user_id,
        target_user_id = %user_id,
        new_tier = %payload.new_tier,
        "Admin subscription change successful"
    );

    Ok(Json(ApiResponse::success(
        "User subscription changed successfully",
        change_response,
    )))
}

/// ユーザーのサブスクリプション履歴取得
pub async fn get_user_subscription_history_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Query(pagination): Query<PaginationQuery>,
) -> AppResult<Json<ApiResponse<SubscriptionHistoryResponse>>> {
    let page = pagination.page.unwrap_or(1).max(1);
    let page_size = pagination.page_size.unwrap_or(20).max(1).min(100);

    let (history, total) = app_state
        .subscription_service
        .get_user_subscription_history(user.claims.user_id, page, page_size)
        .await?;

    let response = SubscriptionHistoryResponse {
        user_id: user.claims.user_id,
        history,
        pagination: PaginationMeta::new(page, page_size, total),
    };

    Ok(Json(ApiResponse::success(
        "Subscription history retrieved successfully",
        response,
    )))
}

/// ユーザーのサブスクリプション統計取得
pub async fn get_user_subscription_stats_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
) -> AppResult<Json<ApiResponse<UserSubscriptionStatsResponse>>> {
    let stats = app_state
        .subscription_service
        .get_user_subscription_stats(user.claims.user_id)
        .await?;

    let response = UserSubscriptionStatsResponse::from_stats(stats);

    Ok(Json(ApiResponse::success(
        "Subscription statistics retrieved successfully",
        response,
    )))
}

/// サブスクリプション階層別統計取得（管理者用）
pub async fn get_subscription_tier_stats_handler(
    State(app_state): State<AppState>,
    _admin: AuthenticatedUserWithRole,
) -> AppResult<Json<ApiResponse<Vec<SubscriptionTierStatsResponse>>>> {
    let tier_stats = app_state
        .subscription_service
        .get_subscription_tier_stats()
        .await?;

    let response: Vec<SubscriptionTierStatsResponse> = tier_stats
        .into_iter()
        .map(SubscriptionTierStatsResponse::from_stats)
        .collect();

    Ok(Json(ApiResponse::success(
        "Subscription tier statistics retrieved successfully",
        response,
    )))
}

/// 期間内のサブスクリプション履歴取得（管理者用）
pub async fn get_subscription_history_by_period_handler(
    State(app_state): State<AppState>,
    _admin: AuthenticatedUserWithRole,
    Query(query): Query<PeriodQuery>,
) -> AppResult<Json<ApiResponse<Vec<SubscriptionChangeInfo>>>> {
    let end_date = query.end_date.unwrap_or_else(Utc::now);
    let start_date = query
        .start_date
        .unwrap_or_else(|| end_date - Duration::days(30));

    if start_date > end_date {
        return Err(AppError::BadRequest(
            "Start date must be before end date".to_string(),
        ));
    }

    let history = app_state
        .subscription_service
        .get_subscription_history_by_date_range(start_date, end_date)
        .await?;

    Ok(Json(ApiResponse::success(
        "Subscription history for period retrieved successfully",
        history,
    )))
}

/// 期間クエリパラメータ
#[derive(Debug, Deserialize)]
pub struct PeriodQuery {
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}

/// 管理者用サブスクリプション概要取得
pub async fn get_subscription_overview_handler(
    State(app_state): State<AppState>,
    _admin: AuthenticatedUserWithRole,
    Query(query): Query<AdminStatsQuery>,
) -> AppResult<Json<ApiResponse<SubscriptionOverviewResponse>>> {
    let days = query.days.unwrap_or(30) as i64;
    let start_date = Utc::now() - Duration::days(days);

    // 並行して各統計を取得
    let (distribution, tier_changes, upgrades, downgrades, tier_stats) = tokio::try_join!(
        app_state
            .subscription_service
            .get_subscription_distribution(),
        app_state.subscription_service.get_tier_change_statistics(),
        app_state.subscription_service.get_upgrade_history(),
        app_state.subscription_service.get_downgrade_history(),
        app_state.subscription_service.get_subscription_tier_stats()
    )?;

    // 期間内のアップグレード・ダウングレードをフィルタ
    let recent_upgrades: Vec<_> = upgrades
        .into_iter()
        .filter(|u| u.changed_at >= start_date)
        .collect();
    let recent_downgrades: Vec<_> = downgrades
        .into_iter()
        .filter(|d| d.changed_at >= start_date)
        .collect();

    let response = SubscriptionOverviewResponse {
        total_users: distribution.iter().map(|(_, count)| count).sum(),
        distribution: distribution
            .into_iter()
            .map(|(tier, count)| TierDistribution {
                tier,
                user_count: count,
            })
            .collect(),
        tier_changes: tier_changes
            .into_iter()
            .map(|(tier, count)| TierChangeStats {
                tier,
                change_count: count,
            })
            .collect(),
        recent_upgrades_count: recent_upgrades.len() as u64,
        recent_downgrades_count: recent_downgrades.len() as u64,
        period_days: days as u32,
        tier_stats: tier_stats
            .into_iter()
            .map(SubscriptionTierStatsResponse::from_stats)
            .collect(),
        // 収益情報は仮実装
        revenue_stats: if query.include_revenue.unwrap_or(false) {
            Some(RevenueStats {
                monthly_recurring_revenue: 0.0,
                annual_recurring_revenue: 0.0,
                average_revenue_per_user: 0.0,
            })
        } else {
            None
        },
    };

    Ok(Json(ApiResponse::success(
        "Subscription overview retrieved successfully",
        response,
    )))
}

/// サブスクリプション用ルーター
pub fn subscription_router() -> Router<AppState> {
    Router::new()
        // ユーザー向けエンドポイント
        .route("/current", get(get_current_subscription_handler))
        .route("/upgrade", post(upgrade_subscription_handler))
        .route("/downgrade", post(downgrade_subscription_handler))
        .route("/tiers", get(get_available_tiers_handler))
        .route("/history", get(get_user_subscription_history_handler))
        .route("/stats", get(get_user_subscription_stats_handler))
}

/// 管理者用サブスクリプションルーター
pub fn admin_subscription_router() -> Router<AppState> {
    Router::new()
        .route("/users/:user_id", patch(change_user_subscription_handler))
        .route("/overview", get(get_subscription_overview_handler))
        .route("/stats/tiers", get(get_subscription_tier_stats_handler))
        .route("/history", get(get_subscription_history_by_period_handler))
}

/// 管理者権限を要求するルーターを生成
pub fn subscription_router_with_state() -> Router<AppState> {
    Router::new()
        .nest("/subscriptions", subscription_router())
        .nest("/admin/subscriptions", admin_subscription_router())
}
