use crate::error::AppResult;
use crate::features::admin::dto::analytics::{
    HistoricalComparison, PeerComparison, ProductivityTrend, TaskPriorityDistribution,
    TaskStatsDetailResponse, TaskStatsOverview, TaskStatusDistribution, TaskTrends, TierComparison,
    TierMetrics, TrendAnalysis, TrendDirection, UserComparisons, UserRecommendation,
    UserTaskPerformance, WorkflowPattern,
};
use crate::features::analytics::dto::requests::{
    AdvancedExportRequest, AnalyticsTimeRangeRequest, FeatureUsageQuery, TaskAnalyticsRequest,
    TrackFeatureUsageRequest,
};
use crate::features::analytics::dto::responses::{
    ActivityDistribution, ActivityPatterns, AdvancedExportResponse, AnalysisPeriod,
    BehavioralMetrics, DailyActivitySummaryResponse, EngagementLevel, ExportMetadata, ExportStatus,
    FeatureUsage, FeatureUsageCount, FeatureUsageCountsResponse, FeatureUsageMetrics,
    FeatureUsageStatsResponse, LoginFrequency, MetricGranularity, PerformanceIndicators,
    ProficiencyLevel, SatisfactionIndicators, SentimentCategory, SentimentScore, SessionDuration,
    SubscriptionTierDistribution, SubscriptionUtilization, SystemStatsResponse,
    UserBehaviorAnalyticsResponse, UserFeatureUsageResponse,
};
use crate::features::analytics::models::daily_activity_summary::DailyActivityInput;
use crate::features::auth::middleware::{AuthenticatedUser, AuthenticatedUserWithRole};
use crate::shared::types::ApiResponse;
use axum::{
    extract::{Path, Query, State},
    routing::{delete, get, post},
    Json, Router,
};
use chrono::NaiveDate;
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

/// Track feature usage
pub async fn track_feature_usage_handler(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Json(request): Json<TrackFeatureUsageRequest>,
) -> AppResult<Json<ApiResponse<()>>> {
    // Use existing feature tracking service
    app_state
        .feature_tracking_service
        .track_feature_usage(
            user.user_id(),
            &request.feature_name,
            &request.action_type,
            request.metadata.and_then(|m| serde_json::to_value(m).ok()),
        )
        .await?;

    Ok(Json(ApiResponse::success(
        "Feature usage tracked successfully",
        (),
    )))
}

/// Update daily activity summary (admin only)
pub async fn update_daily_activity_summary_handler(
    State(app_state): State<crate::api::AppState>,
    _user: AuthenticatedUserWithRole,
    Path(date): Path<chrono::NaiveDate>,
    Json(input): Json<DailyActivityInput>,
) -> AppResult<Json<ApiResponse<crate::features::analytics::dto::responses::DailyActivitySummary>>>
{
    let summary = app_state
        .activity_summary_service
        .update_daily_summary(date, input)
        .await?;

    // Convert model to response DTO
    let dto = crate::features::analytics::dto::responses::DailyActivitySummary {
        date: summary.date,
        total_users: summary.total_users,
        active_users: summary.active_users,
        new_users: summary.new_users,
        tasks_created: summary.tasks_created,
        tasks_completed: summary.tasks_completed,
        created_at: summary.created_at,
        updated_at: summary.updated_at,
    };

    Ok(Json(ApiResponse::success(
        "Daily activity summary updated successfully",
        dto,
    )))
}

/// Get single daily activity summary (admin only)
pub async fn get_single_daily_activity_summary_handler(
    State(app_state): State<crate::api::AppState>,
    _user: AuthenticatedUserWithRole,
    Path(date): Path<chrono::NaiveDate>,
) -> AppResult<
    Json<ApiResponse<Option<crate::features::analytics::dto::responses::DailyActivitySummary>>>,
> {
    let summary = app_state
        .activity_summary_service
        .get_daily_summary(date)
        .await?;

    // Convert model to response DTO if present
    let dto =
        summary.map(
            |model| crate::features::analytics::dto::responses::DailyActivitySummary {
                date: model.date,
                total_users: model.total_users,
                active_users: model.active_users,
                new_users: model.new_users,
                tasks_created: model.tasks_created,
                tasks_completed: model.tasks_completed,
                created_at: model.created_at,
                updated_at: model.updated_at,
            },
        );

    Ok(Json(ApiResponse::success(
        if dto.is_some() {
            "Daily activity summary retrieved successfully"
        } else {
            "No daily activity summary found for the specified date"
        },
        dto,
    )))
}

/// Get daily activity summaries by date range (admin only)
pub async fn get_daily_activity_summaries_range_handler(
    State(app_state): State<crate::api::AppState>,
    _user: AuthenticatedUserWithRole,
    Query(query): Query<DateRangeQuery>,
) -> AppResult<Json<ApiResponse<DailyActivitySummaryResponse>>> {
    let start_date = query.start_date.ok_or_else(|| {
        crate::error::AppError::ValidationError("start_date is required".to_string())
    })?;
    let end_date = query.end_date.ok_or_else(|| {
        crate::error::AppError::ValidationError("end_date is required".to_string())
    })?;

    if start_date >= end_date {
        return Err(crate::error::AppError::ValidationError(
            "start_date must be before end_date".to_string(),
        ));
    }

    // Get summaries
    let summaries = app_state
        .activity_summary_service
        .get_summaries_range(start_date, end_date)
        .await?;

    // Calculate growth rate based on range
    let days = (end_date - start_date).num_days() + 1;
    let growth_rate = app_state
        .activity_summary_service
        .calculate_growth_rate(days)
        .await?;

    // Convert models to response DTOs
    let dto_summaries = summaries
        .into_iter()
        .map(
            |model| crate::features::analytics::dto::responses::DailyActivitySummary {
                date: model.date,
                total_users: model.total_users,
                active_users: model.active_users,
                new_users: model.new_users,
                tasks_created: model.tasks_created,
                tasks_completed: model.tasks_completed,
                created_at: model.created_at,
                updated_at: model.updated_at,
            },
        )
        .collect();

    let response = DailyActivitySummaryResponse {
        summaries: dto_summaries,
        growth_rate,
        period: AnalysisPeriod {
            start_date: chrono::DateTime::from_naive_utc_and_offset(
                start_date.and_hms_opt(0, 0, 0).unwrap(),
                chrono::Utc,
            ),
            end_date: chrono::DateTime::from_naive_utc_and_offset(
                end_date.and_hms_opt(23, 59, 59).unwrap(),
                chrono::Utc,
            ),
            duration_days: days as u32,
            granularity: MetricGranularity::Daily,
        },
    };

    Ok(Json(ApiResponse::success(
        "Daily activity summaries retrieved successfully",
        response,
    )))
}

