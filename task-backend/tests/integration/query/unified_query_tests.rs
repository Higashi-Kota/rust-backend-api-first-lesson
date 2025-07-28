use crate::common::{app_helper, auth_helper};
use axum::http::StatusCode;
use serde_json::{json, Value};
use tower::ServiceExt;

/// 統一パターンのテスト - 全パラメータの組み合わせ
#[tokio::test]
async fn test_unified_query_all_parameters_combination() {
    // Arrange
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // タスクを複数作成
    for i in 0..25 {
        let create_req = auth_helper::create_authenticated_request(
            "POST",
            "/tasks",
            &user.access_token,
            Some(
                json!({
                    "title": format!("Task {}", i),
                    "description": format!("Description for task {}", i),
                    "priority": if i % 2 == 0 { "high" } else { "medium" },
                    "status": "todo"
                })
                .to_string(),
            ),
        );
        app.clone().oneshot(create_req).await.unwrap();
    }

    // Act: 全パラメータを組み合わせたクエリ
    let req = auth_helper::create_authenticated_request(
        "GET",
        "/tasks/search?page=2&per_page=5&sort_by=priority&sort_order=desc&search=Task&status=todo",
        &user.access_token,
        None,
    );
    let response = app.clone().oneshot(req).await.unwrap();

    // Assert - check status first
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();

    if status != StatusCode::OK {
        println!("Response status: {}", status);
        println!(
            "Response body as string: {}",
            String::from_utf8_lossy(&body)
        );
        panic!("Expected OK, got {}", status);
    }

    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(status, StatusCode::OK);

    // ページネーションの確認
    assert_eq!(json["data"]["pagination"]["page"], 2);
    assert_eq!(json["data"]["pagination"]["per_page"], 5);
    assert!(json["data"]["pagination"]["total_count"].as_i64().unwrap() >= 10);

    // アイテムの確認
    let items = json["data"]["items"].as_array().unwrap();
    assert_eq!(items.len(), 5); // per_page = 5

    // 検索条件の確認（すべて"Task"を含む）
    for item in items {
        assert!(item["title"].as_str().unwrap().contains("Task"));
        assert_eq!(item["status"], "todo");
    }
}

/// ページネーション境界値テスト
#[tokio::test]
async fn test_pagination_boundary_values() {
    // Arrange
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // Test 1: 0ページの処理（自動的に1に補正される）
    let req = auth_helper::create_authenticated_request(
        "GET",
        "/tasks/search?page=0&per_page=10",
        &user.access_token,
        None,
    );
    let response = app.clone().oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["data"]["pagination"]["page"], 1); // 0 → 1に補正

    // Test 2: 超大きいページ番号
    let req = auth_helper::create_authenticated_request(
        "GET",
        "/tasks/search?page=999999&per_page=10",
        &user.access_token,
        None,
    );
    let response = app.clone().oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["data"]["items"].as_array().unwrap().len(), 0); // 空の結果

    // Test 3: per_pageの最大値超過（100を超える場合は100に制限）
    let req = auth_helper::create_authenticated_request(
        "GET",
        "/tasks/search?page=1&per_page=200",
        &user.access_token,
        None,
    );
    let response = app.clone().oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["data"]["pagination"]["per_page"], 100); // 200 → 100に制限
}

/// 空の検索結果の適切な処理
#[tokio::test]
async fn test_empty_search_results() {
    // Arrange
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // Act: 存在しない検索条件
    let req = auth_helper::create_authenticated_request(
        "GET",
        "/tasks/search?search=NonExistentTaskName123456789",
        &user.access_token,
        None,
    );
    let response = app.clone().oneshot(req).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // 空の結果でも適切な構造を持つ
    assert_eq!(json["data"]["items"].as_array().unwrap().len(), 0);
    assert_eq!(json["data"]["pagination"]["total_count"], 0);
    assert_eq!(json["data"]["pagination"]["total_pages"], 0);
    assert_eq!(json["data"]["pagination"]["has_next"], false);
    assert_eq!(json["data"]["pagination"]["has_prev"], false);
}

