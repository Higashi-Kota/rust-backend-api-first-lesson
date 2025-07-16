// task-backend/tests/integration/analytics/user_analytics_tests.rs

use axum::{
    body,
    http::{Request, StatusCode},
};
use serde_json::Value;
use tower::ServiceExt;

use crate::common::{app_helper, auth_helper};

#[tokio::test]
async fn test_get_user_behavior_analytics() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create and authenticate a member user
    let member_user = auth_helper::create_and_authenticate_member(&app).await;

    // Test user behavior analytics
    let req = auth_helper::create_authenticated_request(
        "GET",
        "/analytics/behavior",
        &member_user.access_token,
        None,
    );

    let response = app
        .clone()
        .oneshot(req)
        .await
        .expect("Failed to get user behavior analytics");

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).expect("Failed to parse JSON");

    // Verify response structure
    assert!(body["data"]["user_id"].is_string());
    assert!(body["data"]["analysis_period"].is_object());
    assert!(body["data"]["behavioral_metrics"].is_object());
    assert!(body["data"]["activity_patterns"].is_object());
    assert!(body["data"]["feature_usage"].is_object());
    assert!(body["data"]["performance_indicators"].is_object());
    assert!(body["data"]["recommendations"].is_array());
    assert!(body["data"]["generated_at"].is_number());

    // Verify behavioral metrics structure
    let behavioral_metrics = &body["data"]["behavioral_metrics"];
    assert!(behavioral_metrics["login_frequency"].is_object());
    assert!(behavioral_metrics["session_duration"].is_object());
    assert!(behavioral_metrics["activity_score"].is_number());
    assert!(behavioral_metrics["engagement_level"].is_string());
    assert!(behavioral_metrics["feature_adoption_rate"].is_number());
    assert!(behavioral_metrics["consistency_score"].is_number());

    // Verify activity patterns structure
    let activity_patterns = &body["data"]["activity_patterns"];
    assert!(activity_patterns["peak_activity_hours"].is_array());
    assert!(activity_patterns["most_active_days"].is_array());
    assert!(activity_patterns["activity_distribution"].is_object());
    assert!(activity_patterns["workflow_patterns"].is_array());
    assert!(activity_patterns["seasonal_trends"].is_array());

    // Verify recommendations
    let recommendations = body["data"]["recommendations"].as_array().unwrap();
    assert!(!recommendations.is_empty());

    for recommendation in recommendations {
        assert!(recommendation["recommendation_type"].is_string());
        assert!(recommendation["title"].is_string());
        assert!(recommendation["description"].is_string());
        assert!(recommendation["priority"].is_string());
        assert!(recommendation["expected_impact"].is_string());
    }
}

#[tokio::test]
async fn test_get_user_behavior_analytics_with_comparisons() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create and authenticate a member user
    let member_user = auth_helper::create_and_authenticate_member(&app).await;

    // Test user behavior analytics with comparisons
    let req = auth_helper::create_authenticated_request(
        "GET",
        "/analytics/behavior?include_comparisons=true",
        &member_user.access_token,
        None,
    );

    let response = app
        .clone()
        .oneshot(req)
        .await
        .expect("Failed to get user behavior analytics");

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).expect("Failed to parse JSON");

    // Verify comparisons are included
    assert!(body["data"]["comparisons"].is_object());

    let comparisons = &body["data"]["comparisons"];
    assert!(comparisons["peer_comparison"].is_object());
    assert!(comparisons["historical_comparison"].is_object());
    assert!(comparisons["tier_comparison"].is_object());

    // Verify peer comparison
    let peer_comparison = &comparisons["peer_comparison"];
    assert!(peer_comparison["percentile_rank"].is_number());
    assert!(peer_comparison["above_average_metrics"].is_array());
    assert!(peer_comparison["below_average_metrics"].is_array());
    assert!(peer_comparison["peer_group_size"].is_number());
    assert!(peer_comparison["benchmark_score"].is_number());

    // Verify historical comparison
    let historical_comparison = &comparisons["historical_comparison"];
    assert!(historical_comparison["improvement_areas"].is_array());
    assert!(historical_comparison["declining_areas"].is_array());
    assert!(historical_comparison["consistency_score"].is_number());
    assert!(historical_comparison["growth_rate"].is_number());
    assert!(historical_comparison["trend_analysis"].is_object());

    // Verify tier comparison
    let tier_comparison = &comparisons["tier_comparison"];
    assert!(tier_comparison["current_tier"].is_string());
    assert!(tier_comparison["tier_average_metrics"].is_object());
    assert!(tier_comparison["tier_percentile"].is_number());
}