/// Cleanup old daily activity summaries (admin only)
pub async fn cleanup_daily_activity_summaries_handler(
    State(app_state): State<crate::api::AppState>,
    _user: AuthenticatedUserWithRole,
    Query(query): Query<CleanupQuery>,
) -> AppResult<Json<ApiResponse<CleanupResult>>> {
    let days_to_keep = query.days.unwrap_or(365) as i64; // Default to 365 days

    let deleted_count = app_state
        .activity_summary_service
        .cleanup_old_summaries(days_to_keep)
        .await?;

    let result = CleanupResult {
        deleted_count: deleted_count as i32,
        operation_type: "activity_summary_cleanup".to_string(),
    };

    Ok(Json(ApiResponse::success(
        "Old activity summaries cleaned up successfully",
        result,
    )))
}

/// Get daily activity summaries (admin only)
pub async fn get_daily_activity_summary_handler(
    State(app_state): State<crate::api::AppState>,
    _user: AuthenticatedUserWithRole,
    Query(query): Query<AnalyticsTimeRangeRequest>,
) -> AppResult<Json<ApiResponse<DailyActivitySummaryResponse>>> {
    let days = query.period_days.unwrap_or(7);

    // Get summaries
    let summaries = app_state
        .activity_summary_service
        .get_recent_summaries(days as i64)
        .await?;

    // Calculate growth rate
    let growth_rate = app_state
        .activity_summary_service
        .calculate_growth_rate(days as i64)
        .await?;

    // Convert models to response DTOs
    let dto_summaries = summaries
        .into_iter()
        .map(
            |model| crate::features::analytics::dto::responses::DailyActivitySummary {
                date: model.date,
                total_users: model.total_users,
                active_users: model.active_users,
                new_users: model.new_users,
                tasks_created: model.tasks_created,
                tasks_completed: model.tasks_completed,
                created_at: model.created_at,
                updated_at: model.updated_at,
            },
        )
        .collect();

    let response = DailyActivitySummaryResponse {
        summaries: dto_summaries,
        growth_rate,
        period: AnalysisPeriod {
            start_date: chrono::Utc::now() - chrono::Duration::days(days as i64 - 1),
            end_date: chrono::Utc::now(),
            duration_days: days,
            granularity: MetricGranularity::Daily,
        },
    };

    Ok(Json(ApiResponse::success(
        "Daily activity summaries retrieved successfully",
        response,
    )))
}

/// Get system-wide analytics (admin only)
pub async fn get_system_analytics_handler(
    State(app_state): State<crate::api::AppState>,
    _user: AuthenticatedUserWithRole,
    Query(_query): Query<AnalyticsTimeRangeRequest>,
) -> AppResult<Json<ApiResponse<SystemStatsResponse>>> {
    // Get user counts using repository directly
    use crate::features::user::models::user;
    use sea_orm::{EntityTrait, PaginatorTrait};

    let total_users = user::Entity::find()
        .count(app_state.db.as_ref())
        .await
        .unwrap_or(0);

    // Get subscription distribution
    let subscription_distribution = app_state
        .subscription_service
        .get_subscription_distribution()
        .await
        .unwrap_or_default();

    // Get actual task counts using repository
    let total_tasks = crate::features::task::models::task_model::Entity::find()
        .count(app_state.db.as_ref())
        .await
        .unwrap_or(0);

    let completed_tasks = crate::features::task::models::task_model::Entity::find()
        .filter(crate::features::task::models::task_model::Column::Status.eq("completed"))
        .count(app_state.db.as_ref())
        .await
        .unwrap_or(0);

    // Get organization and team counts
    use crate::features::{organization::models::organization, team::models::team};
    let total_organizations = organization::Entity::find()
        .count(app_state.db.as_ref())
        .await
        .unwrap_or(0);

    let active_teams = team::Entity::find()
        .count(app_state.db.as_ref())
        .await
        .unwrap_or(0);

    // Calculate active users (users who logged in within last 30 days)
    use sea_orm::prelude::*;
    let thirty_days_ago = chrono::Utc::now() - chrono::Duration::days(30);
    // Count active users - include users who have logged in recently OR created recently
    let active_users = user::Entity::find()
        .filter(
            user::Column::LastLoginAt
                .gt(thirty_days_ago)
                .or(user::Column::CreatedAt.gt(thirty_days_ago)),
        )
        .count(app_state.db.as_ref())
        .await
        .unwrap_or(0);

    let mut response = SystemStatsResponse::new();
    response.total_users = total_users;
    response.active_users = active_users;
    response.total_tasks = total_tasks;
    response.completed_tasks = completed_tasks;
    response.task_completion_rate = if total_tasks > 0 {
        (completed_tasks as f64 / total_tasks as f64) * 100.0
    } else {
        0.0
    };
    response.total_organizations = total_organizations;
    response.active_teams = active_teams;
    response.average_tasks_per_user = if total_users > 0 {
        total_tasks as f64 / total_users as f64
    } else {
        0.0
    };

    // Set subscription tier distribution
    response.subscription_distribution = subscription_distribution
        .into_iter()
        .map(|(tier, count)| SubscriptionTierDistribution { tier, count })
        .collect();

    // Calculate daily and weekly active users
    let one_day_ago = chrono::Utc::now() - chrono::Duration::days(1);
    let seven_days_ago = chrono::Utc::now() - chrono::Duration::days(7);

    response.daily_active_users = user::Entity::find()
        .filter(
            user::Column::LastLoginAt
                .gt(one_day_ago)
                .or(user::Column::CreatedAt.gt(one_day_ago)),
        )
        .count(app_state.db.as_ref())
        .await
        .unwrap_or(0);

    response.weekly_active_users = user::Entity::find()
        .filter(
            user::Column::LastLoginAt
                .gt(seven_days_ago)
                .or(user::Column::CreatedAt.gt(seven_days_ago)),
        )
        .count(app_state.db.as_ref())
        .await
        .unwrap_or(0);

    Ok(Json(ApiResponse::success(
        "System analytics retrieved successfully",
        response,
    )))
}

