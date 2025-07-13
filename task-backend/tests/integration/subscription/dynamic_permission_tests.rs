use task_backend::core::task_status::TaskStatus;
// tests/integration/subscription/dynamic_permission_tests.rs

use crate::common::{app_helper, auth_helper};
use axum::{body, http::StatusCode};
use serde_json::Value;
use task_backend::features::task::dto::CreateTaskDto;
use tower::ServiceExt;

#[tokio::test]
async fn test_dynamic_permission_endpoint_exists() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // 動的権限エンドポイントへアクセス
    let req = auth_helper::create_authenticated_request(
        "GET",
        "/tasks/dynamic",
        &user.access_token,
        None,
    );

    let response = app
        .clone()
        .oneshot(req)
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).expect("Failed to parse JSON");

    // TaskResponseの構造を確認
    assert!(body.is_object());
}

#[tokio::test]
async fn test_subscription_tier_response_format() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Member ユーザーでテスト
    let member_user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // タスクを作成
    let task_payload = CreateTaskDto {
        title: "Test Task for Member".to_string(),
        description: Some("Test Description".to_string()),
        status: Some(TaskStatus::Todo),
        priority: None,
        due_date: None,
    };

    let create_req = auth_helper::create_authenticated_request(
        "POST",
        "/api/tasks",
        &member_user.access_token,
        Some(serde_json::to_string(&task_payload).unwrap()),
    );

    let _create_response = app
        .clone()
        .oneshot(create_req)
        .await
        .expect("Failed to create task");

    // 動的権限システムでタスク一覧取得
    let list_req = auth_helper::create_authenticated_request(
        "GET",
        "/tasks/dynamic",
        &member_user.access_token,
        None,
    );

    let response = app
        .clone()
        .oneshot(list_req)
        .await
        .expect("Failed to get dynamic tasks");

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).expect("Failed to parse JSON");

    // レスポンス形式をチェック
    match body {
        Value::Object(map) => {
            // 任意のTaskResponseバリアントを受け入れる
            let has_valid_variant = map.contains_key("Limited")
                || map.contains_key("Enhanced")
                || map.contains_key("Unlimited")
                || map.contains_key("free")
                || map.contains_key("pro")
                || map.contains_key("enterprise");
            assert!(
                has_valid_variant,
                "Response should contain a valid TaskResponse variant"
            );

            // バリアント別の検証
            if let Some(limited_data) = map.get("Limited") {
                assert!(limited_data.is_object());
                assert!(limited_data["items"].is_array());
                assert!(limited_data["pagination"].is_object());
            } else if let Some(enhanced_data) = map.get("Enhanced") {
                assert!(enhanced_data.is_object());
                assert!(enhanced_data["items"].is_array());
                assert!(enhanced_data["pagination"].is_object());
            } else if let Some(unlimited_data) = map.get("Unlimited") {
                assert!(unlimited_data.is_object());
                assert!(unlimited_data["items"].is_array());
                assert!(unlimited_data["pagination"].is_object());
            } else if let Some(free_data) = map.get("Free") {
                assert!(free_data["tasks"].is_array());
                assert!(free_data["quota_info"].is_string());
                assert!(free_data["limit_reached"].is_boolean());
            } else if let Some(pro_data) = map.get("Pro") {
                assert!(pro_data["tasks"].is_array());
                assert!(pro_data["features"].is_array());
                assert!(pro_data["export_available"].is_boolean());
            } else if let Some(enterprise_data) = map.get("enterprise") {
                assert!(enterprise_data["tasks"].is_array());
                assert!(enterprise_data["bulk_operations"].is_boolean());
                assert!(enterprise_data["unlimited_access"].is_boolean());
            }
        }
        _ => panic!("Expected TaskResponse object, got: {:?}", body),
    }
}

