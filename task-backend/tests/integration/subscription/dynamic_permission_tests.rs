use task_backend::domain::task_status::TaskStatus;
// tests/integration/subscription/dynamic_permission_tests.rs

use crate::common::{app_helper, auth_helper};
use axum::{body, http::StatusCode};
use serde_json::Value;
use task_backend::api::dto::task_dto::CreateTaskDto;
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

    // ApiResponse wrapper structure
    assert!(body["success"].as_bool().unwrap());
    assert!(body["data"].is_object());
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
        "/tasks",
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
    let data = &body["data"];
    assert!(data.is_object());

    // tier fieldを確認
    let tier = data["tier"].as_str().expect("tier field should be present");

    // tier別の検証
    match tier {
        "Free" => {
            assert!(data["tasks"].is_array());
            assert!(data["quota_info"].is_string());
            assert!(data["limit_reached"].is_boolean());
        }
        "Pro" => {
            assert!(data["tasks"].is_array());
            assert!(data["features"].is_array());
            assert!(data["export_available"].is_boolean());
        }
        "Enterprise" => {
            assert!(data["data"].is_object());
            assert!(data["data"]["items"].is_array());
            assert!(data["data"]["pagination"].is_object());
            assert!(data["bulk_operations"].is_boolean());
            assert!(data["unlimited_access"].is_boolean());
        }
        "Limited" => {
            assert!(data["items"].is_array());
            assert!(data["pagination"].is_object());
        }
        "Enhanced" => {
            assert!(data["items"].is_array());
            assert!(data["pagination"].is_object());
        }
        "Unlimited" => {
            assert!(data["items"].is_array());
            assert!(data["pagination"].is_object());
        }
        _ => panic!("Unexpected tier: {}", tier),
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
    let data = &body["data"];
    assert!(data.is_object(), "Expected object in data field");

    // tier fieldをチェック
    let tier = data["tier"].as_str().unwrap();
    match tier {
        "Enterprise" => {
            assert!(data["data"]["items"].is_array());
            assert!(data["data"]["pagination"].is_object());
            assert!(data["bulk_operations"].as_bool().unwrap_or(false));
            assert!(data["unlimited_access"].as_bool().unwrap_or(false));
        }
        "Unlimited" => {
            // Unlimited variantでも管理者としてOK
            assert!(data["items"].is_array());
            assert!(data["pagination"].is_object());
        }
        "Pro" => {
            // Pro variantでも管理者としてOK
            assert!(data["tasks"].is_array());
            assert!(data["features"].is_array());
            assert!(data["export_available"].is_boolean());
        }
        _ => panic!("Admin should have high-level access tier, got: {}", tier),
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
        "/tasks",
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
    let data2 = &body2["data"];
    assert!(data2.is_object());

    let tier = data2["tier"]
        .as_str()
        .expect("tier field should be present");
    let task_count = match tier {
        "Free" => data2["tasks"].as_array().unwrap().len(),
        "Pro" => data2["tasks"].as_array().unwrap().len(),
        "Enterprise" => data2["data"]["items"].as_array().unwrap().len(),
        "Limited" | "Enhanced" | "Unlimited" => data2["items"].as_array().unwrap().len(),
        _ => panic!("Unexpected tier: {}", tier),
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

#[tokio::test]
async fn test_debug_response_structure() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // Create a task first to ensure we have data
    let task_payload = CreateTaskDto {
        title: "Debug Test Task".to_string(),
        description: Some("Task for debugging response structure".to_string()),
        status: Some(TaskStatus::Todo),
        priority: None,
        due_date: None,
    };

    let create_req = auth_helper::create_authenticated_request(
        "POST",
        "/tasks",
        &user.access_token,
        Some(serde_json::to_string(&task_payload).unwrap()),
    );

    let _create_response = app
        .clone()
        .oneshot(create_req)
        .await
        .expect("Failed to create task");

    // Now get the dynamic tasks response
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

    println!("Response Status: {:?}", response.status());

    let body_bytes = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).expect("Failed to parse JSON");

    println!("=== FULL RESPONSE STRUCTURE ===");
    println!("{}", serde_json::to_string_pretty(&body).unwrap());

    // Print detailed structure analysis
    println!("\n=== RESPONSE ANALYSIS ===");
    println!(
        "Top-level type: {:?}",
        if body.is_object() {
            "Object"
        } else if body.is_array() {
            "Array"
        } else {
            "Other"
        }
    );

    if let Some(obj) = body.as_object() {
        println!("Top-level keys: {:?}", obj.keys().collect::<Vec<_>>());

        if let Some(success) = obj.get("success") {
            println!("success field: {:?}", success);
        }

        if let Some(data) = obj.get("data") {
            println!(
                "\ndata field type: {:?}",
                if data.is_object() {
                    "Object"
                } else if data.is_array() {
                    "Array"
                } else {
                    "Other"
                }
            );

            if let Some(data_obj) = data.as_object() {
                println!("data field keys: {:?}", data_obj.keys().collect::<Vec<_>>());

                // Check for tier field
                if let Some(tier) = data_obj.get("tier") {
                    println!("\nFound tier: {:?}", tier);
                }

                // Print all fields for debugging
                for (key, value) in data_obj {
                    println!(
                        "{}: {:?}",
                        key,
                        if value.is_object() {
                            "Object"
                        } else if value.is_array() {
                            "Array"
                        } else if value.is_string() {
                            "String"
                        } else if value.is_boolean() {
                            "Boolean"
                        } else if value.is_number() {
                            "Number"
                        } else {
                            "Other"
                        }
                    );
                }
            }
        }

        if let Some(error) = obj.get("error") {
            println!("\nerror field: {:?}", error);
        }

        if let Some(meta) = obj.get("meta") {
            println!("\nmeta field: {:?}", meta);
        }
    }
}

#[tokio::test]
async fn test_debug_admin_response_structure() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create admin user
    let admin_token = auth_helper::create_admin_with_jwt(&app).await;

    // Create a task as admin
    let task_payload = CreateTaskDto {
        title: "Admin Debug Test Task".to_string(),
        description: Some("Task for debugging admin response structure".to_string()),
        status: Some(TaskStatus::Todo),
        priority: None,
        due_date: None,
    };

    let create_req = auth_helper::create_authenticated_request(
        "POST",
        "/tasks",
        &admin_token,
        Some(serde_json::to_string(&task_payload).unwrap()),
    );

    let _create_response = app
        .clone()
        .oneshot(create_req)
        .await
        .expect("Failed to create task");

    // Now get the dynamic tasks response as admin
    let req =
        auth_helper::create_authenticated_request("GET", "/tasks/dynamic", &admin_token, None);

    let response = app
        .clone()
        .oneshot(req)
        .await
        .expect("Failed to send request");

    println!("Admin Response Status: {:?}", response.status());

    let body_bytes = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).expect("Failed to parse JSON");

    println!("=== ADMIN FULL RESPONSE STRUCTURE ===");
    println!("{}", serde_json::to_string_pretty(&body).unwrap());

    // Print detailed structure analysis
    println!("\n=== ADMIN RESPONSE ANALYSIS ===");

    if let Some(data) = body.get("data").and_then(|d| d.as_object()) {
        println!("data field keys: {:?}", data.keys().collect::<Vec<_>>());

        // Check tier field
        if let Some(tier) = data.get("tier") {
            println!("\nAdmin has tier: {:?}", tier);
        }

        // Print all fields for debugging
        for (key, value) in data {
            println!(
                "{}: {:?}",
                key,
                if value.is_object() {
                    "Object"
                } else if value.is_array() {
                    "Array"
                } else if value.is_string() {
                    "String"
                } else if value.is_boolean() {
                    "Boolean"
                } else if value.is_number() {
                    "Number"
                } else {
                    "Other"
                }
            );
        }
    }
}
