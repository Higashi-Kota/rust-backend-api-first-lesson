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
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use uuid::Uuid;
use validator::Validate;

use super::super::dto::subscription::*;
use super::super::dto::SubscriptionOverviewResponse;
use crate::features::subscription::models::history::SubscriptionChangeInfo;

/// 管理者用統計クエリパラメータ
#[derive(Debug, Deserialize)]
pub struct AdminStatsQuery {
    pub include_revenue: Option<bool>,
    pub days: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct SubscriptionHistoryAdminResponse {
    pub histories: Vec<SubscriptionChangeInfo>,
    pub tier_stats: Vec<TierChangeStats>,
    pub change_summary: ChangeSummary,
}

#[derive(Debug, Serialize)]
pub struct TierChangeStats {
    pub tier: String,
    pub count: u64,
}

#[derive(Debug, Serialize)]
pub struct ChangeSummary {
    pub total_changes: u64,
    pub upgrades_count: u64,
    pub downgrades_count: u64,
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
        admin_id = %admin.user_id(),
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
            Some(admin.user_id()),
            payload.reason.clone(),
        )
        .await?;

    let change_response = SubscriptionChangeResponse::new(
        updated_user.into(),
        current_user.subscription_tier,
        payload.new_tier.clone(),
        payload.reason,
        Some(admin.user_id()),
    );