#[tokio::test]
async fn test_get_user_behavior_analytics_other_user_denied() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create and authenticate a member user
    let member_user = auth_helper::create_and_authenticate_member(&app).await;

    // Create another user
    let other_user = auth_helper::create_and_authenticate_member(&app).await;

    // Try to access other user's behavior analytics
    let req = auth_helper::create_authenticated_request(
        "GET",
        &format!("/analytics/behavior?user_id={}", other_user.user_id),
        &member_user.access_token,
        None,
    );

    let response = app
        .clone()
        .oneshot(req)
        .await
        .expect("Failed to get user behavior analytics");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_get_user_behavior_analytics_admin_access() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create and authenticate an admin user
    let admin_token = auth_helper::create_and_authenticate_admin(&app).await;

    // Create a member user
    let member_user = auth_helper::create_and_authenticate_member(&app).await;

    // Admin should be able to access other user's behavior analytics
    let req = auth_helper::create_authenticated_request(
        "GET",
        &format!("/analytics/behavior?user_id={}", member_user.user_id),
        &admin_token,
        None,
    );

    let response = app
        .clone()
        .oneshot(req)
        .await
        .expect("Failed to get user behavior analytics");

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).expect("Failed to parse JSON");

    // Verify the response is for the requested user
    assert_eq!(
        body["data"]["user_id"].as_str(),
        Some(member_user.user_id.to_string().as_str())
    );
}

#[tokio::test]
async fn test_bulk_user_operation_admin() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create and authenticate an admin user
    let admin_token = auth_helper::create_and_authenticate_admin(&app).await;

    // Create some member users for bulk operation
    let user1 = auth_helper::create_and_authenticate_member(&app).await;
    let user2 = auth_helper::create_and_authenticate_member(&app).await;

    let bulk_request = serde_json::json!({
        "user_ids": [user1.user_id, user2.user_id],
        "operation": "UpdateSubscription",
        "parameters": {
            "new_tier": "pro"
        },
        "notify_users": true
    });

    let req = auth_helper::create_authenticated_request(
        "POST",
        "/admin/users/bulk-operations",
        &admin_token,
        Some(serde_json::to_string(&bulk_request).unwrap()),
    );

    let response = app
        .clone()
        .oneshot(req)
        .await
        .expect("Failed to perform bulk user operation");

    // Debug: Print the response status and body if not OK
    let status = response.status();
    if status != StatusCode::OK {
        let body_bytes = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8_lossy(&body_bytes);
        println!("Expected 200 OK, got: {}", status);
        println!("Response body: {}", body_str);
        panic!("Bulk operation test failed with status: {}", status);
    }

    assert_eq!(status, StatusCode::OK);

    let body_bytes = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).expect("Failed to parse JSON");

    // Verify response structure
    assert!(body["data"]["operation_id"].is_string());
    assert_eq!(
        body["data"]["operation"].as_str(),
        Some("update_subscription")
    );
    assert_eq!(body["data"]["total_users"].as_u64(), Some(2));
    assert!(body["data"]["successful_operations"].is_number());
    assert!(body["data"]["failed_operations"].is_number());
    assert!(body["data"]["results"].is_array());
    assert!(body["data"]["execution_time_ms"].is_number());
    assert!(body["data"]["executed_at"].is_number());

    // Verify results structure
    let results = body["data"]["results"].as_array().unwrap();
    assert_eq!(results.len(), 2);

    for result in results {
        assert!(result["user_id"].is_string());
        assert!(result["success"].is_boolean());
        // エラーは失敗した場合のみ存在
        if !result["success"].as_bool().unwrap() {
            assert!(result["error"].is_object());
            assert!(result["error"]["message"].is_string());
        }
    }

    // Check that we have both successful and potentially failed operations
    let total_operations = body["data"]["successful_operations"].as_u64().unwrap()
        + body["data"]["failed_operations"].as_u64().unwrap();
    assert_eq!(total_operations, 2);
}

