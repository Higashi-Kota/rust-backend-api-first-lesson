// tests/integration/subscription/admin_subscription_endpoints_test.rs
// Integration tests for admin subscription management endpoints

use crate::common::{app_helper, auth_helper};
use axum::{
    body::{self},
    http::StatusCode,
};
use serde_json::{json, Value};
use tower::ServiceExt;

#[tokio::test]
async fn test_subscription_history_all_endpoint() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create admin
    let admin_token = auth_helper::create_admin_with_jwt(&app).await;

    // Test the /admin/subscription/history/all endpoint
    let req = auth_helper::create_authenticated_request(
        "GET",
        "/admin/subscription/history/all?page=1&per_page=10",
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
}

#[tokio::test]
async fn test_subscription_analytics_endpoint() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create admin
    let admin_token = auth_helper::create_admin_with_jwt(&app).await;

    // Test the /admin/subscription/analytics endpoint
    let req = auth_helper::create_authenticated_request(
        "GET",
        "/admin/subscription/analytics",
        &admin_token,
        None,
    );

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let response: Value = serde_json::from_slice(&body).unwrap();

    // Verify response structure
    assert!(response["success"].as_bool().unwrap());
    assert!(response["data"]["tier_distribution"].is_array());
    assert!(response["data"]["total_upgrades"].is_number());
    assert!(response["data"]["total_downgrades"].is_number());
}

#[tokio::test]
async fn test_subscription_history_search_endpoint() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create admin and a test user
    let admin_token = auth_helper::create_admin_with_jwt(&app).await;
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // Change user's subscription to create history
    let tier_change = json!({
        "new_tier": "pro"
    });

    let req = auth_helper::create_authenticated_request(
        "PUT",
        &format!("/users/{}/subscription", user.id),
        &admin_token,
        Some(serde_json::to_string(&tier_change).unwrap()),
    );

    app.clone().oneshot(req).await.unwrap();

    // Search for the history
    let req = auth_helper::create_authenticated_request(
        "GET",
        "/admin/subscription/history/search?tier=pro",
        &admin_token,
        None,
    );

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let response: Value = serde_json::from_slice(&body).unwrap();

    // Verify response
    assert!(response["success"].as_bool().unwrap());
    assert!(response["data"]["items"].is_array());
}

#[tokio::test]
async fn test_user_subscription_history_endpoint() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create a user
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // User accesses their own subscription history
    let req = auth_helper::create_authenticated_request(
        "GET",
        &format!("/users/{}/subscription/history", user.id),
        &user.access_token,
        None,
    );

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let response: Value = serde_json::from_slice(&body).unwrap();

    // Verify response structure - note: different structure for user endpoint
    assert!(response["data"]["user_id"].is_string());
    assert!(response["data"]["history"].is_array());
    assert!(response["data"]["stats"].is_object());
}