    info!(
        admin_id = %admin.user_id(),
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
    let page_size = pagination.per_page.unwrap_or(20).clamp(1, 100);

    let (history, total) = app_state
        .subscription_service
        .get_user_subscription_history(user.claims.user_id, page as u64, page_size as u64)
        .await?;

    let stats = app_state
        .subscription_service
        .get_user_subscription_stats(user.claims.user_id)
        .await?;

    let response = SubscriptionHistoryResponse {
        user_id: user.claims.user_id,
        history,
        pagination: Some(PaginationMeta::new(page, page_size, total as i64)),
        stats,
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
) -> AppResult<Json<ApiResponse<crate::features::subscription::dto::UserSubscriptionStatsResponse>>>
{
    let stats = app_state
        .subscription_service
        .get_user_subscription_stats(user.claims.user_id)
        .await?;

    let response =
        crate::features::subscription::dto::UserSubscriptionStatsResponse::from_stats(stats);

    Ok(Json(ApiResponse::success(
        "Subscription statistics retrieved successfully",
        response,
    )))
}

/// サブスクリプション階層別統計取得（管理者用）
pub async fn get_subscription_tier_stats_handler(
    State(app_state): State<AppState>,
    _admin: AuthenticatedUserWithRole,
) -> AppResult<
    Json<ApiResponse<Vec<crate::features::subscription::dto::SubscriptionTierStatsResponse>>>,
> {
    let tier_stats = app_state
        .subscription_service
        .get_subscription_tier_stats()
        .await?;

    let response: Vec<crate::features::subscription::dto::SubscriptionTierStatsResponse> =
        tier_stats
            .into_iter()
            .map(crate::features::subscription::dto::SubscriptionTierStatsResponse::from_stats)
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
    let recent_upgrades: Vec<SubscriptionChangeInfo> = upgrades
        .into_iter()
        .filter(|u| u.changed_at >= start_date)
        .collect();
    let recent_downgrades: Vec<SubscriptionChangeInfo> = downgrades
        .into_iter()
        .filter(|d| d.changed_at >= start_date)
        .collect();

    let response = crate::features::subscription::dto::SubscriptionOverviewResponse {
        total_users: distribution.iter().map(|(_, count)| count).sum(),
        distribution: distribution
            .into_iter()
            .map(
                |(tier, count)| crate::features::subscription::dto::TierDistribution {
                    tier,
                    user_count: count,
                },
            )
            .collect(),
        tier_changes: tier_changes
            .into_iter()
            .map(
                |(tier, count)| crate::features::subscription::dto::TierChangeStats {
                    tier,
                    change_count: count,
                },
            )
            .collect(),
        recent_upgrades_count: recent_upgrades.len() as u64,
        recent_downgrades_count: recent_downgrades.len() as u64,
        period_days: days as u32,
        tier_stats: tier_stats
            .into_iter()
            .map(crate::features::subscription::dto::SubscriptionTierStatsResponse::from_stats)
            .collect(),
        // 収益情報は仮実装
        revenue_stats: if query.include_revenue.unwrap_or(false) {
            Some(crate::features::subscription::dto::RevenueStats {
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
        .route("/users/{user_id}", patch(change_user_subscription_handler))
        .route("/overview", get(get_subscription_overview_handler))
        .route("/stats/tiers", get(get_subscription_tier_stats_handler))
        .route("/history", get(get_subscription_history_by_period_handler))
        .route("/history/search", get(search_subscription_history_handler))
        .route("/analytics", get(get_subscription_analytics_handler))
}

// Legacy admin subscription router removed - use new endpoints

/// Get subscription history for a specific user - allows both user self-access and admin access
pub async fn get_user_subscription_history_by_id_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Path(user_id): Path<Uuid>,
    Query(query): Query<PaginationQuery>,
) -> AppResult<Json<serde_json::Value>> {
    // Check authorization: users can only access their own history unless they're admins
    if user.user_id() != user_id && !user.is_admin() {
        return Err(AppError::Forbidden(
            "You can only access your own subscription history".to_string(),
        ));
    }

    let page = query.page.unwrap_or(1) as u64;
    let per_page = query.per_page.unwrap_or(20) as u64;

    let (history, _total_count) = app_state
        .subscription_service
        .get_user_subscription_history(user_id, page, per_page)
        .await?;

    // Get stats
    let stats = app_state
        .subscription_service
        .get_user_subscription_stats(user_id)
        .await?;

    // Get current user info for current tier
    let user_profile = app_state.user_service.get_user_profile(user_id).await?;

    // Format response to match test expectations
    let response = serde_json::json!({
        "user_id": user_id,
        "history": history,
        "stats": {
            "total_changes": stats.total_changes,
            "upgrade_count": stats.upgrade_count,
            "downgrade_count": stats.downgrade_count,
            "current_tier": user_profile.subscription_tier,
            "days_at_current_tier": stats.days_at_current_tier,
            "has_ever_upgraded": stats.has_ever_upgraded,
        }
    });

    Ok(Json(response))
}

/// Search subscription history by tier (admin)
async fn search_subscription_history_handler(
    State(app_state): State<AppState>,
    _admin: AuthenticatedUserWithRole,
    Query(query): Query<SubscriptionSearchQuery>,
) -> AppResult<Json<ApiResponse<Vec<SubscriptionChangeInfo>>>> {
    let end_date = Utc::now();
    let start_date = end_date - Duration::days(30); // Default to last 30 days

    let mut history = app_state
        .subscription_service
        .get_subscription_history_by_date_range(start_date, end_date)
        .await?;

    // Filter by tier if provided
    if let Some(tier) = query.tier {
        history.retain(|h| h.new_tier == tier || h.previous_tier.as_ref() == Some(&tier));
    }

    Ok(Json(ApiResponse::success(
        "Subscription history search completed",
        history,
    )))
}

/// Get subscription analytics (admin)
async fn get_subscription_analytics_handler(
    State(app_state): State<AppState>,
    _admin: AuthenticatedUserWithRole,
) -> AppResult<Json<ApiResponse<SubscriptionAnalyticsResponse>>> {
    let distribution = app_state
        .subscription_service
        .get_subscription_distribution()
        .await?;

    let response = SubscriptionAnalyticsResponse {
        total_subscriptions: distribution.iter().map(|(_, count)| *count as i64).sum(),
        tier_distribution: distribution
            .into_iter()
            .map(|(tier, count)| TierDistribution {
                tier,
                count: count as i64,
            })
            .collect(),
        growth_rate: 0.0, // TODO: Implement growth rate calculation
        churn_rate: 0.0,  // TODO: Implement churn rate calculation
    };

    Ok(Json(ApiResponse::success(
        "Subscription analytics retrieved successfully",
        response,
    )))
}

#[derive(Debug, Deserialize)]
struct SubscriptionSearchQuery {
    tier: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SubscriptionAnalyticsResponse {
    pub total_subscriptions: i64,
    pub tier_distribution: Vec<TierDistribution>,
    pub growth_rate: f64,
    pub churn_rate: f64,
}

#[derive(Debug, Serialize)]
pub struct TierDistribution {
    pub tier: String,
    pub count: i64,
}

/// 管理者権限を要求するルーターを生成
pub fn subscription_router_with_state() -> Router<AppState> {
    Router::new()
        .nest("/subscriptions", subscription_router())
        .nest("/admin/subscriptions", admin_subscription_router())
        // Legacy paths for backward compatibility
        .route(
            "/users/{user_id}/subscription/history",
            get(get_user_subscription_history_by_id_handler),
        )
}