#[tokio::test]
async fn test_bulk_user_operation_member_denied() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create and authenticate a member user
    let member_user = auth_helper::create_and_authenticate_member(&app).await;

    let bulk_request = serde_json::json!({
        "user_ids": [member_user.user_id],
        "operation": "UpdateSubscription",
        "parameters": {
            "new_tier": "pro"
        }
    });

    let req = auth_helper::create_authenticated_request(
        "POST",
        "/admin/users/bulk-operations",
        &member_user.access_token,
        Some(serde_json::to_string(&bulk_request).unwrap()),
    );

    let response = app
        .clone()
        .oneshot(req)
        .await
        .expect("Failed to perform bulk user operation");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_bulk_user_operation_validation_error() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create and authenticate an admin user
    let admin_token = auth_helper::create_and_authenticate_admin(&app).await;

    let invalid_request = serde_json::json!({
        "user_ids": [], // Empty user_ids array
        "operation": "UpdateSubscription"
    });

    let req = auth_helper::create_authenticated_request(
        "POST",
        "/admin/users/bulk-operations",
        &admin_token,
        Some(serde_json::to_string(&invalid_request).unwrap()),
    );

    let response = app
        .clone()
        .oneshot(req)
        .await
        .expect("Failed to perform bulk user operation");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_advanced_export() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create and authenticate a member user
    let member_user = auth_helper::create_and_authenticate_member(&app).await;

    let export_request = serde_json::json!({
        "export_type": "Tasks",
        "format": "Csv",
        "max_records": 500,
        "include_metadata": true,
        "custom_fields": ["id", "title", "status", "created_at"]
    });

    let req = auth_helper::create_authenticated_request(
        "POST",
        "/exports/advanced",
        &member_user.access_token,
        Some(serde_json::to_string(&export_request).unwrap()),
    );

    let response = app
        .clone()
        .oneshot(req)
        .await
        .expect("Failed to create advanced export");

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).expect("Failed to parse JSON");

    // Verify response structure
    assert!(body["data"]["export_id"].is_string());
    assert_eq!(body["data"]["export_type"].as_str(), Some("Tasks"));
    assert_eq!(body["data"]["format"].as_str(), Some("Csv"));
    assert_eq!(body["data"]["total_records"].as_u64(), Some(500));
    assert!(body["data"]["file_size_bytes"].is_number());
    assert!(body["data"]["download_url"].is_string());
    assert!(body["data"]["expires_at"].is_number());
    assert!(body["data"]["metadata"].is_object());
    assert_eq!(
        body["data"]["processing_status"].as_str(),
        Some("Completed")
    );
    assert!(body["data"]["created_at"].is_number());

    // Verify metadata structure
    let metadata = &body["data"]["metadata"];
    assert!(metadata["filters_applied"].is_object());
    assert!(metadata["columns_included"].is_array());
    assert!(metadata["data_version"].is_string());
    assert!(metadata["export_source"].is_string());
    assert!(metadata["checksum"].is_string());

    // Verify columns_included matches custom_fields
    let columns = metadata["columns_included"].as_array().unwrap();
    assert_eq!(columns.len(), 4);
    assert!(columns.iter().any(|c| c.as_str() == Some("id")));
    assert!(columns.iter().any(|c| c.as_str() == Some("title")));
    assert!(columns.iter().any(|c| c.as_str() == Some("status")));
    assert!(columns.iter().any(|c| c.as_str() == Some("created_at")));
}

#[tokio::test]
async fn test_advanced_export_admin_only_types() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create and authenticate a member user
    let member_user = auth_helper::create_and_authenticate_member(&app).await;

    // Try to export admin-only type (Users)
    let export_request = serde_json::json!({
        "export_type": "Users",
        "format": "Json",
        "max_records": 100
    });

    let req = auth_helper::create_authenticated_request(
        "POST",
        "/exports/advanced",
        &member_user.access_token,
        Some(serde_json::to_string(&export_request).unwrap()),
    );

    let response = app
        .clone()
        .oneshot(req)
        .await
        .expect("Failed to create advanced export");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    // Now test with admin user
    let admin_token = auth_helper::create_and_authenticate_admin(&app).await;

    let req = auth_helper::create_authenticated_request(
        "POST",
        "/exports/advanced",
        &admin_token,
        Some(serde_json::to_string(&export_request).unwrap()),
    );

    let response = app
        .clone()
        .oneshot(req)
        .await
        .expect("Failed to create advanced export");

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_advanced_export_validation_error() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Create and authenticate a member user
    let member_user = auth_helper::create_and_authenticate_member(&app).await;

    let invalid_request = serde_json::json!({
        "export_type": "Tasks",
        "format": "Csv",
        "max_records": 200000 // Invalid: > 100000
    });

    let req = auth_helper::create_authenticated_request(
        "POST",
        "/exports/advanced",
        &member_user.access_token,
        Some(serde_json::to_string(&invalid_request).unwrap()),
    );

    let response = app
        .clone()
        .oneshot(req)
        .await
        .expect("Failed to create advanced export");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_user_analytics_endpoints_unauthorized() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    let endpoints = vec![
        ("GET", "/analytics/behavior"),
        ("POST", "/admin/users/bulk-operations"),
        ("POST", "/exports/advanced"),
    ];

    for (method, endpoint) in endpoints {
        let req = Request::builder()
            .method(method)
            .uri(endpoint)
            .header("content-type", "application/json")
            .body(body::Body::empty())
            .unwrap();

        let response = app
            .clone()
            .oneshot(req)
            .await
            .expect("Failed to make request");
        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "Endpoint {} should require authentication",
            endpoint
        );
    }
}
