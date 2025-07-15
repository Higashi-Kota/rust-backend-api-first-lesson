// tests/integration/tasks/crud_tests.rs

use axum::{
    body::{self, Body},
    http::{Request, StatusCode},
};
use serde_json::Value;
use tower::ServiceExt;

use crate::common::{app_helper, auth_helper, test_data};

#[tokio::test]
async fn test_create_task_with_authentication() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // タスク作成
    let task_data = test_data::create_test_task();

    let req = auth_helper::create_authenticated_request(
        "POST",
        "/tasks",
        &user.access_token,
        Some(serde_json::to_string(&task_data).unwrap()),
    );

    let res = app.clone().oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::CREATED);
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let task: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(task["data"]["title"], "Test Task");
    assert_eq!(task["data"]["status"], "todo");
    assert!(task["data"]["id"].is_string());
    assert!(task["data"]["user_id"].is_string());
    assert_eq!(task["data"]["user_id"], user.id.to_string());
}

#[tokio::test]
async fn test_create_task_without_authentication() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // 認証なしでタスク作成試行
    let task_data = test_data::create_test_task();

    let req = Request::builder()
        .uri("/tasks")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&task_data).unwrap()))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let error: Value = serde_json::from_slice(&body).unwrap();

    // Error response format
    assert!(!error["success"].as_bool().unwrap());
    assert!(error["data"].is_null());
    assert!(error["error"].is_object());
    assert_eq!(error["error"]["code"], "UNAUTHORIZED");
}