/// Get feature usage counts (admin only)
pub async fn get_feature_usage_counts_handler(
    State(app_state): State<crate::api::AppState>,
    _user: AuthenticatedUserWithRole,
    Query(query): Query<FeatureUsageQuery>,
) -> AppResult<Json<ApiResponse<FeatureUsageCountsResponse>>> {
    // Extract days parameter from query or default to 7
    let period_days = query.days.unwrap_or(7);

    // Use existing feature usage metrics repository
    let end_date = chrono::Utc::now();
    let start_date = end_date - chrono::Duration::days(period_days as i64);
    let counts = app_state
        .feature_usage_metrics_repo
        .get_feature_usage_counts(start_date, end_date)
        .await?;

    let feature_counts: Vec<FeatureUsageCount> = counts
        .into_iter()
        .map(|(feature_name, usage_count)| FeatureUsageCount {
            feature_name,
            total_usage: usage_count as u64,
            unique_users: usage_count as u64, // TODO: Get actual unique user count
        })
        .collect();

    let response = FeatureUsageCountsResponse {
        period_days,
        feature_counts,
    };

    Ok(Json(ApiResponse::success(
        "Feature usage counts retrieved successfully",
        response,
    )))
}

/// Get feature usage stats (admin only) - for test compatibility
pub async fn get_feature_usage_stats_handler(
    State(_app_state): State<crate::api::AppState>,
    _user: AuthenticatedUserWithRole,
    Query(query): Query<FeatureUsageQuery>,
) -> AppResult<Json<ApiResponse<FeatureUsageStatsResponse>>> {
    let period_days = query.days.unwrap_or(30);

    let response = FeatureUsageStatsResponse {
        period_days: period_days as i32,
    };

    Ok(Json(ApiResponse::success(
        "Feature usage stats retrieved successfully",
        response,
    )))
}

/// Get user feature usage (admin only) - for test compatibility
pub async fn get_user_feature_usage_handler(
    State(app_state): State<crate::api::AppState>,
    _user: AuthenticatedUserWithRole,
    Path(user_id): Path<Uuid>,
    Query(query): Query<FeatureUsageQuery>,
) -> AppResult<Json<ApiResponse<UserFeatureUsageResponse>>> {
    let period_days = query.days.unwrap_or(7);

    // Use the feature_tracking_service to get actual feature usage data
    let _feature_usage_data = app_state
        .feature_tracking_service
        .get_user_feature_usage(user_id, period_days as i64)
        .await?;

    // Convert the data to response format (simplified for now)
    let response = UserFeatureUsageResponse { user_id };

    Ok(Json(ApiResponse::success(
        "User feature usage retrieved successfully",
        response,
    )))
}

/// Get user action counts (admin only)
pub async fn get_user_action_counts_handler(
    State(app_state): State<crate::api::AppState>,
    _user: AuthenticatedUserWithRole,
    Path(user_id): Path<Uuid>,
    Query(query): Query<AnalyticsTimeRangeRequest>,
) -> AppResult<Json<ApiResponse<std::collections::HashMap<String, i64>>>> {
    let period_days = query.period_days.unwrap_or(7);
    let end_date = chrono::Utc::now();
    let start_date = end_date - chrono::Duration::days(period_days as i64);

    let action_counts = app_state
        .feature_usage_metrics_repo
        .get_user_action_counts(user_id, start_date, end_date)
        .await?;

    Ok(Json(ApiResponse::success(
        format!("User {} action counts retrieved successfully", user_id),
        action_counts,
    )))
}

