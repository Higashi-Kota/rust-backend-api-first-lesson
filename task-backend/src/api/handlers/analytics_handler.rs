// task-backend/src/api/handlers/analytics_handler.rs

use crate::api::dto::analytics_dto::*;
use crate::api::dto::common::{ApiResponse, OperationResult};
use crate::api::AppState;
use crate::domain::subscription_tier::SubscriptionTier;
use crate::error::{AppError, AppResult};
use crate::middleware::auth::{AuthenticatedUser, AuthenticatedUserWithRole};
use axum::{
    extract::{Json, Path, Query, State},
    routing::{get, post},
    Router,
};
use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use serde_json::json;
use tracing::{info, warn};
use uuid::Uuid;
use validator::Validate;

// --- Query Parameters ---

/// 統計期間パラメータ
#[derive(Debug, Deserialize, Validate)]
pub struct StatsPeriodQuery {
    #[validate(range(min = 1, max = 365, message = "Days must be between 1 and 365"))]
    pub days: Option<u32>,
    pub include_trends: Option<bool>,
    pub detailed: Option<bool>,
}

// --- Handler Functions ---

/// システム全体の統計を取得（管理者のみ）
pub async fn get_system_stats_handler(
    State(_app_state): State<AppState>,
    admin_user: AuthenticatedUserWithRole,
    Query(query): Query<StatsPeriodQuery>,
) -> AppResult<Json<ApiResponse<SystemStatsResponse>>> {
    // 管理者権限チェック
    if !admin_user.is_admin() {
        warn!(
            user_id = %admin_user.user_id(),
            role = ?admin_user.role().map(|r| &r.name),
            "Access denied: Admin permission required for system stats"
        );
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    // バリデーション
    query.validate().map_err(|validation_errors| {
        warn!(
            "System stats query validation failed: {}",
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

    let days = query.days.unwrap_or(30);
    let include_trends = query.include_trends.unwrap_or(false);

    info!(
        admin_id = %admin_user.user_id(),
        days = %days,
        include_trends = %include_trends,
        "System stats requested"
    );

    // システム統計を生成（実際の実装では適切なサービスから取得）
    let mut stats = SystemStatsResponse::new();

    // モックデータを設定
    stats.overview = SystemOverview {
        total_users: 1250,
        active_users_last_30_days: 890,
        total_tasks: 15680,
        completed_tasks: 12340,
        system_uptime_days: 365,
        database_size_mb: 2450.5,
    };

    stats.user_metrics = UserMetrics {
        new_registrations_today: 12,
        new_registrations_this_week: 85,
        new_registrations_this_month: 340,
        active_users_today: 234,
        retention_rate_30_days: 78.5,
        average_session_duration_minutes: 45.2,
        user_distribution_by_tier: vec![
            TierDistribution {
                tier: SubscriptionTier::Free,
                count: 750,
                percentage: 60.0,
            },
            TierDistribution {
                tier: SubscriptionTier::Pro,
                count: 350,
                percentage: 28.0,
            },
            TierDistribution {
                tier: SubscriptionTier::Enterprise,
                count: 150,
                percentage: 12.0,
            },
        ],
    };

    stats.task_metrics = TaskMetrics {
        tasks_created_today: 456,
        tasks_completed_today: 378,
        tasks_created_this_week: 2890,
        tasks_completed_this_week: 2234,
        average_completion_time_hours: 18.5,
        completion_rate_percentage: 78.7,
        top_task_categories: vec![
            TaskCategoryStats {
                category: "Development".to_string(),
                total_tasks: 4567,
                completed_tasks: 3890,
                completion_rate: 85.2,
            },
            TaskCategoryStats {
                category: "Marketing".to_string(),
                total_tasks: 2890,
                completed_tasks: 2234,
                completion_rate: 77.3,
            },
            TaskCategoryStats {
                category: "Support".to_string(),
                total_tasks: 1890,
                completed_tasks: 1567,
                completion_rate: 82.9,
            },
        ],
    };

    stats.subscription_metrics = SubscriptionMetrics {
        conversion_rate_percentage: 12.5,
        churn_rate_percentage: 3.2,
        average_revenue_per_user: 45.67,
        monthly_recurring_revenue: 56780.0,
        upgrade_rate_percentage: 8.9,
        downgrade_rate_percentage: 2.1,
    };

    info!(
        admin_id = %admin_user.user_id(),
        total_users = %stats.overview.total_users,
        active_users = %stats.overview.active_users_last_30_days,
        "System stats generated"
    );

    Ok(Json(ApiResponse::success(
        "System statistics retrieved successfully",
        stats,
    )))
}

/// ユーザー個人のアクティビティ統計を取得
pub async fn get_user_activity_handler(
    State(_app_state): State<AppState>,
    user: AuthenticatedUser,
    Query(query): Query<StatsPeriodQuery>,
) -> AppResult<Json<ApiResponse<UserActivityResponse>>> {
    // バリデーション
    query.validate().map_err(|validation_errors| {
        warn!(
            "User activity query validation failed: {}",
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

    let days = query.days.unwrap_or(30);
    let period_end = Utc::now();
    let period_start = period_end - Duration::days(days as i64);

    info!(
        user_id = %user.claims.user_id,
        days = %days,
        "User activity stats requested"
    );

    // ユーザーアクティビティを生成（実際の実装では適切なサービスから取得）
    let daily_activities = generate_mock_daily_activities(period_start, period_end);

    let total_tasks_created = daily_activities
        .iter()
        .map(|d| d.tasks_created as u64)
        .sum();
    let total_tasks_completed = daily_activities
        .iter()
        .map(|d| d.tasks_completed as u64)
        .sum();
    let total_login_days = daily_activities
        .iter()
        .filter(|d| d.login_count > 0)
        .count() as u64;
    let average_daily_tasks = if days > 0 {
        total_tasks_created as f64 / days as f64
    } else {
        0.0
    };
    let completion_rate = if total_tasks_created > 0 {
        (total_tasks_completed as f64 / total_tasks_created as f64) * 100.0
    } else {
        0.0
    };

    let productivity_score = ActivitySummary::calculate_productivity_score(
        completion_rate / 100.0,
        average_daily_tasks,
        total_login_days as f64 / days as f64,
    );

    let summary = ActivitySummary {
        total_tasks_created,
        total_tasks_completed,
        total_login_days,
        average_daily_tasks,
        completion_rate_percentage: completion_rate,
        most_active_day: "Monday".to_string(),
        productivity_score,
    };

    let response = UserActivityResponse {
        user_id: user.claims.user_id,
        daily_activities,
        summary,
        period_start,
        period_end,
    };

    info!(
        user_id = %user.claims.user_id,
        total_tasks_created = %response.summary.total_tasks_created,
        completion_rate = %response.summary.completion_rate_percentage,
        productivity_score = %response.summary.productivity_score,
        "User activity stats generated"
    );

    // ApiResponse::success_with_metadataを活用
    let metadata = json!({
        "query_period_days": days,
        "period_start": period_start.to_rfc3339(),
        "period_end": period_end.to_rfc3339(),
        "calculation_timestamp": Utc::now().to_rfc3339(),
        "api_version": "v1"
    });

    Ok(Json(ApiResponse::success_with_metadata(
        "User activity statistics retrieved successfully",
        response,
        metadata,
    )))
}

/// 管理者用ユーザーアクティビティ統計を取得
pub async fn get_user_activity_admin_handler(
    State(_app_state): State<AppState>,
    admin_user: AuthenticatedUserWithRole,
    Path(target_user_id): Path<Uuid>,
    Query(query): Query<StatsPeriodQuery>,
) -> AppResult<Json<ApiResponse<UserActivityResponse>>> {
    // 管理者権限チェック
    if !admin_user.is_admin() {
        warn!(
            user_id = %admin_user.user_id(),
            target_user_id = %target_user_id,
            "Access denied: Admin permission required for user activity stats"
        );
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    // バリデーション
    query.validate().map_err(|validation_errors| {
        warn!(
            "Admin user activity query validation failed: {}",
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

    let days = query.days.unwrap_or(30);
    let period_end = Utc::now();
    let period_start = period_end - Duration::days(days as i64);

    info!(
        admin_id = %admin_user.user_id(),
        target_user_id = %target_user_id,
        days = %days,
        "Admin user activity stats requested"
    );

    // ターゲットユーザーのアクティビティを生成
    let daily_activities = generate_mock_daily_activities(period_start, period_end);

    let total_tasks_created = daily_activities
        .iter()
        .map(|d| d.tasks_created as u64)
        .sum();
    let total_tasks_completed = daily_activities
        .iter()
        .map(|d| d.tasks_completed as u64)
        .sum();
    let total_login_days = daily_activities
        .iter()
        .filter(|d| d.login_count > 0)
        .count() as u64;
    let average_daily_tasks = if days > 0 {
        total_tasks_created as f64 / days as f64
    } else {
        0.0
    };
    let completion_rate = if total_tasks_created > 0 {
        (total_tasks_completed as f64 / total_tasks_created as f64) * 100.0
    } else {
        0.0
    };

    let productivity_score = ActivitySummary::calculate_productivity_score(
        completion_rate / 100.0,
        average_daily_tasks,
        total_login_days as f64 / days as f64,
    );

    let summary = ActivitySummary {
        total_tasks_created,
        total_tasks_completed,
        total_login_days,
        average_daily_tasks,
        completion_rate_percentage: completion_rate,
        most_active_day: "Tuesday".to_string(),
        productivity_score,
    };

    let response = UserActivityResponse {
        user_id: target_user_id,
        daily_activities,
        summary,
        period_start,
        period_end,
    };

    info!(
        admin_id = %admin_user.user_id(),
        target_user_id = %target_user_id,
        total_tasks_created = %response.summary.total_tasks_created,
        completion_rate = %response.summary.completion_rate_percentage,
        "Admin user activity stats generated"
    );

    Ok(Json(ApiResponse::success(
        "User activity statistics retrieved successfully",
        response,
    )))
}

/// タスク統計詳細を取得
pub async fn get_task_stats_handler(
    State(_app_state): State<AppState>,
    user: AuthenticatedUser,
    Query(query): Query<StatsPeriodQuery>,
) -> AppResult<Json<ApiResponse<TaskStatsDetailResponse>>> {
    // バリデーション
    query.validate().map_err(|validation_errors| {
        warn!("Task stats query validation failed: {}", validation_errors);
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

    let days = query.days.unwrap_or(30);
    let detailed = query.detailed.unwrap_or(false);

    info!(
        user_id = %user.claims.user_id,
        days = %days,
        detailed = %detailed,
        "Task stats requested"
    );

    // タスク統計を生成（実際の実装では適切なサービスから取得）
    let overview = TaskStatsOverview {
        total_tasks: 156,
        completed_tasks: 123,
        pending_tasks: 28,
        overdue_tasks: 5,
        average_completion_days: 2.3,
        completion_rate: TaskStatsOverview::calculate_completion_rate(123, 156),
    };

    let status_distribution = vec![
        TaskStatusDistribution {
            status: "completed".to_string(),
            count: 123,
            percentage: 78.8,
        },
        TaskStatusDistribution {
            status: "in_progress".to_string(),
            count: 18,
            percentage: 11.5,
        },
        TaskStatusDistribution {
            status: "todo".to_string(),
            count: 10,
            percentage: 6.4,
        },
        TaskStatusDistribution {
            status: "cancelled".to_string(),
            count: 5,
            percentage: 3.2,
        },
    ];

    let priority_distribution = vec![
        TaskPriorityDistribution {
            priority: "high".to_string(),
            count: 45,
            percentage: 28.8,
            average_completion_days: 1.8,
        },
        TaskPriorityDistribution {
            priority: "medium".to_string(),
            count: 78,
            percentage: 50.0,
            average_completion_days: 2.5,
        },
        TaskPriorityDistribution {
            priority: "low".to_string(),
            count: 33,
            percentage: 21.2,
            average_completion_days: 3.2,
        },
    ];

    let trends = generate_mock_task_trends();
    let user_performance = if detailed && user.claims.is_admin() {
        Some(generate_mock_user_performance())
    } else {
        None
    };

    let response = TaskStatsDetailResponse {
        overview,
        status_distribution,
        priority_distribution,
        trends,
        user_performance,
        pagination: None,
    };

    info!(
        user_id = %user.claims.user_id,
        total_tasks = %response.overview.total_tasks,
        completion_rate = %response.overview.completion_rate,
        "Task stats generated"
    );

    Ok(Json(ApiResponse::success(
        "Task statistics retrieved successfully",
        response,
    )))
}

/// ユーザー行動分析を取得
pub async fn get_user_behavior_analytics_handler(
    State(_app_state): State<AppState>,
    user: AuthenticatedUser,
    Query(query): Query<UserBehaviorAnalyticsQuery>,
) -> AppResult<Json<ApiResponse<UserBehaviorAnalyticsResponse>>> {
    let target_user_id = query.user_id.unwrap_or(user.claims.user_id);

    // 権限チェック: 自分のデータまたは管理者
    if target_user_id != user.claims.user_id && !user.claims.is_admin() {
        warn!(
            user_id = %user.claims.user_id,
            target_user_id = %target_user_id,
            "Access denied: Cannot access other user's behavior analytics"
        );
        return Err(AppError::Forbidden(
            "Cannot access other user's data".to_string(),
        ));
    }

    let from_date = query
        .from_date
        .unwrap_or_else(|| Utc::now() - Duration::days(30));
    let to_date = query.to_date.unwrap_or_else(Utc::now);
    let include_comparisons = query.include_comparisons.unwrap_or(false);

    info!(
        user_id = %user.claims.user_id,
        target_user_id = %target_user_id,
        from_date = %from_date,
        to_date = %to_date,
        include_comparisons = %include_comparisons,
        "User behavior analytics requested"
    );

    // 行動分析データを生成（実際の実装では適切なサービスから取得）
    let analysis_period = AnalysisPeriod {
        start_date: from_date,
        end_date: to_date,
        duration_days: (to_date - from_date).num_days() as u32,
        granularity: MetricGranularity::Daily,
    };

    let behavioral_metrics = BehavioralMetrics {
        login_frequency: LoginFrequency {
            daily_average: 1.2,
            weekly_average: 5.4,
            monthly_average: 22.8,
            consistency_score: 78.5,
            longest_streak_days: 15,
            current_streak_days: 7,
        },
        session_duration: SessionDuration {
            average_minutes: 45.2,
            median_minutes: 38.0,
            longest_session_minutes: 180.0,
            shortest_session_minutes: 5.0,
            session_count: 89,
        },
        activity_score: 75.6,
        engagement_level: BehavioralMetrics::calculate_engagement_level(75.6),
        feature_adoption_rate: 68.9,
        consistency_score: 82.3,
    };

    let activity_patterns = ActivityPatterns {
        peak_activity_hours: vec![9, 10, 14, 15],
        most_active_days: vec![
            "Monday".to_string(),
            "Tuesday".to_string(),
            "Wednesday".to_string(),
        ],
        activity_distribution: ActivityDistribution {
            morning: 35.2,
            afternoon: 45.8,
            evening: 19.0,
            weekday: 78.5,
            weekend: 21.5,
        },
        workflow_patterns: vec![WorkflowPattern {
            pattern_name: "Morning Productivity".to_string(),
            frequency: 15,
            efficiency_score: 87.3,
            steps: vec![WorkflowStep {
                step_name: "Email Review".to_string(),
                order: 1,
                duration_minutes: 15.0,
                success_rate: 95.0,
                common_errors: vec!["Missed important emails".to_string()],
            }],
            average_duration_minutes: 120.0,
        }],
        seasonal_trends: vec![SeasonalTrend {
            season: "Winter".to_string(),
            activity_multiplier: 1.15,
            peak_months: vec!["January".to_string(), "February".to_string()],
            trend_strength: 0.8,
        }],
    };

    let feature_usage = FeatureUsageMetrics {
        most_used_features: vec![FeatureUsage {
            feature_name: "Task Management".to_string(),
            usage_count: 156,
            usage_percentage: 89.2,
            last_used: Utc::now() - Duration::hours(2),
            proficiency_level: ProficiencyLevel::Advanced,
        }],
        least_used_features: vec![FeatureUsage {
            feature_name: "Advanced Reporting".to_string(),
            usage_count: 3,
            usage_percentage: 1.8,
            last_used: Utc::now() - Duration::days(15),
            proficiency_level: ProficiencyLevel::Beginner,
        }],
        feature_progression: vec![FeatureProgression {
            feature_name: "Task Management".to_string(),
            adoption_date: Utc::now() - Duration::days(90),
            proficiency_level: ProficiencyLevel::Advanced,
            usage_trend: TrendDirection::Stable,
            mastery_percentage: 85.6,
        }],
        subscription_utilization: SubscriptionUtilization {
            current_tier: SubscriptionTier::Pro,
            tier_utilization_percentage: 67.8,
            underutilized_features: vec!["Bulk Operations".to_string(), "API Access".to_string()],
            upgrade_recommendations: vec![UpgradeRecommendation {
                recommended_tier: SubscriptionTier::Enterprise,
                reason: "Heavy usage of advanced features".to_string(),
                expected_benefits: vec![
                    "Unlimited API calls".to_string(),
                    "Priority support".to_string(),
                ],
                confidence_score: 78.9,
                estimated_roi: 145.6,
            }],
            cost_efficiency_score: 72.3,
        },
    };

    let performance_indicators = PerformanceIndicators {
        task_completion_rate: 85.7,
        average_task_duration: 2.3,
        productivity_score: 78.9,
        error_rate: 2.1,
        satisfaction_indicators: SatisfactionIndicators {
            feature_satisfaction_score: 82.5,
            performance_satisfaction_score: 79.3,
            overall_satisfaction_score: 81.2,
            nps_score: Some(8.5),
            feedback_sentiment: SentimentScore {
                positive_percentage: 78.5,
                negative_percentage: 12.3,
                neutral_percentage: 9.2,
                overall_sentiment: SentimentCategory::Positive,
            },
        },
    };

    let comparisons = if include_comparisons {
        Some(UserComparisons {
            peer_comparison: PeerComparison {
                percentile_rank: 75.6,
                above_average_metrics: vec![
                    "Task Completion".to_string(),
                    "Login Frequency".to_string(),
                ],
                below_average_metrics: vec!["Feature Adoption".to_string()],
                peer_group_size: 234,
                benchmark_score: 82.3,
            },
            historical_comparison: HistoricalComparison {
                improvement_areas: vec![ImprovementArea {
                    metric_name: "Task Completion Rate".to_string(),
                    improvement_percentage: 12.5,
                    timeframe: "Last 30 days".to_string(),
                    recommended_actions: vec!["Continue current workflow".to_string()],
                }],
                declining_areas: vec![DecliningArea {
                    metric_name: "Feature Usage".to_string(),
                    decline_percentage: 5.2,
                    timeframe: "Last 14 days".to_string(),
                    intervention_suggestions: vec!["Explore new features".to_string()],
                }],
                consistency_score: 78.9,
                growth_rate: 8.7,
                trend_analysis: TrendAnalysis {
                    trend_direction: TrendDirection::Increasing,
                    trend_strength: 0.8,
                    seasonality_detected: true,
                    forecast_accuracy: 85.2,
                },
            },
            tier_comparison: TierComparison {
                current_tier: SubscriptionTier::Pro,
                tier_average_metrics: TierMetrics {
                    average_activity_score: 68.4,
                    average_feature_usage: 54.7,
                    average_productivity_score: 72.1,
                    tier_satisfaction_score: 76.8,
                },
                tier_percentile: 78.9,
                upgrade_impact_prediction: Some(UpgradeImpact {
                    productivity_increase: 15.6,
                    feature_access_improvement: 25.8,
                    estimated_roi: 145.6,
                    payback_period_months: 6,
                }),
            },
        })
    } else {
        None
    };

    let recommendations = vec![
        UserRecommendation {
            recommendation_type: RecommendationType::ProductivityImprovement,
            title: "Optimize Morning Routine".to_string(),
            description: "Your productivity peaks in the morning. Consider scheduling important tasks during 9-11 AM.".to_string(),
            priority: RecommendationPriority::Medium,
            expected_impact: "15% productivity increase".to_string(),
            action_url: Some("/dashboard/schedule".to_string()),
        },
        UserRecommendation {
            recommendation_type: RecommendationType::FeatureAdoption,
            title: "Explore Bulk Operations".to_string(),
            description: "You have access to bulk operations but haven't used them. This could save 30% time on repetitive tasks.".to_string(),
            priority: RecommendationPriority::High,
            expected_impact: "30% time savings on bulk tasks".to_string(),
            action_url: Some("/features/bulk-operations".to_string()),
        },
    ];

    let response = UserBehaviorAnalyticsResponse {
        user_id: target_user_id,
        analysis_period,
        behavioral_metrics,
        activity_patterns,
        feature_usage,
        performance_indicators,
        comparisons,
        recommendations,
        generated_at: Utc::now(),
    };

    info!(
        user_id = %user.claims.user_id,
        target_user_id = %target_user_id,
        activity_score = %response.behavioral_metrics.activity_score,
        engagement_level = ?response.behavioral_metrics.engagement_level,
        "User behavior analytics generated"
    );

    Ok(Json(ApiResponse::success(
        "User behavior analytics retrieved successfully",
        response,
    )))
}

/// 高度なエクスポートを実行
pub async fn advanced_export_handler(
    State(_app_state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<AdvancedExportRequest>,
) -> AppResult<Json<ApiResponse<AdvancedExportResponse>>> {
    // バリデーション
    payload.validate().map_err(|validation_errors| {
        warn!("Advanced export validation failed: {}", validation_errors);
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

    // エクスポート権限チェック（実装に応じて調整）
    let can_export = match payload.export_type {
        ExportType::Users | ExportType::SystemMetrics => user.claims.is_admin(),
        _ => true, // 他のタイプは一般ユーザーも可能
    };

    if !can_export {
        warn!(
            user_id = %user.claims.user_id,
            export_type = ?payload.export_type,
            "Access denied: Insufficient permissions for export type"
        );
        return Err(AppError::Forbidden(
            "Insufficient permissions for this export type".to_string(),
        ));
    }

    let export_id = Uuid::new_v4();

    info!(
        user_id = %user.claims.user_id,
        export_id = %export_id,
        export_type = ?payload.export_type,
        format = ?payload.format,
        max_records = ?payload.max_records,
        "Advanced export requested"
    );

    // エクスポート処理をシミュレート
    let total_records = payload.max_records.unwrap_or(1000);
    let file_size_bytes = (total_records as u64 * 250) + 1024; // Estimate file size
    let expires_at = Utc::now() + Duration::days(7);

    let filters_applied = payload.filters.unwrap_or(ExportFilters {
        date_range: None,
        user_filter: None,
        status_filter: None,
        subscription_filter: None,
        custom_filters: None,
    });

    let metadata = ExportMetadata {
        filters_applied,
        columns_included: payload.custom_fields.unwrap_or_else(|| {
            vec![
                "id".to_string(),
                "created_at".to_string(),
                "updated_at".to_string(),
            ]
        }),
        data_version: "1.0".to_string(),
        export_source: "analytics_api".to_string(),
        checksum: format!("sha256:{}", Uuid::new_v4()),
        compression: match payload.format {
            ExportFormat::Json | ExportFormat::Csv => Some("gzip".to_string()),
            _ => None,
        },
    };

    let response = AdvancedExportResponse {
        export_id,
        export_type: payload.export_type,
        format: payload.format,
        total_records,
        file_size_bytes,
        download_url: Some(format!("/api/exports/{}/download", export_id)),
        expires_at,
        metadata,
        processing_status: ExportStatus::Completed,
        created_at: Utc::now(),
    };

    info!(
        user_id = %user.claims.user_id,
        export_id = %export_id,
        total_records = %response.total_records,
        file_size_bytes = %response.file_size_bytes,
        "Advanced export completed"
    );

    // エクスポートレスポンスをOperationResult::createdでラップして作成済みを明示
    let export_result = OperationResult::created(response);

    Ok(Json(ApiResponse::success(
        "Advanced export completed successfully",
        export_result.item,
    )))
}

// --- Helper Functions ---

fn generate_mock_daily_activities(start: DateTime<Utc>, end: DateTime<Utc>) -> Vec<DailyActivity> {
    let mut activities = Vec::new();
    let mut current = start;

    while current <= end {
        activities.push(DailyActivity {
            date: current,
            tasks_created: (current.timestamp() % 10) as u32,
            tasks_completed: (current.timestamp() % 8) as u32,
            login_count: if current.timestamp() % 3 == 0 { 1 } else { 0 },
            session_duration_minutes: (current.timestamp() % 60) as u32,
        });
        current += Duration::days(1);
    }

    activities
}

fn generate_mock_task_trends() -> TaskTrends {
    let weekly_creation = vec![
        WeeklyTrend {
            week_start: Utc::now() - Duration::days(21),
            count: 45,
            change_from_previous_week: 12.5,
        },
        WeeklyTrend {
            week_start: Utc::now() - Duration::days(14),
            count: 52,
            change_from_previous_week: 15.6,
        },
        WeeklyTrend {
            week_start: Utc::now() - Duration::days(7),
            count: 48,
            change_from_previous_week: -7.7,
        },
    ];

    let weekly_completion = vec![
        WeeklyTrend {
            week_start: Utc::now() - Duration::days(21),
            count: 38,
            change_from_previous_week: 8.6,
        },
        WeeklyTrend {
            week_start: Utc::now() - Duration::days(14),
            count: 45,
            change_from_previous_week: 18.4,
        },
        WeeklyTrend {
            week_start: Utc::now() - Duration::days(7),
            count: 41,
            change_from_previous_week: -8.9,
        },
    ];

    TaskTrends {
        weekly_creation,
        weekly_completion,
        completion_velocity: 0.89,
        productivity_trend: ProductivityTrend {
            direction: "increasing".to_string(),
            change_percentage: 5.2,
            prediction_next_week: 47.0,
        },
    }
}

fn generate_mock_user_performance() -> Vec<UserTaskPerformance> {
    vec![
        UserTaskPerformance {
            user_id: Uuid::new_v4(),
            username: "alice_smith".to_string(),
            tasks_created: 89,
            tasks_completed: 76,
            completion_rate: 85.4,
            average_completion_time_hours: 18.2,
            productivity_score: 8.7,
        },
        UserTaskPerformance {
            user_id: Uuid::new_v4(),
            username: "bob_jones".to_string(),
            tasks_created: 67,
            tasks_completed: 52,
            completion_rate: 77.6,
            average_completion_time_hours: 24.5,
            productivity_score: 7.2,
        },
        UserTaskPerformance {
            user_id: Uuid::new_v4(),
            username: "carol_wilson".to_string(),
            tasks_created: 112,
            tasks_completed: 98,
            completion_rate: 87.5,
            average_completion_time_hours: 16.8,
            productivity_score: 9.1,
        },
    ]
}

// --- ルーター ---

/// アナリティクスルーターを作成
pub fn analytics_router(app_state: AppState) -> Router {
    Router::new()
        // システム統計（管理者のみ）
        .route("/admin/analytics/system", get(get_system_stats_handler))
        // ユーザー統計
        .route("/analytics/activity", get(get_user_activity_handler))
        .route("/analytics/tasks", get(get_task_stats_handler))
        // 管理者用ユーザー統計
        .route(
            "/admin/analytics/users/{id}/activity",
            get(get_user_activity_admin_handler),
        )
        // User Analytics & Management APIs (新機能)
        .route(
            "/analytics/behavior",
            get(get_user_behavior_analytics_handler),
        )
        .route("/exports/advanced", post(advanced_export_handler))
        .with_state(app_state)
}

/// アナリティクスルーターをAppStateから作成
pub fn analytics_router_with_state(app_state: AppState) -> Router {
    analytics_router(app_state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_period_query_validation() {
        let valid_query = StatsPeriodQuery {
            days: Some(30),
            include_trends: Some(true),
            detailed: Some(false),
        };
        assert!(valid_query.validate().is_ok());

        let invalid_query = StatsPeriodQuery {
            days: Some(400), // Invalid: > 365
            include_trends: Some(true),
            detailed: Some(false),
        };
        assert!(invalid_query.validate().is_err());
    }

    #[test]
    fn test_generate_mock_daily_activities() {
        let start = Utc::now() - Duration::days(7);
        let end = Utc::now();
        let activities = generate_mock_daily_activities(start, end);

        assert!(!activities.is_empty());
        assert!(activities.len() <= 8); // 7 days + maybe 1 extra depending on timing

        for activity in &activities {
            assert!(activity.date >= start);
            assert!(activity.date <= end);
            assert!(activity.tasks_created < 10);
            assert!(activity.tasks_completed < 8);
        }
    }

    #[test]
    fn test_generate_mock_task_trends() {
        let trends = generate_mock_task_trends();

        assert_eq!(trends.weekly_creation.len(), 3);
        assert_eq!(trends.weekly_completion.len(), 3);
        assert!(trends.completion_velocity > 0.0);
        assert_eq!(trends.productivity_trend.direction, "increasing");
    }

    #[test]
    fn test_generate_mock_user_performance() {
        let performance = generate_mock_user_performance();

        assert_eq!(performance.len(), 3);

        for user_perf in &performance {
            assert!(!user_perf.username.is_empty());
            assert!(user_perf.completion_rate <= 100.0);
            assert!(user_perf.productivity_score <= 10.0);
            assert!(user_perf.average_completion_time_hours > 0.0);
        }
    }
}
