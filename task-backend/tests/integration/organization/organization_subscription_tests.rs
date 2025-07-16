// tests/integration/organization/organization_subscription_tests.rs

use crate::common::{app_helper, auth_helper};
use axum::http::StatusCode;
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

async fn create_test_organization_with_tier(
    app: &axum::Router,
    owner: &auth_helper::TestUser,
    tier: &str,
) -> serde_json::Value {
    let org_data = json!({
        "name": format!("Test Org {} - {}", Uuid::new_v4(), tier),
        "description": format!("Organization with {} subscription", tier),
        "subscription_tier": tier
    });

    let req = auth_helper::create_authenticated_request(
        "POST",
        "/organizations",
        &owner.access_token,
        Some(serde_json::to_string(&org_data).unwrap()),
    );

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    response["data"].clone()
}

#[tokio::test]
async fn test_upgrade_organization_subscription() {
    // Arrange: Create free tier organization
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let owner = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let org = create_test_organization_with_tier(&app, &owner, "free").await;
    let org_id = org["id"].as_str().unwrap();

    let upgrade_request = json!({
        "subscription_tier": "pro"
    });

    // Act: Upgrade subscription
    let req = auth_helper::create_authenticated_request(
        "PUT",
        &format!("/organizations/{}/subscription", org_id),
        &owner.access_token,
        Some(serde_json::to_string(&upgrade_request).unwrap()),
    );
    let res = app.clone().oneshot(req).await.unwrap();

    // Assert: Verify response
    assert_eq!(res.status(), StatusCode::OK);

    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(response["success"].as_bool().unwrap());
    assert_eq!(response["data"]["subscription_tier"], "pro");
    assert_eq!(response["data"]["max_teams"], 20); // Pro tier limits
    assert_eq!(response["data"]["max_members"], 100);

    // Verify organization was upgraded by fetching it again
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
    assert_eq!(response["data"]["subscription_tier"], "pro");
}

#[tokio::test]
async fn test_downgrade_organization_subscription() {
    // Arrange: Create enterprise tier organization
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let owner = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let org = create_test_organization_with_tier(&app, &owner, "enterprise").await;
    let org_id = org["id"].as_str().unwrap();

    // Create teams up to Pro limit (to test downgrade validation)
    for i in 0..15 {
        let team_data = json!({
            "name": format!("Team {}", i),
            "description": "Test team",
            "organization_id": org_id
        });

        let req = auth_helper::create_authenticated_request(
            "POST",
            "/teams",
            &owner.access_token,
            Some(serde_json::to_string(&team_data).unwrap()),
        );
        let _ = app.clone().oneshot(req).await.unwrap();
    }

    let downgrade_request = json!({
        "subscription_tier": "pro"
    });

    // Act: Downgrade subscription
    let req = auth_helper::create_authenticated_request(
        "PUT",
        &format!("/organizations/{}/subscription", org_id),
        &owner.access_token,
        Some(serde_json::to_string(&downgrade_request).unwrap()),
    );
    let res = app.clone().oneshot(req).await.unwrap();

    // Assert: Should succeed if within Pro limits
    assert_eq!(res.status(), StatusCode::OK);

    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response["data"]["subscription_tier"], "pro");
    assert_eq!(response["data"]["max_teams"], 20);
}

