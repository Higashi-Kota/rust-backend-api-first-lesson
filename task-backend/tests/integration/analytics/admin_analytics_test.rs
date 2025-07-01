// task-backend/tests/integration/admin_analytics_test.rs

use axum::{
    body::{self, Body},
    http::{Request, StatusCode},
};
use serde_json::Value;
use tower::ServiceExt;

use crate::common::{app_helper, auth_helper};

#[tokio::test]
async fn test_admin_get_system_analytics() {
    // Setup
    let (app, _schema, _db) = app_helper::setup_full_app().await;

    // Create admin user
    let admin_token = auth_helper::create_admin_with_jwt(&app).await;

    // Make request to get system analytics
    let request = Request::builder()
        .uri("/admin/analytics/system")
        .method("GET")
        .header("Authorization", format!("Bearer {}", admin_token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Check response structure
    assert_eq!(json["success"], true);
    assert_eq!(json["message"], "System analytics retrieved successfully");

    let data = &json["data"];
    assert!(data["total_users"].is_number());
    assert!(data["active_users"].is_number());
    assert!(data["total_tasks"].is_number());
    assert!(data["completed_tasks"].is_number());
    assert!(data["active_teams"].is_number());
    assert!(data["total_organizations"].is_number());
    assert!(data["user_growth_rate"].is_number());
    assert!(data["task_completion_rate"].is_number());
    assert!(data["average_tasks_per_user"].is_number());
    assert!(data["subscription_distribution"].is_array());
    assert!(data["suspicious_ips"].is_array());
    assert!(data["daily_active_users"].is_number());
    assert!(data["weekly_active_users"].is_number());
}

#[tokio::test]
async fn test_member_cannot_access_system_analytics() {
    // Setup
    let (app, _schema, _db) = app_helper::setup_full_app().await;

    // Create member user
    let member_signup =
        auth_helper::create_test_user_with_info("member@example.com", "member_user");
    let member_user = auth_helper::signup_test_user(&app, member_signup)
        .await
        .unwrap();

    // Try to access system analytics endpoint
    let request = Request::builder()
        .uri("/admin/analytics/system")
        .method("GET")
        .header(
            "Authorization",
            format!("Bearer {}", member_user.access_token),
        )
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_admin_get_subscription_history() {
    // Setup
    let (app, _schema, _db) = app_helper::setup_full_app().await;

    // Create admin user
    let admin_token = auth_helper::create_admin_with_jwt(&app).await;

    // Create some test users and simulate subscription changes
    for i in 0..3 {
        let email = format!("test+sub{}@example.com", i);
        let signup_data = auth_helper::create_test_user_with_info(&email, &format!("user{}", i));
        let _ = auth_helper::signup_test_user(&app, signup_data).await;
    }

    // Get subscription history
    let request = Request::builder()
        .uri("/admin/subscription/history")
        .method("GET")
        .header("Authorization", format!("Bearer {}", admin_token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], true);
    assert!(json["data"]["histories"].is_array());
    assert!(json["data"]["tier_stats"].is_array());
    assert!(json["data"]["change_summary"].is_object());
}

#[tokio::test]
async fn test_admin_get_subscription_history_with_date_range() {
    // Setup
    let (app, _schema, _db) = app_helper::setup_full_app().await;

    // Create admin user
    let admin_token = auth_helper::create_admin_with_jwt(&app).await;

    // Get subscription history with date range
    let request = Request::builder()
        .uri("/admin/subscription/history?start_date=2024-01-01T00:00:00Z&end_date=2024-12-31T23:59:59Z")
        .method("GET")
        .header("Authorization", format!("Bearer {}", admin_token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], true);
    assert!(json["data"]["histories"].is_array());
}

#[tokio::test]
async fn test_admin_get_subscription_history_with_filter() {
    // Setup
    let (app, _schema, _db) = app_helper::setup_full_app().await;

    // Create admin user
    let admin_token = auth_helper::create_admin_with_jwt(&app).await;

    // Test upgrade filter
    let request = Request::builder()
        .uri("/admin/subscription/history?filter=upgrades")
        .method("GET")
        .header("Authorization", format!("Bearer {}", admin_token))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Test downgrade filter
    let request = Request::builder()
        .uri("/admin/subscription/history?filter=downgrades")
        .method("GET")
        .header("Authorization", format!("Bearer {}", admin_token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_user_get_own_subscription_history() {
    // Setup
    let (app, _schema, _db) = app_helper::setup_full_app().await;

    // Create and login user
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // Get own subscription history
    let request = Request::builder()
        .uri(format!("/users/{}/subscription/history", user.id))
        .method("GET")
        .header("Authorization", format!("Bearer {}", user.access_token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();

    if status != StatusCode::OK {
        let body = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let error_text = String::from_utf8_lossy(&body);
        eprintln!("User subscription history error: {}", error_text);
        eprintln!("User ID: {}", user.id);
        eprintln!("Access token: {}", user.access_token);
        panic!("Expected OK status, got: {:?}", status);
    }

    assert_eq!(status, StatusCode::OK);

    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Check SubscriptionHistoryResponse structure
    assert!(json["user_id"].is_string());
    assert!(json["history"].is_array());
    assert!(json["stats"].is_object());
}

#[tokio::test]
async fn test_user_cannot_get_other_user_subscription_history() {
    // Setup
    let (app, _schema, _db) = app_helper::setup_full_app().await;

    // Create two users
    let user1 = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let user2_signup = auth_helper::create_test_user_with_info("user2@example.com", "user2");
    let user2 = auth_helper::signup_test_user(&app, user2_signup)
        .await
        .unwrap();

    // User1 tries to access user2's subscription history
    let request = Request::builder()
        .uri(format!("/users/{}/subscription/history", user2.id))
        .method("GET")
        .header("Authorization", format!("Bearer {}", user1.access_token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_admin_can_get_any_user_subscription_history() {
    // Setup
    let (app, _schema, _db) = app_helper::setup_full_app().await;

    // Create admin and regular user
    let admin_token = auth_helper::create_admin_with_jwt(&app).await;
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // Admin accesses user's subscription history
    let request = Request::builder()
        .uri(format!("/users/{}/subscription/history", user.id))
        .method("GET")
        .header("Authorization", format!("Bearer {}", admin_token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Admin can access user's subscription history - check SubscriptionHistoryResponse structure
    assert!(json["user_id"].is_string());
    assert!(json["history"].is_array());
    assert!(json["stats"].is_object());
}
