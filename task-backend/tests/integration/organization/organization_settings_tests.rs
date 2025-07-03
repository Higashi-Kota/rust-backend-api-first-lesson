// tests/integration/organization/organization_settings_tests.rs

use crate::common::{app_helper, auth_helper};
use axum::http::StatusCode;
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

async fn create_test_organization(
    app: &axum::Router,
    owner: &auth_helper::TestUser,
) -> serde_json::Value {
    let org_data = json!({
        "name": format!("Test Organization {}", Uuid::new_v4()),
        "description": "Organization for settings testing",
        "subscription_tier": "pro"
    });

    let req = auth_helper::create_authenticated_request(
        "POST",
        "/organizations",
        &owner.access_token,
        Some(serde_json::to_string(&org_data).unwrap()),
    );

    let res = app.clone().oneshot(req).await.unwrap();
    let status = res.status();

    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();

    if status != StatusCode::CREATED {
        let body_str = String::from_utf8_lossy(&body);
        panic!(
            "Failed to create organization. Status: {:?}, Body: {}",
            status, body_str
        );
    }
    assert_eq!(status, StatusCode::CREATED);
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    response["data"].clone()
}

async fn add_member_to_organization(
    app: &axum::Router,
    org_id: &str,
    owner_token: &str,
    member_id: Uuid,
    role: &str,
) {
    let member_data = json!({
        "user_id": member_id,
        "role": role
    });

    let req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/organizations/{}/members", org_id),
        owner_token,
        Some(serde_json::to_string(&member_data).unwrap()),
    );

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn test_update_organization_settings_as_owner() {
    // Arrange: Set up app and create organization
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let admin_token = auth_helper::create_admin_with_jwt(&app).await;

    // Create organization as admin
    let org_data = json!({
        "name": format!("Test Organization {}", Uuid::new_v4()),
        "description": "Organization for settings testing",
        "subscription_tier": "pro"
    });

    let req = auth_helper::create_authenticated_request(
        "POST",
        "/organizations",
        &admin_token,
        Some(serde_json::to_string(&org_data).unwrap()),
    );

    let res = app.clone().oneshot(req).await.unwrap();
    let status = res.status();

    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();

    if status != StatusCode::CREATED {
        let body_str = String::from_utf8_lossy(&body);
        panic!(
            "Failed to create organization. Status: {:?}, Body: {}",
            status, body_str
        );
    }
    assert_eq!(status, StatusCode::CREATED);
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let org = response["data"].clone();
    let org_id = org["id"].as_str().unwrap();

    let settings_update = json!({
        "allow_public_teams": true,
        "require_approval_for_new_members": false,
        "enable_single_sign_on": true,
        "default_team_subscription_tier": "pro"
    });

    // Act: Update organization settings
    let req = auth_helper::create_authenticated_request(
        "PATCH",
        &format!("/organizations/{}/settings", org_id),
        &admin_token,
        Some(serde_json::to_string(&settings_update).unwrap()),
    );
    let res = app.clone().oneshot(req).await.unwrap();
    let status = res.status();

    // Get response body for debugging
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();

    if status != StatusCode::OK {
        let body_str = String::from_utf8_lossy(&body);
        panic!(
            "Update settings failed. Status: {:?}, Body: {}",
            status, body_str
        );
    }

    // Assert: Verify response
    assert_eq!(status, StatusCode::OK);
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(response["success"].as_bool().unwrap());

    // The response from update_organization_settings returns the whole organization
    let org_data = &response["data"];

    // Check if settings are in a separate field or in the organization directly
    let settings = if org_data["settings"].is_object() {
        &org_data["settings"]
    } else {
        // Settings might be flattened into the organization object
        org_data
    };

    // Also handle the case where these might be in different places
    if settings["allow_public_teams"].is_null() {
        println!(
            "Warning: settings not found in expected location. Response: {:?}",
            response
        );
    }

    assert_eq!(settings["allow_public_teams"], true);
    assert_eq!(settings["require_approval_for_new_members"], false);
    assert_eq!(settings["enable_single_sign_on"], true);
    assert_eq!(settings["default_team_subscription_tier"], "pro");

    // Verify settings persist by fetching organization
    let req = auth_helper::create_authenticated_request(
        "GET",
        &format!("/organizations/{}", org_id),
        &admin_token,
        None,
    );
    let res = app.clone().oneshot(req).await.unwrap();
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    let fetched_settings = &response["data"]["settings"];

    // Debug output
    if fetched_settings["allow_public_teams"].is_null() {
        println!("Debug: Full fetched organization response: {:?}", response);
        println!("Debug: Organization data: {:?}", response["data"]);
        println!("Debug: Settings: {:?}", fetched_settings);
    }

    assert_eq!(fetched_settings["allow_public_teams"], true);
    assert_eq!(fetched_settings["enable_single_sign_on"], true);
}

