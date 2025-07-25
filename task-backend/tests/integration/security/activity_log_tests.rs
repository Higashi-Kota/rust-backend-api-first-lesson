use crate::common::{
    app_helper::setup_full_app,
    auth_helper::{create_and_authenticate_user, create_authenticated_request},
};
use axum::http::StatusCode;
use serde_json::json;
use serde_json::Value;
use tower::ServiceExt;

// ヘルパー関数：認証済みリクエストを作成（activity_log_testsのローカル版）
fn create_request(
    method: &str,
    uri: &str,
    body_or_token: Option<String>,
) -> axum::http::Request<axum::body::Body> {
    use axum::body::Body;
    use axum::http::Request;

    let mut request_builder = Request::builder()
        .uri(uri)
        .method(method)
        .header("Content-Type", "application/json");

    // Check if it's a token (for GET requests) or body (for POST requests)
    if method == "GET" && body_or_token.is_some() {
        request_builder = request_builder.header(
            "Authorization",
            format!("Bearer {}", body_or_token.unwrap()),
        );
        request_builder.body(Body::empty()).unwrap()
    } else {
        match body_or_token {
            Some(body_content) => request_builder.body(Body::from(body_content)).unwrap(),
            None => request_builder.body(Body::empty()).unwrap(),
        }
    }
}