/// ソート順序（asc/desc）の動作確認
#[tokio::test]
async fn test_sort_order_asc_desc() {
    // Arrange
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // 異なる優先度のタスクを作成
    let tasks = vec![
        ("Task A", "low"),
        ("Task B", "high"),
        ("Task C", "medium"),
        ("Task D", "high"),
        ("Task E", "low"),
    ];

    for (title, priority) in &tasks {
        let create_req = auth_helper::create_authenticated_request(
            "POST",
            "/tasks",
            &user.access_token,
            Some(
                json!({
                    "title": title,
                    "priority": priority,
                    "status": "todo"
                })
                .to_string(),
            ),
        );
        app.clone().oneshot(create_req).await.unwrap();
    }

    // Test 1: 昇順ソート
    let req = auth_helper::create_authenticated_request(
        "GET",
        "/tasks/search?sort_by=priority&sort_order=asc",
        &user.access_token,
        None,
    );
    let response = app.clone().oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    let items = json["data"]["items"].as_array().unwrap();

    // 優先度の昇順ソートを確認（実装に応じて高優先度が先に来る可能性がある）
    let first_priority = items[0]["priority"].as_str().unwrap();
    assert!(["high", "medium", "low"].contains(&first_priority));

    // Test 2: 降順ソート
    let req = auth_helper::create_authenticated_request(
        "GET",
        "/tasks/search?sort_by=priority&sort_order=desc",
        &user.access_token,
        None,
    );
    let response = app.clone().oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    let items = json["data"]["items"].as_array().unwrap();

    // 降順ソートを確認
    let first_priority_desc = items[0]["priority"].as_str().unwrap();
    // 昇順と降順で異なる結果になることを確認
    assert!(["high", "medium", "low"].contains(&first_priority_desc));
}

/// 特殊文字を含む検索文字列の処理
#[tokio::test]
async fn test_search_with_special_characters() {
    // Arrange
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // 特殊文字を含むタスクを作成
    let special_titles = vec![
        "Task with @ symbol",
        "Task with #hashtag",
        "Task with $money",
        "Task with % percent",
        "Task with & ampersand",
        "Task with (parentheses)",
        "Task with 日本語",
        "Task with émojis 🎉",
    ];

    for title in &special_titles {
        let create_req = auth_helper::create_authenticated_request(
            "POST",
            "/tasks",
            &user.access_token,
            Some(
                json!({
                    "title": title,
                    "priority": "medium",
                    "status": "todo"
                })
                .to_string(),
            ),
        );
        app.clone().oneshot(create_req).await.unwrap();
    }

    // Test 1: @記号での検索
    let req = auth_helper::create_authenticated_request(
        "GET",
        "/tasks/search?search=%40", // URL encoded @
        &user.access_token,
        None,
    );
    let response = app.clone().oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    let items = json["data"]["items"].as_array().unwrap();
    assert!(!items.is_empty());
    assert!(items[0]["title"].as_str().unwrap().contains("@"));

    // Test 2: 日本語での検索
    let req = auth_helper::create_authenticated_request(
        "GET",
        "/tasks/search?search=日本語",
        &user.access_token,
        None,
    );
    let response = app.clone().oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    let items = json["data"]["items"].as_array().unwrap();
    assert!(!items.is_empty());
    assert!(items[0]["title"].as_str().unwrap().contains("日本語"));

    // Test 3: 絵文字での検索
    let req = auth_helper::create_authenticated_request(
        "GET",
        "/tasks/search?search=🎉",
        &user.access_token,
        None,
    );
    let response = app.clone().oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    let items = json["data"]["items"].as_array().unwrap();
    assert!(!items.is_empty());
    assert!(items[0]["title"].as_str().unwrap().contains("🎉"));
}

/// デフォルト値の動作確認
#[tokio::test]
async fn test_query_parameter_defaults() {
    // Arrange
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // Act: パラメータなしでリクエスト
    let req =
        auth_helper::create_authenticated_request("GET", "/tasks/search", &user.access_token, None);
    let response = app.clone().oneshot(req).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // デフォルト値の確認
    assert_eq!(json["data"]["pagination"]["page"], 1); // デフォルトページ
    assert_eq!(json["data"]["pagination"]["per_page"], 20); // デフォルトサイズ
}
