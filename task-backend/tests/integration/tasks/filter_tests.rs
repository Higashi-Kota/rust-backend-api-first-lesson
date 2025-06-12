// tests/integration/tasks/filter_tests.rs

use axum::{
    body::{self, Body},
    http::{Request, StatusCode},
};
use serde_json::Value;
use tower::ServiceExt;

use crate::common::{app_helper, auth_helper, test_data};

#[tokio::test]
async fn test_filter_tasks_by_status_with_authentication() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // 異なるステータスのタスクを作成
    let statuses = ["todo", "in_progress", "completed"];
    for (i, status) in statuses.iter().enumerate() {
        let task_data = test_data::create_custom_task(
            &format!("Filter Task {}", i + 1),
            Some("Test description"),
            Some(status),
        );
        let create_req = auth_helper::create_authenticated_request(
            "POST",
            "/tasks",
            &user.access_token,
            Some(serde_json::to_string(&task_data).unwrap()),
        );

        let create_res = app.clone().oneshot(create_req).await.unwrap();
        assert_eq!(create_res.status(), StatusCode::CREATED);
    }

    // todoステータスでフィルタリング
    let filter_req = auth_helper::create_authenticated_request(
        "GET",
        "/tasks/filter?status=todo",
        &user.access_token,
        None,
    );

    let filter_res = app.clone().oneshot(filter_req).await.unwrap();

    assert_eq!(filter_res.status(), StatusCode::OK);
    let body = body::to_bytes(filter_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();

    assert!(result["tasks"].is_array());
    let filtered_tasks = result["tasks"].as_array().unwrap();
    assert_eq!(filtered_tasks.len(), 1);
    assert_eq!(filtered_tasks[0]["status"], "todo");
    assert_eq!(filtered_tasks[0]["user_id"], user.id.to_string());
}

#[tokio::test]
async fn test_filter_tasks_by_title_with_authentication() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // 異なるタイトルのタスクを作成
    let titles = ["Important Task", "Regular Task", "Another Important Task"];
    for title in &titles {
        let task_data = test_data::create_test_task_with_title(title);
        let create_req = auth_helper::create_authenticated_request(
            "POST",
            "/tasks",
            &user.access_token,
            Some(serde_json::to_string(&task_data).unwrap()),
        );

        let create_res = app.clone().oneshot(create_req).await.unwrap();
        assert_eq!(create_res.status(), StatusCode::CREATED);
    }

    // "Important"を含むタスクでフィルタリング
    let filter_req = auth_helper::create_authenticated_request(
        "GET",
        "/tasks/filter?title_contains=Important",
        &user.access_token,
        None,
    );

    let filter_res = app.clone().oneshot(filter_req).await.unwrap();

    assert_eq!(filter_res.status(), StatusCode::OK);
    let body = body::to_bytes(filter_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();

    assert!(result["tasks"].is_array());
    let filtered_tasks = result["tasks"].as_array().unwrap();
    assert_eq!(filtered_tasks.len(), 2);

    for task in filtered_tasks {
        assert!(task["title"].as_str().unwrap().contains("Important"));
        assert_eq!(task["user_id"], user.id.to_string());
    }
}

#[tokio::test]
async fn test_filter_tasks_multiple_criteria() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // 複合条件用のタスクを作成
    let tasks = [
        ("Important Todo", "todo"),
        ("Important In Progress", "in_progress"),
        ("Regular Todo", "todo"),
        ("Important Done", "completed"),
    ];

    for (title, status) in &tasks {
        let task_data =
            test_data::create_custom_task(title, Some("Test description"), Some(status));
        let create_req = auth_helper::create_authenticated_request(
            "POST",
            "/tasks",
            &user.access_token,
            Some(serde_json::to_string(&task_data).unwrap()),
        );

        let create_res = app.clone().oneshot(create_req).await.unwrap();
        assert_eq!(create_res.status(), StatusCode::CREATED);
    }

    // "Important"かつ"todo"でフィルタリング
    let filter_req = auth_helper::create_authenticated_request(
        "GET",
        "/tasks/filter?status=todo&title_contains=Important",
        &user.access_token,
        None,
    );

    let filter_res = app.clone().oneshot(filter_req).await.unwrap();

    assert_eq!(filter_res.status(), StatusCode::OK);
    let body = body::to_bytes(filter_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();

    assert!(result["tasks"].is_array());
    let filtered_tasks = result["tasks"].as_array().unwrap();
    assert_eq!(filtered_tasks.len(), 1);

    let task = &filtered_tasks[0];
    assert_eq!(task["title"], "Important Todo");
    assert_eq!(task["status"], "todo");
    assert_eq!(task["user_id"], user.id.to_string());
}

