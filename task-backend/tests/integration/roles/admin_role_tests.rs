// task-backend/tests/integration/admin_role_tests.rs
use axum::{
    body::{self, Body},
    http::{Request, StatusCode},
};
use serde_json::Value;
use tower::ServiceExt;
use uuid::Uuid;

use crate::common::{app_helper, auth_helper};

#[tokio::test]
async fn test_admin_list_roles() {
    // Setup
    let (app, _schema, _db) = app_helper::setup_full_app().await;

    // Create admin user
    let admin_token = auth_helper::create_admin_with_jwt(&app).await;

    // Make request to list roles
    let request = Request::builder()
        .uri("/admin/analytics/roles")
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
    assert_eq!(json["message"], "Roles retrieved successfully");
    assert!(json["data"]["roles"].is_array());
    assert!(json["data"]["pagination"].is_object());

    // Check pagination
    let pagination = &json["data"]["pagination"];
    assert!(pagination["page"].as_i64().unwrap() >= 1);
    assert!(pagination["per_page"].as_i64().unwrap() > 0);
    assert!(pagination["total_count"].as_i64().unwrap() >= 2); // At least admin and member roles
}

#[tokio::test]
async fn test_admin_list_roles_with_pagination() {
    // Setup
    let (app, _schema, _db) = app_helper::setup_full_app().await;

    // Create admin user
    let admin_token = auth_helper::create_admin_with_jwt(&app).await;

    // Make request with pagination
    let request = Request::builder()
        .uri("/admin/analytics/roles?page=1&page_size=10")
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

    let pagination = &json["data"]["pagination"];
    assert_eq!(pagination["page"], 1);
    assert_eq!(pagination["per_page"], 10);
}

#[tokio::test]
async fn test_admin_list_active_roles_only() {
    // Setup
    let (app, _schema, _db) = app_helper::setup_full_app().await;

    // Create admin user
    let admin_token = auth_helper::create_admin_with_jwt(&app).await;

    // Make request for active roles only
    let request = Request::builder()
        .uri("/admin/analytics/roles?active_only=true")
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

    // All roles should be active
    let roles = json["data"]["roles"].as_array().unwrap();
    for role in roles {
        assert_eq!(role["is_active"], true);
    }
}

#[tokio::test]
async fn test_admin_get_role_with_subscription() {
    // Setup
    let (app, _schema, _db) = app_helper::setup_full_app().await;

    // Create admin user
    let admin_token = auth_helper::create_admin_with_jwt(&app).await;

    // First get the list of roles to find admin role ID
    let list_request = Request::builder()
        .uri("/admin/analytics/roles")
        .method("GET")
        .header("Authorization", format!("Bearer {}", admin_token))
        .body(Body::empty())
        .unwrap();

    let list_response = app.clone().oneshot(list_request).await.unwrap();
    let list_body = body::to_bytes(list_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let list_json: Value = serde_json::from_slice(&list_body).unwrap();

    // Find admin role
    let admin_role = list_json["data"]["roles"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["name"] == "admin")
        .unwrap();
    let admin_role_id = admin_role["id"].as_str().unwrap();

    // Get role with subscription info
    let request = Request::builder()
        .uri(format!("/admin/analytics/roles/{}/subscription", admin_role_id).as_str())
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
    assert_eq!(json["success"], true);
    let data = &json["data"];
    assert_eq!(data["name"], "admin");
    assert!(data["permissions"].is_object());
    assert!(data["subscription_info"].is_object());
}

#[tokio::test]
async fn test_admin_get_role_with_different_subscription_tiers() {
    // Setup
    let (app, _schema, _db) = app_helper::setup_full_app().await;

    // Create admin user
    let admin_token = auth_helper::create_admin_with_jwt(&app).await;

    // First get member role ID
    let list_request = Request::builder()
        .uri("/admin/analytics/roles")
        .method("GET")
        .header("Authorization", format!("Bearer {}", admin_token))
        .body(Body::empty())
        .unwrap();

    let list_response = app.clone().oneshot(list_request).await.unwrap();
    let list_body = body::to_bytes(list_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let list_json: Value = serde_json::from_slice(&list_body).unwrap();

    let member_role = list_json["data"]["roles"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["name"] == "member")
        .unwrap();
    let member_role_id = member_role["id"].as_str().unwrap();

    // Test different subscription tiers
    let tiers = vec!["free", "pro", "enterprise"];
    for tier in tiers {
        let request = Request::builder()
            .uri(
                format!(
                    "/admin/analytics/roles/{}/subscription?tier={}",
                    member_role_id, tier
                )
                .as_str(),
            )
            .method("GET")
            .header("Authorization", format!("Bearer {}", admin_token))
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        let permissions = &json["data"]["permissions"]["subscription_based_permissions"];
        let tier_perms = permissions
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["tier"] == tier)
            .unwrap();

        assert!(tier_perms["additional_permissions"].is_object());
    }
}

#[tokio::test]
async fn test_member_cannot_access_admin_roles() {
    // Setup
    let (app, _schema, _db) = app_helper::setup_full_app().await;

    // Create member user
    let member_signup =
        auth_helper::create_test_user_with_info("member@example.com", "member_user");
    let member_user = auth_helper::signup_test_user(&app, member_signup)
        .await
        .unwrap();

    // Try to access admin roles endpoint
    let request = Request::builder()
        .uri("/admin/analytics/roles")
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
async fn test_admin_get_role_with_invalid_id() {
    // Setup
    let (app, _schema, _db) = app_helper::setup_full_app().await;

    // Create admin user
    let admin_token = auth_helper::create_admin_with_jwt(&app).await;

    // Try to get non-existent role
    let invalid_id = Uuid::new_v4();
    let request = Request::builder()
        .uri(format!("/admin/analytics/roles/{}/subscription", invalid_id).as_str())
        .method("GET")
        .header("Authorization", format!("Bearer {}", admin_token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_admin_list_roles_pagination_edge_cases() {
    // Setup
    let (app, _schema, _db) = app_helper::setup_full_app().await;

    // Create admin user
    let admin_token = auth_helper::create_admin_with_jwt(&app).await;

    // Test with page=0 (should be corrected to 1)
    let request = Request::builder()
        .uri("/admin/analytics/roles?page=0")
        .method("GET")
        .header("Authorization", format!("Bearer {}", admin_token))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["data"]["pagination"]["page"], 1);

    // Test with very large page_size (should be clamped to 100)
    let request = Request::builder()
        .uri("/admin/analytics/roles?page_size=1000")
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
    assert_eq!(json["data"]["pagination"]["per_page"], 100);
}
