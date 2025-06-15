// tests/integration/tasks/extended_task_api_tests.rs

use crate::common::{app_helper, auth_helper, test_data};
use axum::{
    body::{self, Body},
    http::{Request, StatusCode},
};
use serde_json::Value;
use tower::ServiceExt;

/// 新しく追加されたTask APIの統合テスト

#[tokio::test]
async fn test_get_user_task_stats_with_empty_tasks() {
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let user_token = auth_helper::create_member_with_jwt(&app).await;

    // タスクが存在しない状態での統計取得
    let req = auth_helper::create_authenticated_request("GET", "/tasks/stats", &user_token, None);

    let response = app.clone().oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let stats: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(stats["total_tasks"], 0);
    assert_eq!(stats["completed_tasks"], 0);
    assert_eq!(stats["pending_tasks"], 0);
    assert_eq!(stats["in_progress_tasks"], 0);
    assert_eq!(stats["completion_rate"], 0.0);

    // ステータス分布の確認
    let status_distribution = &stats["status_distribution"];
    assert_eq!(status_distribution["pending"], 0);
    assert_eq!(status_distribution["in_progress"], 0);
    assert_eq!(status_distribution["completed"], 0);
    assert_eq!(status_distribution["other"], 0);
}

#[tokio::test]
async fn test_get_user_task_stats_with_mixed_tasks() {
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let user_token = auth_helper::create_member_with_jwt(&app).await;

    // 異なるステータスのタスクを作成
    let tasks_data = [
        ("Task 1", "todo"),
        ("Task 2", "in_progress"),
        ("Task 3", "completed"),
        ("Task 4", "completed"),
        ("Task 5", "todo"),
    ];

    for (title, status) in &tasks_data {
        let task_data = test_data::create_custom_task(title, Some("Test task"), Some(status));
        let req = auth_helper::create_authenticated_request(
            "POST",
            "/tasks",
            &user_token,
            Some(serde_json::to_string(&task_data).unwrap()),
        );

        let response = app.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    // 統計情報を取得
    let req = auth_helper::create_authenticated_request("GET", "/tasks/stats", &user_token, None);

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let stats: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(stats["total_tasks"], 5);
    assert_eq!(stats["completed_tasks"], 2);
    assert_eq!(stats["pending_tasks"], 2); // "todo" tasks are counted as pending
    assert_eq!(stats["in_progress_tasks"], 1);
    assert_eq!(stats["completion_rate"], 40.0); // 2/5 = 40%

    // ステータス分布の確認
    let status_distribution = &stats["status_distribution"];
    assert_eq!(status_distribution["pending"], 2); // "todo" maps to pending
    assert_eq!(status_distribution["in_progress"], 1);
    assert_eq!(status_distribution["completed"], 2);
    assert_eq!(status_distribution["other"], 0);
}

#[tokio::test]
async fn test_get_user_task_stats_without_authentication() {
    let (app, _schema, _db) = app_helper::setup_full_app().await;

    // 認証なしでタスク統計取得を試行
    let req = Request::builder()
        .uri("/tasks/stats")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_bulk_update_status_with_valid_data() {
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let user_token = auth_helper::create_member_with_jwt(&app).await;

    // 複数のタスクを作成
    let mut task_ids = Vec::new();
    for i in 1..=3 {
        let task_data = test_data::create_custom_task(
            &format!("Task {}", i),
            Some("Test task for bulk update"),
            Some("todo"),
        );
        let req = auth_helper::create_authenticated_request(
            "POST",
            "/tasks",
            &user_token,
            Some(serde_json::to_string(&task_data).unwrap()),
        );

        let response = app.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);

        let body = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let task_data: Value = serde_json::from_slice(&body).unwrap();
        task_ids.push(task_data["id"].as_str().unwrap().to_string());
    }

    // 一括ステータス更新
    let req = auth_helper::create_authenticated_request(
        "PATCH",
        "/tasks/batch/status",
        &user_token,
        Some(
            serde_json::to_string(&serde_json::json!({
                "task_ids": task_ids,
                "status": "completed"
            }))
            .unwrap(),
        ),
    );

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(result["updated_count"], 3);
    assert_eq!(result["new_status"], "completed");
    assert!(result["errors"].as_array().unwrap().is_empty());

    // 統計を確認して更新が反映されているかチェック
    let req = auth_helper::create_authenticated_request("GET", "/tasks/stats", &user_token, None);

    let response = app.clone().oneshot(req).await.unwrap();
    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let stats: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(stats["completed_tasks"], 3);
    assert_eq!(stats["pending_tasks"], 0); // All tasks were updated to completed
}

#[tokio::test]
async fn test_bulk_update_status_with_invalid_task_ids() {
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let user_token = auth_helper::create_member_with_jwt(&app).await;

    // 存在しないタスクIDで一括更新を試行
    let fake_task_ids = [
        "550e8400-e29b-41d4-a716-446655440000",
        "550e8400-e29b-41d4-a716-446655440001",
    ];

    let req = auth_helper::create_authenticated_request(
        "PATCH",
        "/tasks/batch/status",
        &user_token,
        Some(
            serde_json::to_string(&serde_json::json!({
                "task_ids": fake_task_ids,
                "status": "completed"
            }))
            .unwrap(),
        ),
    );

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(result["updated_count"], 0);
    assert_eq!(result["new_status"], "completed");
    assert_eq!(result["errors"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_bulk_update_status_with_invalid_status() {
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let user_token = auth_helper::create_member_with_jwt(&app).await;

    // 無効なステータスで一括更新を試行
    let req = auth_helper::create_authenticated_request(
        "PATCH",
        "/tasks/batch/status",
        &user_token,
        Some(
            serde_json::to_string(&serde_json::json!({
                "task_ids": ["550e8400-e29b-41d4-a716-446655440000"],
                "status": "invalid_status"
            }))
            .unwrap(),
        ),
    );

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_bulk_update_status_with_empty_task_ids() {
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let user_token = auth_helper::create_member_with_jwt(&app).await;

    // 空のタスクIDリストで一括更新を試行
    let req = auth_helper::create_authenticated_request(
        "PATCH",
        "/tasks/batch/status",
        &user_token,
        Some(
            serde_json::to_string(&serde_json::json!({
                "task_ids": [],
                "status": "completed"
            }))
            .unwrap(),
        ),
    );

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(result["updated_count"], 0);
    assert_eq!(result["new_status"], "completed");
    assert!(result["errors"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_bulk_update_status_with_malformed_uuid() {
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let user_token = auth_helper::create_member_with_jwt(&app).await;

    // 不正な形式のUUIDで一括更新を試行
    let req = auth_helper::create_authenticated_request(
        "PATCH",
        "/tasks/batch/status",
        &user_token,
        Some(
            serde_json::to_string(&serde_json::json!({
                "task_ids": ["invalid-uuid", "another-bad-uuid"],
                "status": "completed"
            }))
            .unwrap(),
        ),
    );

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(result["updated_count"], 0);
    assert_eq!(result["new_status"], "completed");
    assert_eq!(result["errors"].as_array().unwrap().len(), 2);

    // エラーメッセージに"Invalid UUID"が含まれることを確認
    let errors = result["errors"].as_array().unwrap();
    for error in errors {
        let error_str = error.as_str().unwrap();
        assert!(error_str.contains("Invalid UUID"));
    }
}

#[tokio::test]
async fn test_bulk_update_status_user_isolation() {
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let user1_token = auth_helper::create_member_with_jwt(&app).await;
    let user2_token = auth_helper::create_member_with_jwt(&app).await;

    // ユーザー1がタスクを作成
    let task_data = test_data::create_custom_task(
        "User 1 Task",
        Some("Task belonging to user 1"),
        Some("todo"),
    );
    let req = auth_helper::create_authenticated_request(
        "POST",
        "/tasks",
        &user1_token,
        Some(serde_json::to_string(&task_data).unwrap()),
    );

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let task_data: Value = serde_json::from_slice(&body).unwrap();
    let task_id = task_data["id"].as_str().unwrap();

    // ユーザー2がユーザー1のタスクの一括更新を試行
    let req = auth_helper::create_authenticated_request(
        "PATCH",
        "/tasks/batch/status",
        &user2_token,
        Some(
            serde_json::to_string(&serde_json::json!({
                "task_ids": [task_id],
                "status": "completed"
            }))
            .unwrap(),
        ),
    );

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(result["updated_count"], 0);
    assert_eq!(result["errors"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn test_extended_task_api_response_format() {
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let user_token = auth_helper::create_member_with_jwt(&app).await;

    // タスク統計のレスポンス形式をテスト
    let req = auth_helper::create_authenticated_request("GET", "/tasks/stats", &user_token, None);

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let stats: Value = serde_json::from_slice(&body).unwrap();

    // 必須フィールドの存在確認
    assert!(stats.get("total_tasks").is_some());
    assert!(stats.get("completed_tasks").is_some());
    assert!(stats.get("pending_tasks").is_some());
    assert!(stats.get("in_progress_tasks").is_some());
    assert!(stats.get("completion_rate").is_some());
    assert!(stats.get("status_distribution").is_some());

    // ステータス分布の必須フィールド確認
    let status_distribution = &stats["status_distribution"];
    assert!(status_distribution.get("pending").is_some());
    assert!(status_distribution.get("in_progress").is_some());
    assert!(status_distribution.get("completed").is_some());
    assert!(status_distribution.get("other").is_some());

    // データ型の確認
    assert!(stats["total_tasks"].is_u64());
    assert!(stats["completed_tasks"].is_u64());
    assert!(stats["completion_rate"].is_f64());
}

#[tokio::test]
async fn test_extended_task_api_content_type_headers() {
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let user_token = auth_helper::create_member_with_jwt(&app).await;

    // レスポンスヘッダーの確認
    let req = auth_helper::create_authenticated_request("GET", "/tasks/stats", &user_token, None);

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let headers = response.headers();
    assert_eq!(
        headers.get("content-type").unwrap().to_str().unwrap(),
        "application/json"
    );
}
