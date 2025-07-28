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
    // Arrange: Set up test environment with actual data
    let (app, _schema, db) = app_helper::setup_full_app().await;
    let admin_token = auth_helper::create_admin_with_jwt(&app).await;

    // Create test users with different subscription tiers
    let users = vec![
        ("free1@example.com", "free"),
        ("free2@example.com", "free"),
        ("pro1@example.com", "pro"),
        ("enterprise1@example.com", "enterprise"),
    ];

    for (email, tier) in users {
        let signup_data =
            auth_helper::create_test_user_with_info(email, email.split('@').next().unwrap());
        let user = auth_helper::signup_test_user(&app, signup_data)
            .await
            .unwrap();

        // Update user tier if not free
        if tier != "free" {
            use sea_orm::{ActiveModelTrait, EntityTrait, Set};
            use task_backend::domain::user_model;
            let user_entity = user_model::Entity::find_by_id(user.id)
                .one(&db.connection)
                .await
                .unwrap()
                .unwrap();
            let mut user_active: user_model::ActiveModel = user_entity.into();
            user_active.subscription_tier = Set(tier.to_string());
            user_active.update(&db.connection).await.unwrap();
        }
    }

    // Create tasks for some users
    let user1 = auth_helper::setup_authenticated_user(&app).await.unwrap();
    for i in 0..5 {
        let task_data = serde_json::json!({
            "title": format!("Task {}", i),
            "description": format!("Description {}", i),
            "status": if i < 3 { "completed" } else { "todo" }
        });
        let req = auth_helper::create_authenticated_request(
            "POST",
            "/tasks",
            &user1.access_token,
            Some(task_data.to_string()),
        );
        app.clone().oneshot(req).await.unwrap();
    }

    // Act: Make request to get system analytics
    let request = Request::builder()
        .uri("/admin/analytics/system")
        .method("GET")
        .header("Authorization", format!("Bearer {}", admin_token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Assert: Verify response contains actual data
    assert_eq!(response.status(), StatusCode::OK);

    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["success"], true);

    // Verify actual values based on created data
    assert!(json["data"]["total_users"].as_u64().unwrap() >= 5); // At least the users we created
    assert!(json["data"]["active_users"].as_u64().unwrap() >= 1); // At least user1 who created tasks
    assert_eq!(json["data"]["total_tasks"].as_u64().unwrap(), 5); // Exactly 5 tasks created
    assert_eq!(json["data"]["completed_tasks"].as_u64().unwrap(), 3); // 3 completed tasks
                                                                      // Teams may or may not exist, so just check the field is present
    assert!(json["data"]["active_teams"].is_u64());
    // Organizations may or may not exist, so just check the field is present
    assert!(json["data"]["total_organizations"].is_u64());

    // Verify calculated rates
    let task_completion_rate = json["data"]["task_completion_rate"].as_f64().unwrap();
    assert!((task_completion_rate - 60.0).abs() < 0.1); // 3/5 = 60%

    let avg_tasks_per_user = json["data"]["average_tasks_per_user"].as_f64().unwrap();
    assert!(avg_tasks_per_user > 0.0 && avg_tasks_per_user <= 5.0);

    // Verify subscription distribution
    let sub_dist = json["data"]["subscription_distribution"]
        .as_array()
        .unwrap();
    assert!(!sub_dist.is_empty());

    let mut tier_counts = std::collections::HashMap::new();
    for tier_info in json["data"]["subscription_distribution"]
        .as_array()
        .unwrap()
    {
        let tier = tier_info["tier"].as_str().unwrap();
        let count = tier_info["count"].as_u64().unwrap();
        tier_counts.insert(tier.to_string(), count);
    }

    assert!(tier_counts.get("free").unwrap_or(&0) >= &2);
    assert!(tier_counts.get("pro").unwrap_or(&0) >= &1);
    assert!(tier_counts.get("enterprise").unwrap_or(&0) >= &1);

    // Verify suspicious IPs array exists (may be empty)
    let suspicious_ips = json["data"]["suspicious_ips"].as_array().unwrap();
    assert_eq!(suspicious_ips.len(), 0); // No suspicious activity in test

    assert!(json["data"]["daily_active_users"].as_u64().unwrap() >= 1);
    assert!(json["data"]["weekly_active_users"].as_u64().unwrap() >= 1);
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
    // Arrange: Set up test environment with subscription changes
    let (app, _schema, db) = app_helper::setup_full_app().await;
    let admin_token = auth_helper::create_admin_with_jwt(&app).await;

    // Create test users and simulate subscription changes
    let test_users = vec![
        ("test+sub1@example.com", "user1", "pro"),
        ("test+sub2@example.com", "user2", "enterprise"),
        ("test+sub3@example.com", "user3", "free"),
    ];

    for (email, username, target_tier) in test_users {
        let signup_data = auth_helper::create_test_user_with_info(email, username);
        let user = auth_helper::signup_test_user(&app, signup_data)
            .await
            .unwrap();

        // Simulate subscription change
        if target_tier != "free" {
            use sea_orm::{ActiveModelTrait, Set};
            use task_backend::domain::subscription_history_model;

            let history = subscription_history_model::ActiveModel {
                user_id: Set(user.id),
                previous_tier: Set(Some("free".to_string())),
                new_tier: Set(target_tier.to_string()),
                reason: Set(Some("Test upgrade".to_string())),
                changed_by: Set(Some(user.id)),
                ..Default::default()
            };
            history.insert(&db.connection).await.unwrap();
        }
    }

    // Act: Get subscription history
    let request = Request::builder()
        .uri("/admin/subscription/history")
        .method("GET")
        .header("Authorization", format!("Bearer {}", admin_token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Assert: Verify actual subscription history data
    assert_eq!(response.status(), StatusCode::OK);

    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], true);

    // Verify histories array contains actual changes
    let histories = json["data"]["history"].as_array().unwrap();
    assert_eq!(histories.len(), 2); // 2 upgrades (pro and enterprise)

    // Verify each history entry
    for history in histories {
        assert_eq!(history["previous_tier"], "free");
        assert!(history["new_tier"] == "pro" || history["new_tier"] == "enterprise");
        assert_eq!(history["is_upgrade"], true);
        assert_eq!(history["is_downgrade"], false);
        assert_eq!(history["reason"], "Test upgrade");
        assert!(history["changed_at"].is_number());
    }

    // Verify tier stats
    let tier_stats = json["data"]["stats"]["tier_distribution"]
        .as_array()
        .unwrap();
    assert!(!tier_stats.is_empty());

    let mut tier_map = std::collections::HashMap::new();
    for stat in tier_stats {
        tier_map.insert(
            stat["tier"].as_str().unwrap().to_string(),
            stat["count"].as_u64().unwrap(),
        );
    }

    // tier_stats shows how many times each tier was changed TO
    // We created 2 upgrades: 1 to pro and 1 to enterprise
    assert_eq!(tier_map.get("pro").unwrap_or(&0), &1);
    assert_eq!(tier_map.get("enterprise").unwrap_or(&0), &1);

    // Verify change summary
    let change_summary = &json["data"]["change_summary"];
    assert_eq!(change_summary["total_changes"].as_u64().unwrap(), 2);
    assert_eq!(change_summary["upgrades_count"].as_u64().unwrap(), 2);
    assert_eq!(change_summary["downgrades_count"].as_u64().unwrap(), 0);
}