/// アクティビティログが正しく記録されることを確認
#[tokio::test]
async fn test_activity_log_recording() {
    let (app, _schema, _db) = setup_full_app().await;

    // テストユーザーを作成
    let user = create_and_authenticate_user(&app).await;

    // タスクを作成（これがアクティビティログに記録される）
    let task_data = json!({
        "title": "Test Task",
        "description": "This is a test task",
        "status": "todo"
    });

    let response = app
        .clone()
        .oneshot(create_authenticated_request(
            "POST",
            "/tasks",
            &user.access_token,
            Some(serde_json::to_string(&task_data).unwrap()),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    // アクティビティログが非同期で記録されるため、少し待機
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // 自分のアクティビティログを取得
    let response = app
        .clone()
        .oneshot(create_request(
            "GET",
            "/activity-logs/me",
            Some(user.access_token.clone()),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();

    // ログが記録されていることを確認
    assert!(body["total"].as_u64().unwrap() >= 1);
    assert!(!body["logs"].as_array().unwrap().is_empty());

    // 最新のログを確認
    let latest_log = &body["logs"][0];
    assert_eq!(latest_log["user_id"], user.id.to_string());
    assert_eq!(latest_log["action"], "create_task");
    assert_eq!(latest_log["resource_type"], "task");
}

/// ユーザーは自分のログのみ取得できることを確認
#[tokio::test]
async fn test_user_can_only_see_own_logs() {
    let (app, _schema, _db) = setup_full_app().await;

    // 2人のユーザーを作成
    let user1 = create_and_authenticate_user(&app).await;
    let user2 = create_and_authenticate_user(&app).await;

    // User1でタスクを作成
    let task_data = json!({
        "title": "User1 Task",
        "description": "Task by user1",
        "status": "todo"
    });

    let response = app
        .clone()
        .oneshot(create_authenticated_request(
            "POST",
            "/tasks",
            &user1.access_token,
            Some(serde_json::to_string(&task_data).unwrap()),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    // User2が自分のログを取得
    let response = app
        .clone()
        .oneshot(create_request(
            "GET",
            "/activity-logs/me",
            Some(user2.access_token.clone()),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();

    // User2のログにはUser1のアクティビティが含まれていないことを確認
    let logs = body["logs"].as_array().unwrap();
    for log in logs {
        assert_eq!(log["user_id"], user2.id.to_string());
    }
}

/// 管理者は全ユーザーのログを取得できることを確認
#[tokio::test]
async fn test_admin_can_see_all_logs() {
    let (app, _schema, _db) = setup_full_app().await;

    // 一般ユーザーを作成
    let user = create_and_authenticate_user(&app).await;

    // 管理者を作成（既存の管理者でサインイン）
    let admin_signin = json!({
        "identifier": "admin@example.com",
        "password": "Adm1n$ecurE2024!"
    });

    let response = app
        .clone()
        .oneshot(create_request(
            "POST",
            "/auth/signin",
            Some(serde_json::to_string(&admin_signin).unwrap()),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let signin_response: Value = serde_json::from_slice(&body_bytes).unwrap();
    let admin_token = signin_response["data"]["tokens"]["access_token"]
        .as_str()
        .unwrap();

    // アクティビティログが非同期で記録されるため、少し待機
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // ユーザーでタスクを作成
    let task_data = json!({
        "title": "User Task",
        "description": "Task by regular user",
        "status": "todo"
    });

    let response = app
        .clone()
        .oneshot(create_authenticated_request(
            "POST",
            "/tasks",
            &user.access_token,
            Some(serde_json::to_string(&task_data).unwrap()),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    // アクティビティログが非同期で記録されるため、少し待機
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    // 管理者が全ログを取得（ページサイズを大きくして全てのログを取得）
    let response = app
        .clone()
        .oneshot(create_request(
            "GET",
            "/admin/activity-logs?per_page=100",
            Some(admin_token.to_string()),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();

    // レスポンスはActivityLogResponseの構造（logs, total, page, per_page）
    // 一般ユーザーのログが含まれていることを確認
    let logs = body["logs"].as_array().unwrap();
    let user_log_found = logs.iter().any(|log| log["user_id"] == user.id.to_string());
    assert!(
        user_log_found,
        "User log not found in response. Available logs: {:?}",
        logs
    );
}

/// 一般ユーザーは管理者用エンドポイントにアクセスできないことを確認
#[tokio::test]
async fn test_regular_user_cannot_access_admin_logs() {
    let (app, _schema, _db) = setup_full_app().await;

    // 一般ユーザーを作成
    let user = create_and_authenticate_user(&app).await;

    // 管理者用エンドポイントにアクセス
    let response = app
        .clone()
        .oneshot(create_request(
            "GET",
            "/admin/activity-logs",
            Some(user.access_token.clone()),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

/// フィルタリング機能のテスト
#[tokio::test]
async fn test_activity_log_filtering() {
    let (app, _schema, _db) = setup_full_app().await;

    // 管理者でサインイン
    let admin_signin = json!({
        "identifier": "admin@example.com",
        "password": "Adm1n$ecurE2024!"
    });

    let response = app
        .clone()
        .oneshot(create_request(
            "POST",
            "/auth/signin",
            Some(serde_json::to_string(&admin_signin).unwrap()),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let signin_response: Value = serde_json::from_slice(&body_bytes).unwrap();
    let admin_token = signin_response["data"]["tokens"]["access_token"]
        .as_str()
        .unwrap();

    // アクティビティログが非同期で記録されるため、少し待機
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // 複数のアクティビティを生成
    // 1. タスク作成
    let task_data = json!({
        "title": "Test Task",
        "description": "Test",
        "status": "todo"
    });

    let response = app
        .clone()
        .oneshot(create_authenticated_request(
            "POST",
            "/tasks",
            admin_token,
            Some(serde_json::to_string(&task_data).unwrap()),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let task_response: Value = serde_json::from_slice(&body_bytes).unwrap();
    let task_id = task_response["data"]["id"].as_str().unwrap();

    // 2. タスク更新
    let update_data = json!({
        "title": "Updated Task",
        "status": "in_progress"
    });

    let response = app
        .clone()
        .oneshot(create_authenticated_request(
            "PATCH",
            &format!("/tasks/{}", task_id),
            admin_token,
            Some(serde_json::to_string(&update_data).unwrap()),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // resource_typeでフィルタリング
    let response = app
        .clone()
        .oneshot(create_request(
            "GET",
            "/admin/activity-logs?resource_type=task",
            Some(admin_token.to_string()),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();

    // すべてのログがtaskリソースであることを確認
    let logs = body["logs"].as_array().unwrap();
    for log in logs {
        assert_eq!(log["resource_type"], "task");
    }

    // actionでフィルタリング
    let response = app
        .clone()
        .oneshot(create_request(
            "GET",
            "/admin/activity-logs?action=update_task",
            Some(admin_token.to_string()),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();

    // すべてのログがupdate_taskアクションであることを確認
    let logs = body["logs"].as_array().unwrap();
    for log in logs {
        assert_eq!(log["action"], "update_task");
    }
}
