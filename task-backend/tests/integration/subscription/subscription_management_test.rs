// tests/integration/subscription_management_test.rs

use crate::common::{app_helper, auth_helper};
use axum::{
    body::{self},
    http::StatusCode,
};
use serde_json::{json, Value};
use tower::ServiceExt;

#[tokio::test]
async fn test_admin_subscription_history_search() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create admin
    let admin_token = auth_helper::create_admin_with_jwt(&app).await;

    // Create multiple users with different subscription tiers
    for i in 0..5 {
        let signup_data = auth_helper::create_test_user_with_info(
            &format!("tier_user{}@example.com", i),
            &format!("tier_user{}", i),
        );
        let user = auth_helper::signup_test_user(&app, signup_data)
            .await
            .unwrap();

        // Change some users to different tiers
        if i % 2 == 0 {
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
        }
    }

    // Search for pro tier changes
    let req = auth_helper::create_authenticated_request(
        "GET",
        "/admin/subscription/history/search?tier=pro&page=1&per_page=10",
        &admin_token,
        None,
    );

    let res = app.clone().oneshot(req).await.unwrap();
    let status = res.status();

    if status != StatusCode::OK {
        let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
        let error_text = String::from_utf8_lossy(&body);
        eprintln!("Search error: Status: {}, Body: {}", status, error_text);
        panic!("Expected OK status, got: {:?}", status);
    }

    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let response: Value = serde_json::from_slice(&body).unwrap();

    // Verify search results
    assert!(response["success"].as_bool().unwrap());
    assert!(response["data"]["items"].is_array());
    assert!(response["data"]["pagination"].is_object());

    let histories = response["data"]["items"].as_array().unwrap();
    for history in histories {
        assert_eq!(history["new_tier"].as_str().unwrap(), "pro");
    }
}

#[tokio::test]
async fn test_admin_subscription_analytics() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create admin
    let admin_token = auth_helper::create_admin_with_jwt(&app).await;

    // Create users with various subscription changes
    for i in 0..3 {
        let signup_data = auth_helper::create_test_user_with_info(
            &format!("analytics_user{}@example.com", i),
            &format!("analytics_user{}", i),
        );
        let user = auth_helper::signup_test_user(&app, signup_data)
            .await
            .unwrap();

        // Simulate subscription journey: free -> pro -> enterprise
        let tier_change_pro = json!({
            "new_tier": "pro"
        });

        let req = auth_helper::create_authenticated_request(
            "PUT",
            &format!("/users/{}/subscription", user.id),
            &admin_token,
            Some(serde_json::to_string(&tier_change_pro).unwrap()),
        );
        app.clone().oneshot(req).await.unwrap();

        if i == 0 {
            // One user upgrades to enterprise
            let tier_change_enterprise = json!({
                "new_tier": "enterprise"
            });

            let req = auth_helper::create_authenticated_request(
                "PUT",
                &format!("/users/{}/subscription", user.id),
                &admin_token,
                Some(serde_json::to_string(&tier_change_enterprise).unwrap()),
            );
            app.clone().oneshot(req).await.unwrap();
        }
    }

    // Get subscription analytics
    let req = auth_helper::create_authenticated_request(
        "GET",
        "/admin/subscription/analytics",
        &admin_token,
        None,
    );

    let res = app.clone().oneshot(req).await.unwrap();
    let status = res.status();

    if status != StatusCode::OK {
        let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
        let error_text = String::from_utf8_lossy(&body);
        eprintln!("Analytics error: Status: {}, Body: {}", status, error_text);
        panic!("Expected OK status, got: {:?}", status);
    }

    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let response: Value = serde_json::from_slice(&body).unwrap();

    // Verify analytics structure
    assert!(response["success"].as_bool().unwrap());
    assert!(response["data"]["tier_distribution"].is_array());
    assert!(response["data"]["recent_upgrades"].is_array());
    assert!(response["data"]["recent_downgrades"].is_array());
    assert!(response["data"]["total_upgrades"].as_u64().unwrap() >= 3);
    assert!(response["data"]["total_downgrades"].as_u64().unwrap() == 0);
}