#[tokio::test]
async fn test_admin_get_subscription_history_with_date_range() {
    // Setup
    let (app, _schema, _db) = app_helper::setup_full_app().await;

    // Create admin user
    let admin_token = auth_helper::create_admin_with_jwt(&app).await;

    // Get subscription history with date range
    // Use Unix timestamps for dates
    let start_timestamp = 1704067200; // 2024-01-01T00:00:00Z
    let end_timestamp = 1735689599; // 2024-12-31T23:59:59Z
    let request = Request::builder()
        .uri(format!(
            "/admin/subscription/history?created_after={}&created_before={}",
            start_timestamp, end_timestamp
        ))
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
    assert!(json["data"]["history"].is_array());
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
    // Arrange: Create user with subscription history
    let (app, _schema, db) = app_helper::setup_full_app().await;
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // Add subscription history for the user
    use sea_orm::{ActiveModelTrait, Set};
    use task_backend::domain::subscription_history_model;

    // Simulate upgrade from free to pro
    let history1 = subscription_history_model::ActiveModel {
        user_id: Set(user.id),
        previous_tier: Set(Some("free".to_string())),
        new_tier: Set("pro".to_string()),
        reason: Set(Some("User requested upgrade".to_string())),
        changed_by: Set(Some(user.id)),
        ..Default::default()
    };
    history1.insert(&db.connection).await.unwrap();

    // Simulate later downgrade from pro to free
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    let history2 = subscription_history_model::ActiveModel {
        user_id: Set(user.id),
        previous_tier: Set(Some("pro".to_string())),
        new_tier: Set("free".to_string()),
        reason: Set(Some("Subscription expired".to_string())),
        changed_by: Set(None),
        ..Default::default()
    };
    history2.insert(&db.connection).await.unwrap();

    // Act: Get own subscription history
    let request = Request::builder()
        .uri(format!("/users/{}/subscription/history", user.id))
        .method("GET")
        .header("Authorization", format!("Bearer {}", user.access_token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Assert: Verify user's subscription history
    assert_eq!(response.status(), StatusCode::OK);

    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Verify response structure and data
    assert_eq!(json["data"]["user_id"], user.id.to_string());

    let history = json["data"]["history"].as_array().unwrap();
    assert_eq!(history.len(), 2); // Two history entries

    // Verify first entry (should be most recent - downgrade)
    assert_eq!(history[0]["previous_tier"], "pro");
    assert_eq!(history[0]["new_tier"], "free");
    assert_eq!(history[0]["is_downgrade"], true);
    assert_eq!(history[0]["is_upgrade"], false);
    assert_eq!(history[0]["reason"], "Subscription expired");

    // Verify second entry (older - upgrade)
    assert_eq!(history[1]["previous_tier"], "free");
    assert_eq!(history[1]["new_tier"], "pro");
    assert_eq!(history[1]["is_upgrade"], true);
    assert_eq!(history[1]["is_downgrade"], false);
    assert_eq!(history[1]["reason"], "User requested upgrade");

    // Verify stats
    let stats = &json["data"]["stats"];
    assert_eq!(stats["total_changes"].as_u64().unwrap(), 2);
    assert_eq!(stats["upgrade_count"].as_u64().unwrap(), 1);
    assert_eq!(stats["downgrade_count"].as_u64().unwrap(), 1);
    assert_eq!(stats["current_tier"], "free"); // Current tier after downgrade
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
    assert!(json["data"]["user_id"].is_string());
    assert!(json["data"]["history"].is_array());
    assert!(json["data"]["stats"].is_object());
}