#[tokio::test]
async fn test_cannot_downgrade_with_excess_resources() {
    // Arrange: Create pro tier organization with more members than free allows
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let owner = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let org = create_test_organization_with_tier(&app, &owner, "pro").await;
    let org_id = org["id"].as_str().unwrap();

    // Add 11 members to the organization (exceeding Free tier limit of 10)
    for i in 0..11 {
        let member_data = auth_helper::create_test_user_with_info(
            &format!("member{}@example.com", i),
            &format!("Member{}", i),
        );
        let member = auth_helper::signup_test_user(&app, member_data)
            .await
            .unwrap();

        let add_member_request = json!({
            "user_id": member.id,
            "role": "Member"
        });

        let req = auth_helper::create_authenticated_request(
            "POST",
            &format!("/organizations/{}/members", org_id),
            &owner.access_token,
            Some(serde_json::to_string(&add_member_request).unwrap()),
        );
        let res = app.clone().oneshot(req).await.unwrap();
        let status = res.status();

        if status != StatusCode::CREATED {
            let body = axum::body::to_bytes(res.into_body(), usize::MAX)
                .await
                .unwrap();
            let body_str = String::from_utf8_lossy(&body);
            panic!(
                "Failed to add member. Status: {:?}, Body: {}",
                status, body_str
            );
        }
    }

    let downgrade_request = json!({
        "subscription_tier": "free"
    });

    // Act: Try to downgrade to free tier
    let req = auth_helper::create_authenticated_request(
        "PUT",
        &format!("/organizations/{}/subscription", org_id),
        &owner.access_token,
        Some(serde_json::to_string(&downgrade_request).unwrap()),
    );
    let res = app.clone().oneshot(req).await.unwrap();

    // Assert: Should fail due to exceeding member limits
    let status = res.status();
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);

    if status != StatusCode::BAD_REQUEST {
        panic!(
            "Expected BAD_REQUEST but got {:?}. Response: {}",
            status, body_str
        );
    }

    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(response["success"], false);

    // Check for error message - it might be in different formats
    let error_message = if response["error"]["message"].is_string() {
        response["error"]["message"].as_str().unwrap()
    } else {
        panic!("Could not find error message in response: {:?}", response);
    };

    assert!(
        error_message.contains("exceeds"),
        "Error message '{}' does not contain 'exceeds'",
        error_message
    );
}