#[tokio::test]
async fn test_admin_delete_subscription_history() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create admin and user
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

    // Get subscription history to find the ID
    let req = auth_helper::create_authenticated_request(
        "GET",
        "/admin/subscription/history/all?page=1&per_page=10",
        &admin_token,
        None,
    );

    let res = app.clone().oneshot(req).await.unwrap();
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let response: Value = serde_json::from_slice(&body).unwrap();

    let histories = response["data"]["items"].as_array().unwrap();
    assert!(!histories.is_empty());

    let history_id = histories[0]["id"].as_str().unwrap();

    // Delete specific history record
    let req = auth_helper::create_authenticated_request(
        "DELETE",
        &format!("/admin/subscription/history/{}", history_id),
        &admin_token,
        None,
    );

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // Verify deletion by trying to search for it
    let req = auth_helper::create_authenticated_request(
        "GET",
        "/admin/subscription/history/all?page=1&per_page=10",
        &admin_token,
        None,
    );

    let res = app.clone().oneshot(req).await.unwrap();
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let response: Value = serde_json::from_slice(&body).unwrap();

    let remaining_histories = response["data"]["items"].as_array().unwrap();
    let deleted_found = remaining_histories
        .iter()
        .any(|h| h["id"].as_str().unwrap() == history_id);
    assert!(!deleted_found);
}

#[tokio::test]
async fn test_admin_delete_user_subscription_history() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create admin and user
    let admin_token = auth_helper::create_admin_with_jwt(&app).await;
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // Create multiple subscription changes
    for tier in &["pro", "enterprise", "pro"] {
        let tier_change = json!({
            "new_tier": tier
        });

        let req = auth_helper::create_authenticated_request(
            "PUT",
            &format!("/users/{}/subscription", user.id),
            &admin_token,
            Some(serde_json::to_string(&tier_change).unwrap()),
        );
        app.clone().oneshot(req).await.unwrap();
    }

    // Delete all subscription history for the user
    let req = auth_helper::create_authenticated_request(
        "DELETE",
        &format!("/admin/users/{}/subscription-history", user.id),
        &admin_token,
        None,
    );

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let response: Value = serde_json::from_slice(&body).unwrap();

    assert!(response["success"].as_bool().unwrap());
    assert!(response["data"]["deleted_count"].as_u64().unwrap() >= 3);

    // Verify history is deleted
    let req = auth_helper::create_authenticated_request(
        "GET",
        &format!("/users/{}/subscription/history", user.id),
        &admin_token,
        None,
    );

    let res = app.clone().oneshot(req).await.unwrap();
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let response: Value = serde_json::from_slice(&body).unwrap();

    let history = response["data"]["history"].as_array().unwrap();
    assert!(history.is_empty());
}

#[tokio::test]
async fn test_update_organization_subscription() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create admin
    let admin_token = auth_helper::create_admin_with_jwt(&app).await;

    // Create an organization first
    let create_org = json!({
        "name": "Test Organization",
        "description": "Organization for subscription test",
        "subscription_tier": "free"
    });

    let req = auth_helper::create_authenticated_request(
        "POST",
        "/organizations",
        &admin_token,
        Some(serde_json::to_string(&create_org).unwrap()),
    );

    let res = app.clone().oneshot(req).await.unwrap();
    let status = res.status();

    if status != StatusCode::CREATED {
        let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
        let error_text = String::from_utf8_lossy(&body);
        eprintln!("Create org error: Status: {}, Body: {}", status, error_text);
        panic!("Expected CREATED status, got: {:?}", status);
    }

    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let response: Value = serde_json::from_slice(&body).unwrap();
    let org_id = response["data"]["id"].as_str().unwrap();

    // Update organization subscription
    let subscription_update = json!({
        "subscription_tier": "enterprise"
    });

    let req = auth_helper::create_authenticated_request(
        "PUT",
        &format!("/organizations/{}/subscription", org_id),
        &admin_token,
        Some(serde_json::to_string(&subscription_update).unwrap()),
    );

    let res = app.clone().oneshot(req).await.unwrap();
    let status = res.status();

    if status != StatusCode::OK {
        let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
        let error_text = String::from_utf8_lossy(&body);
        eprintln!(
            "Update subscription error: Status: {}, Body: {}",
            status, error_text
        );
        panic!("Expected OK status, got: {:?}", status);
    }

    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let response: Value = serde_json::from_slice(&body).unwrap();

    // Verify update
    assert!(response["success"].as_bool().unwrap());
    assert_eq!(
        response["data"]["subscription_tier"].as_str().unwrap(),
        "enterprise"
    );
}

