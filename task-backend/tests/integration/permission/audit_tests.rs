// task-backend/tests/integration/permission/audit_tests.rs

use axum::{
    body,
    http::{Request, StatusCode},
};
use serde_json::Value;
use tower::ServiceExt;

use crate::common::{app_helper, auth_helper};

#[tokio::test]
async fn test_check_resource_permission_authenticated() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create and authenticate a member user
    let member_user = auth_helper::create_and_authenticate_member(&app).await;

    // Test resource permission check
    let req = auth_helper::create_authenticated_request(
        "GET",
        "/permissions/resources/tasks/actions/read/check",
        &member_user.access_token,
        None,
    );

    let response = app
        .clone()
        .oneshot(req)
        .await
        .expect("Failed to get resource permission");

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).expect("Failed to parse JSON");

    // Verify response structure
    assert!(body["data"]["user_id"].is_string());
    assert_eq!(body["data"]["resource"].as_str(), Some("tasks"));
    assert_eq!(body["data"]["action"].as_str(), Some("read"));
    assert!(body["data"]["allowed"].is_boolean());
    assert!(body["data"]["checked_at"].is_string());

    // Member should be allowed to read tasks
    assert_eq!(body["data"]["allowed"].as_bool(), Some(true));
}

#[tokio::test]
async fn test_check_resource_permission_denied() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create and authenticate a member user
    let member_user = auth_helper::create_and_authenticate_member(&app).await;

    // Test resource permission check for admin action
    let req = auth_helper::create_authenticated_request(
        "GET",
        "/permissions/resources/users/actions/delete/check",
        &member_user.access_token,
        None,
    );

    let response = app
        .clone()
        .oneshot(req)
        .await
        .expect("Failed to get resource permission");

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).expect("Failed to parse JSON");

    // Member should not be allowed to delete users
    assert_eq!(body["data"]["allowed"].as_bool(), Some(false));
    assert!(body["data"]["reason"].is_string());
    assert!(body["data"]["subscription_requirements"].is_object());
}

#[tokio::test]
async fn test_bulk_permission_check() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create and authenticate a member user
    let member_user = auth_helper::create_and_authenticate_member(&app).await;

    let bulk_request = serde_json::json!({
        "checks": [
            {
                "resource": "tasks",
                "action": "read"
            },
            {
                "resource": "tasks",
                "action": "create"
            },
            {
                "resource": "users",
                "action": "delete"
            }
        ]
    });

    let req = auth_helper::create_authenticated_request(
        "POST",
        "/permissions/bulk-check",
        &member_user.access_token,
        Some(serde_json::to_string(&bulk_request).unwrap()),
    );

    let response = app
        .clone()
        .oneshot(req)
        .await
        .expect("Failed to bulk check permissions");

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).expect("Failed to parse JSON");

    // Verify response structure
    assert!(body["data"]["user_id"].is_string());
    assert!(body["data"]["checks"].is_array());
    assert!(body["data"]["summary"].is_object());
    assert!(body["data"]["execution_time_ms"].is_number());
    assert!(body["data"]["checked_at"].is_string());

    let checks = body["data"]["checks"].as_array().unwrap();
    assert_eq!(checks.len(), 3);

    // Check individual results
    let task_read = &checks[0];
    assert_eq!(task_read["resource"].as_str(), Some("tasks"));
    assert_eq!(task_read["action"].as_str(), Some("read"));
    assert_eq!(task_read["allowed"].as_bool(), Some(true));

    let task_create = &checks[1];
    assert_eq!(task_create["resource"].as_str(), Some("tasks"));
    assert_eq!(task_create["action"].as_str(), Some("create"));
    assert_eq!(task_create["allowed"].as_bool(), Some(true));

    let user_delete = &checks[2];
    assert_eq!(user_delete["resource"].as_str(), Some("users"));
    assert_eq!(user_delete["action"].as_str(), Some("delete"));
    assert_eq!(user_delete["allowed"].as_bool(), Some(false));

    // Verify summary
    let summary = &body["data"]["summary"];
    assert_eq!(summary["total_checks"].as_u64(), Some(3));
    assert_eq!(summary["allowed_count"].as_u64(), Some(2));
    assert_eq!(summary["denied_count"].as_u64(), Some(1));
    assert!(summary["success_rate"].as_f64().unwrap() > 0.0);
}

#[tokio::test]
async fn test_bulk_permission_check_validation_error() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create and authenticate a member user
    let member_user = auth_helper::create_and_authenticate_member(&app).await;

    let invalid_request = serde_json::json!({
        "checks": [
            {
                "resource": "", // Invalid empty resource
                "action": "read"
            }
        ]
    });

    let req = auth_helper::create_authenticated_request(
        "POST",
        "/permissions/bulk-check",
        &member_user.access_token,
        Some(serde_json::to_string(&invalid_request).unwrap()),
    );

    let response = app
        .clone()
        .oneshot(req)
        .await
        .expect("Failed to bulk check permissions");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_user_effective_permissions_own() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create and authenticate a member user
    let member_user = auth_helper::create_and_authenticate_member(&app).await;

    let req = auth_helper::create_authenticated_request(
        "GET",
        &format!("/permissions/user/{}/effective", member_user.user_id),
        &member_user.access_token,
        None,
    );

    let response = app
        .clone()
        .oneshot(req)
        .await
        .expect("Failed to get effective permissions");

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).expect("Failed to parse JSON");

    // Verify response structure
    assert!(body["data"]["user_id"].is_string());
    assert!(body["data"]["role"].is_object());
    assert!(body["data"]["subscription_tier"].is_string());
    assert!(body["data"]["effective_permissions"].is_array());
    assert!(body["data"]["inherited_permissions"].is_array());
    assert!(body["data"]["denied_permissions"].is_array());
    assert!(body["data"]["permission_summary"].is_object());
    assert!(body["data"]["last_updated"].is_string());

    // Verify role info
    let role = &body["data"]["role"];
    assert!(role["role_id"].is_string());
    assert!(role["role_name"].is_string());
    assert!(role["is_active"].as_bool().unwrap());

    // Verify permission summary
    let summary = &body["data"]["permission_summary"];
    assert!(summary["total_permissions"].is_number());
    assert!(summary["effective_permissions"].is_number());
    assert!(summary["coverage_percentage"].is_number());
    assert!(summary["highest_scope"].is_string());
}

