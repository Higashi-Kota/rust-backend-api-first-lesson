// tests/integration/tasks/pagination_tests.rs

use axum::{
    body::{self, Body},
    http::{Request, StatusCode},
};
use serde_json::Value;
use tower::ServiceExt;

use crate::common::{app_helper, auth_helper, test_data};

#[tokio::test]
async fn test_paginated_tasks_with_authentication() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // ページネーション用のタスクを作成（12個）
    for i in 1..=12 {
        let task_data = test_data::create_test_task_with_title(&format!("Pagination Task {}", i));
        let create_req = auth_helper::create_authenticated_request(
            "POST",
            "/tasks",
            &user.access_token,
            Some(serde_json::to_string(&task_data).unwrap()),
        );

        let create_res = app.clone().oneshot(create_req).await.unwrap();
        assert_eq!(create_res.status(), StatusCode::CREATED);
    }

    // 1ページ目を取得（ページサイズ5）
    let page1_req = auth_helper::create_authenticated_request(
        "GET",
        "/tasks/paginated?page=1&page_size=5",
        &user.access_token,
        None,
    );

    let page1_res = app.clone().oneshot(page1_req).await.unwrap();

    assert_eq!(page1_res.status(), StatusCode::OK);
    let body = body::to_bytes(page1_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let page1_result: Value = serde_json::from_slice(&body).unwrap();

    // 1ページ目の検証
    assert!(page1_result["items"].is_array());
    let page1_tasks = page1_result["items"].as_array().unwrap();
    assert_eq!(page1_tasks.len(), 5);

    // 全てのタスクが自分のものであることを確認
    for task in page1_tasks {
        assert_eq!(task["user_id"], user.id.to_string());
    }

    // ページネーション情報の検証
    let pagination = &page1_result["pagination"];
    assert_eq!(pagination["page"], 1);
    assert_eq!(pagination["per_page"], 5);
    assert_eq!(pagination["total_count"], 12);
    assert_eq!(pagination["has_next"], true);
    assert_eq!(pagination["has_prev"], false);
    assert_eq!(pagination["total_pages"], 3);
}

#[tokio::test]
async fn test_paginated_tasks_second_page() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // ページネーション用のタスクを作成（8個）
    for i in 1..=8 {
        let task_data = test_data::create_test_task_with_title(&format!("Page Test Task {}", i));
        let create_req = auth_helper::create_authenticated_request(
            "POST",
            "/tasks",
            &user.access_token,
            Some(serde_json::to_string(&task_data).unwrap()),
        );

        let create_res = app.clone().oneshot(create_req).await.unwrap();
        assert_eq!(create_res.status(), StatusCode::CREATED);
    }

    // 2ページ目を取得（ページサイズ3）
    let page2_req = auth_helper::create_authenticated_request(
        "GET",
        "/tasks/paginated?page=2&page_size=3",
        &user.access_token,
        None,
    );

    let page2_res = app.clone().oneshot(page2_req).await.unwrap();

    assert_eq!(page2_res.status(), StatusCode::OK);
    let body = body::to_bytes(page2_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let page2_result: Value = serde_json::from_slice(&body).unwrap();

    // 2ページ目の検証
    let page2_tasks = page2_result["items"].as_array().unwrap();
    assert_eq!(page2_tasks.len(), 3);

    // ページネーション情報の検証
    let pagination = &page2_result["pagination"];
    assert_eq!(pagination["page"], 2);
    assert_eq!(pagination["per_page"], 3);
    assert_eq!(pagination["total_count"], 8);
    assert_eq!(pagination["has_next"], true);
    assert_eq!(pagination["has_prev"], true);
}

#[tokio::test]
async fn test_paginated_tasks_last_page() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // ページネーション用のタスクを作成（7個）
    for i in 1..=7 {
        let task_data = test_data::create_test_task_with_title(&format!("Last Page Task {}", i));
        let create_req = auth_helper::create_authenticated_request(
            "POST",
            "/tasks",
            &user.access_token,
            Some(serde_json::to_string(&task_data).unwrap()),
        );

        let create_res = app.clone().oneshot(create_req).await.unwrap();
        assert_eq!(create_res.status(), StatusCode::CREATED);
    }

    // 最後のページを取得（ページサイズ3、3ページ目）
    let last_page_req = auth_helper::create_authenticated_request(
        "GET",
        "/tasks/paginated?page=3&page_size=3",
        &user.access_token,
        None,
    );

    let last_page_res = app.clone().oneshot(last_page_req).await.unwrap();

    assert_eq!(last_page_res.status(), StatusCode::OK);
    let body = body::to_bytes(last_page_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let last_page_result: Value = serde_json::from_slice(&body).unwrap();

    // 最後のページの検証（1個のタスクのみ）
    let last_page_tasks = last_page_result["items"].as_array().unwrap();
    assert_eq!(last_page_tasks.len(), 1);

    // ページネーション情報の検証
    let pagination = &last_page_result["pagination"];
    assert_eq!(pagination["page"], 3);
    assert_eq!(pagination["per_page"], 3);
    assert_eq!(pagination["total_count"], 7);
    assert_eq!(pagination["has_next"], false);
    assert_eq!(pagination["has_prev"], true);
}