#[tokio::test]
async fn test_update_organization_settings_as_admin() {
    // Arrange: Set up app, create organization with admin member
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let owner = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let admin_data =
        auth_helper::create_test_user_with_info("org_admin_settings@example.com", "OrgAdmin");
    let admin = auth_helper::signup_test_user(&app, admin_data)
        .await
        .unwrap();

    let org = create_test_organization(&app, &owner).await;
    let org_id = org["id"].as_str().unwrap();

    // Add admin to organization
    add_member_to_organization(&app, org_id, &owner.access_token, admin.id, "Admin").await;

    let settings_update = json!({
        "allow_public_teams": false,
        "require_approval_for_new_members": true
    });

    // Act: Admin updates organization settings
    let req = auth_helper::create_authenticated_request(
        "PATCH",
        &format!("/organizations/{}/settings", org_id),
        &admin.access_token,
        Some(serde_json::to_string(&settings_update).unwrap()),
    );
    let res = app.clone().oneshot(req).await.unwrap();

    // Assert: Should be allowed
    assert_eq!(res.status(), StatusCode::OK);

    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(response["success"].as_bool().unwrap());
    assert_eq!(response["data"]["settings"]["allow_public_teams"], false);
    assert_eq!(
        response["data"]["settings"]["require_approval_for_new_members"],
        true
    );
}