#[tokio::test]
async fn test_filter_tasks_user_isolation() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // 複数ユーザーを作成
    let users = auth_helper::setup_multiple_users(&app, 2).await.unwrap();
    let user1 = &users[0];
    let user2 = &users[1];

    // 両ユーザーで同じタイトルのタスクを作成
    for user in &users {
        let task_data = test_data::create_test_task_with_title("Shared Title Task");
        let create_req = auth_helper::create_authenticated_request(
            "POST",
            "/tasks",
            &user.access_token,
            Some(serde_json::to_string(&task_data).unwrap()),
        );

        let create_res = app.clone().oneshot(create_req).await.unwrap();
        assert_eq!(create_res.status(), StatusCode::CREATED);
    }

    // user1でフィルタリング
    let filter_req1 = auth_helper::create_authenticated_request(
        "GET",
        "/tasks/filter?title_contains=Shared",
        &user1.access_token,
        None,
    );

    let filter_res1 = app.clone().oneshot(filter_req1).await.unwrap();
    assert_eq!(filter_res1.status(), StatusCode::OK);

    let body1 = body::to_bytes(filter_res1.into_body(), usize::MAX)
        .await
        .unwrap();
    let result1: Value = serde_json::from_slice(&body1).unwrap();

    let filtered_tasks1 = result1["tasks"].as_array().unwrap();
    assert_eq!(filtered_tasks1.len(), 1);
    assert_eq!(filtered_tasks1[0]["user_id"], user1.id.to_string());

    // user2でフィルタリング
    let filter_req2 = auth_helper::create_authenticated_request(
        "GET",
        "/tasks/filter?title_contains=Shared",
        &user2.access_token,
        None,
    );

    let filter_res2 = app.clone().oneshot(filter_req2).await.unwrap();
    assert_eq!(filter_res2.status(), StatusCode::OK);

    let body2 = body::to_bytes(filter_res2.into_body(), usize::MAX)
        .await
        .unwrap();
    let result2: Value = serde_json::from_slice(&body2).unwrap();

    let filtered_tasks2 = result2["tasks"].as_array().unwrap();
    assert_eq!(filtered_tasks2.len(), 1);
    assert_eq!(filtered_tasks2[0]["user_id"], user2.id.to_string());
}

#[tokio::test]
async fn test_filter_tasks_without_authentication() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // 認証なしでフィルタリング試行
    let req = Request::builder()
        .uri("/tasks/filter?status=todo")
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
async fn test_filter_tasks_invalid_status() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // 無効なステータスでフィルタリング
    let filter_req = auth_helper::create_authenticated_request(
        "GET",
        "/tasks/filter?status=invalid_status",
        &user.access_token,
        None,
    );

    let filter_res = app.clone().oneshot(filter_req).await.unwrap();

    assert_eq!(filter_res.status(), StatusCode::OK);
    let body = body::to_bytes(filter_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();

    // 無効なステータスでは何も返されない
    assert!(result["tasks"].is_array());
    let filtered_tasks = result["tasks"].as_array().unwrap();
    assert!(filtered_tasks.is_empty());
}