#[tokio::test]
async fn test_paginated_tasks_user_isolation() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // 複数ユーザーを作成
    let users = auth_helper::setup_multiple_users(&app, 2).await.unwrap();
    let user1 = &users[0];
    let user2 = &users[1];

    // user1で3つのタスクを作成
    for i in 1..=3 {
        let task_data =
            test_data::create_test_task_with_title(&format!("User1 Paginated Task {}", i));
        let create_req = auth_helper::create_authenticated_request(
            "POST",
            "/tasks",
            &user1.access_token,
            Some(serde_json::to_string(&task_data).unwrap()),
        );

        let create_res = app.clone().oneshot(create_req).await.unwrap();
        assert_eq!(create_res.status(), StatusCode::CREATED);
    }

    // user2で5つのタスクを作成
    for i in 1..=5 {
        let task_data =
            test_data::create_test_task_with_title(&format!("User2 Paginated Task {}", i));
        let create_req = auth_helper::create_authenticated_request(
            "POST",
            "/tasks",
            &user2.access_token,
            Some(serde_json::to_string(&task_data).unwrap()),
        );

        let create_res = app.clone().oneshot(create_req).await.unwrap();
        assert_eq!(create_res.status(), StatusCode::CREATED);
    }

    // user1でページネーション取得
    let user1_req = auth_helper::create_authenticated_request(
        "GET",
        "/tasks/paginated?page=1&page_size=10",
        &user1.access_token,
        None,
    );

    let user1_res = app.clone().oneshot(user1_req).await.unwrap();
    assert_eq!(user1_res.status(), StatusCode::OK);

    let body1 = body::to_bytes(user1_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let user1_result: Value = serde_json::from_slice(&body1).unwrap();

    // user1は自分のタスク3つのみが見える
    let user1_tasks = user1_result["items"].as_array().unwrap();
    assert_eq!(user1_tasks.len(), 3);
    assert_eq!(user1_result["pagination"]["total_count"], 3);

    for task in user1_tasks {
        assert_eq!(task["user_id"], user1.id.to_string());
    }

    // user2でページネーション取得
    let user2_req = auth_helper::create_authenticated_request(
        "GET",
        "/tasks/paginated?page=1&page_size=10",
        &user2.access_token,
        None,
    );

    let user2_res = app.clone().oneshot(user2_req).await.unwrap();
    assert_eq!(user2_res.status(), StatusCode::OK);

    let body2 = body::to_bytes(user2_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let user2_result: Value = serde_json::from_slice(&body2).unwrap();

    // user2は自分のタスク5つのみが見える
    let user2_tasks = user2_result["items"].as_array().unwrap();
    assert_eq!(user2_tasks.len(), 5);
    assert_eq!(user2_result["pagination"]["total_count"], 5);

    for task in user2_tasks {
        assert_eq!(task["user_id"], user2.id.to_string());
    }
}

#[tokio::test]
async fn test_paginated_tasks_without_authentication() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // 認証なしでページネーション取得試行
    let req = Request::builder()
        .uri("/tasks/paginated?page=1&page_size=5")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let error: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(error["error_type"], "unauthorized");
}

#[tokio::test]
async fn test_paginated_tasks_invalid_page_number() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // 1つのタスクを作成
    let task_data = test_data::create_test_task();
    let create_req = auth_helper::create_authenticated_request(
        "POST",
        "/tasks",
        &user.access_token,
        Some(serde_json::to_string(&task_data).unwrap()),
    );

    let create_res = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(create_res.status(), StatusCode::CREATED);

    // 存在しないページ番号でアクセス
    let invalid_page_req = auth_helper::create_authenticated_request(
        "GET",
        "/tasks/paginated?page=10&page_size=5",
        &user.access_token,
        None,
    );

    let invalid_page_res = app.clone().oneshot(invalid_page_req).await.unwrap();

    assert_eq!(invalid_page_res.status(), StatusCode::OK);
    let body = body::to_bytes(invalid_page_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();

    // 存在しないページでは空の配列が返される
    let tasks = result["items"].as_array().unwrap();
    assert!(tasks.is_empty());

    let pagination = &result["pagination"];
    assert_eq!(pagination["page"], 10);
    assert_eq!(pagination["total_count"], 1);
    assert_eq!(pagination["has_next"], false);
    assert_eq!(pagination["has_prev"], true);
}

#[tokio::test]
async fn test_paginated_tasks_invalid_page_size() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // 無効なページサイズ（0またはマイナス）
    let invalid_size_req = auth_helper::create_authenticated_request(
        "GET",
        "/tasks/paginated?page=1&page_size=0",
        &user.access_token,
        None,
    );

    let invalid_size_res = app.clone().oneshot(invalid_size_req).await.unwrap();

    // 無効なページサイズではバリデーションエラーまたはデフォルト値適用
    assert!(
        invalid_size_res.status() == StatusCode::BAD_REQUEST
            || invalid_size_res.status() == StatusCode::OK
    );
}

