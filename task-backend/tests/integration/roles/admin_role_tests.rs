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
    // Arrange: Set up environment with predefined roles
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let admin_token = auth_helper::create_admin_with_jwt(&app).await;

    // Act: Make request to list roles
    let request = Request::builder()
        .uri("/admin/analytics/roles")
        .method("GET")
        .header("Authorization", format!("Bearer {}", admin_token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Assert: Verify response contains actual role data
    assert_eq!(response.status(), StatusCode::OK);

    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], true);
    assert_eq!(json["message"], "Roles retrieved successfully");

    // Verify roles array contains actual roles
    let roles = json["data"]["roles"].as_array().unwrap();
    assert!(
        roles.len() >= 2,
        "Should have at least admin and member roles"
    );

    // Find and verify admin role
    let admin_role = roles
        .iter()
        .find(|r| r["name"] == "admin")
        .expect("Admin role should exist");
    assert!(admin_role["id"].is_string());
    assert_eq!(admin_role["is_active"], true);
    assert_eq!(admin_role["is_system_role"], true);
    assert!(admin_role["user_count"].as_u64().unwrap() >= 1); // At least the admin we created

    // Find and verify member role
    let member_role = roles
        .iter()
        .find(|r| r["name"] == "member")
        .expect("Member role should exist");
    assert!(member_role["id"].is_string());
    assert_eq!(member_role["is_active"], true);
    assert_eq!(member_role["is_system_role"], true);
    assert!(member_role["user_count"].is_u64()); // User count should be a valid number

    // Verify pagination data
    let pagination = &json["data"]["pagination"];
    assert_eq!(pagination["page"], 1);
    assert_eq!(pagination["per_page"], 20); // Default page size
    assert_eq!(
        pagination["total_count"].as_i64().unwrap(),
        roles.len() as i64
    );
    assert_eq!(pagination["total_pages"], 1); // All roles fit in one page
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
    // Arrange: Set up environment and get role ID
    let (app, _schema, _db) = app_helper::setup_full_app().await;
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

    // Act: Get role with subscription info
    let request = Request::builder()
        .uri(format!("/admin/analytics/roles/{}/subscription", admin_role_id).as_str())
        .method("GET")
        .header("Authorization", format!("Bearer {}", admin_token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Assert: Verify detailed role information
    assert_eq!(response.status(), StatusCode::OK);

    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], true);

    let data = &json["data"];
    assert_eq!(data["name"], "admin");
    assert_eq!(data["id"], admin_role_id);

    // Verify permissions structure contains actual permissions
    let permissions = &data["permissions"];
    let base_perms = &permissions["base_permissions"];

    // Admin should have all permissions
    assert_eq!(base_perms["tasks"]["create"], true);
    assert_eq!(base_perms["tasks"]["read"], true);
    assert_eq!(base_perms["tasks"]["update"], true);
    assert_eq!(base_perms["tasks"]["delete"], true);
    assert_eq!(base_perms["teams"]["create"], true);
    assert_eq!(base_perms["teams"]["manage"], true);
    assert_eq!(base_perms["users"]["manage"], true);
    assert_eq!(base_perms["admin"]["full_access"], true);

    // Verify subscription info
    let sub_info = &data["subscription_info"];
    assert!(sub_info["applicable_tiers"]
        .as_array()
        .unwrap()
        .contains(&serde_json::json!("all")));
    assert_eq!(sub_info["tier_independent"], true); // Admin role works for all tiers
}

#[tokio::test]
async fn test_admin_get_role_with_different_subscription_tiers() {
    // Arrange: Set up environment and get member role ID
    let (app, _schema, _db) = app_helper::setup_full_app().await;
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

    // Act & Assert: Test different subscription tiers
    let tier_expectations = vec![
        ("free", vec![("max_tasks", 10), ("max_teams", 1)]),
        (
            "pro",
            vec![("max_tasks", 100), ("max_teams", 5), ("bulk_operations", 1)],
        ),
        (
            "enterprise",
            vec![
                ("max_tasks", -1),
                ("max_teams", -1),
                ("bulk_operations", 1),
                ("api_access", 1),
            ],
        ),
    ];

    for (tier, expected_perms) in tier_expectations {
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

        // Verify subscription-based permissions for this tier
        let permissions = &json["data"]["permissions"]["subscription_based_permissions"];
        let tier_perms = permissions
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["tier"] == tier)
            .unwrap_or_else(|| panic!("Should have permissions for {} tier", tier));

        assert_eq!(tier_perms["tier"], tier);

        // Verify tier-specific permissions
        let additional_perms = &tier_perms["additional_permissions"];
        for (perm_name, expected_value) in expected_perms {
            if expected_value == -1 {
                // -1 means unlimited
                assert_eq!(
                    additional_perms[perm_name],
                    serde_json::json!("unlimited"),
                    "Permission {} should be unlimited for {} tier",
                    perm_name,
                    tier
                );
            } else if expected_value == 1
                && (perm_name == "bulk_operations" || perm_name == "api_access")
            {
                // These specific features are boolean when value is 1
                assert_eq!(
                    additional_perms[perm_name], true,
                    "Permission {} should be enabled for {} tier",
                    perm_name, tier
                );
            } else {
                // Numeric limit (including max_teams = 1)
                assert_eq!(
                    additional_perms[perm_name].as_i64().unwrap(),
                    expected_value as i64,
                    "Permission {} should be {} for {} tier",
                    perm_name,
                    expected_value,
                    tier
                );
            }
        }
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
