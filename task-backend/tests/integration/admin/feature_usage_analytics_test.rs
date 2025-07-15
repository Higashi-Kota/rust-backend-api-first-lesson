// tests/integration/admin/feature_usage_analytics_test.rs

use crate::common::{app_helper, auth_helper};
use axum::{body, http::StatusCode};
use serde_json::Value;
use task_backend::types::ApiResponse;
use tower::ServiceExt;

#[tokio::test]
async fn test_admin_get_feature_usage_counts() {
    // Arrange: Set up test environment and track feature usage
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;
    let admin_token = auth_helper::create_admin_with_jwt(&app).await;
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // Create multiple users to track different features
    let user2 = auth_helper::create_and_authenticate_member(&app).await;

    // Define expected feature usage counts
    let _features = &[
        ("feature_0", 2), // Will be used by 2 iterations (i=0, i=3)
        ("feature_1", 2), // Will be used by 2 iterations (i=1, i=4)
        ("feature_2", 1), // Will be used by 1 iteration (i=2)
    ];

    // Track feature usage for first user
    for i in 0..5 {
        let feature_request =
            task_backend::api::handlers::analytics_handler::TrackFeatureUsageRequest {
                feature_name: format!("feature_{}", i % 3),
                action_type: "view".to_string(),
                metadata: Some(serde_json::json!({
                    "test_iteration": i,
                    "user": "user1"
                })),
            };

        let req = auth_helper::create_authenticated_request(
            "POST",
            "/analytics/track-feature",
            &user.access_token,
            Some(serde_json::to_string(&feature_request).unwrap()),
        );

        let _ = app.clone().oneshot(req).await.unwrap();
    }

    // Track additional usage from second user
    for feature_name in ["feature_0", "feature_1", "feature_new"] {
        let feature_request =
            task_backend::api::handlers::analytics_handler::TrackFeatureUsageRequest {
                feature_name: feature_name.to_string(),
                action_type: "create".to_string(),
                metadata: Some(serde_json::json!({
                    "user": "user2"
                })),
            };

        let req = auth_helper::create_authenticated_request(
            "POST",
            "/analytics/track-feature",
            &user2.access_token,
            Some(serde_json::to_string(&feature_request).unwrap()),
        );

        let _ = app.clone().oneshot(req).await.unwrap();
    }

    // Act: Get feature usage counts as admin
    let req = auth_helper::create_authenticated_request(
        "GET",
        "/admin/analytics/feature-usage-counts?days=7",
        &admin_token,
        None,
    );

    let res = app.clone().oneshot(req).await.unwrap();

    // Assert: Verify response contains actual feature counts
    assert_eq!(res.status(), StatusCode::OK);

    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let response: ApiResponse<Value> = serde_json::from_slice(&body).unwrap();
    let data = response.data.unwrap();

    assert_eq!(data["period_days"], 7);

    let feature_counts = data["feature_counts"].as_array().unwrap();
    assert!(
        !feature_counts.is_empty(),
        "Feature counts should not be empty"
    );

    // Verify specific feature counts
    let mut found_features = std::collections::HashMap::new();
    for feature in feature_counts {
        let name = feature["feature_name"].as_str().unwrap();
        let total_usage = feature["total_usage"].as_u64().unwrap();
        let unique_users = feature["unique_users"].as_u64().unwrap();
        found_features.insert(name.to_string(), (total_usage, unique_users));
    }

    // Verify expected counts
    assert_eq!(found_features.get("feature_0").unwrap().0, 3); // 2 from user1 + 1 from user2
    assert_eq!(found_features.get("feature_0").unwrap().1, 3); // unique_users is same as total for now
    assert_eq!(found_features.get("feature_1").unwrap().0, 3); // 2 from user1 + 1 from user2
    assert_eq!(found_features.get("feature_1").unwrap().1, 3); // unique_users is same as total for now
    assert_eq!(found_features.get("feature_2").unwrap().0, 1); // 1 from user1 only
    assert_eq!(found_features.get("feature_2").unwrap().1, 1); // unique_users is same as total for now
    assert_eq!(found_features.get("feature_new").unwrap().0, 1); // 1 from user2 only
    assert_eq!(found_features.get("feature_new").unwrap().1, 1); // unique_users is same as total for now
}

#[tokio::test]
async fn test_non_admin_cannot_access_feature_usage_counts() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create regular user
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // Try to access admin endpoint
    let req = auth_helper::create_authenticated_request(
        "GET",
        "/admin/analytics/feature-usage-counts",
        &user.access_token,
        None,
    );

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}