#[tokio::test]
async fn test_paginated_tasks_large_page_size() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // 3つのタスクを作成
    for i in 1..=3 {
        let task_data = test_data::create_test_task_with_title(&format!("Large Size Task {}", i));
        let create_req = auth_helper::create_authenticated_request(
            "POST",
            "/tasks",
            &user.access_token,
            Some(serde_json::to_string(&task_data).unwrap()),
        );

        let create_res = app.clone().oneshot(create_req).await.unwrap();
        assert_eq!(create_res.status(), StatusCode::CREATED);
    }

    // 非常に大きなページサイズで取得
    let large_size_req = auth_helper::create_authenticated_request(
        "GET",
        "/tasks/paginated?page=1&page_size=1000",
        &user.access_token,
        None,
    );

    let large_size_res = app.clone().oneshot(large_size_req).await.unwrap();

    assert_eq!(large_size_res.status(), StatusCode::OK);
    let body = body::to_bytes(large_size_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();

    // 実際のタスク数（3つ）が返される
    let tasks = result["items"].as_array().unwrap();
    assert_eq!(tasks.len(), 3);

    let pagination = &result["pagination"];
    assert_eq!(pagination["total_count"], 3);
    assert_eq!(pagination["page"], 1);
    assert_eq!(pagination["has_next"], false);
}

#[tokio::test]
async fn test_paginated_tasks_default_parameters() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // 複数のタスクを作成
    for i in 1..=15 {
        let task_data =
            test_data::create_test_task_with_title(&format!("Default Param Task {}", i));
        let create_req = auth_helper::create_authenticated_request(
            "POST",
            "/tasks",
            &user.access_token,
            Some(serde_json::to_string(&task_data).unwrap()),
        );

        let create_res = app.clone().oneshot(create_req).await.unwrap();
        assert_eq!(create_res.status(), StatusCode::CREATED);
    }

    // パラメータなしでページネーション取得
    let default_req = auth_helper::create_authenticated_request(
        "GET",
        "/tasks/paginated",
        &user.access_token,
        None,
    );

    let default_res = app.clone().oneshot(default_req).await.unwrap();

    assert_eq!(default_res.status(), StatusCode::OK);
    let body = body::to_bytes(default_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();

    // デフォルト値が適用されることを確認
    let pagination = &result["pagination"];
    assert_eq!(pagination["page"], 1);
    assert!(pagination["per_page"].as_i64().unwrap() > 0);
    assert_eq!(pagination["total_count"], 15);

    let tasks = result["items"].as_array().unwrap();
    assert!(!tasks.is_empty());
    assert!(tasks.len() <= pagination["per_page"].as_i64().unwrap() as usize);
}

#[tokio::test]
async fn test_paginated_tasks_empty_result() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // ユーザー登録とログイン（タスクは作成しない）
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // タスクなしでページネーション取得
    let empty_req = auth_helper::create_authenticated_request(
        "GET",
        "/tasks/paginated?page=1&page_size=10",
        &user.access_token,
        None,
    );

    let empty_res = app.clone().oneshot(empty_req).await.unwrap();

    assert_eq!(empty_res.status(), StatusCode::OK);
    let body = body::to_bytes(empty_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();

    // 空の結果
    let tasks = result["items"].as_array().unwrap();
    assert!(tasks.is_empty());

    let pagination = &result["pagination"];
    assert_eq!(pagination["page"], 1);
    assert_eq!(pagination["per_page"], 10);
    assert_eq!(pagination["total_count"], 0);
    assert_eq!(pagination["total_pages"], 0);
    assert_eq!(pagination["has_next"], false);
    assert_eq!(pagination["has_prev"], false);
}

#[tokio::test]
async fn test_paginated_tasks_sorting_order() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // 順番がわかるようにタスクを作成
    for i in 1..=5 {
        let task_data = test_data::create_test_task_with_title(&format!("Sort Test Task {:02}", i));
        let create_req = auth_helper::create_authenticated_request(
            "POST",
            "/tasks",
            &user.access_token,
            Some(serde_json::to_string(&task_data).unwrap()),
        );

        let create_res = app.clone().oneshot(create_req).await.unwrap();
        assert_eq!(create_res.status(), StatusCode::CREATED);

        // 少し待機して作成時間に差をつける
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    // ページネーション取得
    let sort_req = auth_helper::create_authenticated_request(
        "GET",
        "/tasks/paginated?page=1&page_size=5",
        &user.access_token,
        None,
    );

    let sort_res = app.clone().oneshot(sort_req).await.unwrap();

    assert_eq!(sort_res.status(), StatusCode::OK);
    let body = body::to_bytes(sort_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();

    let tasks = result["items"].as_array().unwrap();
    assert_eq!(tasks.len(), 5);

    // ソート順序の確認（実装によって昇順・降順が異なる）
    // ここでは一貫性があることを確認
    let first_task_time = tasks[0]["created_at"].as_str().unwrap();
    let last_task_time = tasks[4]["created_at"].as_str().unwrap();

    // 時間文字列として比較（ISO 8601 フォーマットの場合）
    assert!(first_task_time != last_task_time);
}
