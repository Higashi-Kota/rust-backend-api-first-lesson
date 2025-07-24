// tests/integration/tasks/search_tests.rs
// 検索クエリパターンの統合テスト

use crate::common::{
    app_helper::{create_request, setup_full_app},
    auth_helper::create_and_authenticate_user,
};
use axum::body::to_bytes;
use axum::http::StatusCode;
use serde_json::json;
use task_backend::api::dto::common::PaginatedResponse;
use task_backend::api::dto::task_dto::TaskDto;
use task_backend::types::ApiResponse;
use tower::ServiceExt;

#[tokio::test]
async fn test_search_tasks_basic() {
    // Arrange
    let (app, _schema, _db) = setup_full_app().await;
    let user = create_and_authenticate_user(&app).await;

    // 複数のタスクを作成
    for i in 0..5 {
        let task_data = json!({
            "title": format!("Task {} for search", i),
            "description": if i % 2 == 0 { "bug fix" } else { "feature" },
            "status": if i % 2 == 0 { "in_progress" } else { "todo" }
        });

        let response = app
            .clone()
            .oneshot(create_request(
                "POST",
                "/tasks",
                &user.access_token,
                &task_data,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    // Act: 統一検索エンドポイントを使用
    let response = app
        .oneshot(create_request(
            "GET",
            "/tasks/search?search=Task&page=1&per_page=10",
            &user.access_token,
            &json!({}),
        ))
        .await
        .unwrap();

    // Assert
    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    if status != StatusCode::OK {
        let error_text = String::from_utf8_lossy(&body);
        panic!("Expected OK status, got {}: {}", status, error_text);
    }
    let result: ApiResponse<PaginatedResponse<TaskDto>> = serde_json::from_slice(&body).unwrap();

    assert!(result.success);
    let data = result.data.unwrap();
    assert_eq!(data.items.len(), 5);
    assert_eq!(data.pagination.total_count, 5);
}

#[tokio::test]
async fn test_search_with_filters() {
    // Arrange
    let (app, _schema, _db) = setup_full_app().await;
    let user = create_and_authenticate_user(&app).await;

    // 異なるステータスのタスクを作成
    let statuses = ["todo", "in_progress", "completed"];
    for (i, status) in statuses.iter().enumerate() {
        let task_data = json!({
            "title": format!("Task {} with status", i),
            "status": status
        });

        let response = app
            .clone()
            .oneshot(create_request(
                "POST",
                "/tasks",
                &user.access_token,
                &task_data,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    // Act: ステータスフィルタを使用
    let response = app
        .oneshot(create_request(
            "GET",
            "/tasks/search?status=in_progress",
            &user.access_token,
            &json!({}),
        ))
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let result: ApiResponse<PaginatedResponse<TaskDto>> = serde_json::from_slice(&body).unwrap();

    assert!(result.success);
    let data = result.data.unwrap();
    assert_eq!(data.items.len(), 1);
    assert_eq!(data.items[0].status.as_str(), "in_progress");
}

#[tokio::test]
async fn test_search_with_sorting() {
    // Arrange
    let (app, _schema, _db) = setup_full_app().await;
    let user = create_and_authenticate_user(&app).await;

    // タイトルが異なるタスクを作成
    let titles = ["Charlie Task", "Alpha Task", "Bravo Task"];
    for title in &titles {
        let task_data = json!({
            "title": title
        });

        let response = app
            .clone()
            .oneshot(create_request(
                "POST",
                "/tasks",
                &user.access_token,
                &task_data,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    // Act: タイトルでソート（昇順）
    let response = app
        .oneshot(create_request(
            "GET",
            "/tasks/search?sort_by=title&sort_order=asc",
            &user.access_token,
            &json!({}),
        ))
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let result: ApiResponse<PaginatedResponse<TaskDto>> = serde_json::from_slice(&body).unwrap();

    assert!(result.success);
    let data = result.data.unwrap();
    assert_eq!(data.items.len(), 3);
    assert_eq!(data.items[0].title, "Alpha Task");
    assert_eq!(data.items[1].title, "Bravo Task");
    assert_eq!(data.items[2].title, "Charlie Task");
}

#[tokio::test]
async fn test_search_with_pagination() {
    // Arrange
    let (app, _schema, _db) = setup_full_app().await;
    let user = create_and_authenticate_user(&app).await;

    // 9個のタスクを作成（Freeティアの制限内）
    for i in 0..9 {
        let task_data = json!({
            "title": format!("Task {:02}", i)
        });

        let response = app
            .clone()
            .oneshot(create_request(
                "POST",
                "/tasks",
                &user.access_token,
                &task_data,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    // Act: 2ページ目を取得（page_size=5）
    let response = app
        .oneshot(create_request(
            "GET",
            "/tasks/search?page=2&per_page=5&sort_by=created_at&sort_order=asc",
            &user.access_token,
            &json!({}),
        ))
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let result: ApiResponse<PaginatedResponse<TaskDto>> = serde_json::from_slice(&body).unwrap();

    assert!(result.success);
    let data = result.data.unwrap();
    assert_eq!(data.items.len(), 4); // 9個のタスクで、2ページ目は残り4個
    assert_eq!(data.pagination.page, 2);
    assert_eq!(data.pagination.per_page, 5);
    assert_eq!(data.pagination.total_count, 9);
    assert_eq!(data.pagination.total_pages, 2);
    assert!(!data.pagination.has_next); // 2ページで終わり
    assert!(data.pagination.has_prev);
}

#[tokio::test]
async fn test_search_complex_query() {
    // Arrange
    let (app, _schema, _db) = setup_full_app().await;
    let user = create_and_authenticate_user(&app).await;

    // 様々なタスクを作成
    let tasks = vec![
        json!({"title": "Fix bug in login", "status": "in_progress", "priority": "high"}),
        json!({"title": "Add feature X", "status": "todo", "priority": "medium"}),
        json!({"title": "Fix bug in API", "status": "in_progress", "priority": "high"}),
        json!({"title": "Update docs", "status": "completed", "priority": "low"}),
        json!({"title": "Fix UI issue", "status": "todo", "priority": "high"}),
    ];

    for task_data in &tasks {
        let response = app
            .clone()
            .oneshot(create_request(
                "POST",
                "/tasks",
                &user.access_token,
                task_data,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    // Act: 複雑なクエリ（検索語 + ステータス + 優先度 + ソート）
    let response = app
        .oneshot(create_request(
            "GET",
            "/tasks/search?search=Fix&status=in_progress&priority=high&sort_by=title&sort_order=asc",
            &user.access_token,
            &json!({}),
        ))
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let result: ApiResponse<PaginatedResponse<TaskDto>> = serde_json::from_slice(&body).unwrap();

    assert!(result.success);
    let data = result.data.unwrap();
    assert_eq!(data.items.len(), 2);
    assert!(data.items[0].title.contains("Fix"));
    assert!(data.items[1].title.contains("Fix"));
    assert_eq!(data.items[0].status.as_str(), "in_progress");
    assert_eq!(data.items[1].status.as_str(), "in_progress");
}

#[tokio::test]
async fn test_search_empty_result() {
    // Arrange
    let (app, _schema, _db) = setup_full_app().await;
    let user = create_and_authenticate_user(&app).await;

    // Act: 存在しない検索語でクエリ
    let response = app
        .oneshot(create_request(
            "GET",
            "/tasks/search?search=NonExistentTask",
            &user.access_token,
            &json!({}),
        ))
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let result: ApiResponse<PaginatedResponse<TaskDto>> = serde_json::from_slice(&body).unwrap();

    assert!(result.success);
    let data = result.data.unwrap();
    assert_eq!(data.items.len(), 0);
    assert_eq!(data.pagination.total_count, 0);
}

#[tokio::test]
async fn test_search_invalid_sort_field() {
    // Arrange
    let (app, _schema, _db) = setup_full_app().await;
    let user = create_and_authenticate_user(&app).await;

    // Act: 許可されていないソートフィールドを使用
    let response = app
        .oneshot(create_request(
            "GET",
            "/tasks/search?sort_by=invalid_field",
            &user.access_token,
            &json!({}),
        ))
        .await
        .unwrap();

    // Assert: 無効なソートフィールドは無視される
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let result: ApiResponse<PaginatedResponse<TaskDto>> = serde_json::from_slice(&body).unwrap();

    assert!(result.success);
    assert!(result.data.is_some());
}

#[tokio::test]
async fn test_search_date_range_filter() {
    // Arrange
    let (app, _schema, _db) = setup_full_app().await;
    let user = create_and_authenticate_user(&app).await;

    // 異なる期限のタスクを作成
    let now = chrono::Utc::now();
    let tasks = vec![
        json!({
            "title": "Past task",
            "due_date": (now - chrono::Duration::days(7)).timestamp()
        }),
        json!({
            "title": "Current task",
            "due_date": now.timestamp()
        }),
        json!({
            "title": "Future task",
            "due_date": (now + chrono::Duration::days(7)).timestamp()
        }),
    ];

    for task_data in &tasks {
        let response = app
            .clone()
            .oneshot(create_request(
                "POST",
                "/tasks",
                &user.access_token,
                task_data,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    // Act: 期限が今日より前のタスクを検索
    let response = app
        .oneshot(create_request(
            "GET",
            &format!("/tasks/search?due_date_before={}", now.timestamp()),
            &user.access_token,
            &json!({}),
        ))
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let result: ApiResponse<PaginatedResponse<TaskDto>> = serde_json::from_slice(&body).unwrap();

    assert!(result.success);
    let data = result.data.unwrap();
    assert_eq!(data.items.len(), 1); // Past task のみ（due_date_beforeは厳密な不等号）
}
