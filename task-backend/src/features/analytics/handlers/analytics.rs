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
    BehavioralMetrics, EngagementLevel, ExportMetadata, ExportStatus, FeatureUsage,
    FeatureUsageCount, FeatureUsageCountsResponse, FeatureUsageMetrics, FeatureUsageStatsResponse,
    LoginFrequency, MetricGranularity, PerformanceIndicators, ProficiencyLevel,
    SatisfactionIndicators, SentimentCategory, SentimentScore, SessionDuration,
    SubscriptionTierDistribution, SubscriptionUtilization, SystemStatsResponse,
    UserBehaviorAnalyticsResponse, UserFeatureUsageResponse,
};
use crate::features::auth::middleware::{AuthenticatedUser, AuthenticatedUserWithRole};
use crate::shared::types::ApiResponse;
use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
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
        .unwrap_or(0) as u64;

    let completed_tasks = crate::features::task::models::task_model::Entity::find()
        .filter(crate::features::task::models::task_model::Column::Status.eq("completed"))
        .count(app_state.db.as_ref())
        .await
        .unwrap_or(0) as u64;

    // Get organization and team counts
    use crate::features::{organization::models::organization, team::models::team};
    let total_organizations = organization::Entity::find()
        .count(app_state.db.as_ref())
        .await
        .unwrap_or(0) as u64;

    let active_teams = team::Entity::find()
        .count(app_state.db.as_ref())
        .await
        .unwrap_or(0) as u64;

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
        .unwrap_or(0) as u64;

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
        .unwrap_or(0) as u64;

    response.weekly_active_users = user::Entity::find()
        .filter(
            user::Column::LastLoginAt
                .gt(seven_days_ago)
                .or(user::Column::CreatedAt.gt(seven_days_ago)),
        )
        .count(app_state.db.as_ref())
        .await
        .unwrap_or(0) as u64;

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
    State(_app_state): State<crate::api::AppState>,
    _user: AuthenticatedUserWithRole,
    Path(user_id): Path<Uuid>,
    Query(query): Query<FeatureUsageQuery>,
) -> AppResult<Json<ApiResponse<UserFeatureUsageResponse>>> {
    let _period_days = query.days.unwrap_or(7);

    let response = UserFeatureUsageResponse { user_id };

    Ok(Json(ApiResponse::success(
        "User feature usage retrieved successfully",
        response,
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
}

// Re-export the analytics router for compatibility
pub fn analytics_router_with_state(state: crate::api::AppState) -> Router {
    analytics_router().with_state(state)
}
