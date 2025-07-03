// tests/integration/admin/feature_usage_analytics_test.rs

use crate::common::{app_helper, auth_helper};
use axum::{body, http::StatusCode};
use serde_json::Value;
use task_backend::api::dto::common::ApiResponse;
use tower::ServiceExt;

#[tokio::test]
async fn test_admin_get_feature_usage_counts() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create admin user
    let admin_token = auth_helper::create_admin_with_jwt(&app).await;

    // Track some feature usage first
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // Track multiple feature usages
    for i in 0..5 {
        let feature_request =
            task_backend::api::handlers::analytics_handler::TrackFeatureUsageRequest {
                feature_name: format!("feature_{}", i % 3),
                action_type: "view".to_string(),
                metadata: None,
            };

        let req = auth_helper::create_authenticated_request(
            "POST",
            "/analytics/track-feature",
            &user.access_token,
            Some(serde_json::to_string(&feature_request).unwrap()),
        );

        let _ = app.clone().oneshot(req).await.unwrap();
    }

    // Get feature usage counts as admin
    let req = auth_helper::create_authenticated_request(
        "GET",
        "/admin/analytics/feature-usage-counts?days=7",
        &admin_token,
        None,
    );

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let response: ApiResponse<Value> = serde_json::from_slice(&body).unwrap();
    let data = response.data.unwrap();

    assert_eq!(data["period_days"], 7);
    assert!(data["feature_counts"].is_array());
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