#[tokio::test]
async fn test_get_task_with_authentication() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // タスク作成
    let task_data = test_data::create_test_task();
    let create_req = auth_helper::create_authenticated_request(
        "POST",
        "/tasks",
        &user.access_token,
        Some(serde_json::to_string(&task_data).unwrap()),
    );

    let create_res = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(create_res.status(), StatusCode::CREATED);

    let create_body = body::to_bytes(create_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let created_task: Value = serde_json::from_slice(&create_body).unwrap();
    let task_id = created_task["data"]["id"].as_str().unwrap();

    // タスク取得
    let get_req = auth_helper::create_authenticated_request(
        "GET",
        &format!("/tasks/{}", task_id),
        &user.access_token,
        None,
    );

    let get_res = app.clone().oneshot(get_req).await.unwrap();

    assert_eq!(get_res.status(), StatusCode::OK);
    let body = body::to_bytes(get_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let task: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(task["data"]["id"], task_id);
    assert_eq!(task["data"]["title"], "Test Task");
    assert_eq!(task["data"]["user_id"], user.id.to_string());
}

#[tokio::test]
async fn test_get_task_of_another_user() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // 複数ユーザーを作成
    let users = auth_helper::setup_multiple_users(&app, 2).await.unwrap();
    let user1 = &users[0];
    let user2 = &users[1];

    // user1でタスク作成
    let task_data = test_data::create_test_task();
    let create_req = auth_helper::create_authenticated_request(
        "POST",
        "/tasks",
        &user1.access_token,
        Some(serde_json::to_string(&task_data).unwrap()),
    );

    let create_res = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(create_res.status(), StatusCode::CREATED);

    let create_body = body::to_bytes(create_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let created_task: Value = serde_json::from_slice(&create_body).unwrap();
    let task_id = created_task["data"]["id"].as_str().unwrap();

    // user2でタスク取得試行
    let get_req = auth_helper::create_authenticated_request(
        "GET",
        &format!("/tasks/{}", task_id),
        &user2.access_token,
        None,
    );

    let get_res = app.clone().oneshot(get_req).await.unwrap();

    // 他のユーザーのタスクにはアクセスできない
    assert_eq!(get_res.status(), StatusCode::NOT_FOUND);
    let body = body::to_bytes(get_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let error: Value = serde_json::from_slice(&body).unwrap();

    // Error response format
    assert!(!error["success"].as_bool().unwrap());
    assert!(error["data"].is_null());
    assert!(error["error"].is_object());
    assert_eq!(error["error"]["code"], "NOT_FOUND");
}

#[tokio::test]
async fn test_list_tasks_with_authentication() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // 複数のタスクを作成
    for i in 1..=3 {
        let task_data = test_data::create_test_task_with_title(&format!("Auth Task {}", i));
        let create_req = auth_helper::create_authenticated_request(
            "POST",
            "/tasks",
            &user.access_token,
            Some(serde_json::to_string(&task_data).unwrap()),
        );

        let create_res = app.clone().oneshot(create_req).await.unwrap();
        assert_eq!(create_res.status(), StatusCode::CREATED);
    }

    // タスク一覧取得
    let list_req =
        auth_helper::create_authenticated_request("GET", "/tasks", &user.access_token, None);

    let list_res = app.clone().oneshot(list_req).await.unwrap();

    assert_eq!(list_res.status(), StatusCode::OK);
    let body = body::to_bytes(list_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let tasks: Value = serde_json::from_slice(&body).unwrap();

    // 自分が作成したタスクのみが返される
    assert!(tasks["data"].is_array());
    let task_array = tasks["data"].as_array().unwrap();
    assert_eq!(task_array.len(), 3);

    // 全てのタスクが自分のものであることを確認
    for task in task_array {
        assert_eq!(task["user_id"], user.id.to_string());
    }
}

#[tokio::test]
async fn test_list_tasks_user_isolation() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // 複数ユーザーを作成
    let users = auth_helper::setup_multiple_users(&app, 2).await.unwrap();
    let user1 = &users[0];
    let user2 = &users[1];

    // user1で2つのタスクを作成
    for i in 1..=2 {
        let task_data = test_data::create_test_task_with_title(&format!("User1 Task {}", i));
        let create_req = auth_helper::create_authenticated_request(
            "POST",
            "/tasks",
            &user1.access_token,
            Some(serde_json::to_string(&task_data).unwrap()),
        );

        let create_res = app.clone().oneshot(create_req).await.unwrap();
        assert_eq!(create_res.status(), StatusCode::CREATED);
    }

    // user2で3つのタスクを作成
    for i in 1..=3 {
        let task_data = test_data::create_test_task_with_title(&format!("User2 Task {}", i));
        let create_req = auth_helper::create_authenticated_request(
            "POST",
            "/tasks",
            &user2.access_token,
            Some(serde_json::to_string(&task_data).unwrap()),
        );

        let create_res = app.clone().oneshot(create_req).await.unwrap();
        assert_eq!(create_res.status(), StatusCode::CREATED);
    }

    // user1でタスク一覧取得
    let list_req1 =
        auth_helper::create_authenticated_request("GET", "/tasks", &user1.access_token, None);

    let list_res1 = app.clone().oneshot(list_req1).await.unwrap();
    assert_eq!(list_res1.status(), StatusCode::OK);

    let body1 = body::to_bytes(list_res1.into_body(), usize::MAX)
        .await
        .unwrap();
    let tasks1: Value = serde_json::from_slice(&body1).unwrap();

    // user1は自分のタスク2つのみが見える
    assert_eq!(tasks1["data"].as_array().unwrap().len(), 2);
    for task in tasks1["data"].as_array().unwrap() {
        assert_eq!(task["user_id"], user1.id.to_string());
    }

    // user2でタスク一覧取得
    let list_req2 =
        auth_helper::create_authenticated_request("GET", "/tasks", &user2.access_token, None);

    let list_res2 = app.clone().oneshot(list_req2).await.unwrap();
    assert_eq!(list_res2.status(), StatusCode::OK);

    let body2 = body::to_bytes(list_res2.into_body(), usize::MAX)
        .await
        .unwrap();
    let tasks2: Value = serde_json::from_slice(&body2).unwrap();

    // user2は自分のタスク3つのみが見える
    assert_eq!(tasks2["data"].as_array().unwrap().len(), 3);
    for task in tasks2["data"].as_array().unwrap() {
        assert_eq!(task["user_id"], user2.id.to_string());
    }
}

#[tokio::test]
async fn test_update_task_with_authentication() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // タスク作成
    let task_data = test_data::create_test_task();
    let create_req = auth_helper::create_authenticated_request(
        "POST",
        "/tasks",
        &user.access_token,
        Some(serde_json::to_string(&task_data).unwrap()),
    );

    let create_res = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(create_res.status(), StatusCode::CREATED);

    let create_body = body::to_bytes(create_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let created_task: Value = serde_json::from_slice(&create_body).unwrap();
    let task_id = created_task["data"]["id"].as_str().unwrap();

    // タスク更新
    let update_data = test_data::create_partial_update_task_title("Updated Auth Task");
    let update_req = auth_helper::create_authenticated_request(
        "PATCH",
        &format!("/tasks/{}", task_id),
        &user.access_token,
        Some(serde_json::to_string(&update_data).unwrap()),
    );

    let update_res = app.clone().oneshot(update_req).await.unwrap();

    assert_eq!(update_res.status(), StatusCode::OK);
    let body = body::to_bytes(update_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let updated_task: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(updated_task["data"]["id"], task_id);
    assert_eq!(updated_task["data"]["title"], "Updated Auth Task");
    assert_eq!(updated_task["data"]["user_id"], user.id.to_string());
}

#[tokio::test]
async fn test_update_task_of_another_user() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // 複数ユーザーを作成
    let users = auth_helper::setup_multiple_users(&app, 2).await.unwrap();
    let user1 = &users[0];
    let user2 = &users[1];

    // user1でタスク作成
    let task_data = test_data::create_test_task();
    let create_req = auth_helper::create_authenticated_request(
        "POST",
        "/tasks",
        &user1.access_token,
        Some(serde_json::to_string(&task_data).unwrap()),
    );

    let create_res = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(create_res.status(), StatusCode::CREATED);

    let create_body = body::to_bytes(create_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let created_task: Value = serde_json::from_slice(&create_body).unwrap();
    let task_id = created_task["data"]["id"].as_str().unwrap();

    // user2でタスク更新試行
    let update_data = test_data::create_partial_update_task_title("Unauthorized Update");
    let update_req = auth_helper::create_authenticated_request(
        "PATCH",
        &format!("/tasks/{}", task_id),
        &user2.access_token,
        Some(serde_json::to_string(&update_data).unwrap()),
    );

    let update_res = app.clone().oneshot(update_req).await.unwrap();

    // 他のユーザーのタスクは更新できない
    assert_eq!(update_res.status(), StatusCode::NOT_FOUND);
    let body = body::to_bytes(update_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let error: Value = serde_json::from_slice(&body).unwrap();

    // Error response format
    assert!(!error["success"].as_bool().unwrap());
    assert!(error["data"].is_null());
    assert!(error["error"].is_object());
    assert_eq!(error["error"]["code"], "NOT_FOUND");
}

#[tokio::test]
async fn test_delete_task_with_authentication() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // タスク作成
    let task_data = test_data::create_test_task();
    let create_req = auth_helper::create_authenticated_request(
        "POST",
        "/tasks",
        &user.access_token,
        Some(serde_json::to_string(&task_data).unwrap()),
    );

    let create_res = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(create_res.status(), StatusCode::CREATED);

    let create_body = body::to_bytes(create_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let created_task: Value = serde_json::from_slice(&create_body).unwrap();
    let task_id = created_task["data"]["id"].as_str().unwrap();

    // タスク削除
    let delete_req = auth_helper::create_authenticated_request(
        "DELETE",
        &format!("/tasks/{}", task_id),
        &user.access_token,
        None,
    );

    let delete_res = app.clone().oneshot(delete_req).await.unwrap();

    assert_eq!(delete_res.status(), StatusCode::NO_CONTENT);

    // 削除されたことを確認
    let get_req = auth_helper::create_authenticated_request(
        "GET",
        &format!("/tasks/{}", task_id),
        &user.access_token,
        None,
    );

    let get_res = app.clone().oneshot(get_req).await.unwrap();
    assert_eq!(get_res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_task_of_another_user() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // 複数ユーザーを作成
    let users = auth_helper::setup_multiple_users(&app, 2).await.unwrap();
    let user1 = &users[0];
    let user2 = &users[1];

    // user1でタスク作成
    let task_data = test_data::create_test_task();
    let create_req = auth_helper::create_authenticated_request(
        "POST",
        "/tasks",
        &user1.access_token,
        Some(serde_json::to_string(&task_data).unwrap()),
    );

    let create_res = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(create_res.status(), StatusCode::CREATED);

    let create_body = body::to_bytes(create_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let created_task: Value = serde_json::from_slice(&create_body).unwrap();
    let task_id = created_task["data"]["id"].as_str().unwrap();

    // user2でタスク削除試行
    let delete_req = auth_helper::create_authenticated_request(
        "DELETE",
        &format!("/tasks/{}", task_id),
        &user2.access_token,
        None,
    );

    let delete_res = app.clone().oneshot(delete_req).await.unwrap();

    // 他のユーザーのタスクは削除できない
    assert_eq!(delete_res.status(), StatusCode::NOT_FOUND);
    let body = body::to_bytes(delete_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let error: Value = serde_json::from_slice(&body).unwrap();

    // Error response format
    assert!(!error["success"].as_bool().unwrap());
    assert!(error["data"].is_null());
    assert!(error["error"].is_object());
    assert_eq!(error["error"]["code"], "NOT_FOUND");

    // 元のユーザーではまだアクセス可能であることを確認
    let get_req = auth_helper::create_authenticated_request(
        "GET",
        &format!("/tasks/{}", task_id),
        &user1.access_token,
        None,
    );

    let get_res = app.clone().oneshot(get_req).await.unwrap();
    assert_eq!(get_res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_task_validation_with_authentication() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // 無効なタスクデータで作成試行
    let invalid_task = test_data::create_invalid_task_empty_title();

    let req = auth_helper::create_authenticated_request(
        "POST",
        "/tasks",
        &user.access_token,
        Some(serde_json::to_string(&invalid_task).unwrap()),
    );

    let res = app.clone().oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let error: Value = serde_json::from_slice(&body).unwrap();

    // Error response format
    assert!(!error["success"].as_bool().unwrap());
    assert!(error["data"].is_null());
    assert!(error["error"].is_object());
    assert!(
        (error["error"]["code"] == "VALIDATION_ERROR"
            || error["error"]["code"] == "VALIDATION_ERRORS")
    );

    // Check validation details if present
    if let Some(details) = error["error"]["details"].as_array() {
        assert!(!details.is_empty());
    }
}

#[tokio::test]
async fn test_task_with_invalid_uuid() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // 無効なUUIDでタスク取得試行
    let req = auth_helper::create_authenticated_request(
        "GET",
        "/tasks/not-a-uuid",
        &user.access_token,
        None,
    );

    let res = app.clone().oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let error: Value = serde_json::from_slice(&body).unwrap();

    // Error response format
    assert!(!error["success"].as_bool().unwrap());
    assert!(error["data"].is_null());
    assert!(error["error"].is_object());
    assert!(
        (error["error"]["code"] == "VALIDATION_ERROR"
            || error["error"]["code"] == "VALIDATION_ERRORS")
    );
}
