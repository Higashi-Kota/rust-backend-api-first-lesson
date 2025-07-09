// tests/integration/admin_task_stats_test.rs

use axum::{body, http::StatusCode};
use serde_json::Value;
use task_backend::api::dto::analytics_dto::TaskStatsDetailResponse;
use task_backend::shared::types::common::ApiResponse;
use tower::ServiceExt;

use crate::common::{app_helper, auth_helper};

#[tokio::test]
async fn test_admin_task_stats_api() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // 管理者ユーザーでログイン
    let admin_token = auth_helper::create_and_authenticate_admin(&app).await;

    // 通常ユーザーを作成してタスクを追加
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // いくつかのタスクを作成
    for i in 0..5 {
        let task_data = serde_json::json!({
            "title": format!("Test Task {}", i),
            "description": "Test description",
            "status": "todo"
        });

        let req = auth_helper::create_authenticated_request(
            "POST",
            "/tasks",
            &user.access_token,
            Some(serde_json::to_string(&task_data).unwrap()),
        );

        let _ = app.clone().oneshot(req).await.unwrap();
    }

    // いくつかのタスクを完了状態にする
    // まずタスク一覧を取得
    let list_req =
        auth_helper::create_authenticated_request("GET", "/tasks", &user.access_token, None);

    let list_res = app.clone().oneshot(list_req).await.unwrap();
    let body = body::to_bytes(list_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let tasks: Value = serde_json::from_slice(&body).unwrap();
    let task_array = tasks.as_array().unwrap();

    // 最初の3つを完了にする
    for (i, task) in task_array.iter().enumerate() {
        if i < 3 {
            let task_id = task["id"].as_str().unwrap();
            let update_data = serde_json::json!({
                "status": "completed"
            });

            let update_req = auth_helper::create_authenticated_request(
                "PATCH",
                &format!("/tasks/{}", task_id),
                &user.access_token,
                Some(serde_json::to_string(&update_data).unwrap()),
            );

            let _ = app.clone().oneshot(update_req).await.unwrap();
        }
    }

    // 統計を取得（詳細なし）
    let stats_req =
        auth_helper::create_authenticated_request("GET", "/admin/tasks/stats", &admin_token, None);

    let stats_res = app.clone().oneshot(stats_req).await.unwrap();
    assert_eq!(stats_res.status(), StatusCode::OK);

    let body = body::to_bytes(stats_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let stats: ApiResponse<TaskStatsDetailResponse> = serde_json::from_slice(&body).unwrap();

    assert!(stats.success);
    let data = stats.data.unwrap();
    assert_eq!(data.overview.total_tasks, 5);
    assert_eq!(data.overview.completed_tasks, 3);
    assert_eq!(data.overview.pending_tasks, 2);
    assert!(data.user_performance.is_none());

    // 統計を取得（詳細あり）
    let detailed_req = auth_helper::create_authenticated_request(
        "GET",
        "/admin/tasks/stats?include_details=true",
        &admin_token,
        None,
    );

    let detailed_res = app.clone().oneshot(detailed_req).await.unwrap();
    assert_eq!(detailed_res.status(), StatusCode::OK);

    let body = body::to_bytes(detailed_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let detailed_stats: ApiResponse<TaskStatsDetailResponse> =
        serde_json::from_slice(&body).unwrap();

    assert!(detailed_stats.success);
    let detailed_data = detailed_stats.data.unwrap();
    assert!(detailed_data.user_performance.is_some());

    let performances = detailed_data.user_performance.unwrap();
    assert_eq!(performances.len(), 1);
    assert_eq!(performances[0].tasks_created, 5);
    assert_eq!(performances[0].tasks_completed, 3);
    assert_eq!(performances[0].completion_rate, 60.0);
}

#[tokio::test]
async fn test_admin_task_stats_requires_admin_permission() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // 通常ユーザーでログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // 統計取得を試みる
    let stats_req = auth_helper::create_authenticated_request(
        "GET",
        "/admin/tasks/stats",
        &user.access_token,
        None,
    );

    let stats_res = app.clone().oneshot(stats_req).await.unwrap();

    // 権限エラーを確認
    assert_eq!(stats_res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_admin_task_stats_status_distribution() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // 管理者ユーザーでログイン
    let admin_token = auth_helper::create_and_authenticate_admin(&app).await;

    // 統計を取得
    let stats_req =
        auth_helper::create_authenticated_request("GET", "/admin/tasks/stats", &admin_token, None);

    let stats_res = app.clone().oneshot(stats_req).await.unwrap();
    assert_eq!(stats_res.status(), StatusCode::OK);

    let body = body::to_bytes(stats_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let stats: ApiResponse<TaskStatsDetailResponse> = serde_json::from_slice(&body).unwrap();

    assert!(stats.success);
    let data = stats.data.unwrap();

    // ステータス分布が存在することを確認
    assert!(!data.status_distribution.is_empty());

    // 各ステータスが含まれていることを確認
    let statuses: Vec<&str> = data
        .status_distribution
        .iter()
        .map(|s| s.status.as_str())
        .collect();

    assert!(statuses.contains(&"todo"));
    assert!(statuses.contains(&"in_progress"));
    assert!(statuses.contains(&"completed"));
}