#[tokio::test]
async fn test_get_user_effective_permissions_other_user_denied() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create and authenticate a member user
    let member_user = auth_helper::create_and_authenticate_member(&app).await;

    // Create another user
    let other_user = auth_helper::create_and_authenticate_member(&app).await;

    // Try to access other user's permissions
    let req = auth_helper::create_authenticated_request(
        "GET",
        &format!("/permissions/user/{}/effective", other_user.user_id),
        &member_user.access_token,
        None,
    );

    let response = app
        .clone()
        .oneshot(req)
        .await
        .expect("Failed to get effective permissions");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_get_user_effective_permissions_with_inherited() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create and authenticate a member user
    let member_user = auth_helper::create_and_authenticate_member(&app).await;

    let req = auth_helper::create_authenticated_request(
        "GET",
        &format!(
            "/permissions/user/{}/effective?include_inherited=true",
            member_user.user_id
        ),
        &member_user.access_token,
        None,
    );

    let response = app
        .clone()
        .oneshot(req)
        .await
        .expect("Failed to get effective permissions");

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).expect("Failed to parse JSON");

    // For member users, inherited_permissions should be empty
    let inherited = body["data"]["inherited_permissions"].as_array().unwrap();
    assert_eq!(inherited.len(), 0);
}

#[tokio::test]
async fn test_get_system_permission_audit_admin() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create and authenticate an admin user
    let admin_token = auth_helper::create_and_authenticate_admin(&app).await;

    let req = auth_helper::create_authenticated_request(
        "GET",
        "/admin/permissions/audit",
        &admin_token,
        None,
    );

    let response = app
        .clone()
        .oneshot(req)
        .await
        .expect("Failed to get system audit");

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).expect("Failed to parse JSON");

    // Verify response structure
    assert!(body["data"]["audit_entries"].is_array());
    assert!(body["data"]["summary"].is_object());
    assert!(body["data"]["total_entries"].is_number());
    assert!(body["data"]["filtered_entries"].is_number());
    assert!(body["data"]["audit_period"].is_object());

    // Verify audit entries
    let entries = body["data"]["audit_entries"].as_array().unwrap();
    assert!(!entries.is_empty());

    let first_entry = &entries[0];
    assert!(first_entry["id"].is_string());
    assert!(first_entry["user_id"].is_string());
    assert!(first_entry["resource"].is_string());
    assert!(first_entry["action"].is_string());
    assert!(first_entry["result"].is_string());
    assert!(first_entry["timestamp"].is_string());

    // Verify summary
    let summary = &body["data"]["summary"];
    assert!(summary["total_checks"].is_number());
    assert!(summary["allowed_checks"].is_number());
    assert!(summary["denied_checks"].is_number());
    assert!(summary["success_rate"].is_number());
    assert!(summary["most_accessed_resource"].is_string());
    assert!(summary["most_denied_action"].is_string());

    // Verify audit period
    let period = &body["data"]["audit_period"];
    assert!(period["start_date"].is_string());
    assert!(period["end_date"].is_string());
    assert!(period["duration_hours"].is_number());
}

#[tokio::test]
async fn test_get_system_permission_audit_member_denied() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create and authenticate a member user
    let member_user = auth_helper::create_and_authenticate_member(&app).await;

    let req = auth_helper::create_authenticated_request(
        "GET",
        "/admin/permissions/audit",
        &member_user.access_token,
        None,
    );

    let response = app
        .clone()
        .oneshot(req)
        .await
        .expect("Failed to get system audit");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_get_system_permission_audit_with_filters() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create and authenticate an admin user
    let admin_token = auth_helper::create_and_authenticate_admin(&app).await;

    let req = auth_helper::create_authenticated_request(
        "GET",
        "/admin/permissions/audit?resource=tasks&action=read",
        &admin_token,
        None,
    );

    let response = app
        .clone()
        .oneshot(req)
        .await
        .expect("Failed to get system audit");

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).expect("Failed to parse JSON");

    // Verify filtered results
    let entries = body["data"]["audit_entries"].as_array().unwrap();
    assert!(!entries.is_empty());

    // All entries should match the filter
    for entry in entries {
        assert_eq!(entry["resource"].as_str(), Some("tasks"));
        assert_eq!(entry["action"].as_str(), Some("read"));
    }
}

#[tokio::test]
async fn test_permission_audit_endpoints_unauthorized() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    let endpoints = vec![
        ("GET", "/permissions/resources/tasks/actions/read/check"),
        ("POST", "/permissions/bulk-check"),
        (
            "GET",
            "/permissions/user/00000000-0000-0000-0000-000000000000/effective",
        ),
        ("GET", "/admin/permissions/audit"),
    ];

    for (method, endpoint) in endpoints {
        let req = Request::builder()
            .method(method)
            .uri(endpoint)
            .header("content-type", "application/json")
            .body(body::Body::empty())
            .unwrap();

        let response = app
            .clone()
            .oneshot(req)
            .await
            .expect("Failed to make request");
        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "Endpoint {} should require authentication",
            endpoint
        );
    }
}
