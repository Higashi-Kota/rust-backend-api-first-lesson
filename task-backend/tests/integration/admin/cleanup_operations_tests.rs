use axum::{
    body::{self, Body},
    http::{Request, StatusCode},
};
use serde_json::Value;
use tower::ServiceExt;

use crate::common::{app_helper, auth_helper};

#[tokio::test]
async fn test_admin_list_bulk_operations_success() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create admin user and get token
    let admin_token = auth_helper::create_admin_with_jwt(&app).await;

    // List bulk operations
    let request = Request::builder()
        .uri("/admin/cleanup/bulk-operations")
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

    assert!(json["success"].as_bool().unwrap());
    assert_eq!(json["data"]["items"].as_array().unwrap().len(), 0); // No operations yet
}

#[tokio::test]
async fn test_admin_cleanup_bulk_operations_success() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create admin user and get token
    let admin_token = auth_helper::create_admin_with_jwt(&app).await;

    // Cleanup bulk operations older than 90 days (default)
    let request = Request::builder()
        .uri("/admin/cleanup/bulk-operations")
        .method("DELETE")
        .header("Authorization", format!("Bearer {}", admin_token))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert!(json["success"].as_bool().unwrap());
    assert_eq!(
        json["data"]["operation_type"],
        "bulk_operation_history_cleanup"
    );
    assert_eq!(json["data"]["deleted_count"], 0); // No old operations to delete
}

#[tokio::test]
async fn test_admin_cleanup_daily_summaries_success() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create admin user and get token
    let admin_token = auth_helper::create_admin_with_jwt(&app).await;

    // Cleanup daily summaries older than 365 days (default)
    let request = Request::builder()
        .uri("/admin/cleanup/daily-summaries")
        .method("DELETE")
        .header("Authorization", format!("Bearer {}", admin_token))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert!(json["success"].as_bool().unwrap());
    assert_eq!(
        json["data"]["operation_type"],
        "daily_activity_summary_cleanup"
    );
    assert_eq!(json["data"]["deleted_count"], 0); // No old summaries to delete
}

#[tokio::test]
async fn test_admin_get_user_feature_metrics_success() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create admin user and get token
    let admin_token = auth_helper::create_admin_with_jwt(&app).await;

    // Create a test user to get metrics for
    let test_user = auth_helper::create_and_authenticate_member(&app).await;

    // Get feature metrics for the test user
    let request = Request::builder()
        .uri(format!(
            "/admin/cleanup/feature-metrics?user_id={}",
            test_user.user_id
        ))
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

    assert!(json["success"].as_bool().unwrap());
    assert_eq!(json["data"]["user_id"], test_user.user_id.to_string());
    assert!(json["data"]["action_counts"]
        .as_object()
        .unwrap()
        .is_empty()); // No metrics tracked yet
}

#[tokio::test]
async fn test_admin_cleanup_feature_metrics_success() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create admin user and get token
    let admin_token = auth_helper::create_admin_with_jwt(&app).await;

    // Cleanup feature metrics older than 180 days (default)
    let request = Request::builder()
        .uri("/admin/cleanup/feature-metrics")
        .method("DELETE")
        .header("Authorization", format!("Bearer {}", admin_token))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert!(json["success"].as_bool().unwrap());
    assert_eq!(
        json["data"]["operation_type"],
        "feature_usage_metrics_cleanup"
    );
    assert_eq!(json["data"]["deleted_count"], 0); // No old metrics to delete
}

#[tokio::test]
async fn test_admin_cleanup_operations_forbidden_for_non_admin() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create regular member user and get token
    let member_token = auth_helper::create_member_with_jwt(&app).await;

    // Try to access cleanup endpoints as non-admin
    let endpoints = vec![
        ("/admin/cleanup/bulk-operations", "GET"),
        ("/admin/cleanup/bulk-operations", "DELETE"),
        ("/admin/cleanup/daily-summaries", "DELETE"),
        ("/admin/cleanup/feature-metrics", "GET"),
        ("/admin/cleanup/feature-metrics", "DELETE"),
    ];

    for (endpoint, method) in endpoints {
        let request = Request::builder()
            .uri(endpoint)
            .method(method)
            .header("Authorization", format!("Bearer {}", member_token))
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();

        assert_eq!(
            response.status(),
            StatusCode::FORBIDDEN,
            "Expected 403 for endpoint {} with method {}",
            endpoint,
            method
        );
    }
}

#[tokio::test]
async fn test_admin_cleanup_with_custom_days_parameter() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create admin user and get token
    let admin_token = auth_helper::create_admin_with_jwt(&app).await;

    // Try to cleanup with too few days (should fail)
    let request = Request::builder()
        .uri("/admin/cleanup/bulk-operations?days=20")
        .method("DELETE")
        .header("Authorization", format!("Bearer {}", admin_token))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST); // Validation error

    // Cleanup with valid days parameter
    let request = Request::builder()
        .uri("/admin/cleanup/bulk-operations?days=60")
        .method("DELETE")
        .header("Authorization", format!("Bearer {}", admin_token))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