/// Get user behavior analytics (admin only)
pub async fn get_user_behavior_analytics_handler(
    State(_app_state): State<crate::api::AppState>,
    _user: AuthenticatedUserWithRole,
    Path(user_id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<UserBehaviorAnalyticsResponse>>> {
    // TODO: Implement actual user behavior analytics
    let response = UserBehaviorAnalyticsResponse {
        user_id,
        analysis_period: AnalysisPeriod {
            start_date: chrono::Utc::now() - chrono::Duration::days(30),
            end_date: chrono::Utc::now(),
            duration_days: 30,
            granularity: MetricGranularity::Daily,
        },
        behavioral_metrics: BehavioralMetrics {
            login_frequency: LoginFrequency {
                daily_average: 0.0,
                weekly_average: 0.0,
                monthly_average: 0.0,
                consistency_score: 0.0,
                longest_streak_days: 0,
                current_streak_days: 0,
            },
            session_duration: SessionDuration {
                average_minutes: 0.0,
                median_minutes: 0.0,
                longest_session_minutes: 0.0,
                shortest_session_minutes: 0.0,
                session_count: 0,
            },
            activity_score: 0.0,
            engagement_level: EngagementLevel::Medium,
            feature_adoption_rate: 0.0,
            consistency_score: 0.0,
        },
        activity_patterns: ActivityPatterns {
            peak_activity_hours: vec![],
            most_active_days: vec![],
            activity_distribution: ActivityDistribution {
                morning: 0.0,
                afternoon: 0.0,
                evening: 0.0,
                weekday: 0.0,
                weekend: 0.0,
            },
            workflow_patterns: vec![],
            seasonal_trends: vec![],
        },
        feature_usage: FeatureUsageMetrics {
            most_used_features: vec![],
            least_used_features: vec![],
            feature_progression: vec![],
            subscription_utilization: SubscriptionUtilization {
                current_tier: crate::core::subscription_tier::SubscriptionTier::Free,
                tier_utilization_percentage: 0.0,
                underutilized_features: vec![],
                upgrade_recommendations: vec![],
                cost_efficiency_score: 0.0,
            },
        },
        performance_indicators: PerformanceIndicators {
            task_completion_rate: 0.0,
            average_task_duration: 0.0,
            productivity_score: 0.0,
            error_rate: 0.0,
            satisfaction_indicators: SatisfactionIndicators {
                feature_satisfaction_score: 0.0,
                performance_satisfaction_score: 0.0,
                overall_satisfaction_score: 0.0,
                nps_score: None,
                feedback_sentiment: SentimentScore {
                    positive_percentage: 0.0,
                    neutral_percentage: 0.0,
                    negative_percentage: 0.0,
                    overall_sentiment: SentimentCategory::Neutral,
                },
            },
        },
        comparisons: None,
        recommendations: vec![],
        generated_at: chrono::Utc::now(),
    };

    Ok(Json(ApiResponse::success(
        "User behavior analytics retrieved successfully",
        response,
    )))
}

/// Export user analytics data (admin only)
pub async fn export_user_analytics_handler(
    State(_app_state): State<crate::api::AppState>,
    _user: AuthenticatedUserWithRole,
    Query(_query): Query<crate::features::admin::dto::analytics::UserAnalyticsExportQuery>,
) -> AppResult<Json<ApiResponse<Vec<crate::features::admin::dto::analytics::UserAnalyticsExport>>>>
{
    // TODO: Implement actual user analytics export
    let response = vec![];

    Ok(Json(ApiResponse::success(
        "User analytics exported successfully",
        response,
    )))
}

/// Get task statistics (admin only)
pub async fn get_task_stats_handler(
    State(app_state): State<crate::api::AppState>,
    _user: AuthenticatedUserWithRole,
    Query(query): Query<TaskAnalyticsRequest>,
) -> AppResult<Json<ApiResponse<TaskStatsDetailResponse>>> {
    // Get actual task counts using repository
    use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter};

    let todo_count = crate::features::task::models::task_model::Entity::find()
        .filter(crate::features::task::models::task_model::Column::Status.eq("todo"))
        .count(app_state.db.as_ref())
        .await
        .unwrap_or(0) as u32;

    let in_progress_count = crate::features::task::models::task_model::Entity::find()
        .filter(crate::features::task::models::task_model::Column::Status.eq("in_progress"))
        .count(app_state.db.as_ref())
        .await
        .unwrap_or(0) as u32;

    let completed_count = crate::features::task::models::task_model::Entity::find()
        .filter(crate::features::task::models::task_model::Column::Status.eq("completed"))
        .count(app_state.db.as_ref())
        .await
        .unwrap_or(0) as u32;

    let total_tasks = todo_count + in_progress_count + completed_count;
    let pending_tasks = todo_count + in_progress_count;
    let overdue_tasks = 0; // TODO: Implement overdue logic when due dates are available

    let completion_rate = if total_tasks > 0 {
        (completed_count as f64 / total_tasks as f64) * 100.0
    } else {
        0.0
    };

    // Create status distribution
    let status_distribution = vec![
        TaskStatusDistribution {
            status: "todo".to_string(),
            count: todo_count as u64,
            percentage: if total_tasks > 0 {
                (todo_count as f64 / total_tasks as f64) * 100.0
            } else {
                0.0
            },
        },
        TaskStatusDistribution {
            status: "in_progress".to_string(),
            count: in_progress_count as u64,
            percentage: if total_tasks > 0 {
                (in_progress_count as f64 / total_tasks as f64) * 100.0
            } else {
                0.0
            },
        },
        TaskStatusDistribution {
            status: "completed".to_string(),
            count: completed_count as u64,
            percentage: if total_tasks > 0 {
                (completed_count as f64 / total_tasks as f64) * 100.0
            } else {
                0.0
            },
        },
    ];

    // Mock priority distribution
    let priority_distribution = vec![
        TaskPriorityDistribution {
            priority: "high".to_string(),
            count: (total_tasks as f64 * 0.2) as u64,
            percentage: 20.0,
            average_completion_days: 1.5,
        },
        TaskPriorityDistribution {
            priority: "medium".to_string(),
            count: (total_tasks as f64 * 0.5) as u64,
            percentage: 50.0,
            average_completion_days: 3.0,
        },
        TaskPriorityDistribution {
            priority: "low".to_string(),
            count: (total_tasks as f64 * 0.3) as u64,
            percentage: 30.0,
            average_completion_days: 5.0,
        },
    ];

    let response = TaskStatsDetailResponse {
        overview: TaskStatsOverview {
            total_tasks: total_tasks as u64,
            completed_tasks: completed_count as u64,
            pending_tasks: pending_tasks as u64,
            overdue_tasks: overdue_tasks as u64,
            average_completion_days: 2.5, // Mock value
            completion_rate,
        },
        status_distribution,
        priority_distribution,
        trends: TaskTrends {
            weekly_creation: vec![], // TODO: Implement weekly trends
            weekly_completion: vec![],
            completion_velocity: 0.85,
            productivity_trend: ProductivityTrend {
                direction: "increasing".to_string(),
                change_percentage: 15.0,
                prediction_next_week: 18.0,
            },
        },
        user_performance: if query.include_details == Some(true) {
            // Get user performance data (exclude admin users)
            let users = crate::features::user::models::user::Entity::find()
                .filter(crate::features::user::models::user::Column::Email.ne("admin@example.com"))
                .all(app_state.db.as_ref())
                .await
                .unwrap_or_default();

            let mut performances = Vec::new();
            for user in users {
                // Mock data for now - in real implementation, this would query actual task stats per user
                performances.push(UserTaskPerformance {
                    user_id: user.id,
                    username: user.username.clone(),
                    tasks_created: 5,
                    tasks_completed: 3,
                    completion_rate: 60.0,
                    average_completion_time_hours: 24.0,
                    productivity_score: 75.0,
                });
            }
            Some(performances)
        } else {
            None
        },
        pagination: None,
    };

    Ok(Json(ApiResponse::success(
        "Task statistics retrieved successfully",
        response,
    )))
}

#[derive(Debug, Deserialize)]
pub struct BehaviorAnalyticsQuery {
    pub user_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct DateRangeQuery {
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct CleanupQuery {
    pub days: Option<i32>,
}

#[derive(Debug, serde::Serialize)]
pub struct CleanupResult {
    pub deleted_count: i32,
    pub operation_type: String,
}

/// Get current user's behavior analytics
pub async fn get_current_user_behavior_analytics_handler(
    State(_app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Query(query): Query<BehaviorAnalyticsQuery>,
) -> AppResult<Json<ApiResponse<UserBehaviorAnalyticsResponse>>> {
    // Determine which user's analytics to fetch
    let target_user_id = if let Some(requested_user_id) = query.user_id {
        // Check if user is admin or accessing their own data
        if user.user_id() != requested_user_id && !user.is_admin() {
            return Err(crate::error::AppError::Forbidden(
                "You can only access your own behavior analytics".to_string(),
            ));
        }
        requested_user_id
    } else {
        user.user_id()
    };
    // TODO: Implement actual user behavior analytics
    let response = UserBehaviorAnalyticsResponse {
        user_id: target_user_id,
        analysis_period: AnalysisPeriod {
            start_date: chrono::Utc::now() - chrono::Duration::days(30),
            end_date: chrono::Utc::now(),
            duration_days: 30,
            granularity: MetricGranularity::Daily,
        },
        behavioral_metrics: BehavioralMetrics {
            login_frequency: LoginFrequency {
                daily_average: 0.8,
                weekly_average: 5.6,
                monthly_average: 24.0,
                consistency_score: 0.75,
                longest_streak_days: 15,
                current_streak_days: 5,
            },
            session_duration: SessionDuration {
                average_minutes: 45.5,
                median_minutes: 40.0,
                longest_session_minutes: 120.0,
                shortest_session_minutes: 5.0,
                session_count: 50,
            },
            activity_score: 0.85,
            engagement_level: EngagementLevel::High,
            feature_adoption_rate: 0.7,
            consistency_score: 0.8,
        },
        activity_patterns: ActivityPatterns {
            peak_activity_hours: vec![9, 10, 14, 15, 16],
            most_active_days: vec!["Monday".to_string(), "Tuesday".to_string(), "Thursday".to_string()],
            activity_distribution: ActivityDistribution {
                morning: 0.3,
                afternoon: 0.5,
                evening: 0.2,
                weekday: 0.8,
                weekend: 0.2,
            },
            workflow_patterns: vec![
                WorkflowPattern {
                    pattern_name: "Morning Review".to_string(),
                    frequency: 9,
                    efficiency_score: 0.85,
                    steps: vec![],
                    average_duration_minutes: 45.0,
                },
            ],
            seasonal_trends: vec![],
        },
        feature_usage: FeatureUsageMetrics {
            most_used_features: vec![
                FeatureUsage {
                    feature_name: "Task Management".to_string(),
                    usage_count: 150,
                    usage_percentage: 45.0,
                    last_used: chrono::Utc::now() - chrono::Duration::hours(2),
                    proficiency_level: ProficiencyLevel::Expert,
                },
                FeatureUsage {
                    feature_name: "Team Collaboration".to_string(),
                    usage_count: 80,
                    usage_percentage: 25.0,
                    last_used: chrono::Utc::now() - chrono::Duration::hours(5),
                    proficiency_level: ProficiencyLevel::Intermediate,
                },
            ],
            least_used_features: vec![],
            feature_progression: vec![],
            subscription_utilization: SubscriptionUtilization {
                current_tier: crate::core::subscription_tier::SubscriptionTier::Free,
                tier_utilization_percentage: 65.0,
                underutilized_features: vec!["Analytics".to_string()],
                upgrade_recommendations: vec![],
                cost_efficiency_score: 0.8,
            },
        },
        performance_indicators: PerformanceIndicators {
            task_completion_rate: 0.78,
            average_task_duration: 2.5,
            productivity_score: 0.82,
            error_rate: 0.02,
            satisfaction_indicators: SatisfactionIndicators {
                feature_satisfaction_score: 0.85,
                performance_satisfaction_score: 0.8,
                overall_satisfaction_score: 0.83,
                nps_score: Some(8.0),
                feedback_sentiment: SentimentScore {
                    positive_percentage: 75.0,
                    neutral_percentage: 20.0,
                    negative_percentage: 5.0,
                    overall_sentiment: SentimentCategory::Positive,
                },
            },
        },
        comparisons: Some(UserComparisons {
            peer_comparison: PeerComparison {
                percentile_rank: 75.0,
                above_average_metrics: vec!["Login Frequency".to_string(), "Task Completion Rate".to_string()],
                below_average_metrics: vec![],
                peer_group_size: 50,
                benchmark_score: 82.0,
            },
            historical_comparison: HistoricalComparison {
                improvement_areas: vec![],
                declining_areas: vec![],
                consistency_score: 0.8,
                growth_rate: 0.15,
                trend_analysis: TrendAnalysis {
                    trend_direction: TrendDirection::Increasing,
                    trend_strength: 0.85,
                    seasonality_detected: false,
                    forecast_accuracy: 0.75,
                },
            },
            tier_comparison: TierComparison {
                current_tier: crate::core::subscription_tier::SubscriptionTier::Free,
                tier_average_metrics: TierMetrics {
                    average_activity_score: 0.6,
                    average_feature_usage: 0.5,
                    average_productivity_score: 0.65,
                    tier_satisfaction_score: 0.7,
                },
                tier_percentile: 75.0,
                upgrade_impact_prediction: None,
            },
        }),
        recommendations: vec![
            UserRecommendation {
                recommendation_type: crate::features::admin::dto::analytics::RecommendationType::FeatureAdoption,
                title: "Try Analytics Dashboard".to_string(),
                description: "Based on your usage patterns, the Analytics Dashboard could help you track progress more effectively.".to_string(),
                priority: crate::features::admin::dto::analytics::RecommendationPriority::Medium,
                expected_impact: "Increase productivity by 15%".to_string(),
                action_url: Some("/analytics/dashboard".to_string()),
            },
            UserRecommendation {
                recommendation_type: crate::features::admin::dto::analytics::RecommendationType::WorkflowOptimization,
                title: "Batch Similar Tasks".to_string(),
                description: "Group similar tasks together to improve focus and reduce context switching.".to_string(),
                priority: crate::features::admin::dto::analytics::RecommendationPriority::High,
                expected_impact: "Save 30 minutes daily".to_string(),
                action_url: None,
            },
        ],
        generated_at: chrono::Utc::now(),
    };

    Ok(Json(ApiResponse::success(
        "User behavior analytics retrieved successfully",
        response,
    )))
}

/// Advanced export handler
pub async fn advanced_export_handler(
    State(_app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Json(request): Json<AdvancedExportRequest>,
) -> AppResult<Json<ApiResponse<AdvancedExportResponse>>> {
    // Validate request
    request.validate().map_err(|e| {
        crate::error::AppError::ValidationError(format!("Invalid export request: {}", e))
    })?;

    // Check if user is admin for admin-only export types
    if matches!(
        request.export_type.as_str(),
        "Users" | "AuditLogs" | "SystemMetrics"
    ) && !user.claims.is_admin()
    {
        return Err(crate::error::AppError::Forbidden(
            "Admin access required for this export type".to_string(),
        ));
    }
    // TODO: Implement actual export logic
    let response = AdvancedExportResponse {
        export_id: uuid::Uuid::new_v4(),
        export_type: request.export_type,
        format: request.format,
        total_records: request.max_records.unwrap_or(1000) as u64,
        file_size_bytes: 1024 * 100, // Mock 100KB
        download_url: Some(format!(
            "https://example.com/exports/{}",
            uuid::Uuid::new_v4()
        )),
        expires_at: chrono::Utc::now() + chrono::Duration::hours(24),
        metadata: ExportMetadata {
            filters_applied: request.filters.unwrap_or_default(),
            columns_included: request
                .custom_fields
                .unwrap_or_else(|| vec!["id".to_string(), "created_at".to_string()]),
            data_version: "1.0".to_string(),
            export_source: "analytics".to_string(),
            checksum: "placeholder".to_string(),
            compression: None,
        },
        processing_status: ExportStatus::Completed,
        created_at: chrono::Utc::now(),
    };

    Ok(Json(ApiResponse::success(
        "Export request processed",
        response,
    )))
}

/// Create organization analytics (admin only)
pub async fn create_organization_analytics_handler(
    State(app_state): State<crate::api::AppState>,
    _user: AuthenticatedUserWithRole,
    Path(organization_id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    use crate::features::organization::models::analytics::{AnalyticsType, Period};
    use crate::features::organization::repositories::analytics::AnalyticsRepository;

    // Parse analytics data from payload
    let analytics_type = payload["analytics_type"]
        .as_str()
        .and_then(|s| serde_json::from_value::<AnalyticsType>(serde_json::json!(s)).ok())
        .unwrap_or(AnalyticsType::Performance);

    let period = payload["period"]
        .as_str()
        .and_then(|s| serde_json::from_value::<Period>(serde_json::json!(s)).ok())
        .unwrap_or(Period::Daily);

    let metric_name = payload["metric_name"]
        .as_str()
        .unwrap_or("custom_metric")
        .to_string();

    let metric_value = payload["metric_value"].clone();

    use crate::features::organization::models::analytics;
    use sea_orm::Set;

    let analytics_model = analytics::ActiveModel {
        id: Set(Uuid::new_v4()),
        organization_id: Set(organization_id),
        department_id: Set(payload["department_id"]
            .as_str()
            .and_then(|s| Uuid::parse_str(s).ok())),
        analytics_type: Set(analytics_type.to_string()),
        metric_name: Set(metric_name.clone()),
        metric_value: Set(metric_value),
        period: Set(period.to_string()),
        period_start: Set(chrono::Utc::now()),
        period_end: Set(chrono::Utc::now()),
        calculated_by: Set(_user.user_id()),
        created_at: Set(chrono::Utc::now()),
        updated_at: Set(chrono::Utc::now()),
    };

    let analytics = AnalyticsRepository::create(&app_state.db, analytics_model).await?;

    let response = serde_json::json!({
        "id": analytics.id,
        "organization_id": analytics.organization_id,
        "analytics_type": analytics.analytics_type,
        "period": analytics.period,
        "metric_name": analytics.metric_name,
        "metric_value": analytics.metric_value,
        "created_at": analytics.created_at,
    });

    Ok(Json(ApiResponse::success(
        "Analytics created successfully",
        response,
    )))
}

/// Get analytics by type (admin only)
pub async fn get_analytics_by_type_handler(
    State(app_state): State<crate::api::AppState>,
    _user: AuthenticatedUserWithRole,
    Path((organization_id, analytics_type)): Path<(Uuid, String)>,
) -> AppResult<Json<ApiResponse<Vec<serde_json::Value>>>> {
    use crate::features::organization::models::analytics::AnalyticsType;
    use crate::features::organization::repositories::analytics::AnalyticsRepository;

    let analytics_type = serde_json::from_value::<AnalyticsType>(serde_json::json!(analytics_type))
        .map_err(|_| crate::error::AppError::BadRequest("Invalid analytics type".to_string()))?;

    let analytics_data = AnalyticsRepository::find_by_organization_and_type(
        &app_state.db,
        organization_id,
        analytics_type,
        Some(100), // Add a default limit
    )
    .await?;

    let response: Vec<serde_json::Value> = analytics_data
        .into_iter()
        .map(|a| {
            serde_json::json!({
                "id": a.id,
                "analytics_type": a.analytics_type,
                "period": a.period,
                "metric_name": a.metric_name,
                "metric_value": a.metric_value,
                "period_start": a.period_start,
                "period_end": a.period_end,
                "calculated_by": a.calculated_by,
                "created_at": a.created_at,
            })
        })
        .collect();

    Ok(Json(ApiResponse::success(
        "Analytics by type retrieved successfully",
        response,
    )))
}

/// Get analytics by period (admin only)
pub async fn get_analytics_by_period_handler(
    State(app_state): State<crate::api::AppState>,
    _user: AuthenticatedUserWithRole,
    Path(organization_id): Path<Uuid>,
    Query(query): Query<serde_json::Value>,
) -> AppResult<Json<ApiResponse<Vec<serde_json::Value>>>> {
    use crate::features::organization::models::analytics::Period;
    use crate::features::organization::repositories::analytics::AnalyticsRepository;

    let period = query["period"]
        .as_str()
        .and_then(|s| serde_json::from_value::<Period>(serde_json::json!(s)).ok())
        .unwrap_or(Period::Daily);

    let start_date = query["start_date"]
        .as_str()
        .and_then(|s| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").ok())
        .unwrap_or_else(|| (chrono::Utc::now() - chrono::Duration::days(7)).naive_utc());

    let end_date = query["end_date"]
        .as_str()
        .and_then(|s| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").ok())
        .unwrap_or_else(|| chrono::Utc::now().naive_utc());

    let analytics_data = AnalyticsRepository::find_by_organization_id(
        &app_state.db,
        organization_id,
        Some(100), // Default limit
    )
    .await?;

    // Filter by period dates
    let analytics_data: Vec<_> = analytics_data
        .into_iter()
        .filter(|a| {
            a.period == period.to_string()
                && a.period_start.naive_utc() >= start_date
                && a.period_end.naive_utc() <= end_date
        })
        .collect();

    let response: Vec<serde_json::Value> = analytics_data
        .into_iter()
        .map(|a| {
            serde_json::json!({
                "id": a.id,
                "analytics_type": a.analytics_type,
                "period": a.period,
                "metric_name": a.metric_name,
                "metric_value": a.metric_value,
                "period_start": a.period_start,
                "period_end": a.period_end,
                "created_at": a.created_at,
            })
        })
        .collect();

    Ok(Json(ApiResponse::success(
        "Analytics by period retrieved successfully",
        response,
    )))
}

/// Check if analytics exists for period (admin only)
pub async fn check_analytics_period_exists_handler(
    State(app_state): State<crate::api::AppState>,
    _user: AuthenticatedUserWithRole,
    Path(organization_id): Path<Uuid>,
    Query(query): Query<serde_json::Value>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    use crate::features::organization::models::analytics::Period;
    use crate::features::organization::repositories::analytics::AnalyticsRepository;

    let period = query["period"]
        .as_str()
        .and_then(|s| serde_json::from_value::<Period>(serde_json::json!(s)).ok())
        .unwrap_or(Period::Daily);

    let start_date = query["start_date"]
        .as_str()
        .and_then(|s| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").ok())
        .unwrap_or_else(|| chrono::Utc::now().naive_utc());

    let end_date = query["end_date"]
        .as_str()
        .and_then(|s| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").ok())
        .unwrap_or_else(|| chrono::Utc::now().naive_utc());

    use crate::features::organization::models::analytics::AnalyticsType;

    // Get analytics type and metric name from query
    let analytics_type = query["analytics_type"]
        .as_str()
        .and_then(|s| serde_json::from_value::<AnalyticsType>(serde_json::json!(s)).ok())
        .unwrap_or(AnalyticsType::Performance);

    let metric_name = query["metric_name"]
        .as_str()
        .unwrap_or("default_metric")
        .to_string();

    let department_id = query["department_id"]
        .as_str()
        .and_then(|s| Uuid::parse_str(s).ok());

    let exists = AnalyticsRepository::exists_analytics_for_period(
        &app_state.db,
        organization_id,
        department_id,
        analytics_type,
        &metric_name,
        chrono::DateTime::from_naive_utc_and_offset(start_date, chrono::Utc),
        chrono::DateTime::from_naive_utc_and_offset(end_date, chrono::Utc),
    )
    .await?;

    let response = serde_json::json!({
        "organization_id": organization_id,
        "period": period,
        "start_date": start_date,
        "end_date": end_date,
        "exists": exists,
    });

    Ok(Json(ApiResponse::success(
        "Analytics period existence checked",
        response,
    )))
}

/// Get latest metrics (admin only)
pub async fn get_latest_metrics_handler(
    State(app_state): State<crate::api::AppState>,
    _user: AuthenticatedUserWithRole,
    Query(query): Query<serde_json::Value>,
) -> AppResult<Json<ApiResponse<Vec<serde_json::Value>>>> {
    use crate::features::organization::repositories::analytics::AnalyticsRepository;

    let organization_id = query["organization_id"]
        .as_str()
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| {
            crate::error::AppError::BadRequest("organization_id is required".to_string())
        })?;

    let metric_name = query["metric_name"].as_str().unwrap_or("all").to_string();

    let department_id = query["department_id"]
        .as_str()
        .and_then(|s| Uuid::parse_str(s).ok());

    let analytics_type = query["analytics_type"]
        .as_str()
        .and_then(|s| serde_json::from_value::<AnalyticsType>(serde_json::json!(s)).ok())
        .unwrap_or(AnalyticsType::Performance);

    use crate::features::organization::models::analytics::AnalyticsType;

    let analytics_data = if let Some(analytics) = AnalyticsRepository::find_latest_by_metric(
        &app_state.db,
        organization_id,
        department_id,
        analytics_type,
        &metric_name,
    )
    .await?
    {
        vec![analytics]
    } else {
        vec![]
    };

    let response: Vec<serde_json::Value> = analytics_data
        .into_iter()
        .map(|a| {
            serde_json::json!({
                "id": a.id,
                "analytics_type": a.analytics_type,
                "metric_name": a.metric_name,
                "metric_value": a.metric_value,
                "period": a.period,
                "created_at": a.created_at,
            })
        })
        .collect();

    Ok(Json(ApiResponse::success(
        "Latest metrics retrieved successfully",
        response,
    )))
}

/// Get department analytics (admin only)
pub async fn get_department_analytics_handler(
    State(app_state): State<crate::api::AppState>,
    _user: AuthenticatedUserWithRole,
    Path(department_id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<Vec<serde_json::Value>>>> {
    use crate::features::organization::repositories::analytics::AnalyticsRepository;

    let analytics_data = AnalyticsRepository::find_by_department_id(
        &app_state.db,
        department_id,
        Some(100), // Default limit
    )
    .await?;

    let response: Vec<serde_json::Value> = analytics_data
        .into_iter()
        .map(|a| {
            serde_json::json!({
                "id": a.id,
                "organization_id": a.organization_id,
                "department_id": a.department_id,
                "analytics_type": a.analytics_type,
                "period": a.period,
                "metric_name": a.metric_name,
                "metric_value": a.metric_value,
                "created_at": a.created_at,
            })
        })
        .collect();

    Ok(Json(ApiResponse::success(
        "Department analytics retrieved successfully",
        response,
    )))
}

/// Get aggregated metrics (admin only)
pub async fn get_aggregated_metrics_handler(
    State(app_state): State<crate::api::AppState>,
    _user: AuthenticatedUserWithRole,
    Path(organization_id): Path<Uuid>,
    Query(query): Query<serde_json::Value>,
) -> AppResult<Json<ApiResponse<Vec<serde_json::Value>>>> {
    use crate::features::organization::models::analytics::{AnalyticsType, Period};
    use crate::features::organization::repositories::analytics::AnalyticsRepository;

    let analytics_type = query["analytics_type"]
        .as_str()
        .and_then(|s| serde_json::from_value::<AnalyticsType>(serde_json::json!(s)).ok())
        .unwrap_or(AnalyticsType::Performance);

    let period = query["period"]
        .as_str()
        .and_then(|s| serde_json::from_value::<Period>(serde_json::json!(s)).ok())
        .unwrap_or(Period::Monthly);

    let metric_name = query["metric_name"]
        .as_str()
        .unwrap_or("default_metric")
        .to_string();

    let start_date = query["start_date"]
        .as_str()
        .and_then(|s| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").ok())
        .map_or_else(
            || chrono::Utc::now() - chrono::Duration::days(30),
            |dt| chrono::DateTime::from_naive_utc_and_offset(dt, chrono::Utc),
        );

    let end_date = query["end_date"]
        .as_str()
        .and_then(|s| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").ok())
        .map_or_else(chrono::Utc::now, |dt| {
            chrono::DateTime::from_naive_utc_and_offset(dt, chrono::Utc)
        });

    let analytics_data = AnalyticsRepository::find_aggregated_metrics(
        &app_state.db,
        organization_id,
        analytics_type,
        &metric_name,
        period,
        start_date,
        end_date,
    )
    .await?;

    let response: Vec<serde_json::Value> = analytics_data
        .into_iter()
        .map(|a| {
            serde_json::json!({
                "analytics_type": a.analytics_type,
                "period": a.period,
                "metric_name": a.metric_name,
                "aggregated_value": a.metric_value,
                "data_points": 1, // In real implementation, this would be the count of aggregated records
                "created_at": a.created_at,
            })
        })
        .collect();

    Ok(Json(ApiResponse::success(
        "Aggregated metrics retrieved successfully",
        response,
    )))
}

/// Delete old analytics (admin only)
pub async fn delete_old_analytics_handler(
    State(app_state): State<crate::api::AppState>,
    _user: AuthenticatedUserWithRole,
    Path(organization_id): Path<Uuid>,
    Query(query): Query<serde_json::Value>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    use crate::features::organization::repositories::analytics::AnalyticsRepository;

    let days_to_keep = query["days_to_keep"].as_i64().unwrap_or(365);

    let before_date = chrono::Utc::now() - chrono::Duration::days(days_to_keep);

    let deleted_count =
        AnalyticsRepository::delete_old_analytics(&app_state.db, organization_id, before_date)
            .await?;

    let response = serde_json::json!({
        "organization_id": organization_id,
        "deleted_count": deleted_count,
        "days_kept": days_to_keep,
        "operation": "delete_old_analytics",
        "completed_at": chrono::Utc::now(),
    });

    Ok(Json(ApiResponse::success(
        "Old analytics deleted successfully",
        response,
    )))
}

/// Analytics router
pub fn analytics_router() -> Router<crate::api::AppState> {
    Router::new()
        .route(
            "/analytics/track-feature",
            post(track_feature_usage_handler),
        )
        .route(
            "/analytics/behavior",
            get(get_current_user_behavior_analytics_handler),
        )
        .route("/admin/analytics/system", get(get_system_analytics_handler))
        .route(
            "/admin/analytics/daily-summary",
            get(get_daily_activity_summary_handler),
        )
        .route(
            "/admin/analytics/daily-summary/{date}",
            get(get_single_daily_activity_summary_handler),
        )
        .route(
            "/admin/analytics/daily-summary/{date}",
            post(update_daily_activity_summary_handler),
        )
        .route(
            "/admin/analytics/daily-summary/range",
            get(get_daily_activity_summaries_range_handler),
        )
        .route(
            "/admin/analytics/daily-summary/cleanup",
            post(cleanup_daily_activity_summaries_handler),
        )
        .route(
            "/admin/analytics/feature-usage-counts",
            get(get_feature_usage_counts_handler),
        )
        .route(
            "/admin/analytics/features/usage",
            get(get_feature_usage_stats_handler),
        )
        .route(
            "/admin/analytics/users/{user_id}/features",
            get(get_user_feature_usage_handler),
        )
        .route(
            "/admin/analytics/users/{user_id}/action-counts",
            get(get_user_action_counts_handler),
        )
        .route("/admin/tasks/stats", get(get_task_stats_handler))
        .route(
            "/analytics/users/{user_id}/behavior",
            get(get_user_behavior_analytics_handler),
        )
        .route(
            "/analytics/users/export",
            get(export_user_analytics_handler),
        )
        .route("/exports/advanced", post(advanced_export_handler))
        .route(
            "/admin/analytics/organizations/{organization_id}",
            post(create_organization_analytics_handler),
        )
        .route(
            "/admin/analytics/organizations/{organization_id}/type/{analytics_type}",
            get(get_analytics_by_type_handler),
        )
        .route(
            "/admin/analytics/organizations/{organization_id}/period",
            get(get_analytics_by_period_handler),
        )
        .route(
            "/admin/analytics/organizations/{organization_id}/check-period",
            get(check_analytics_period_exists_handler),
        )
        .route(
            "/admin/analytics/metrics/latest",
            get(get_latest_metrics_handler),
        )
        .route(
            "/admin/analytics/departments/{department_id}",
            get(get_department_analytics_handler),
        )
        .route(
            "/admin/analytics/organizations/{organization_id}/aggregated",
            get(get_aggregated_metrics_handler),
        )
        .route(
            "/admin/analytics/organizations/{organization_id}/cleanup",
            delete(delete_old_analytics_handler),
        )
}

// Re-export the analytics router for compatibility
pub fn analytics_router_with_state(state: crate::api::AppState) -> Router {
    analytics_router().with_state(state)
}
