// tests/integration/analytics/behavior_analytics_tests.rs

use crate::common::{app_helper, auth_helper};
use axum::{
    body::{self, Body},
    http::{Request, StatusCode},
};
use task_backend::features::analytics::dto::requests::TrackFeatureUsageRequest;
use task_backend::features::analytics::dto::responses::{
    FeatureUsageStatsResponse, UserFeatureUsageResponse,
};
use task_backend::shared::types::common::ApiResponse;
use tower::ServiceExt;

#[tokio::test]
async fn test_track_feature_usage() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create and authenticate user
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // Track feature usage
    let feature_request = TrackFeatureUsageRequest {
        feature_name: "user_profile".to_string(),
        action_type: "view".to_string(),
        metadata: None,
    };

    let req = auth_helper::create_authenticated_request(
        "POST",
        "/analytics/track-feature",
        &user.access_token,
        Some(serde_json::to_string(&feature_request).unwrap()),
    );

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_feature_usage_stats_as_admin() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create admin user
    let admin_token = auth_helper::create_admin_with_jwt(&app).await;

    // Get feature usage stats
    let req = auth_helper::create_authenticated_request(
        "GET",
        "/admin/analytics/features/usage?days=30",
        &admin_token,
        None,
    );

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let response: ApiResponse<FeatureUsageStatsResponse> = serde_json::from_slice(&body).unwrap();
    let stats = response.data.unwrap();
    assert_eq!(stats.period_days, 30);
}

#[tokio::test]
async fn test_get_user_feature_usage_as_admin() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create test user
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // Create admin user
    let admin_token = auth_helper::create_admin_with_jwt(&app).await;

    // Get user feature usage
    let req = auth_helper::create_authenticated_request(
        "GET",
        &format!("/admin/analytics/users/{}/features?days=7", user.id),
        &admin_token,
        None,
    );

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let response: ApiResponse<UserFeatureUsageResponse> = serde_json::from_slice(&body).unwrap();
    let usage = response.data.unwrap();
    assert_eq!(usage.user_id, user.id);
}

#[tokio::test]
async fn test_get_feature_usage_requires_auth() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Try to access without auth
    let req = Request::builder()
        .uri("/admin/analytics/features/usage?days=30")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_non_admin_cannot_access_analytics() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create regular user
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // Try to access admin analytics endpoint
    let req = auth_helper::create_authenticated_request(
        "GET",
        "/admin/analytics/features/usage?days=30",
        &user.access_token,
        None,
    );

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}
