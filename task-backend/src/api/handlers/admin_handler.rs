// task-backend/src/api/handlers/admin_handler.rs

use crate::api::dto::admin_organization_dto::*;
use crate::api::dto::admin_role_dto::*;
use crate::api::dto::subscription_history_dto::*;
use crate::api::dto::team_invitation_dto::*;
use crate::api::dto::user_dto::{
    UpdateUserSettingsRequest, UserSettingsDto, UserWithRoleResponse, UsersByLanguageResponse,
    UsersWithNotificationResponse,
};
use crate::error::{AppError, AppResult};
use crate::features::admin::dto::subscription_history::{
    DeleteHistoryResponse, SubscriptionAnalyticsResponse, SubscriptionHistorySearchQuery,
    SubscriptionTierDistribution,
};
use crate::features::admin::dto::{
    AdminBulkCreateTasksRequest, AdminBulkDeleteTasksRequest, AdminBulkOperationResponse,
    AdminBulkUpdateTasksRequest, AdminTaskStatsResponse, BulkOperationHistoryResponse,
    BulkOperationListQuery, ChangeUserSubscriptionRequest, ChangeUserSubscriptionResponse,
    CleanupResultResponse, UserFeatureMetricsResponse,
};
use crate::features::auth::middleware::{AuthenticatedUser, AuthenticatedUserWithRole};
use crate::features::task::dto::*;
use crate::shared::types::pagination::{PaginatedResponse, PaginationQuery};
use crate::shared::types::ApiResponse;
use crate::utils::permission::{PermissionChecker, PermissionType};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::{Duration, Utc};
use std::collections::HashMap;
use tracing::info;
use uuid::Uuid;
use validator::Validate;

// === タスク管理API ===

