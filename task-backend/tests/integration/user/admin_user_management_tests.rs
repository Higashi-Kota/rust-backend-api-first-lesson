// tests/integration/user/admin_user_management_test.rs
// Integration tests for admin user management endpoints

use crate::common::{app_helper, auth_helper};
use axum::{
    body::{self},
    http::StatusCode,
};
use serde_json::Value;
use tower::ServiceExt;

#[tokio::test]
async fn test_admin_users_with_roles_endpoint() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create admin
    let admin_token = auth_helper::create_admin_with_jwt(&app).await;

    // Test the /admin/users/roles endpoint
    let req = auth_helper::create_authenticated_request(
        "GET",
        "/admin/users/roles?page=1&per_page=10",
        &admin_token,
        None,
    );

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let response: Value = serde_json::from_slice(&body).unwrap();

    // Verify response structure
    assert!(response["success"].as_bool().unwrap());
    assert!(response["data"]["items"].is_array());
    assert!(response["data"]["pagination"].is_object());
    assert!(response["data"]["role_summary"].is_array());
}