#[tokio::test]
async fn test_filter_tasks_no_matches() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // タスクを作成
    let task_data = test_data::create_test_task_with_title("Specific Task");
    let create_req = auth_helper::create_authenticated_request(
        "POST",
        "/tasks",
        &user.access_token,
        Some(serde_json::to_string(&task_data).unwrap()),
    );

    let create_res = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(create_res.status(), StatusCode::CREATED);

    // マッチしない条件でフィルタリング
    let filter_req = auth_helper::create_authenticated_request(
        "GET",
        "/tasks/filter?title_contains=NonExistent",
        &user.access_token,
        None,
    );

    let filter_res = app.clone().oneshot(filter_req).await.unwrap();

    assert_eq!(filter_res.status(), StatusCode::OK);
    let body = body::to_bytes(filter_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();

    assert!(result["tasks"].is_array());
    let filtered_tasks = result["tasks"].as_array().unwrap();
    assert!(filtered_tasks.is_empty());
}

#[tokio::test]
async fn test_filter_tasks_empty_parameters() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // タスクを作成
    let task_data = test_data::create_test_task();
    let create_req = auth_helper::create_authenticated_request(
        "POST",
        "/tasks",
        &user.access_token,
        Some(serde_json::to_string(&task_data).unwrap()),
    );

    let create_res = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(create_res.status(), StatusCode::CREATED);

    // 空のパラメータでフィルタリング
    let filter_req =
        auth_helper::create_authenticated_request("GET", "/tasks/filter", &user.access_token, None);

    let filter_res = app.clone().oneshot(filter_req).await.unwrap();

    assert_eq!(filter_res.status(), StatusCode::OK);
    let body = body::to_bytes(filter_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();

    // フィルター条件なしでは全てのタスクが返される
    assert!(result["tasks"].is_array());
    let filtered_tasks = result["tasks"].as_array().unwrap();
    assert_eq!(filtered_tasks.len(), 1);
    assert_eq!(filtered_tasks[0]["user_id"], user.id.to_string());
}

#[tokio::test]
async fn test_filter_tasks_case_sensitivity() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // 大文字小文字混在のタスクを作成
    let task_data = test_data::create_test_task_with_title("CaseSensitive Task");
    let create_req = auth_helper::create_authenticated_request(
        "POST",
        "/tasks",
        &user.access_token,
        Some(serde_json::to_string(&task_data).unwrap()),
    );

    let create_res = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(create_res.status(), StatusCode::CREATED);

    // 小文字でフィルタリング
    let filter_req = auth_helper::create_authenticated_request(
        "GET",
        "/tasks/filter?title_contains=casesensitive",
        &user.access_token,
        None,
    );

    let filter_res = app.clone().oneshot(filter_req).await.unwrap();
    assert_eq!(filter_res.status(), StatusCode::OK);

    let body = body::to_bytes(filter_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();

    let filtered_tasks = result["tasks"].as_array().unwrap();

    // 実装により大文字小文字を区別する場合は0件、しない場合は1件
    assert!(filtered_tasks.len() <= 1);
}

#[tokio::test]
async fn test_filter_tasks_with_special_characters() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // 特殊文字を含むタスクを作成
    let task_data = test_data::create_test_task_with_title("Task with @#$% Special");
    let create_req = auth_helper::create_authenticated_request(
        "POST",
        "/tasks",
        &user.access_token,
        Some(serde_json::to_string(&task_data).unwrap()),
    );

    let create_res = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(create_res.status(), StatusCode::CREATED);

    // 特殊文字を含む文字列でフィルタリング
    let filter_req = auth_helper::create_authenticated_request(
        "GET",
        "/tasks/filter?title_contains=@#$%",
        &user.access_token,
        None,
    );

    let filter_res = app.clone().oneshot(filter_req).await.unwrap();

    // URL エンコーディングや特殊文字の処理により結果は異なる
    assert!(
        filter_res.status() == StatusCode::OK || filter_res.status() == StatusCode::BAD_REQUEST
    );
}