/// 管理者向けタスク詳細取得（制限なし）
pub async fn admin_get_task(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Path(task_id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<TaskResponse>>> {
    if !user.is_admin() {
        return Err(AppError::Forbidden(
            "Administrator access required".to_string(),
        ));
    }

    let task_service = &app_state.task_service;

    // 管理者なので任意のタスクにアクセス可能
    let task_dto = task_service.get_task(task_id).await?;
    let response = TaskResponse::Enterprise {
        tasks: vec![task_dto],
        bulk_operations: true,
        unlimited_access: true,
    };

    Ok(Json(ApiResponse::success(
        "Task retrieved successfully",
        response,
    )))
}

/// 管理者向けタスク一括作成
pub async fn admin_bulk_create_tasks(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Json(request): Json<AdminBulkCreateTasksRequest>,
) -> AppResult<Json<ApiResponse<AdminBulkOperationResponse>>> {
    if !user.is_admin() {
        return Err(AppError::Forbidden(
            "Administrator access required".to_string(),
        ));
    }

    request
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    let task_service = &app_state.task_service;
    let mut success_count = 0;
    let mut errors = Vec::new();

    // タスクを準備
    let total_requested = request.tasks.len();
    let tasks_to_create = request.tasks;

    match task_service.admin_create_tasks_bulk(tasks_to_create).await {
        Ok(created_tasks) => {
            success_count = created_tasks.len();
        }
        Err(e) => {
            errors.push(format!("Bulk create failed: {}", e));
        }
    }

    let response = AdminBulkOperationResponse {
        success_count,
        failed_count: errors.len(),
        total_requested,
        errors,
    };

    Ok(Json(ApiResponse::success(
        "Bulk task creation completed",
        response,
    )))
}

/// 管理者向けタスク一括更新
pub async fn admin_bulk_update_tasks(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Json(request): Json<AdminBulkUpdateTasksRequest>,
) -> AppResult<Json<ApiResponse<AdminBulkOperationResponse>>> {
    if !user.is_admin() {
        return Err(AppError::Forbidden(
            "Administrator access required".to_string(),
        ));
    }

    request
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    let task_service = &app_state.task_service;
    let total_requested = request.updates.len();

    match task_service.admin_update_tasks_bulk(request.updates).await {
        Ok(updated_count) => {
            let response = AdminBulkOperationResponse {
                success_count: updated_count,
                failed_count: 0,
                total_requested,
                errors: vec![],
            };

            Ok(Json(ApiResponse::success(
                "Bulk task update completed",
                response,
            )))
        }
        Err(e) => {
            let response = AdminBulkOperationResponse {
                success_count: 0,
                failed_count: 1,
                total_requested,
                errors: vec![format!("Bulk update failed: {}", e)],
            };

            Ok(Json(ApiResponse::success(
                "Bulk task update failed",
                response,
            )))
        }
    }
}

/// 管理者向けタスク一括削除
pub async fn admin_bulk_delete_tasks(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Json(request): Json<AdminBulkDeleteTasksRequest>,
) -> AppResult<Json<ApiResponse<AdminBulkOperationResponse>>> {
    if !user.is_admin() {
        return Err(AppError::Forbidden(
            "Administrator access required".to_string(),
        ));
    }

    request
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    let task_service = &app_state.task_service;
    let total_requested = request.task_ids.len();

    match task_service.admin_delete_tasks_bulk(request.task_ids).await {
        Ok(rows_affected) => {
            let response = AdminBulkOperationResponse {
                success_count: rows_affected as usize,
                failed_count: 0,
                total_requested,
                errors: vec![],
            };

            Ok(Json(ApiResponse::success(
                "Bulk task deletion completed",
                response,
            )))
        }
        Err(e) => {
            let response = AdminBulkOperationResponse {
                success_count: 0,
                failed_count: 1,
                total_requested,
                errors: vec![format!("Bulk delete failed: {}", e)],
            };

            Ok(Json(ApiResponse::success(
                "Bulk task deletion failed",
                response,
            )))
        }
    }
}

// === 招待管理API（移行） ===

/// 管理者向け期限切れ招待クリーンアップ
pub async fn admin_cleanup_expired_invitations(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
) -> AppResult<Json<ApiResponse<Vec<TeamInvitationResponse>>>> {
    if !user.is_admin() {
        return Err(AppError::Forbidden(
            "Administrator access required".to_string(),
        ));
    }

    let service = &app_state.team_invitation_service;
    let expired_invitations = service.mark_expired_invitations().await?;

    let responses: Vec<TeamInvitationResponse> = expired_invitations
        .into_iter()
        .map(TeamInvitationResponse::from)
        .collect();

    Ok(Json(ApiResponse::success(
        "Expired invitations cleaned up successfully",
        responses,
    )))
}

/// 管理者向け古い招待削除
pub async fn admin_delete_old_invitations(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Query(params): Query<HashMap<String, String>>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    if !user.is_admin() {
        return Err(AppError::Forbidden(
            "Administrator access required".to_string(),
        ));
    }

    let service = &app_state.team_invitation_service;

    let days = params
        .get("days")
        .and_then(|d| d.parse::<u32>().ok())
        .unwrap_or(30);

    if days < 7 {
        return Err(AppError::ValidationError(
            "Cannot delete invitations less than 7 days old".to_string(),
        ));
    }

    let deleted_count = service.cleanup_old_invitations(days).await?;

    Ok(Json(ApiResponse::success(
        "Old invitations deleted successfully",
        serde_json::json!({
            "deleted_count": deleted_count,
            "days": days
        }),
    )))
}

// === 統計・分析API ===

/// 管理者向けタスク詳細統計取得
pub async fn admin_get_task_stats(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUserWithRole,
    Query(query): Query<crate::api::dto::analytics_dto::TaskAnalyticsRequest>,
) -> AppResult<Json<ApiResponse<crate::api::dto::analytics_dto::TaskStatsDetailResponse>>> {
    use crate::api::dto::analytics_dto::*;

    // 管理者権限チェック
    if !user.is_admin() {
        return Err(AppError::Forbidden(
            "Administrator access required".to_string(),
        ));
    }

    let task_service = &app_state.task_service;
    let user_service = &app_state.user_service;

    // 全タスクを取得（実際の実装ではページネーションを考慮すべき）
    let all_tasks = task_service.list_tasks().await?;

    // 基本統計を計算
    let total_tasks = all_tasks.len() as u64;
    let completed_tasks = all_tasks
        .iter()
        .filter(|t| t.status == crate::core::task_status::TaskStatus::Completed)
        .count() as u64;
    let pending_tasks = all_tasks
        .iter()
        .filter(|t| {
            t.status == crate::core::task_status::TaskStatus::Todo
                || t.status == crate::core::task_status::TaskStatus::InProgress
        })
        .count() as u64;
    let overdue_tasks = all_tasks
        .iter()
        .filter(|t| {
            if let Some(due_date) = t.due_date {
                due_date < Utc::now() && t.status != crate::core::task_status::TaskStatus::Completed
            } else {
                false
            }
        })
        .count() as u64;

    let completion_rate = if total_tasks > 0 {
        (completed_tasks as f64 / total_tasks as f64) * 100.0
    } else {
        0.0
    };

    // 平均完了日数を計算（簡易実装）
    let average_completion_days = {
        let completed_with_dates = all_tasks
            .iter()
            .filter(|t| t.status == crate::core::task_status::TaskStatus::Completed)
            .filter_map(|t| t.due_date.map(|d| (t.created_at, d)))
            .collect::<Vec<_>>();

        if completed_with_dates.is_empty() {
            0.0
        } else {
            let total_days: f64 = completed_with_dates
                .iter()
                .map(|(created, due)| (*due - *created).num_days() as f64)
                .sum();
            total_days / completed_with_dates.len() as f64
        }
    };

    let overview = TaskStatsOverview {
        total_tasks,
        completed_tasks,
        pending_tasks,
        overdue_tasks,
        average_completion_days,
        completion_rate,
    };

    // ステータス別分布
    let status_distribution = vec![
        TaskStatusDistribution {
            status: "todo".to_string(),
            count: all_tasks
                .iter()
                .filter(|t| t.status == crate::core::task_status::TaskStatus::Todo)
                .count() as u64,
            percentage: 0.0, // 後で計算
        },
        TaskStatusDistribution {
            status: "in_progress".to_string(),
            count: all_tasks
                .iter()
                .filter(|t| t.status == crate::core::task_status::TaskStatus::InProgress)
                .count() as u64,
            percentage: 0.0,
        },
        TaskStatusDistribution {
            status: "completed".to_string(),
            count: completed_tasks,
            percentage: 0.0,
        },
    ]
    .into_iter()
    .map(|mut dist| {
        dist.percentage = if total_tasks > 0 {
            (dist.count as f64 / total_tasks as f64) * 100.0
        } else {
            0.0
        };
        dist
    })
    .collect();

    // 優先度分布を実際のデータから取得
    let priority_data = task_service.get_priority_distribution().await?;

    // 優先度別の平均完了日数を実際のデータから取得
    let priority_avg_days = task_service
        .get_average_completion_days_by_priority()
        .await?;

    // 優先度別の平均完了日数をマップに変換
    let priority_avg_map: std::collections::HashMap<String, f64> =
        priority_avg_days.into_iter().collect();

    let priority_distribution: Vec<TaskPriorityDistribution> = vec!["high", "medium", "low"]
        .into_iter()
        .map(|priority| {
            let count = priority_data
                .iter()
                .find(|(p, _)| p == priority)
                .map_or(0, |(_, c)| *c);

            let percentage = if total_tasks > 0 {
                (count as f64 / total_tasks as f64) * 100.0
            } else {
                0.0
            };

            let avg_days = priority_avg_map.get(priority).copied().unwrap_or(0.0);

            TaskPriorityDistribution {
                priority: priority.to_string(),
                count,
                percentage,
                average_completion_days: avg_days,
            }
        })
        .collect();

    // トレンドデータを実際のデータから取得（3週間分）
    let trend_data = task_service.get_weekly_trend_data(3).await?;

    let mut weekly_creation = Vec::new();
    let mut weekly_completion = Vec::new();

    for i in 0..trend_data.len() {
        let (week_start, created, completed) = &trend_data[i];

        // 前週との変化率を計算
        let creation_change = if i > 0 {
            let prev_created = trend_data[i - 1].1;
            if prev_created > 0 {
                ((*created as f64 - prev_created as f64) / prev_created as f64) * 100.0
            } else {
                0.0
            }
        } else {
            0.0
        };

        let completion_change = if i > 0 {
            let prev_completed = trend_data[i - 1].2;
            if prev_completed > 0 {
                ((*completed as f64 - prev_completed as f64) / prev_completed as f64) * 100.0
            } else {
                0.0
            }
        } else {
            0.0
        };

        weekly_creation.push(WeeklyTrend {
            week_start: *week_start,
            count: *created,
            change_from_previous_week: creation_change,
        });

        weekly_completion.push(WeeklyTrend {
            week_start: *week_start,
            count: *completed,
            change_from_previous_week: completion_change,
        });
    }

    // 完了速度を計算（完了数/作成数）
    let total_created: u64 = trend_data.iter().map(|(_, c, _)| *c).sum();
    let total_completed: u64 = trend_data.iter().map(|(_, _, c)| *c).sum();
    let completion_velocity = if total_created > 0 {
        total_completed as f64 / total_created as f64
    } else {
        0.0
    };

    // 生産性トレンドを計算
    let (direction, change_percentage) = if weekly_completion.len() >= 2 {
        let recent = weekly_completion.last().unwrap().count as f64;
        let previous = weekly_completion[weekly_completion.len() - 2].count as f64;

        if previous == 0.0 {
            // 前週が0で今週が0より大きい場合は100%増加とする
            if recent > 0.0 {
                ("increasing".to_string(), 100.0)
            } else {
                ("stable".to_string(), 0.0)
            }
        } else if recent > previous {
            (
                "increasing".to_string(),
                ((recent - previous) / previous) * 100.0,
            )
        } else if recent < previous {
            (
                "decreasing".to_string(),
                ((previous - recent) / previous) * 100.0,
            )
        } else {
            ("stable".to_string(), 0.0)
        }
    } else {
        ("stable".to_string(), 0.0)
    };

    // 次週の予測（簡単な線形予測）
    let prediction_next_week = if weekly_completion.len() >= 2 {
        let recent = weekly_completion.last().unwrap().count as f64;
        let trend = weekly_completion.last().unwrap().change_from_previous_week / 100.0;
        recent * (1.0 + trend)
    } else {
        weekly_completion.last().map_or(0.0, |t| t.count as f64)
    };

    let trends = TaskTrends {
        weekly_creation,
        weekly_completion,
        completion_velocity,
        productivity_trend: ProductivityTrend {
            direction,
            change_percentage,
            prediction_next_week,
        },
    };

    // ユーザー別パフォーマンス（要求された場合のみ）
    let user_performance = if query.include_details.unwrap_or(false) {
        // ユーザー別にタスクを集計
        let mut user_tasks: std::collections::HashMap<Uuid, (u64, u64)> =
            std::collections::HashMap::new();

        for task in &all_tasks {
            if let Some(user_id) = task.user_id {
                let entry = user_tasks.entry(user_id).or_insert((0, 0));
                entry.0 += 1; // created
                if task.status == crate::core::task_status::TaskStatus::Completed {
                    entry.1 += 1; // completed
                }
            }
        }

        // ユーザー情報を取得して結果を構築
        let mut performances = vec![];
        for (user_id, (created, completed)) in user_tasks {
            if let Ok(user) = user_service.get_user_profile(user_id).await {
                // count_tasks_for_userを使用して正確な数を取得
                let actual_count = task_service
                    .count_tasks_for_user(user_id)
                    .await
                    .unwrap_or(created);

                let completion_rate = if actual_count > 0 {
                    (completed as f64 / actual_count as f64) * 100.0
                } else {
                    0.0
                };

                // 平均完了時間を取得
                let average_completion_time_hours = task_service
                    .get_user_average_completion_hours(user_id)
                    .await
                    .unwrap_or(48.0); // エラー時はデフォルト値

                // 生産性スコアを計算（完了率、平均完了時間、タスク数を考慮）
                let productivity_score = {
                    let rate_score = completion_rate * 0.5; // 完了率の重み: 50%
                    let speed_score = if average_completion_time_hours > 0.0 {
                        // 24時間以内: 100点、48時間: 50点、72時間以上: 0点
                        (f64::max(72.0 - average_completion_time_hours, 0.0) / 72.0 * 100.0) * 0.3
                    } else {
                        0.0
                    }; // 速度の重み: 30%
                    let volume_score = (completed.min(20) as f64 / 20.0 * 100.0) * 0.2; // 完了数の重み: 20%（20個で満点）

                    (rate_score + speed_score + volume_score).min(100.0)
                };

                performances.push(UserTaskPerformance {
                    user_id,
                    username: user.username,
                    tasks_created: actual_count,
                    tasks_completed: completed,
                    completion_rate,
                    average_completion_time_hours,
                    productivity_score,
                });
            }
        }

        Some(performances)
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

    Ok(Json(ApiResponse::success(
        "Task statistics retrieved successfully",
        response,
    )))
}

/// 管理者向けタスク統計取得
pub async fn admin_get_task_statistics(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUserWithRole,
) -> AppResult<Json<ApiResponse<AdminTaskStatsResponse>>> {
    // can_access_admin_featuresを使用して、管理者またはEnterpriseプランユーザーのアクセスを許可
    if let Some(role) = user.role() {
        if !PermissionChecker::can_access_admin_features(role) {
            return Err(AppError::Forbidden(
                "Administrator or Enterprise subscription required".to_string(),
            ));
        }
    } else if !user.is_admin() {
        return Err(AppError::Forbidden(
            "Administrator access required".to_string(),
        ));
    }

    let task_service = &app_state.task_service;

    // 基本統計を取得（実装はサービス層で）
    let stats = task_service.get_admin_task_statistics().await?;

    Ok(Json(ApiResponse::success(
        "Task statistics retrieved successfully",
        stats,
    )))
}

/// 管理者向けタスク作成（単一）
pub async fn admin_create_task(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Json(request): Json<CreateTaskDto>,
) -> AppResult<Json<ApiResponse<TaskDto>>> {
    if !user.is_admin() {
        return Err(AppError::Forbidden(
            "Administrator access required".to_string(),
        ));
    }

    let task_service = &app_state.task_service;
    let created_task = task_service.create_task(request).await?;

    Ok(Json(ApiResponse::success(
        "Task created successfully",
        created_task,
    )))
}

/// 管理者向けタスク更新（単一）
pub async fn admin_update_task(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Path(task_id): Path<Uuid>,
    Json(request): Json<UpdateTaskDto>,
) -> AppResult<Json<ApiResponse<TaskDto>>> {
    if !user.is_admin() {
        return Err(AppError::Forbidden(
            "Administrator access required".to_string(),
        ));
    }

    let task_service = &app_state.task_service;
    let updated_task = task_service.update_task(task_id, request).await?;

    Ok(Json(ApiResponse::success(
        "Task updated successfully",
        updated_task,
    )))
}

/// 管理者向けタスク削除（単一）
pub async fn admin_delete_task(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Path(task_id): Path<Uuid>,
) -> AppResult<axum::http::StatusCode> {
    if !user.is_admin() {
        return Err(AppError::Forbidden(
            "Administrator access required".to_string(),
        ));
    }

    let task_service = &app_state.task_service;
    task_service.delete_task(task_id).await?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}

/// 管理者向け全タスク一覧取得
pub async fn admin_list_all_tasks(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
) -> AppResult<Json<Vec<TaskDto>>> {
    if !user.is_admin() {
        return Err(AppError::Forbidden(
            "Administrator access required".to_string(),
        ));
    }

    let task_service = &app_state.task_service;
    let tasks = task_service.list_tasks().await?;

    Ok(Json(tasks))
}

/// 管理者向けタスク一覧取得（ページング付き）
pub async fn admin_list_tasks_paginated(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> AppResult<Json<ApiResponse<PaginatedTasksDto>>> {
    if !user.is_admin() {
        return Err(AppError::Forbidden(
            "Administrator access required".to_string(),
        ));
    }

    let page = params
        .get("page")
        .and_then(|p| p.parse::<u64>().ok())
        .unwrap_or(1);
    let page_size = params
        .get("page_size")
        .and_then(|p| p.parse::<u64>().ok())
        .unwrap_or(10)
        .clamp(1, 100);

    let task_service = &app_state.task_service;
    let paginated_tasks = task_service.list_tasks_paginated(page, page_size).await?;

    Ok(Json(ApiResponse::success(
        "Paginated tasks retrieved successfully",
        paginated_tasks,
    )))
}

// レガシーバッチAPI削除 - 新形式のadmin_bulk_*に統一

/// 管理者向け特定ユーザーのタスク一覧取得
pub async fn admin_list_user_tasks(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Path(user_id): Path<Uuid>,
) -> AppResult<Json<Vec<TaskDto>>> {
    if !user.is_admin() {
        return Err(AppError::Forbidden(
            "Administrator access required".to_string(),
        ));
    }

    let task_service = &app_state.task_service;
    let tasks = task_service.list_tasks_for_user(user_id).await?;

    Ok(Json(tasks))
}

// === ロール管理API ===

/// 管理者向けロール一覧取得
pub async fn admin_list_roles(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUserWithRole,
    Query(query): Query<AdminRoleListQuery>,
) -> AppResult<Json<ApiResponse<AdminRoleListResponse>>> {
    // 管理者権限チェック
    if !user.is_admin() {
        return Err(AppError::Forbidden(
            "Administrator access required".to_string(),
        ));
    }

    let role_service = &app_state.role_service;

    // ページネーションパラメータを取得
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).clamp(1, 100);
    let active_only = query.active_only.unwrap_or(false);

    // ロールを取得
    let (roles, total_count) = if active_only {
        role_service
            .list_active_roles_paginated(page, page_size)
            .await?
    } else {
        role_service
            .list_all_roles_paginated(page, page_size)
            .await?
    };

    // レスポンスを構築 - ユーザー数を含める
    let mut role_responses: Vec<crate::api::dto::role_dto::RoleResponse> = Vec::new();

    for role in roles {
        let mut role_response = crate::api::dto::role_dto::RoleResponse::from(role.clone());

        // Get user count for this role
        let user_count = app_state
            .user_service
            .count_users_by_role(role.id)
            .await
            .unwrap_or(0);

        role_response.user_count = user_count as u64;
        role_responses.push(role_response);
    }

    let pagination = crate::api::dto::common::PaginationMeta {
        page,
        per_page: page_size,
        total_pages: ((total_count as f64) / (page_size as f64)).ceil() as i32,
        total_count: total_count as i64,
        has_next: page < ((total_count as f64) / (page_size as f64)).ceil() as i32,
        has_prev: page > 1,
    };

    let response = AdminRoleListResponse {
        roles: role_responses,
        pagination,
    };

    Ok(Json(ApiResponse::success(
        "Roles retrieved successfully",
        response,
    )))
}

/// 管理者向けロール詳細取得（サブスクリプション情報付き）
pub async fn admin_get_role_with_subscription(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUserWithRole,
    Path(role_id): Path<Uuid>,
    Query(params): Query<HashMap<String, String>>,
) -> AppResult<Json<ApiResponse<RoleWithSubscriptionResponse>>> {
    // 管理者権限チェック
    if !user.is_admin() {
        return Err(AppError::Forbidden(
            "Administrator access required".to_string(),
        ));
    }

    let role_service = &app_state.role_service;

    // サブスクリプション階層をクエリパラメータから取得
    let subscription_tier = params
        .get("tier")
        .and_then(|t| crate::core::subscription_tier::SubscriptionTier::from_str(t))
        .unwrap_or(crate::core::subscription_tier::SubscriptionTier::Free);

    // ロールを取得
    let role = role_service
        .get_role_by_id_with_subscription(role_id, subscription_tier)
        .await?;

    // レスポンスを構築
    let response = RoleWithSubscriptionResponse::from_role_with_tier(role, subscription_tier);

    Ok(Json(ApiResponse::success(
        "Role with subscription info retrieved successfully",
        response,
    )))
}

// === 組織管理API ===

/// 管理者向け組織一覧取得
pub async fn admin_list_organizations(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUserWithRole,
    Query(query): Query<AdminOrganizationsRequest>,
) -> AppResult<Json<ApiResponse<AdminOrganizationsResponse>>> {
    // 管理者権限チェック
    if !user.is_admin() {
        return Err(AppError::Forbidden(
            "Administrator access required".to_string(),
        ));
    }

    let organization_service = &app_state.organization_service;

    // ページネーションパラメータ
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).clamp(1, 100);

    // 検索クエリを構築
    let search_query = crate::api::dto::organization_dto::OrganizationSearchQuery {
        name: None,
        owner_id: None,
        subscription_tier: query.subscription_tier,
        page: Some(page as u32),
        page_size: Some(page_size as u32),
    };

    // 組織一覧を取得（管理者なので全組織を取得）
    let (organizations, total_count) = organization_service
        .get_organizations_paginated(search_query, user.user_id())
        .await?;

    // サブスクリプション階層別の統計を計算
    let mut tier_stats: std::collections::HashMap<
        crate::core::subscription_tier::SubscriptionTier,
        crate::api::dto::organization_dto::OrganizationTierStats,
    > = std::collections::HashMap::new();

    for org in &organizations {
        let stats = tier_stats.entry(org.subscription_tier).or_insert(
            crate::api::dto::organization_dto::OrganizationTierStats {
                tier: org.subscription_tier,
                organization_count: 0,
                team_count: 0,
                member_count: 0,
            },
        );
        stats.organization_count += 1;
        stats.team_count += org.current_team_count;
        stats.member_count += org.current_member_count;
    }

    let tier_summary: Vec<_> = tier_stats.into_values().collect();

    let pagination = crate::api::dto::common::PaginationMeta {
        page,
        per_page: page_size,
        total_pages: ((total_count as f64) / (page_size as f64)).ceil() as i32,
        total_count: total_count as i64,
        has_next: page < ((total_count as f64) / (page_size as f64)).ceil() as i32,
        has_prev: page > 1,
    };

    let response = AdminOrganizationsResponse {
        organizations,
        pagination,
        tier_summary,
    };

    Ok(Json(ApiResponse::success(
        "Organizations retrieved successfully",
        response,
    )))
}

/// 管理者向けロール情報付き全ユーザー一覧取得
pub async fn admin_list_users_with_roles(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUserWithRole,
    Query(query): Query<AdminUsersWithRolesRequest>,
) -> AppResult<Json<ApiResponse<AdminUsersWithRolesResponse>>> {
    // 管理者権限チェック
    if !user.is_admin() {
        return Err(AppError::Forbidden(
            "Administrator access required".to_string(),
        ));
    }

    let user_service = &app_state.user_service;

    // ページネーションパラメータ
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).clamp(1, 100);

    // ユーザー一覧を取得
    let (users_with_roles, total_count) = user_service
        .get_all_users_with_roles_paginated(page, page_size, query.role_name.as_deref())
        .await?;

    // UserWithRoleResponseに変換
    let user_responses: Vec<UserWithRoleResponse> = users_with_roles
        .into_iter()
        .map(|user| UserWithRoleResponse {
            id: user.id,
            username: user.username,
            email: user.email,
            is_active: user.is_active,
            email_verified: user.email_verified,
            subscription_tier: user.subscription_tier,
            created_at: user.created_at,
            last_login_at: user.last_login_at,
            role: crate::api::dto::role_dto::RoleResponse::from(user.role),
        })
        .collect();

    // ロール別統計を計算
    let role_stats = user_service.get_user_stats_by_role().await?;
    let role_summary: Vec<RoleSummary> = role_stats
        .into_iter()
        .map(|stats| RoleSummary {
            role_name: stats.role_name,
            role_display_name: stats.role_display_name,
            user_count: stats.total_users,
            active_users: stats.active_users,
            verified_users: stats.verified_users,
        })
        .collect();

    let pagination = crate::api::dto::common::PaginationMeta {
        page,
        per_page: page_size,
        total_pages: ((total_count as f64) / (page_size as f64)).ceil() as i32,
        total_count: total_count as i64,
        has_next: page < ((total_count as f64) / (page_size as f64)).ceil() as i32,
        has_prev: page > 1,
    };

    let response = AdminUsersWithRolesResponse {
        users: user_responses,
        pagination,
        role_summary,
    };

    Ok(Json(ApiResponse::success(
        "Users with roles retrieved successfully",
        response,
    )))
}

/// ユーザーのメンバーステータスをチェック（IsMemberを活用）
pub async fn admin_check_user_member_status(
    State(app_state): State<crate::api::AppState>,
    admin_user: AuthenticatedUserWithRole,
    Path(user_id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    // 管理者権限チェック
    if !admin_user.is_admin() {
        return Err(AppError::Forbidden(
            "Only administrators can check user member status".to_string(),
        ));
    }

    info!(
        admin_id = %admin_user.user_id(),
        target_user_id = %user_id,
        "Admin checking user member status"
    );

    // ユーザー情報を取得
    let user = app_state.user_service.get_user_by_id(user_id).await?;

    // ロール情報を取得
    let role = app_state.role_service.get_role_by_id(user.role_id).await?;

    // IsMemberを使用してメンバーステータスをチェック
    let is_member = PermissionChecker::check_permission_by_role_name(
        &role.name.to_string(),
        PermissionType::IsMember,
        None,
    );

    let member_info = serde_json::json!({
        "user_id": user_id,
        "username": user.username,
        "email": user.email,
        "role_name": role.name.to_string(),
        "is_member": is_member,
        "is_active": user.is_active,
        "email_verified": user.email_verified,
        "subscription_tier": user.subscription_tier,
        "member_permissions": if is_member {
            serde_json::json!({
                "can_create_tasks": true,
                "can_view_own_tasks": true,
                "can_update_own_tasks": true,
                "can_delete_own_tasks": true,
                "can_view_team_tasks": false,
                "can_manage_team": false,
            })
        } else {
            serde_json::json!({
                "can_create_tasks": false,
                "can_view_own_tasks": false,
                "can_update_own_tasks": false,
                "can_delete_own_tasks": false,
                "can_view_team_tasks": false,
                "can_manage_team": false,
            })
        },
    });

    info!(
        admin_id = %admin_user.user_id(),
        target_user_id = %user_id,
        is_member = %is_member,
        "User member status checked"
    );

    Ok(Json(ApiResponse::success(
        "User member status checked successfully",
        member_info,
    )))
}

/// ユーザーのサブスクリプションを変更（管理者専用）
pub async fn change_user_subscription(
    State(app_state): State<crate::api::AppState>,
    admin_user: AuthenticatedUserWithRole,
    Path(user_id): Path<Uuid>,
    Json(request): Json<ChangeUserSubscriptionRequest>,
) -> AppResult<Json<ApiResponse<ChangeUserSubscriptionResponse>>> {
    // 管理者権限チェック
    if !admin_user.is_admin() {
        return Err(AppError::Forbidden(
            "Only administrators can change user subscriptions".to_string(),
        ));
    }

    request.validate()?;

    info!(
        admin_id = %admin_user.user_id(),
        target_user_id = %user_id,
        new_tier = %request.new_tier,
        "Admin changing user subscription"
    );

    // サブスクリプションを変更
    let (updated_user, history) = app_state
        .subscription_service
        .change_subscription_tier(
            user_id,
            request.new_tier.clone(),
            Some(admin_user.user_id()),
            request.reason,
        )
        .await?;

    let response = ChangeUserSubscriptionResponse {
        user_id: updated_user.id,
        previous_tier: history.previous_tier.unwrap_or_else(|| "free".to_string()),
        new_tier: history.new_tier,
        changed_at: history.changed_at,
        history_id: history.id,
    };

    info!(
        admin_id = %admin_user.user_id(),
        target_user_id = %user_id,
        previous_tier = %response.previous_tier,
        new_tier = %response.new_tier,
        "User subscription changed successfully"
    );

    Ok(Json(ApiResponse::success(
        "User subscription changed successfully",
        response,
    )))
}

// === データクリーンアップ・メンテナンスAPI ===

/// 管理者向けバルク操作履歴一覧取得
pub async fn admin_list_bulk_operations(
    State(app_state): State<crate::api::AppState>,
    admin_user: AuthenticatedUserWithRole,
    Query(query): Query<BulkOperationListQuery>,
) -> AppResult<Json<ApiResponse<PaginatedResponse<BulkOperationHistoryResponse>>>> {
    // 管理者権限チェック
    if !admin_user.is_admin() {
        return Err(AppError::Forbidden(
            "Administrator access required".to_string(),
        ));
    }

    info!(
        admin_id = %admin_user.user_id(),
        "Admin listing bulk operation history"
    );

    // 日付範囲が指定されている場合は範囲検索、そうでなければ最新の履歴を取得
    let operations = if let (Some(start_date), Some(end_date)) = (query.start_date, query.end_date)
    {
        app_state
            .bulk_operation_history_repo
            .get_by_date_range(start_date, end_date)
            .await?
    } else {
        // デフォルトで最新100件を取得
        app_state
            .bulk_operation_history_repo
            .get_recent(100)
            .await?
    };

    // ユーザー情報を取得してレスポンスを構築
    let mut responses = Vec::new();
    for op in operations {
        let username =
            if let Ok(user) = app_state.user_service.get_user_by_id(op.performed_by).await {
                Some(user.username)
            } else {
                None
            };

        responses.push(BulkOperationHistoryResponse {
            id: op.id,
            operation_type: op.operation_type,
            performed_by: op.performed_by,
            performed_by_username: username,
            affected_count: op.affected_count,
            status: op.status,
            error_details: op.error_details,
            created_at: op.created_at,
            completed_at: op.completed_at,
        });
    }

    // ページネーション適用
    let page = query.page;
    let per_page = query.per_page;
    let total_count = responses.len() as i64;
    let offset = ((page - 1) * per_page) as usize;
    let limit = per_page as usize;

    let paginated_responses: Vec<BulkOperationHistoryResponse> =
        responses.into_iter().skip(offset).take(limit).collect();

    let response = PaginatedResponse::new(paginated_responses, page, per_page, total_count);

    Ok(Json(ApiResponse::success(
        "Bulk operation history retrieved successfully",
        response,
    )))
}

/// 管理者向け特定ユーザーのバルク操作履歴取得
pub async fn admin_get_user_bulk_operations(
    State(app_state): State<crate::api::AppState>,
    admin_user: AuthenticatedUserWithRole,
    Path(user_id): Path<Uuid>,
    Query(params): Query<HashMap<String, String>>,
) -> AppResult<Json<ApiResponse<Vec<BulkOperationHistoryResponse>>>> {
    // 管理者権限チェック
    if !admin_user.is_admin() {
        return Err(AppError::Forbidden(
            "Administrator access required".to_string(),
        ));
    }

    let limit = params
        .get("limit")
        .and_then(|l| l.parse::<u64>().ok())
        .unwrap_or(50)
        .clamp(1, 200);

    info!(
        admin_id = %admin_user.user_id(),
        target_user_id = %user_id,
        limit = %limit,
        "Admin getting user bulk operation history"
    );

    let operations = app_state
        .bulk_operation_history_repo
        .get_by_user(user_id, limit)
        .await?;

    // レスポンスを構築
    let responses: Vec<BulkOperationHistoryResponse> = operations
        .into_iter()
        .map(|op| BulkOperationHistoryResponse {
            id: op.id,
            operation_type: op.operation_type,
            performed_by: op.performed_by,
            performed_by_username: None, // 既にユーザーが特定されているので不要
            affected_count: op.affected_count,
            status: op.status,
            error_details: op.error_details,
            created_at: op.created_at,
            completed_at: op.completed_at,
        })
        .collect();

    Ok(Json(ApiResponse::success(
        "User bulk operation history retrieved successfully",
        responses,
    )))
}

/// 管理者向け古いバルク操作履歴の削除
pub async fn admin_cleanup_bulk_operations(
    State(app_state): State<crate::api::AppState>,
    admin_user: AuthenticatedUserWithRole,
    Query(params): Query<HashMap<String, String>>,
) -> AppResult<Json<ApiResponse<CleanupResultResponse>>> {
    // 管理者権限チェック
    if !admin_user.is_admin() {
        return Err(AppError::Forbidden(
            "Administrator access required".to_string(),
        ));
    }

    let days = params
        .get("days")
        .and_then(|d| d.parse::<i64>().ok())
        .unwrap_or(90);

    if days < 30 {
        return Err(AppError::ValidationError(
            "Cannot delete bulk operation history less than 30 days old".to_string(),
        ));
    }

    let before_date = Utc::now() - chrono::Duration::days(days);

    info!(
        admin_id = %admin_user.user_id(),
        before_date = %before_date,
        "Admin cleaning up old bulk operation history"
    );

    let deleted_count = app_state
        .bulk_operation_history_repo
        .delete_old_histories(before_date)
        .await?;

    let response = CleanupResultResponse {
        operation_type: "bulk_operation_history_cleanup".to_string(),
        deleted_count,
        before_date: Some(before_date),
        performed_at: Utc::now(),
    };

    Ok(Json(ApiResponse::success(
        format!("Deleted {} bulk operation history records", deleted_count),
        response,
    )))
}

/// 管理者向け古い日次アクティビティサマリーの削除
pub async fn admin_cleanup_daily_summaries(
    State(app_state): State<crate::api::AppState>,
    admin_user: AuthenticatedUserWithRole,
    Query(params): Query<HashMap<String, String>>,
) -> AppResult<Json<ApiResponse<CleanupResultResponse>>> {
    // 管理者権限チェック
    if !admin_user.is_admin() {
        return Err(AppError::Forbidden(
            "Administrator access required".to_string(),
        ));
    }

    let days = params
        .get("days")
        .and_then(|d| d.parse::<i64>().ok())
        .unwrap_or(365);

    if days < 90 {
        return Err(AppError::ValidationError(
            "Cannot delete daily summaries less than 90 days old".to_string(),
        ));
    }

    let before_date = (Utc::now() - chrono::Duration::days(days)).date_naive();

    info!(
        admin_id = %admin_user.user_id(),
        before_date = %before_date,
        "Admin cleaning up old daily activity summaries"
    );

    let deleted_count = app_state
        .daily_activity_summary_repo
        .delete_old_summaries(before_date)
        .await?;

    let response = CleanupResultResponse {
        operation_type: "daily_activity_summary_cleanup".to_string(),
        deleted_count,
        before_date: Some(before_date.and_hms_opt(0, 0, 0).unwrap().and_utc()),
        performed_at: Utc::now(),
    };

    Ok(Json(ApiResponse::success(
        format!("Deleted {} daily activity summary records", deleted_count),
        response,
    )))
}

/// 管理者向けユーザーの機能使用メトリクス取得
pub async fn admin_get_user_feature_metrics(
    State(app_state): State<crate::api::AppState>,
    admin_user: AuthenticatedUserWithRole,
    Query(params): Query<HashMap<String, String>>,
) -> AppResult<Json<ApiResponse<UserFeatureMetricsResponse>>> {
    // 管理者権限チェック
    if !admin_user.is_admin() {
        return Err(AppError::Forbidden(
            "Administrator access required".to_string(),
        ));
    }

    let user_id = params
        .get("user_id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| AppError::BadRequest("Valid user_id parameter is required".to_string()))?;

    let days = params
        .get("days")
        .and_then(|d| d.parse::<i64>().ok())
        .unwrap_or(30);

    let end_date = Utc::now();
    let start_date = end_date - chrono::Duration::days(days);

    info!(
        admin_id = %admin_user.user_id(),
        target_user_id = %user_id,
        start_date = %start_date,
        end_date = %end_date,
        "Admin getting user feature metrics"
    );

    let action_counts = app_state
        .feature_usage_metrics_repo
        .get_user_action_counts(user_id, start_date, end_date)
        .await?;

    let response = UserFeatureMetricsResponse {
        user_id,
        action_counts,
        start_date,
        end_date,
    };

    Ok(Json(ApiResponse::success(
        "User feature metrics retrieved successfully",
        response,
    )))
}

/// 管理者向け古い機能使用メトリクスの削除
pub async fn admin_cleanup_feature_metrics(
    State(app_state): State<crate::api::AppState>,
    admin_user: AuthenticatedUserWithRole,
    Query(params): Query<HashMap<String, String>>,
) -> AppResult<Json<ApiResponse<CleanupResultResponse>>> {
    // 管理者権限チェック
    if !admin_user.is_admin() {
        return Err(AppError::Forbidden(
            "Administrator access required".to_string(),
        ));
    }

    let days = params
        .get("days")
        .and_then(|d| d.parse::<i64>().ok())
        .unwrap_or(180);

    if days < 30 {
        return Err(AppError::ValidationError(
            "Cannot delete feature metrics less than 30 days old".to_string(),
        ));
    }

    let before_date = Utc::now() - chrono::Duration::days(days);

    info!(
        admin_id = %admin_user.user_id(),
        before_date = %before_date,
        "Admin cleaning up old feature usage metrics"
    );

    let deleted_count = app_state
        .feature_usage_metrics_repo
        .delete_old_metrics(before_date)
        .await?;

    let response = CleanupResultResponse {
        operation_type: "feature_usage_metrics_cleanup".to_string(),
        deleted_count,
        before_date: Some(before_date),
        performed_at: Utc::now(),
    };

    Ok(Json(ApiResponse::success(
        format!("Deleted {} feature usage metric records", deleted_count),
        response,
    )))
}

/// 管理者向け機能使用状況のカウント取得
pub async fn admin_get_feature_usage_counts(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Query(params): Query<HashMap<String, String>>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    if !user.is_admin() {
        return Err(AppError::Forbidden(
            "Administrator access required".to_string(),
        ));
    }

    let days = params
        .get("days")
        .and_then(|d| d.parse::<u32>().ok())
        .unwrap_or(30);

    let start_date = Utc::now() - Duration::days(days as i64);
    let end_date = Utc::now();

    let counts = app_state
        .feature_usage_metrics_repo
        .get_feature_usage_counts(start_date, end_date)
        .await?;

    let mut sorted_counts: Vec<_> = counts.into_iter().collect();
    sorted_counts.sort_by(|a, b| b.1.cmp(&a.1));

    Ok(Json(ApiResponse::success(
        "Feature usage counts retrieved successfully",
        serde_json::json!({
            "period_days": days,
            "start_date": start_date.to_rfc3339(),
            "end_date": end_date.to_rfc3339(),
            "feature_counts": sorted_counts
                .into_iter()
                .map(|(name, count)| serde_json::json!({
                    "feature_name": name,
                    "total_usage": count,
                    "unique_users": count, // For now, we don't track unique users separately
                    "usage_count": count
                }))
                .collect::<Vec<_>>()
        }),
    )))
}

/// Admin専用ルーター（統廃合済み）
pub fn admin_router(app_state: crate::api::AppState) -> axum::Router {
    use axum::routing::{delete, get, post, put};

    // admin_only_middlewareを使用してルーター全体に管理者権限チェックを適用
    axum::Router::new()
        // 単一タスク操作
        .route("/admin/tasks", post(admin_create_task))
        .route("/admin/tasks", get(admin_list_all_tasks))
        .route("/admin/tasks/paginated", get(admin_list_tasks_paginated))
        .route("/admin/tasks/{task_id}", get(admin_get_task))
        .route("/admin/tasks/{task_id}", put(admin_update_task))
        .route("/admin/tasks/{task_id}", delete(admin_delete_task))
        // ユーザー固有タスク管理
        .route("/admin/users/{user_id}/tasks", get(admin_list_user_tasks))
        // バッチ操作（統一形式）
        .route("/admin/tasks/bulk/create", post(admin_bulk_create_tasks))
        .route("/admin/tasks/bulk/update", put(admin_bulk_update_tasks))
        .route("/admin/tasks/bulk/delete", delete(admin_bulk_delete_tasks))
        // 統計・管理
        .route("/admin/tasks/statistics", get(admin_get_task_statistics))
        .route("/admin/tasks/stats", get(admin_get_task_stats))
        // 招待管理
        .route(
            "/admin/invitations/cleanup",
            post(admin_cleanup_expired_invitations),
        )
        .route(
            "/admin/invitations/cleanup/old",
            delete(admin_delete_old_invitations),
        )
        // ロール管理
        .route("/admin/analytics/roles", get(admin_list_roles))
        .route(
            "/admin/analytics/roles/{id}/subscription",
            get(admin_get_role_with_subscription),
        )
        // 組織管理
        .route("/admin/organizations", get(admin_list_organizations))
        .route("/admin/users/roles", get(admin_list_users_with_roles))
        // ユーザーステータス管理
        .route(
            "/admin/users/{user_id}/member-status",
            get(admin_check_user_member_status),
        )
        // ユーザーサブスクリプション管理
        .route(
            "/users/{user_id}/subscription",
            put(change_user_subscription),
        )
        // サブスクリプション履歴検索・分析
        .route(
            "/admin/subscription/history/all",
            get(get_all_subscription_history_handler),
        )
        .route(
            "/admin/subscription/history/search",
            get(search_subscription_history_handler),
        )
        .route(
            "/admin/subscription/analytics",
            get(get_subscription_analytics_handler),
        )
        .route(
            "/admin/users/{user_id}/subscription-history",
            delete(delete_user_subscription_history_handler),
        )
        .route(
            "/admin/subscription/history/{id}",
            delete(delete_subscription_history_by_id_handler),
        )
        // ユーザー設定管理
        .route(
            "/admin/users/{user_id}/settings",
            get(admin_get_user_settings)
                .put(admin_update_user_settings)
                .delete(admin_delete_user_settings),
        )
        .route(
            "/admin/users/settings/by-language",
            get(admin_get_users_by_language),
        )
        .route(
            "/admin/users/settings/with-notification",
            get(admin_get_users_with_notification),
        )
        // データクリーンアップ・メンテナンスAPI
        .route(
            "/admin/cleanup/bulk-operations",
            get(admin_list_bulk_operations).delete(admin_cleanup_bulk_operations),
        )
        .route(
            "/admin/cleanup/bulk-operations/user/{user_id}",
            get(admin_get_user_bulk_operations),
        )
        .route(
            "/admin/cleanup/daily-summaries",
            delete(admin_cleanup_daily_summaries),
        )
        .route(
            "/admin/cleanup/feature-metrics",
            get(admin_get_user_feature_metrics).delete(admin_cleanup_feature_metrics),
        )
        // Feature usage analytics
        .route(
            "/admin/analytics/feature-usage-counts",
            get(admin_get_feature_usage_counts),
        )
        // 管理者専用ミドルウェアは、main.rsで適用されるので、ここでは適用しない
        .with_state(app_state)
}

// ============ サブスクリプション履歴検索・分析API ============

/// 全サブスクリプション履歴取得（管理者用）
pub async fn get_all_subscription_history_handler(
    State(app_state): State<crate::api::AppState>,
    admin_user: AuthenticatedUserWithRole,
    Query(pagination): Query<PaginationQuery>,
) -> AppResult<Json<ApiResponse<PaginatedResponse<SubscriptionHistoryItemResponse>>>> {
    // 管理者権限チェック
    if !admin_user.is_admin() {
        return Err(AppError::Forbidden(
            "Only admins can view all subscription history".to_string(),
        ));
    }

    info!(
        admin_id = %admin_user.user_id(),
        "Admin requesting all subscription history"
    );

    let all_histories = app_state.subscription_history_repo.find_all().await?;

    // ページネーション適用
    let (page, per_page) = pagination.get_pagination();
    let total_count = all_histories.len() as i64;
    let offset = pagination.get_offset() as usize;
    let limit = per_page as usize;

    let paginated_histories: Vec<SubscriptionHistoryItemResponse> = all_histories
        .into_iter()
        .skip(offset)
        .take(limit)
        .map(SubscriptionHistoryItemResponse::from)
        .collect();

    let response = PaginatedResponse::new(paginated_histories, page, per_page, total_count);

    Ok(Json(ApiResponse::success(
        "Subscription history retrieved successfully",
        response,
    )))
}

/// サブスクリプション履歴検索（管理者用）
pub async fn search_subscription_history_handler(
    State(app_state): State<crate::api::AppState>,
    admin_user: AuthenticatedUserWithRole,
    Query(search_query): Query<SubscriptionHistorySearchQuery>,
) -> AppResult<Json<ApiResponse<PaginatedResponse<SubscriptionHistoryItemResponse>>>> {
    // 管理者権限チェック
    if !admin_user.is_admin() {
        return Err(AppError::Forbidden(
            "Only admins can search subscription history".to_string(),
        ));
    }

    search_query
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    info!(
        admin_id = %admin_user.user_id(),
        tier = ?search_query.tier,
        user_id = ?search_query.user_id,
        "Admin searching subscription history"
    );

    // 検索条件に基づいてフィルタリング
    let histories = if let Some(tier) = &search_query.tier {
        app_state
            .subscription_history_repo
            .find_by_tier(tier)
            .await?
    } else if let Some(user_id) = search_query.user_id {
        app_state
            .subscription_history_repo
            .find_by_user_id(user_id)
            .await?
    } else if search_query.start_date.is_some() && search_query.end_date.is_some() {
        app_state
            .subscription_history_repo
            .find_by_date_range(
                search_query.start_date.unwrap(),
                search_query.end_date.unwrap(),
            )
            .await?
    } else {
        // 検索条件がない場合は全件取得
        app_state.subscription_history_repo.find_all().await?
    };

    // ページネーション適用
    let page = search_query.page;
    let per_page = search_query.per_page;
    let total_count = histories.len() as i64;
    let offset = ((page - 1) * per_page) as usize;
    let limit = per_page as usize;

    let paginated_histories: Vec<SubscriptionHistoryItemResponse> = histories
        .into_iter()
        .skip(offset)
        .take(limit)
        .map(SubscriptionHistoryItemResponse::from)
        .collect();

    let response = PaginatedResponse::new(paginated_histories, page, per_page, total_count);

    Ok(Json(ApiResponse::success(
        "Subscription history search completed",
        response,
    )))
}

/// サブスクリプション分析データ取得（管理者用）
pub async fn get_subscription_analytics_handler(
    State(app_state): State<crate::api::AppState>,
    admin_user: AuthenticatedUserWithRole,
) -> AppResult<Json<ApiResponse<SubscriptionAnalyticsResponse>>> {
    // 管理者権限チェック
    if !admin_user.is_admin() {
        return Err(AppError::Forbidden(
            "Only admins can view subscription analytics".to_string(),
        ));
    }

    info!(
        admin_id = %admin_user.user_id(),
        "Admin requesting subscription analytics"
    );

    // アップグレード/ダウングレード履歴を取得
    let upgrade_history = app_state.subscription_service.get_upgrade_history().await?;

    let downgrade_history = app_state
        .subscription_service
        .get_downgrade_history()
        .await?;

    // 階層別統計を取得
    let tier_stats = app_state
        .subscription_history_repo
        .get_tier_change_stats()
        .await?;

    // 分析レスポンスを構築
    let analytics = SubscriptionAnalyticsResponse {
        total_upgrades: upgrade_history.len() as u64,
        total_downgrades: downgrade_history.len() as u64,
        tier_distribution: tier_stats
            .into_iter()
            .map(|(tier, count)| SubscriptionTierDistribution {
                tier,
                count,
                percentage: 0.0, // 後で計算
            })
            .collect(),
        recent_upgrades: upgrade_history.into_iter().take(10).collect(),
        recent_downgrades: downgrade_history.into_iter().take(10).collect(),
        monthly_trend: vec![], // 将来の実装用
    };

    // パーセンテージを計算
    let total_changes: u64 = analytics.tier_distribution.iter().map(|t| t.count).sum();

    let mut analytics = analytics;
    for tier in &mut analytics.tier_distribution {
        tier.percentage = if total_changes > 0 {
            (tier.count as f64 / total_changes as f64) * 100.0
        } else {
            0.0
        };
    }

    Ok(Json(ApiResponse::success(
        "Subscription analytics retrieved successfully",
        analytics,
    )))
}

/// 特定ユーザーのサブスクリプション履歴削除（GDPR対応）
pub async fn delete_user_subscription_history_handler(
    State(app_state): State<crate::api::AppState>,
    admin_user: AuthenticatedUserWithRole,
    Path(user_id): Path<Uuid>,
) -> AppResult<(StatusCode, Json<ApiResponse<DeleteHistoryResponse>>)> {
    // 管理者権限チェック
    if !admin_user.is_admin() {
        return Err(AppError::Forbidden(
            "Only admins can delete subscription history".to_string(),
        ));
    }

    info!(
        admin_id = %admin_user.user_id(),
        target_user_id = %user_id,
        "Admin deleting user subscription history"
    );

    let deleted_count = app_state
        .subscription_history_repo
        .delete_by_user_id(user_id)
        .await?;

    let response = DeleteHistoryResponse {
        user_id,
        deleted_count,
        deleted_at: Utc::now(),
    };

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success(
            format!("Deleted {} subscription history records", deleted_count),
            response,
        )),
    ))
}

