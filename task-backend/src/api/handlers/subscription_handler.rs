// task-backend/src/api/handlers/subscription_handler.rs

use crate::api::dto::common::{ApiResponse, OperationResult, PaginationQuery};
use crate::api::dto::subscription_dto::*;
use crate::api::AppState;
use crate::core::subscription_tier::SubscriptionTier;
use crate::error::{AppError, AppResult};
use crate::middleware::auth::{AuthenticatedUser, AuthenticatedUserWithRole};
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
    Query(query): Query<PaginationQuery>,
) -> AppResult<Json<SubscriptionHistoryResponse>> {
    let (page, per_page) = query.get_pagination();

    info!(
        user_id = %user.claims.user_id,
        page = %page,
        per_page = %per_page,
        "Subscription history request"
    );

    // サブスクリプション履歴を取得
    let (history, total_count) = app_state
        .subscription_service
        .get_user_subscription_history(user.claims.user_id, page as u64, per_page as u64)
        .await?;

    let stats = app_state
        .subscription_service
        .get_user_subscription_stats(user.claims.user_id)
        .await?;

    let pagination = crate::api::dto::PaginationMeta::new(page, per_page, total_count as i64);

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

/// 管理者向けサブスクリプション履歴取得（拡張版）
pub async fn get_admin_subscription_history_extended_handler(
    State(app_state): State<AppState>,
    admin_user: AuthenticatedUserWithRole,
    Query(query): Query<SubscriptionHistoryExtendedQuery>,
) -> AppResult<Json<serde_json::Value>> {
    // 管理者権限チェック
    if !admin_user.is_admin() {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    // リポジトリを取得
    let subscription_history_repo = &app_state.subscription_history_repo;

    // 日付範囲の設定
    let end_date = query.end_date.unwrap_or_else(Utc::now);
    let start_date = query
        .start_date
        .unwrap_or_else(|| end_date - Duration::days(30));

    // フィルタによって履歴を取得
    let histories = match query.filter.as_deref() {
        Some("upgrades") => subscription_history_repo
            .find_upgrades()
            .await
            .unwrap_or_default(),
        Some("downgrades") => subscription_history_repo
            .find_downgrades()
            .await
            .unwrap_or_default(),
        _ => subscription_history_repo
            .find_by_date_range(start_date, end_date)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|h| h.into())
            .collect(),
    };

    // 階層変更統計を取得
    let tier_stats_raw = subscription_history_repo
        .get_tier_change_stats()
        .await
        .unwrap_or_default();

    // Convert tuples to objects
    let tier_stats: Vec<_> = tier_stats_raw
        .into_iter()
        .map(|(tier, count)| {
            json!({
                "tier": tier,
                "count": count
            })
        })
        .collect();

    // 変更サマリーを計算
    let total_changes = histories.len();
    let upgrades_count = histories
        .iter()
        .filter(|h| {
            // アップグレードの判定ロジック
            matches!(
                (h.previous_tier.as_deref(), h.new_tier.as_str()),
                (Some("free"), "pro") | (Some("free"), "enterprise") | (Some("pro"), "enterprise")
            )
        })
        .count();
    let downgrades_count = histories
        .iter()
        .filter(|h| {
            // ダウングレードの判定ロジック
            matches!(
                (h.previous_tier.as_deref(), h.new_tier.as_str()),
                (Some("enterprise"), "pro") | (Some("enterprise"), "free") | (Some("pro"), "free")
            )
        })
        .count();

    let response = json!({
        "success": true,
        "message": "Subscription history retrieved successfully",
        "data": {
            "histories": histories,
            "tier_stats": tier_stats,
            "change_summary": {
                "total_changes": total_changes,
                "upgrades_count": upgrades_count,
                "downgrades_count": downgrades_count,
                "date_range": {
                    "start": start_date,
                    "end": end_date
                }
            }
        }
    });

    Ok(Json(response))
}

/// クエリパラメータ（拡張版）
#[derive(Debug, Deserialize)]
pub struct SubscriptionHistoryExtendedQuery {
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub filter: Option<String>, // "upgrades" | "downgrades"
}

/// 管理者向けサブスクリプション履歴取得
pub async fn get_admin_subscription_history_handler(
    State(app_state): State<AppState>,
    admin_user: AuthenticatedUserWithRole,
    Query(query): Query<SubscriptionHistoryQuery>,
) -> AppResult<Json<AdminSubscriptionHistoryResponse>> {
    // 管理者権限チェック
    if !admin_user.is_admin() {
        warn!(
            user_id = %admin_user.user_id(),
            role = ?admin_user.role().map(|r| &r.name),
            "Access denied: Admin permission required for subscription history"
        );
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    // バリデーション
    query.validate().map_err(|validation_errors| {
        warn!(
            "Admin subscription history query validation failed: {}",
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
    let page_size = query.page_size.unwrap_or(50);

    // 日付範囲の設定（デフォルトは過去30日）
    let end_date = query.end_date.unwrap_or_else(Utc::now);
    let start_date = query
        .start_date
        .unwrap_or_else(|| end_date - Duration::days(30));

    info!(
        admin_id = %admin_user.user_id(),
        start_date = %start_date,
        end_date = %end_date,
        change_type = ?query.change_type,
        page = %page,
        page_size = %page_size,
        "Admin subscription history request"
    );

    // サブスクリプション履歴を取得
    let all_changes = app_state
        .subscription_service
        .get_subscription_history_by_date_range(start_date, end_date)
        .await?;

    // 変更タイプでフィルタリング
    let filtered_changes: Vec<_> = match query.change_type {
        Some(SubscriptionChangeType::Upgrade) => {
            all_changes.into_iter().filter(|c| c.is_upgrade).collect()
        }
        Some(SubscriptionChangeType::Downgrade) => {
            all_changes.into_iter().filter(|c| c.is_downgrade).collect()
        }
        _ => all_changes,
    };

    // 統計情報を計算
    let total_changes = filtered_changes.len() as u64;
    let upgrades_count = filtered_changes.iter().filter(|c| c.is_upgrade).count() as u64;
    let downgrades_count = filtered_changes.iter().filter(|c| c.is_downgrade).count() as u64;

    // ページネーション
    let start_idx = ((page - 1) * page_size) as usize;
    let paginated_changes: Vec<_> = filtered_changes
        .into_iter()
        .skip(start_idx)
        .take(page_size as usize)
        .collect();

    let pagination =
        crate::api::dto::PaginationMeta::new(page as i32, page_size as i32, total_changes as i64);

    let response = AdminSubscriptionHistoryResponse {
        changes: paginated_changes,
        pagination,
        summary: SubscriptionHistorySummary {
            total_changes,
            upgrades_count,
            downgrades_count,
            date_range: DateRange {
                start_date,
                end_date,
            },
        },
    };

    info!(
        admin_id = %admin_user.user_id(),
        total_changes = %response.summary.total_changes,
        "Admin subscription history retrieved"
    );

    Ok(Json(response))
}

/// サブスクリプション統計取得（Phase 5.2版）
pub async fn get_subscription_stats_v2_handler(
    State(app_state): State<AppState>,
    admin_user: AuthenticatedUserWithRole,
) -> AppResult<Json<SubscriptionStatsResponseV2>> {
    // 管理者権限チェック
    if !admin_user.is_admin() {
        warn!(
            user_id = %admin_user.user_id(),
            role = ?admin_user.role().map(|r| &r.name),
            "Access denied: Admin permission required for subscription stats v2"
        );
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    info!(
        admin_id = %admin_user.user_id(),
        "Admin subscription stats v2 request"
    );

    // 階層変更統計を取得
    let tier_change_stats = app_state
        .subscription_service
        .get_tier_change_statistics()
        .await?;

    // 統計データを変換
    let total_changes: u64 = tier_change_stats.iter().map(|(_, count)| count).sum();
    let tier_stats: Vec<TierChangeStat> = tier_change_stats
        .into_iter()
        .map(|(tier, count)| {
            let percentage = if total_changes > 0 {
                (count as f64 / total_changes as f64) * 100.0
            } else {
                0.0
            };
            TierChangeStat {
                tier,
                change_count: count,
                percentage,
            }
        })
        .collect();

    // トレンド分析（過去30日間）
    let end_date = Utc::now();
    let start_date = end_date - Duration::days(30);
    let history = app_state
        .subscription_service
        .get_subscription_history_by_date_range(start_date, end_date)
        .await?;

    let upgrades = history.iter().filter(|h| h.is_upgrade).count() as i64;
    let downgrades = history.iter().filter(|h| h.is_downgrade).count() as i64;
    let net_movement = upgrades - downgrades;

    // 成長率とチャーン率の計算（簡易版）
    let total_users = app_state
        .subscription_service
        .get_subscription_tier_stats()
        .await?
        .iter()
        .map(|s| s.total_users)
        .sum::<u64>() as f64;

    let growth_rate = if total_users > 0.0 {
        (upgrades as f64 / total_users) * 100.0
    } else {
        0.0
    };

    let churn_rate = if total_users > 0.0 {
        (downgrades as f64 / total_users) * 100.0
    } else {
        0.0
    };

    // 収益影響の計算（簡易版）
    let price_map = [("free", 0.0), ("pro", 19.99), ("enterprise", 99.99)]
        .into_iter()
        .collect::<std::collections::HashMap<_, _>>();

    let mut upgrades_revenue = 0.0;
    let mut downgrades_revenue_loss = 0.0;

    for change in &history {
        let prev_price = change
            .previous_tier
            .as_ref()
            .and_then(|t| price_map.get(t.to_lowercase().as_str()))
            .copied()
            .unwrap_or(0.0);
        let new_price = price_map
            .get(change.new_tier.to_lowercase().as_str())
            .copied()
            .unwrap_or(0.0);

        let revenue_diff: f64 = new_price - prev_price;
        if revenue_diff > 0.0 {
            upgrades_revenue += revenue_diff;
        } else {
            downgrades_revenue_loss += revenue_diff.abs();
        }
    }

    let revenue_change = upgrades_revenue - downgrades_revenue_loss;
    let revenue_change_percentage = if upgrades_revenue + downgrades_revenue_loss > 0.0 {
        (revenue_change / (upgrades_revenue + downgrades_revenue_loss)) * 100.0
    } else {
        0.0
    };

    let response = SubscriptionStatsResponseV2 {
        tier_change_stats: tier_stats,
        trend_analysis: TrendAnalysis {
            growth_rate,
            churn_rate,
            net_movement,
            period: "last_30_days".to_string(),
        },
        revenue_impact: RevenueImpact {
            revenue_change,
            revenue_change_percentage,
            upgrades_revenue,
            downgrades_revenue_loss,
        },
    };

    info!(
        admin_id = %admin_user.user_id(),
        total_changes = %total_changes,
        net_movement = %net_movement,
        "Subscription stats v2 retrieved"
    );

    Ok(Json(response))
}

/// ユーザー別サブスクリプション履歴取得
pub async fn get_user_subscription_history_handler(
    State(app_state): State<AppState>,
    Path(user_id): Path<Uuid>,
    authenticated_user: AuthenticatedUserWithRole,
    Query(query): Query<PaginationQuery>,
) -> AppResult<Json<SubscriptionHistoryResponse>> {
    // アクセス権限チェック（本人または管理者）
    if authenticated_user.user_id() != user_id && !authenticated_user.is_admin() {
        warn!(
            requester_id = %authenticated_user.user_id(),
            target_user_id = %user_id,
            role = ?authenticated_user.role().map(|r| &r.name),
            "Access denied: User can only view their own subscription history"
        );
        return Err(AppError::Forbidden(
            "You can only view your own subscription history".to_string(),
        ));
    }

    let (page, per_page) = query.get_pagination();

    info!(
        requester_id = %authenticated_user.user_id(),
        target_user_id = %user_id,
        page = %page,
        per_page = %per_page,
        "User subscription history request"
    );

    // サブスクリプション履歴を取得
    let (history, total_count) = app_state
        .subscription_service
        .get_user_subscription_history(user_id, page as u64, per_page as u64)
        .await?;

    let stats = app_state
        .subscription_service
        .get_user_subscription_stats(user_id)
        .await?;

    let pagination = crate::api::dto::PaginationMeta::new(page, per_page, total_count as i64);

    let response = SubscriptionHistoryResponse {
        user_id,
        history,
        pagination: Some(pagination),
        stats,
    };

    info!(
        requester_id = %authenticated_user.user_id(),
        target_user_id = %user_id,
        history_count = %response.history.len(),
        "User subscription history retrieved"
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

/// サブスクリプション履歴詳細取得
pub async fn get_subscription_history_detail_handler(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Path(history_id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<SubscriptionHistoryDetailResponse>>> {
    info!(
        user_id = %user.user_id(),
        history_id = %history_id,
        "Getting subscription history detail"
    );

    // 履歴を取得（find_by_idメソッドを活用）
    let history = app_state
        .subscription_service
        .get_subscription_history_detail(history_id)
        .await?;

    // アクセス権限チェック（自分の履歴または管理者のみ）
    if history.user_id != user.user_id() && !user.is_admin() {
        warn!(
            user_id = %user.user_id(),
            history_user_id = %history.user_id,
            "Access denied: Cannot view other user's subscription history"
        );
        return Err(AppError::Forbidden(
            "You can only view your own subscription history".to_string(),
        ));
    }

    // 変更実行者の情報を取得
    let changed_by_user = if let Some(changed_by_id) = history.changed_by {
        app_state
            .user_service
            .get_user_by_id(changed_by_id)
            .await
            .ok()
            .map(|user| ChangedByUserInfo {
                user_id: user.id,
                username: user.username,
                email: user.email,
                role: "admin".to_string(), // 変更実行者は通常管理者
            })
    } else {
        None
    };

    // 階層情報を取得して比較
    let previous_tier_info = history
        .previous_tier
        .as_ref()
        .map(|tier| CurrentSubscriptionResponse::get_tier_info(tier));
    let new_tier_info = CurrentSubscriptionResponse::get_tier_info(&history.new_tier);

    // 機能の追加・削除を計算
    let (features_added, features_removed) = if let Some(prev_info) = &previous_tier_info {
        let added: Vec<String> = new_tier_info
            .features
            .iter()
            .filter(|f| !prev_info.features.contains(f))
            .cloned()
            .collect();
        let removed: Vec<String> = prev_info
            .features
            .iter()
            .filter(|f| !new_tier_info.features.contains(f))
            .cloned()
            .collect();
        (added, removed)
    } else {
        (new_tier_info.features.clone(), vec![])
    };

    // 制限の変更を計算
    let limits_changed = if let Some(prev_info) = &previous_tier_info {
        LimitsChanged {
            max_tasks_change: calculate_limit_change(
                prev_info.limits.max_tasks,
                new_tier_info.limits.max_tasks,
            ),
            max_projects_change: calculate_limit_change(
                prev_info.limits.max_projects,
                new_tier_info.limits.max_projects,
            ),
            max_team_members_change: calculate_limit_change(
                prev_info.limits.max_team_members,
                new_tier_info.limits.max_team_members,
            ),
            max_storage_mb_change: calculate_limit_change(
                prev_info.limits.max_storage_mb,
                new_tier_info.limits.max_storage_mb,
            ),
            api_requests_per_hour_change: calculate_limit_change(
                prev_info.limits.api_requests_per_hour,
                new_tier_info.limits.api_requests_per_hour,
            ),
        }
    } else {
        LimitsChanged {
            max_tasks_change: new_tier_info.limits.max_tasks.map(|v| v as i64),
            max_projects_change: new_tier_info.limits.max_projects.map(|v| v as i64),
            max_team_members_change: new_tier_info.limits.max_team_members.map(|v| v as i64),
            max_storage_mb_change: new_tier_info.limits.max_storage_mb.map(|v| v as i64),
            api_requests_per_hour_change: new_tier_info
                .limits
                .api_requests_per_hour
                .map(|v| v as i64),
        }
    };

    let response = SubscriptionHistoryDetailResponse {
        history_id: history.id,
        user_id: history.user_id,
        previous_tier: history.previous_tier.clone(),
        new_tier: history.new_tier.clone(),
        change_type: history.change_type(),
        reason: history.reason.clone(),
        changed_at: history.changed_at,
        changed_by: history.changed_by,
        changed_by_user,
        tier_comparison: TierComparison {
            previous_tier_info,
            new_tier_info,
            features_added,
            features_removed,
            limits_changed,
        },
    };

    info!(
        user_id = %user.user_id(),
        history_id = %history_id,
        "Subscription history detail retrieved"
    );

    Ok(Json(ApiResponse::success(
        "Subscription history detail retrieved successfully",
        response,
    )))
}

// ヘルパー関数：制限の変更を計算
fn calculate_limit_change(old: Option<u32>, new: Option<u32>) -> Option<i64> {
    match (old, new) {
        (Some(old_val), Some(new_val)) => Some(new_val as i64 - old_val as i64),
        (None, Some(new_val)) => Some(new_val as i64), // 無制限から制限へ
        (Some(old_val), None) => Some(-(old_val as i64)), // 制限から無制限へ
        (None, None) => None,                          // 両方無制限
    }
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
        .route("/subscriptions/tiers", get(get_available_tiers_handler))
        .route("/subscriptions/upgrade", post(upgrade_subscription_handler))
        .route(
            "/subscriptions/downgrade",
            post(downgrade_subscription_handler),
        )
        .route(
            "/subscriptions/history",
            get(get_subscription_history_handler),
        )
        .route(
            "/subscriptions/history/{id}",
            get(get_subscription_history_detail_handler),
        )
        // ユーザー別履歴（Phase 5.2）
        .route(
            "/users/{id}/subscription/history",
            get(get_user_subscription_history_handler),
        )
        // 管理者向けエンドポイント
        .route(
            "/admin/subscriptions/stats",
            get(get_subscription_stats_handler),
        )
        .route(
            "/admin/subscription/history",
            get(get_admin_subscription_history_extended_handler),
        )
        .route(
            "/admin/subscription/history/v1",
            get(get_admin_subscription_history_handler),
        )
        .route(
            "/admin/subscription/stats",
            get(get_subscription_stats_v2_handler),
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
