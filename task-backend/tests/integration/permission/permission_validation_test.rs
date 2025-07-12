// tests/integration/permission/permission_validation_test.rs
// Integration tests for permission validation endpoints

use crate::common::{app_helper, auth_helper};
use axum::{body, http::StatusCode};
use serde_json::{json, Value};
use tower::ServiceExt;

#[tokio::test]
async fn test_validate_permissions_with_require_all() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create a regular user
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // Check multiple permissions with require_all=true
    let validation_request = json!({
        "permissions": [
            {
                "resource": "tasks",
                "action": "read"
            },
            {
                "resource": "tasks",
                "action": "create"
            }
        ],
        "require_all": true
    });

    let req = auth_helper::create_authenticated_request(
        "POST",
        "/permissions/validate",
        &user.access_token,
        Some(serde_json::to_string(&validation_request).unwrap()),
    );

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let response: Value = serde_json::from_slice(&body).unwrap();

    // Verify response structure
    assert!(response["success"].as_bool().unwrap());
    assert!(response["data"]["overall_result"].as_bool().unwrap());
    assert!(response["data"]["require_all"].as_bool().unwrap());
    assert_eq!(response["data"]["checks"].as_array().unwrap().len(), 2);

    // Verify summary
    let summary = &response["data"]["summary"];
    assert_eq!(summary["total_checks"].as_u64().unwrap(), 2);
    assert_eq!(summary["allowed_count"].as_u64().unwrap(), 2);
    assert_eq!(summary["denied_count"].as_u64().unwrap(), 0);
}

#[tokio::test]
async fn test_validate_permissions_with_require_any() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create a regular user
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // Check multiple permissions with require_all=false
    let validation_request = json!({
        "permissions": [
            {
                "resource": "admin",
                "action": "manage"
            },
            {
                "resource": "tasks",
                "action": "read"
            }
        ],
        "require_all": false
    });

    let req = auth_helper::create_authenticated_request(
        "POST",
        "/permissions/validate",
        &user.access_token,
        Some(serde_json::to_string(&validation_request).unwrap()),
    );

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let response: Value = serde_json::from_slice(&body).unwrap();

    // Verify response - should be true because at least one permission is allowed
    assert!(response["success"].as_bool().unwrap());
    assert!(response["data"]["overall_result"].as_bool().unwrap());
    assert!(!response["data"]["require_all"].as_bool().unwrap());
}

#[tokio::test]
async fn test_get_user_permissions() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create admin for authorization
    let admin_token = auth_helper::create_admin_with_jwt(&app).await;
    let target_user_id = uuid::Uuid::new_v4();

    let req = auth_helper::create_authenticated_request(
        "GET",
        &format!("/permissions/user/{}", target_user_id),
        &admin_token,
        None,
    );

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let response: Value = serde_json::from_slice(&body).unwrap();

    // Verify response structure
    assert!(response["success"].as_bool().unwrap());
    assert_eq!(
        response["data"]["user_id"].as_str().unwrap(),
        target_user_id.to_string()
    );
    assert!(response["data"]["role"].is_object());
    assert!(response["data"]["permissions"].is_array());
    assert!(response["data"]["features"].is_array());
    assert!(response["data"]["effective_scopes"].is_array());
    assert!(response["data"]["last_updated"].is_string());
}

#[tokio::test]
async fn test_get_available_resources() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create a regular user
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    let req = auth_helper::create_authenticated_request(
        "GET",
        "/permissions/resources",
        &user.access_token,
        None,
    );

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let response: Value = serde_json::from_slice(&body).unwrap();

    // Verify response structure
    assert!(response["success"].as_bool().unwrap());
    assert!(response["data"]["resources"].is_array());
    assert!(response["data"]["total_resources"].is_number());
    assert!(response["data"]["accessible_resources"].is_number());
    assert!(response["data"]["restricted_resources"].is_number());
}

#[tokio::test]
async fn test_get_feature_access() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create a regular user
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    let req = auth_helper::create_authenticated_request(
        "GET",
        "/features/available",
        &user.access_token,
        None,
    );

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let response: Value = serde_json::from_slice(&body).unwrap();

    // Verify response structure
    assert!(response["success"].as_bool().unwrap());
    assert!(response["data"]["available_features"].is_array());
    assert!(response["data"]["restricted_features"].is_array());
    assert!(response["data"]["feature_limits"].is_object());
}

#[tokio::test]
async fn test_bulk_permission_check() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create a regular user
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    let bulk_request = json!({
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
                "resource": "admin",
                "action": "manage"
            }
        ]
    });

    let req = auth_helper::create_authenticated_request(
        "POST",
        "/permissions/bulk-check",
        &user.access_token,
        Some(serde_json::to_string(&bulk_request).unwrap()),
    );

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let response: Value = serde_json::from_slice(&body).unwrap();

    // Verify response structure
    assert!(response["success"].as_bool().unwrap());
    assert_eq!(response["data"]["checks"].as_array().unwrap().len(), 3);
    assert!(response["data"]["execution_time_ms"].is_number());
    assert!(response["data"]["summary"]["total_checks"].is_number());
}

#[tokio::test]
async fn test_check_complex_operation_permissions() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create a regular user
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    let complex_request = json!({
        "operation": "bulk_update",
        "resource_type": "tasks"
    });

    let req = auth_helper::create_authenticated_request(
        "POST",
        "/permissions/complex-operation",
        &user.access_token,
        Some(serde_json::to_string(&complex_request).unwrap()),
    );

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let response: Value = serde_json::from_slice(&body).unwrap();

    // Verify response structure
    assert!(response["success"].as_bool().unwrap());
    assert_eq!(
        response["data"]["operation"].as_str().unwrap(),
        "bulk_update"
    );
    assert!(response["data"]["operation_allowed"].is_boolean());
    assert!(response["data"]["permission_details"].is_array());
}