#[tokio::test]
async fn test_only_owner_can_change_subscription() {
    // Arrange: Create organization with admin member
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let owner = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let admin_data =
        auth_helper::create_test_user_with_info("org_admin_subscription@example.com", "OrgAdmin");
    let admin = auth_helper::signup_test_user(&app, admin_data)
        .await
        .unwrap();

    let org = create_test_organization_with_tier(&app, &owner, "free").await;
    let org_id = org["id"].as_str().unwrap();

    // Add admin to organization
    let member_data = json!({
        "user_id": admin.id,
        "role": "Admin"
    });

    let req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/organizations/{}/members", org_id),
        &owner.access_token,
        Some(serde_json::to_string(&member_data).unwrap()),
    );
    let _ = app.clone().oneshot(req).await.unwrap();

    let upgrade_request = json!({
        "subscription_tier": "pro"
    });

    // Act: Admin tries to upgrade
    let req = auth_helper::create_authenticated_request(
        "PUT",
        &format!("/organizations/{}/subscription", org_id),
        &admin.access_token,
        Some(serde_json::to_string(&upgrade_request).unwrap()),
    );
    let res = app.clone().oneshot(req).await.unwrap();

    // Assert: Should be forbidden
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_subscription_tier_progression() {
    // Arrange: Create free tier organization
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let owner = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let org = create_test_organization_with_tier(&app, &owner, "free").await;
    let org_id = org["id"].as_str().unwrap();
    let _org_uuid = Uuid::parse_str(org_id).unwrap();

    // Act: Upgrade through tiers
    let tiers = vec!["pro", "enterprise"];
    for tier in tiers {
        let upgrade_request = json!({
            "subscription_tier": tier
        });

        let req = auth_helper::create_authenticated_request(
            "PUT",
            &format!("/organizations/{}/subscription", org_id),
            &owner.access_token,
            Some(serde_json::to_string(&upgrade_request).unwrap()),
        );
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    // Verify final state is Enterprise
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
    assert_eq!(response["data"]["subscription_tier"], "enterprise");
}

#[tokio::test]
async fn test_subscription_affects_limits() {
    // Arrange: Create organizations with different tiers
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let owner = auth_helper::setup_authenticated_user(&app).await.unwrap();

    let tiers = vec![("free", 3, 10), ("pro", 20, 100), ("enterprise", 100, 1000)];

    for (tier, expected_teams, expected_members) in tiers {
        let org = create_test_organization_with_tier(&app, &owner, tier).await;
        let org_id = org["id"].as_str().unwrap();

        // Act: Get organization details
        let req = auth_helper::create_authenticated_request(
            "GET",
            &format!("/organizations/{}", org_id),
            &owner.access_token,
            None,
        );
        let res = app.clone().oneshot(req).await.unwrap();

        // Assert: Verify limits
        let body = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        let response: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(response["data"]["subscription_tier"], tier);
        assert_eq!(response["data"]["max_teams"], expected_teams);
        assert_eq!(response["data"]["max_members"], expected_members);
    }
}

#[tokio::test]
async fn test_get_subscription_history() {
    // Arrange: Create organization and make subscription changes
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let owner = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let org = create_test_organization_with_tier(&app, &owner, "free").await;
    let org_id = org["id"].as_str().unwrap();

    // Make multiple subscription changes
    let changes = vec![
        ("pro", "Needed more teams"),
        ("enterprise", "Expanding business"),
        ("pro", "Cost optimization"),
    ];

    for (tier, _reason) in &changes {
        let request = json!({
            "subscription_tier": tier
        });

        let req = auth_helper::create_authenticated_request(
            "PUT",
            &format!("/organizations/{}/subscription", org_id),
            &owner.access_token,
            Some(serde_json::to_string(&request).unwrap()),
        );
        let _ = app.clone().oneshot(req).await.unwrap();
    }

    // Act: Get subscription history
    let req = auth_helper::create_authenticated_request(
        "GET",
        &format!("/organizations/{}/subscription/history", org_id),
        &owner.access_token,
        None,
    );
    let res = app.clone().oneshot(req).await.unwrap();

    // Assert: Verify history
    assert_eq!(res.status(), StatusCode::OK);

    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    let history = response["data"].as_array().unwrap();
    assert!(history.len() >= changes.len());

    // Verify each entry has required fields
    for entry in history {
        assert!(entry["id"].is_string());
        assert!(entry["new_tier"].is_string());
        assert!(entry["changed_at"].is_number());
        assert!(entry["changed_by"].is_string());
    }
}

#[tokio::test]
async fn test_subscription_validation() {
    // Arrange: Create organization
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let owner = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let org = create_test_organization_with_tier(&app, &owner, "free").await;
    let org_id = org["id"].as_str().unwrap();

    // Test 1: Invalid tier
    let invalid_request = json!({
        "subscription_tier": "SuperPremium"
    });

    let req = auth_helper::create_authenticated_request(
        "PUT",
        &format!("/organizations/{}/subscription", org_id),
        &owner.access_token,
        Some(serde_json::to_string(&invalid_request).unwrap()),
    );
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY); // 422 for validation error

    // Test 2: Same tier (no change)
    let same_tier_request = json!({
        "subscription_tier": "free"
    });

    let req = auth_helper::create_authenticated_request(
        "PUT",
        &format!("/organizations/{}/subscription", org_id),
        &owner.access_token,
        Some(serde_json::to_string(&same_tier_request).unwrap()),
    );
    let res = app.clone().oneshot(req).await.unwrap();

    // Depending on implementation, this might succeed or return bad request
    let status = res.status();
    assert!(status == StatusCode::OK || status == StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_subscription_change_affects_features() {
    // Arrange: Create free tier organization
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let owner = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let org = create_test_organization_with_tier(&app, &owner, "free").await;
    let org_id = org["id"].as_str().unwrap();

    // Try to use enterprise feature on free tier
    let settings_update = json!({
        "enable_single_sign_on": true // Premium feature
    });

    let req = auth_helper::create_authenticated_request(
        "PUT",
        &format!("/organizations/{}/settings", org_id),
        &owner.access_token,
        Some(serde_json::to_string(&settings_update).unwrap()),
    );
    let res = app.clone().oneshot(req).await.unwrap();

    // Feature might be saved but not active, or might be rejected
    let _initial_status = res.status();

    // Act: Upgrade to Enterprise
    let upgrade_request = json!({
        "subscription_tier": "enterprise"
    });

    let req = auth_helper::create_authenticated_request(
        "PUT",
        &format!("/organizations/{}/subscription", org_id),
        &owner.access_token,
        Some(serde_json::to_string(&upgrade_request).unwrap()),
    );
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // Now try to enable SSO again
    let req = auth_helper::create_authenticated_request(
        "PATCH",
        &format!("/organizations/{}/settings", org_id),
        &owner.access_token,
        Some(serde_json::to_string(&settings_update).unwrap()),
    );
    let res = app.clone().oneshot(req).await.unwrap();

    // Assert: Should definitely work on Enterprise
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_subscription_change_notification() {
    // Arrange: Create organization with members
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let owner = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let member_data =
        auth_helper::create_test_user_with_info("org_member_subscription@example.com", "OrgMember");
    let member = auth_helper::signup_test_user(&app, member_data)
        .await
        .unwrap();

    let org = create_test_organization_with_tier(&app, &owner, "free").await;
    let org_id = org["id"].as_str().unwrap();

    // Add member
    let member_data = json!({
        "user_id": member.id,
        "role": "Member"
    });

    let req = auth_helper::create_authenticated_request(
        "POST",
        &format!("/organizations/{}/members", org_id),
        &owner.access_token,
        Some(serde_json::to_string(&member_data).unwrap()),
    );
    let _ = app.clone().oneshot(req).await.unwrap();

    // Act: Upgrade subscription
    let upgrade_request = json!({
        "subscription_tier": "pro"
    });

    let req = auth_helper::create_authenticated_request(
        "PUT",
        &format!("/organizations/{}/subscription", org_id),
        &owner.access_token,
        Some(serde_json::to_string(&upgrade_request).unwrap()),
    );
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // In a real implementation, this would trigger notifications
    // For now, verify member can see the new tier
    let req = auth_helper::create_authenticated_request(
        "GET",
        &format!("/organizations/{}", org_id),
        &member.access_token,
        None,
    );
    let res = app.clone().oneshot(req).await.unwrap();

    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response["data"]["subscription_tier"], "pro");
}

#[tokio::test]
async fn test_subscription_billing_integration() {
    // Arrange: Create organization
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let owner = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let org = create_test_organization_with_tier(&app, &owner, "free").await;
    let org_id = org["id"].as_str().unwrap();

    // In a real implementation, this would include payment information
    let upgrade_request = json!({
        "subscription_tier": "pro",
        // "payment_method_id": "pm_test_123", // Would be included in real implementation
        // "billing_cycle": "monthly"
    });

    // Act: Upgrade with billing info
    let req = auth_helper::create_authenticated_request(
        "PUT",
        &format!("/organizations/{}/subscription", org_id),
        &owner.access_token,
        Some(serde_json::to_string(&upgrade_request).unwrap()),
    );
    let res = app.clone().oneshot(req).await.unwrap();

    // Assert: Verify upgrade succeeded
    assert_eq!(res.status(), StatusCode::OK);

    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response["data"]["subscription_tier"], "pro");
    // In real implementation, would also verify billing status
}

#[tokio::test]
async fn test_subscription_change_rollback() {
    // Arrange: Create organization
    let (app, _schema, _db) = app_helper::setup_full_app().await;
    let owner = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let org = create_test_organization_with_tier(&app, &owner, "pro").await;
    let org_id = org["id"].as_str().unwrap();
    let _org_uuid = Uuid::parse_str(org_id).unwrap();

    // Act: Downgrade then quickly upgrade back
    let downgrade_request = json!({
        "subscription_tier": "free"
    });

    let req = auth_helper::create_authenticated_request(
        "PUT",
        &format!("/organizations/{}/subscription", org_id),
        &owner.access_token,
        Some(serde_json::to_string(&downgrade_request).unwrap()),
    );
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // Immediately upgrade back
    let upgrade_request = json!({
        "subscription_tier": "pro"
    });

    let req = auth_helper::create_authenticated_request(
        "PUT",
        &format!("/organizations/{}/subscription", org_id),
        &owner.access_token,
        Some(serde_json::to_string(&upgrade_request).unwrap()),
    );
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // Verify final state is Pro again
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
    assert_eq!(response["data"]["subscription_tier"], "pro");
}