/// 特定のサブスクリプション履歴を削除（管理者用）
pub async fn delete_subscription_history_by_id_handler(
    State(app_state): State<crate::api::AppState>,
    admin_user: AuthenticatedUserWithRole,
    Path(history_id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<bool>>> {
    // 管理者権限チェック
    if !admin_user.is_admin() {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    info!(
        admin_id = %admin_user.user_id(),
        history_id = %history_id,
        "Admin deleting specific subscription history record"
    );

    let deleted = app_state
        .subscription_history_repo
        .delete_by_id(history_id)
        .await?;

    Ok(Json(ApiResponse::success(
        if deleted {
            "Subscription history record deleted successfully"
        } else {
            "Subscription history record not found"
        },
        deleted,
    )))
}

// === ユーザー設定管理API ===

/// 管理者向けユーザー設定取得
pub async fn admin_get_user_settings(
    State(app_state): State<crate::api::AppState>,
    admin_user: AuthenticatedUserWithRole,
    Path(user_id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<UserSettingsDto>>> {
    // 管理者権限チェック
    if !admin_user.is_admin() {
        return Err(AppError::Forbidden(
            "Administrator access required".to_string(),
        ));
    }

    info!(
        admin_id = %admin_user.user_id(),
        target_user_id = %user_id,
        "Admin getting user settings"
    );

    // UserServiceを通じてuser_settings_repoにアクセス
    let settings = app_state
        .user_service
        .get_user_settings(user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("User settings not found".to_string()))?;

    Ok(Json(ApiResponse::success(
        "User settings retrieved successfully",
        UserSettingsDto::from(settings),
    )))
}

/// 管理者向けユーザー設定更新
pub async fn admin_update_user_settings(
    State(app_state): State<crate::api::AppState>,
    admin_user: AuthenticatedUserWithRole,
    Path(user_id): Path<Uuid>,
    Json(request): Json<UpdateUserSettingsRequest>,
) -> AppResult<Json<ApiResponse<UserSettingsDto>>> {
    // 管理者権限チェック
    if !admin_user.is_admin() {
        return Err(AppError::Forbidden(
            "Administrator access required".to_string(),
        ));
    }

    request.validate()?;

    info!(
        admin_id = %admin_user.user_id(),
        target_user_id = %user_id,
        "Admin updating user settings"
    );

    let input = crate::domain::user_settings_model::UserSettingsInput {
        language: request.language,
        timezone: request.timezone,
        notifications_enabled: request.notifications_enabled,
        email_notifications: request
            .email_notifications
            .and_then(|v| serde_json::from_value(v).ok()),
        ui_preferences: request
            .ui_preferences
            .and_then(|v| serde_json::from_value(v).ok()),
    };

    let settings = app_state
        .user_service
        .update_user_settings(user_id, input)
        .await?;

    Ok(Json(ApiResponse::success(
        "User settings updated successfully",
        UserSettingsDto::from(settings),
    )))
}

/// 管理者向け言語別ユーザー一覧取得
pub async fn admin_get_users_by_language(
    State(app_state): State<crate::api::AppState>,
    admin_user: AuthenticatedUserWithRole,
    Query(params): Query<HashMap<String, String>>,
) -> AppResult<Json<ApiResponse<UsersByLanguageResponse>>> {
    // 管理者権限チェック
    if !admin_user.is_admin() {
        return Err(AppError::Forbidden(
            "Administrator access required".to_string(),
        ));
    }

    let language = params
        .get("language")
        .ok_or_else(|| AppError::BadRequest("Language parameter is required".to_string()))?;

    info!(
        admin_id = %admin_user.user_id(),
        language = %language,
        "Admin getting users by language"
    );

    let user_ids = app_state
        .user_service
        .get_users_by_language(language)
        .await?;

    let response = UsersByLanguageResponse {
        language: language.clone(),
        user_count: user_ids.len(),
        user_ids,
    };

    Ok(Json(ApiResponse::success(
        "Users retrieved successfully",
        response,
    )))
}

/// 管理者向け通知有効ユーザー一覧取得
pub async fn admin_get_users_with_notification(
    State(app_state): State<crate::api::AppState>,
    admin_user: AuthenticatedUserWithRole,
    Query(params): Query<HashMap<String, String>>,
) -> AppResult<Json<ApiResponse<UsersWithNotificationResponse>>> {
    // 管理者権限チェック
    if !admin_user.is_admin() {
        return Err(AppError::Forbidden(
            "Administrator access required".to_string(),
        ));
    }

    let notification_type = params.get("notification_type").ok_or_else(|| {
        AppError::BadRequest("Notification type parameter is required".to_string())
    })?;

    info!(
        admin_id = %admin_user.user_id(),
        notification_type = %notification_type,
        "Admin getting users with notification enabled"
    );

    let user_ids = app_state
        .user_service
        .get_users_with_notification_enabled(notification_type)
        .await?;

    let response = UsersWithNotificationResponse {
        notification_type: notification_type.clone(),
        user_count: user_ids.len(),
        user_ids,
    };

    Ok(Json(ApiResponse::success(
        "Users retrieved successfully",
        response,
    )))
}

/// 管理者向けユーザー設定削除
pub async fn admin_delete_user_settings(
    State(app_state): State<crate::api::AppState>,
    admin_user: AuthenticatedUserWithRole,
    Path(user_id): Path<Uuid>,
) -> AppResult<StatusCode> {
    // 管理者権限チェック
    if !admin_user.is_admin() {
        return Err(AppError::Forbidden(
            "Administrator access required".to_string(),
        ));
    }

    info!(
        admin_id = %admin_user.user_id(),
        target_user_id = %user_id,
        "Admin deleting user settings"
    );

    let deleted = app_state.user_service.delete_user_settings(user_id).await?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound("User settings not found".to_string()))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::admin::dto::TaskStatusStats;
    use validator::Validate;

    #[test]
    fn test_admin_bulk_create_tasks_request_validation() {
        let valid_request = AdminBulkCreateTasksRequest {
            tasks: vec![CreateTaskDto {
                title: "Test Task".to_string(),
                description: Some("Test Description".to_string()),
                status: None,
                priority: None,
                due_date: None,
            }],
            assign_to_user: Some(Uuid::new_v4()),
        };
        assert!(valid_request.validate().is_ok());

        let invalid_empty_request = AdminBulkCreateTasksRequest {
            tasks: vec![],
            assign_to_user: None,
        };
        assert!(invalid_empty_request.validate().is_err());

        let invalid_too_many_request = AdminBulkCreateTasksRequest {
            tasks: (0..101)
                .map(|i| CreateTaskDto {
                    title: format!("Task {}", i),
                    description: None,
                    status: None,
                    priority: None,
                    due_date: None,
                })
                .collect(),
            assign_to_user: None,
        };
        assert!(invalid_too_many_request.validate().is_err());
    }

    #[test]
    fn test_admin_bulk_update_tasks_request_validation() {
        let valid_request = AdminBulkUpdateTasksRequest {
            updates: vec![BatchUpdateTaskItemDto {
                id: Uuid::new_v4(),
                title: Some("Updated Task".to_string()),
                description: Some("Updated Description".to_string()),
                status: Some(crate::core::task_status::TaskStatus::Completed),
                due_date: None,
            }],
        };
        assert!(valid_request.validate().is_ok());

        let invalid_request = AdminBulkUpdateTasksRequest { updates: vec![] };
        assert!(invalid_request.validate().is_err());
    }

    #[test]
    fn test_admin_bulk_delete_tasks_request_validation() {
        let valid_request = AdminBulkDeleteTasksRequest {
            task_ids: vec![Uuid::new_v4(), Uuid::new_v4()],
        };
        assert!(valid_request.validate().is_ok());

        let invalid_request = AdminBulkDeleteTasksRequest { task_ids: vec![] };
        assert!(invalid_request.validate().is_err());
    }

    #[test]
    fn test_admin_bulk_operation_response() {
        let response = AdminBulkOperationResponse {
            success_count: 5,
            failed_count: 2,
            total_requested: 7,
            errors: vec!["Error 1".to_string(), "Error 2".to_string()],
        };

        assert_eq!(response.success_count, 5);
        assert_eq!(response.failed_count, 2);
        assert_eq!(response.total_requested, 7);
        assert_eq!(response.errors.len(), 2);
    }

    #[test]
    fn test_admin_task_stats_response() {
        let stats = AdminTaskStatsResponse {
            total_tasks: 100,
            tasks_by_status: vec![
                TaskStatusStats {
                    status: "pending".to_string(),
                    count: 30,
                },
                TaskStatusStats {
                    status: "completed".to_string(),
                    count: 70,
                },
            ],
            tasks_by_user: vec![],
            recent_activity: vec![],
        };

        assert_eq!(stats.total_tasks, 100);
        assert_eq!(stats.tasks_by_status.len(), 2);
        assert_eq!(stats.tasks_by_status[0].count, 30);
        assert_eq!(stats.tasks_by_status[1].count, 70);
    }

    #[test]
    fn test_admin_single_task_operations_logic() {
        use crate::core::task_status::TaskStatus;

        // 単一タスク作成のロジックテスト
        let create_request = CreateTaskDto {
            title: "Admin Created Task".to_string(),
            description: Some("Task created by admin".to_string()),
            status: Some(TaskStatus::InProgress),
            priority: None,
            due_date: None,
        };
        assert!(!create_request.title.is_empty());
        assert!(create_request.description.is_some());

        // 単一タスク更新のロジックテスト
        let update_request = UpdateTaskDto {
            title: Some("Updated Task Title".to_string()),
            description: Some("Updated description".to_string()),
            status: Some(TaskStatus::Completed),
            priority: None,
            due_date: None,
        };
        assert!(update_request.title.is_some());
        assert_eq!(update_request.status, Some(TaskStatus::Completed));
    }

    #[test]
    fn test_admin_pagination_logic() {
        // ページネーションパラメータのロジックテスト
        let mut params = std::collections::HashMap::new();
        params.insert("page".to_string(), "2".to_string());
        params.insert("page_size".to_string(), "25".to_string());

        let page = params
            .get("page")
            .and_then(|p| p.parse::<u64>().ok())
            .unwrap_or(1);
        let page_size = params
            .get("page_size")
            .and_then(|p| p.parse::<u64>().ok())
            .unwrap_or(10)
            .clamp(1, 100);

        assert_eq!(page, 2);
        assert_eq!(page_size, 25);

        // 不正な値の場合のテスト
        let mut invalid_params = std::collections::HashMap::new();
        invalid_params.insert("page".to_string(), "invalid".to_string());
        invalid_params.insert("page_size".to_string(), "150".to_string());

        let invalid_page = invalid_params
            .get("page")
            .and_then(|p| p.parse::<u64>().ok())
            .unwrap_or(1);
        let invalid_page_size = invalid_params
            .get("page_size")
            .and_then(|p| p.parse::<u64>().ok())
            .unwrap_or(10)
            .clamp(1, 100);

        assert_eq!(invalid_page, 1); // デフォルト値
        assert_eq!(invalid_page_size, 100); // クランプされた最大値
    }

    #[test]
    fn test_admin_batch_dto_conversion_logic() {
        use crate::core::task_status::TaskStatus;

        // BatchCreateTaskDto のロジックテスト
        let batch_create = BatchCreateTaskDto {
            tasks: vec![
                CreateTaskDto {
                    title: "Batch Task 1".to_string(),
                    description: Some("First batch task".to_string()),
                    status: Some(TaskStatus::Todo),
                    priority: None,
                    due_date: None,
                },
                CreateTaskDto {
                    title: "Batch Task 2".to_string(),
                    description: None,
                    status: Some(TaskStatus::InProgress),
                    priority: None,
                    due_date: None,
                },
            ],
        };
        assert_eq!(batch_create.tasks.len(), 2);
        assert!(batch_create.tasks[0].description.is_some());
        assert!(batch_create.tasks[1].description.is_none());

        // BatchUpdateTaskDto のロジックテスト
        let batch_update = BatchUpdateTaskDto {
            tasks: vec![BatchUpdateTaskItemDto {
                id: Uuid::new_v4(),
                title: Some("Updated Batch Task".to_string()),
                description: Some("Updated description".to_string()),
                status: Some(TaskStatus::Completed),
                due_date: None,
            }],
        };
        assert_eq!(batch_update.tasks.len(), 1);
        assert_eq!(batch_update.tasks[0].status, Some(TaskStatus::Completed));

        // BatchDeleteTaskDto のロジックテスト
        let task_ids = vec![Uuid::new_v4(), Uuid::new_v4()];
        let batch_delete = BatchDeleteTaskDto {
            ids: task_ids.clone(),
        };
        assert_eq!(batch_delete.ids.len(), 2);
        assert_eq!(batch_delete.ids, task_ids);
    }
}
