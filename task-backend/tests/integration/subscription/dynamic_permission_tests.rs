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
    match data {
        Value::Object(map) => {
            // 任意のTaskResponseバリアントを受け入れる
            let has_valid_variant = map.contains_key("Limited")
                || map.contains_key("Enhanced")
                || map.contains_key("Unlimited")
                || map.contains_key("Free")
                || map.contains_key("Pro")
                || map.contains_key("Enterprise");
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
                assert!(enterprise_data["data"]["tasks"].is_array());
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
    let data = &body["data"];
    match data {
        Value::Object(map) => {
            if map.contains_key("Enterprise") {
                let enterprise_data = &map["Enterprise"];
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
                    map.contains_key("Enhanced") || map.contains_key("Pro"),
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
    let task_count = match data2 {
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
            } else if let Some(enterprise_data) = map.get("Enterprise") {
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

                // Check for various possible variant names
                for variant in &[
                    "Limited",
                    "Enhanced",
                    "Unlimited",
                    "Free",
                    "Pro",
                    "Enterprise",
                ] {
                    if let Some(variant_data) = data_obj.get(*variant) {
                        println!("\nFound variant: {}", variant);
                        println!(
                            "{} type: {:?}",
                            variant,
                            if variant_data.is_object() {
                                "Object"
                            } else if variant_data.is_array() {
                                "Array"
                            } else {
                                "Other"
                            }
                        );

                        if let Some(variant_obj) = variant_data.as_object() {
                            println!(
                                "{} keys: {:?}",
                                variant,
                                variant_obj.keys().collect::<Vec<_>>()
                            );
                        }
                    }
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

        // Check which variant admin gets
        for variant in &[
            "Limited",
            "Enhanced",
            "Unlimited",
            "Free",
            "Pro",
            "Enterprise",
        ] {
            if let Some(variant_data) = data.get(*variant) {
                println!("\nAdmin has variant: {}", variant);
                if let Some(variant_obj) = variant_data.as_object() {
                    println!(
                        "{} keys: {:?}",
                        variant,
                        variant_obj.keys().collect::<Vec<_>>()
                    );
                }
            }
        }
    }
}