#[tokio::test]
async fn test_subscription_history_search_filters() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create admin
    let admin_token = auth_helper::create_admin_with_jwt(&app).await;

    // Create a user and make various subscription changes
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // Upgrade to pro
    let tier_change = json!({ "new_tier": "pro" });
    let req = auth_helper::create_authenticated_request(
        "PUT",
        &format!("/users/{}/subscription", user.id),
        &admin_token,
        Some(serde_json::to_string(&tier_change).unwrap()),
    );
    app.clone().oneshot(req).await.unwrap();

    // Upgrade to enterprise
    let tier_change = json!({ "new_tier": "enterprise" });
    let req = auth_helper::create_authenticated_request(
        "PUT",
        &format!("/users/{}/subscription", user.id),
        &admin_token,
        Some(serde_json::to_string(&tier_change).unwrap()),
    );
    app.clone().oneshot(req).await.unwrap();

    // Downgrade back to pro
    let tier_change = json!({ "new_tier": "pro" });
    let req = auth_helper::create_authenticated_request(
        "PUT",
        &format!("/users/{}/subscription", user.id),
        &admin_token,
        Some(serde_json::to_string(&tier_change).unwrap()),
    );
    app.clone().oneshot(req).await.unwrap();

    // Test tier filter (search for Enterprise tier changes)
    let req = auth_helper::create_authenticated_request(
        "GET",
        "/admin/subscription/history/search?tier=enterprise",
        &admin_token,
        None,
    );

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let response: Value = serde_json::from_slice(&body).unwrap();
    let histories = response["data"]["items"].as_array().unwrap();

    // Should have histories with Enterprise tier
    assert!(!histories.is_empty());
    for history in histories {
        // Check if this involves Enterprise tier
        let is_enterprise = history["new_tier"].as_str().unwrap() == "enterprise"
            || history["previous_tier"].as_str().unwrap_or("") == "enterprise";
        assert!(is_enterprise);
    }

    // Test Pro tier filter
    let req = auth_helper::create_authenticated_request(
        "GET",
        "/admin/subscription/history/search?tier=pro",
        &admin_token,
        None,
    );

    let res = app.clone().oneshot(req).await.unwrap();
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let response: Value = serde_json::from_slice(&body).unwrap();
    let histories = response["data"]["items"].as_array().unwrap();

    // Should have histories with Pro tier
    assert!(!histories.is_empty());
    for history in histories {
        let is_pro = history["new_tier"].as_str().unwrap() == "pro"
            || history["previous_tier"].as_str().unwrap_or("") == "pro";
        assert!(is_pro);
    }
}

#[tokio::test]
async fn test_subscription_operations_require_admin() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create regular user
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // Try to access admin endpoints with regular user token
    let history_endpoint = format!("/admin/subscription/history/{}", "test-id");
    let user_history_endpoint = format!("/admin/users/{}/subscription-history", user.id);

    let endpoints = vec![
        ("/admin/subscription/history/all", "GET"),
        ("/admin/subscription/history/search", "GET"),
        ("/admin/subscription/analytics", "GET"),
        (history_endpoint.as_str(), "DELETE"),
        (user_history_endpoint.as_str(), "DELETE"),
    ];

    for (endpoint, method) in endpoints {
        let req =
            auth_helper::create_authenticated_request(method, endpoint, &user.access_token, None);

        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(
            res.status(),
            StatusCode::FORBIDDEN,
            "Endpoint {} should require admin access",
            endpoint
        );
    }
}