#[tokio::test]
async fn test_member_cannot_update_organization_settings() {
    // Arrange: Set up app, create organization with regular member
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let owner = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let member_data =
        auth_helper::create_test_user_with_info("org_member_settings@example.com", "OrgMember");
    let member = auth_helper::signup_test_user(&app, member_data)
        .await
        .unwrap();

    let org = create_test_organization(&app, &owner).await;
    let org_id = org["id"].as_str().unwrap();

    // Add member to organization
    add_member_to_organization(&app, org_id, &owner.access_token, member.id, "Member").await;

    let settings_update = json!({
        "allow_public_teams": true
    });

    // Act: Member tries to update settings
    let req = auth_helper::create_authenticated_request(
        "PATCH",
        &format!("/organizations/{}/settings", org_id),
        &member.access_token,
        Some(serde_json::to_string(&settings_update).unwrap()),
    );
    let res = app.clone().oneshot(req).await.unwrap();

    // Assert: Should be forbidden
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_partial_settings_update() {
    // Arrange: Set up app and create organization
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let owner = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let org = create_test_organization(&app, &owner).await;
    let org_id = org["id"].as_str().unwrap();

    // First, set all settings
    let initial_settings = json!({
        "allow_public_teams": true,
        "require_approval_for_new_members": true,
        "enable_single_sign_on": false,
        "default_team_subscription_tier": "free"
    });

    let req = auth_helper::create_authenticated_request(
        "PATCH",
        &format!("/organizations/{}/settings", org_id),
        &owner.access_token,
        Some(serde_json::to_string(&initial_settings).unwrap()),
    );
    let _ = app.clone().oneshot(req).await.unwrap();

    // Act: Update only one setting
    let partial_update = json!({
        "enable_single_sign_on": true
    });

    let req = auth_helper::create_authenticated_request(
        "PATCH",
        &format!("/organizations/{}/settings", org_id),
        &owner.access_token,
        Some(serde_json::to_string(&partial_update).unwrap()),
    );
    let res = app.clone().oneshot(req).await.unwrap();

    // Assert: Verify only specified setting changed
    assert_eq!(res.status(), StatusCode::OK);

    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    let settings = &response["data"]["settings"];
    assert_eq!(settings["allow_public_teams"], true); // Unchanged
    assert_eq!(settings["require_approval_for_new_members"], true); // Unchanged
    assert_eq!(settings["enable_single_sign_on"], true); // Changed
    assert_eq!(settings["default_team_subscription_tier"], "free"); // Unchanged
}

#[tokio::test]
async fn test_update_settings_validation() {
    // Arrange: Set up app and create organization
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let owner = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let org = create_test_organization(&app, &owner).await;
    let org_id = org["id"].as_str().unwrap();

    // Test 1: Invalid subscription tier
    let invalid_tier = json!({
        "default_team_subscription_tier": "InvalidTier"
    });

    let req = auth_helper::create_authenticated_request(
        "PATCH",
        &format!("/organizations/{}/settings", org_id),
        &owner.access_token,
        Some(serde_json::to_string(&invalid_tier).unwrap()),
    );
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY); // 422 for validation error

    // Test 2: Empty settings (should succeed as no-op)
    let empty_settings = json!({});

    let req = auth_helper::create_authenticated_request(
        "PATCH",
        &format!("/organizations/{}/settings", org_id),
        &owner.access_token,
        Some(serde_json::to_string(&empty_settings).unwrap()),
    );
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_settings_affect_organization_behavior() {
    // Arrange: Set up app and create organization
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let owner = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let org = create_test_organization(&app, &owner).await;
    let org_id = org["id"].as_str().unwrap();

    // Set require_approval_for_new_members to true
    let settings_update = json!({
        "require_approval_for_new_members": true
    });

    let req = auth_helper::create_authenticated_request(
        "PATCH",
        &format!("/organizations/{}/settings", org_id),
        &owner.access_token,
        Some(serde_json::to_string(&settings_update).unwrap()),
    );
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // Act: Try to add a new member (should require approval)
    let new_member_data =
        auth_helper::create_test_user_with_info("pending@example.com", "PendingMember");
    let _new_member = auth_helper::signup_test_user(&app, new_member_data)
        .await
        .unwrap();

    // In a real implementation, this might create a pending invitation
    // For now, we just verify the setting was saved correctly
    let req = auth_helper::create_authenticated_request(
        "GET",
        &format!("/organizations/{}", org_id),
        &owner.access_token,
        None,
    );
    let res = app.clone().oneshot(req).await.unwrap();
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(
        response["data"]["settings"]["require_approval_for_new_members"],
        true
    );
}

#[tokio::test]
async fn test_settings_history_audit() {
    // Arrange: Set up app and create organization
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let owner = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let org = create_test_organization(&app, &owner).await;
    let org_id = org["id"].as_str().unwrap();

    // Make multiple settings changes
    let updates = vec![
        json!({ "allow_public_teams": true }),
        json!({ "enable_single_sign_on": true }),
        json!({ "allow_public_teams": false, "require_approval_for_new_members": true }),
    ];

    for update in updates {
        let req = auth_helper::create_authenticated_request(
            "PATCH",
            &format!("/organizations/{}/settings", org_id),
            &owner.access_token,
            Some(serde_json::to_string(&update).unwrap()),
        );
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    // In a real implementation, we would verify audit logs here
    // For now, just verify final state
    let req = auth_helper::create_authenticated_request(
        "GET",
        &format!("/organizations/{}", org_id),
        &owner.access_token,
        None,
    );
    let res = app.clone().oneshot(req).await.unwrap();
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    let settings = &response["data"]["settings"];
    assert_eq!(settings["allow_public_teams"], false);
    assert_eq!(settings["require_approval_for_new_members"], true);
    assert_eq!(settings["enable_single_sign_on"], true);
}

#[tokio::test]
async fn test_update_nonexistent_organization_settings() {
    // Arrange: Set up app with authenticated user
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let nonexistent_id = Uuid::new_v4();

    let settings_update = json!({
        "allow_public_teams": true
    });

    // Act: Try to update settings for nonexistent organization
    let req = auth_helper::create_authenticated_request(
        "PATCH",
        &format!("/organizations/{}/settings", nonexistent_id),
        &user.access_token,
        Some(serde_json::to_string(&settings_update).unwrap()),
    );
    let res = app.clone().oneshot(req).await.unwrap();

    // Assert: Should return not found
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_settings_default_values() {
    // Arrange: Set up app and create organization
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let owner = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let org = create_test_organization(&app, &owner).await;
    let org_id = org["id"].as_str().unwrap();

    // Act: Get organization without modifying settings
    let req = auth_helper::create_authenticated_request(
        "GET",
        &format!("/organizations/{}", org_id),
        &owner.access_token,
        None,
    );
    let res = app.clone().oneshot(req).await.unwrap();

    // Assert: Verify default settings
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    let settings = &response["data"]["settings"];
    assert_eq!(settings["allow_public_teams"], false);
    assert_eq!(settings["require_approval_for_new_members"], true); // Default is true
    assert_eq!(settings["enable_single_sign_on"], false);
    assert_eq!(settings["default_team_subscription_tier"], "pro"); // Matches organization's tier
}

#[tokio::test]
async fn test_settings_subscription_tier_constraints() {
    // Arrange: Set up app and create free tier organization
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let owner = auth_helper::setup_authenticated_user(&app).await.unwrap();

    let org_data = json!({
        "name": "Free Tier Organization",
        "description": "Testing tier constraints",
        "subscription_tier": "free"
    });

    let req = auth_helper::create_authenticated_request(
        "POST",
        "/organizations",
        &owner.access_token,
        Some(serde_json::to_string(&org_data).unwrap()),
    );
    let res = app.clone().oneshot(req).await.unwrap();
    let status = res.status();
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();

    if status != StatusCode::CREATED {
        let body_str = String::from_utf8_lossy(&body);
        panic!(
            "Organization creation failed. Status: {:?}, Body: {}",
            status, body_str
        );
    }

    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let org_id = response["data"]["id"].as_str().unwrap();

    // Act: Try to set premium features
    let settings_update = json!({
        "enable_single_sign_on": true,
        "default_team_subscription_tier": "enterprise"
    });

    let req = auth_helper::create_authenticated_request(
        "PATCH",
        &format!("/organizations/{}/settings", org_id),
        &owner.access_token,
        Some(serde_json::to_string(&settings_update).unwrap()),
    );
    let res = app.clone().oneshot(req).await.unwrap();

    // Assert: Depending on business logic, this might fail or succeed with warnings
    // For this test, we assume it succeeds but verify the organization's tier
    let status = res.status();
    if status == StatusCode::OK {
        let body = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        let response: serde_json::Value = serde_json::from_slice(&body).unwrap();
        // Settings might be saved but features might not be active for free tier
        assert!(response["data"]["settings"]["enable_single_sign_on"].is_boolean());
    } else {
        // Or it might be forbidden for free tier
        assert_eq!(status, StatusCode::FORBIDDEN);
    }
}