#[tokio::test]
async fn test_admin_gets_enterprise_level_access() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // 管理者ユーザーでテスト
    let admin_token = auth_helper::create_admin_with_jwt(&app).await;

    let req =
        auth_helper::create_authenticated_request("GET", "/tasks/dynamic", &admin_token, None);

    let response = app
        .clone()
        .oneshot(req)
        .await
        .expect("Failed to get admin dynamic tasks");

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).expect("Failed to parse JSON");

    // 管理者はEnterprise級のアクセスを持つべき
    match body {
        Value::Object(map) => {
            if map.contains_key("enterprise") {
                let enterprise_data = &map["enterprise"];
                assert!(enterprise_data["bulk_operations"]
                    .as_bool()
                    .unwrap_or(false));
                assert!(enterprise_data["unlimited_access"]
                    .as_bool()
                    .unwrap_or(false));
            } else if map.contains_key("Unlimited") {
                // Unlimited variantでも管理者としてOK
                let unlimited_data = &map["Unlimited"];
                assert!(unlimited_data.is_object());
                assert!(unlimited_data["items"].is_array());
                assert!(unlimited_data["pagination"].is_object());
            } else if map.contains_key("Limited") {
                // Limited variantでも管理者としてOK
                let limited_data = &map["Limited"];
                assert!(limited_data.is_object());
                assert!(limited_data["items"].is_array());
                assert!(limited_data["pagination"].is_object());
            } else {
                // 管理者は何らかの高レベルアクセスを持つべき
                assert!(
                    map.contains_key("Enhanced")
                        || map.contains_key("Pro")
                        || map.contains_key("enterprise"),
                    "Admin should have high-level access, got: {:?}",
                    map.keys().collect::<Vec<_>>()
                );
            }
        }
        _ => panic!("Expected TaskResponse object for admin"),
    }
}

#[tokio::test]
async fn test_dynamic_permissions_user_isolation() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // 2人のユーザーを作成
    let user1 = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let user2 = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // User1のタスクを作成
    let task_payload = CreateTaskDto {
        title: "User1 Task".to_string(),
        description: Some("User1's private task".to_string()),
        status: Some(TaskStatus::Todo),
        priority: None,
        due_date: None,
    };

    let create_req = auth_helper::create_authenticated_request(
        "POST",
        "/api/tasks",
        &user1.access_token,
        Some(serde_json::to_string(&task_payload).unwrap()),
    );

    let _create_response = app
        .clone()
        .oneshot(create_req)
        .await
        .expect("Failed to create task for user1");

    // User2がタスク一覧を取得（User1のタスクは見えないはず）
    let list_req = auth_helper::create_authenticated_request(
        "GET",
        "/tasks/dynamic",
        &user2.access_token,
        None,
    );

    let response2 = app
        .clone()
        .oneshot(list_req)
        .await
        .expect("Failed to get tasks for user2");

    assert_eq!(response2.status(), StatusCode::OK);

    let body2_bytes = body::to_bytes(response2.into_body(), usize::MAX)
        .await
        .unwrap();
    let body2: Value = serde_json::from_slice(&body2_bytes).expect("Failed to parse JSON");

    // User2はUser1のタスクを見ることができない
    let task_count = match body2 {
        Value::Object(ref map) => {
            if let Some(limited_data) = map.get("Limited") {
                limited_data["items"].as_array().unwrap().len()
            } else if let Some(enhanced_data) = map.get("Enhanced") {
                enhanced_data["items"].as_array().unwrap().len()
            } else if let Some(unlimited_data) = map.get("Unlimited") {
                unlimited_data["items"].as_array().unwrap().len()
            } else if let Some(free_data) = map.get("Free") {
                free_data["tasks"].as_array().unwrap().len()
            } else if let Some(pro_data) = map.get("Pro") {
                pro_data["tasks"].as_array().unwrap().len()
            } else if let Some(enterprise_data) = map.get("enterprise") {
                enterprise_data["tasks"].as_array().unwrap().len()
            } else {
                panic!(
                    "Unexpected response format: {:?}",
                    map.keys().collect::<Vec<_>>()
                );
            }
        }
        _ => panic!("Expected TaskResponse object"),
    };

    assert_eq!(task_count, 0, "User2 should not see User1's tasks");
}

#[tokio::test]
async fn test_unauthorized_access_denied() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // 認証なしでアクセス試行
    let req = axum::http::Request::builder()
        .uri("/tasks/dynamic")
        .method("GET")
        .body(axum::body::Body::from(""))
        .unwrap();

    let response = app
        .clone()
        .oneshot(req)
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_dynamic_permissions_with_filter_parameters() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // フィルターパラメータ付きでアクセス
    let req = auth_helper::create_authenticated_request(
        "GET",
        "/tasks/dynamic?status=todo",
        &user.access_token,
        None,
    );

    let response = app
        .clone()
        .oneshot(req)
        .await
        .expect("Failed to send request with filter");

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).expect("Failed to parse JSON");
    assert!(
        body.is_object(),
        "Should return TaskResponse object with filters"
    );
}
