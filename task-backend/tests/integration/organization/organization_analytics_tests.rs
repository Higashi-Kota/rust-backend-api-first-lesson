use axum::body::to_bytes;
use common::app_helper::setup_full_app;
use task_backend::features::organization::dto::OrganizationAnalyticsResponse;
use task_backend::shared::types::common::ApiResponse;
use tower::ServiceExt;

mod common {
    pub use crate::common::*;
}

#[tokio::test]
async fn test_organization_analytics_success() {
    let (app, _schema, _db) = setup_full_app().await;
    let user = common::auth_helper::create_and_authenticate_user(&app).await;

    // 組織を作成
    let create_response = app
        .clone()
        .oneshot(common::app_helper::create_request(
            "POST",
            "/organizations",
            &user.access_token,
            &serde_json::json!({
                "name": "Test Analytics Organization",
                "description": "Organization for analytics testing",
                "subscription_tier": "free"
            }),
        ))
        .await
        .expect("Failed to send request");

    assert_eq!(create_response.status(), 201);
    let body_bytes = to_bytes(create_response.into_body(), usize::MAX)
        .await
        .expect("Failed to read body");
    let org_body: ApiResponse<serde_json::Value> =
        serde_json::from_slice(&body_bytes).expect("Failed to parse response");
    let organization_id = org_body.data.unwrap()["id"]
        .as_str()
        .unwrap()
        .parse::<uuid::Uuid>()
        .unwrap();

    // 組織分析データを取得
    let response = app
        .oneshot(common::auth_helper::create_authenticated_request(
            "GET",
            &format!("/organizations/{}/analytics", organization_id),
            &user.access_token,
            None,
        ))
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 200);

    let body_bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read body");
    let body: ApiResponse<OrganizationAnalyticsResponse> =
        serde_json::from_slice(&body_bytes).expect("Failed to parse response");

    assert!(body.data.is_some());
    let analytics_response = body.data.unwrap();
    assert_eq!(analytics_response.organization_id, organization_id);
    assert_eq!(
        analytics_response.organization_name,
        "Test Analytics Organization"
    );
    assert_eq!(
        analytics_response.message,
        "Organization analytics retrieved successfully"
    );

    // サマリーの検証
    assert!(analytics_response.summary.total_members >= 1);
    assert!(analytics_response.summary.usage_percentage >= 0.0);
    assert!(analytics_response.summary.usage_percentage <= 100.0);

    // 分析データの検証
    for data in &analytics_response.analytics_data {
        assert!(!data.analytics_type.is_empty());
        assert!(!data.period.is_empty());
        assert!(data.period_start < data.period_end);
        assert!(data.recorded_at <= chrono::Utc::now());
    }
}

#[tokio::test]
async fn test_organization_analytics_forbidden() {
    let (app, _schema, _db) = setup_full_app().await;
    let user = common::auth_helper::create_and_authenticate_user(&app).await;
    let other_user = common::auth_helper::create_and_authenticate_user(&app).await;

    // 組織を作成
    let create_response = app
        .clone()
        .oneshot(common::app_helper::create_request(
            "POST",
            "/organizations",
            &user.access_token,
            &serde_json::json!({
                "name": "Private Organization",
                "description": "Organization for forbidden test",
                "subscription_tier": "free"
            }),
        ))
        .await
        .expect("Failed to send request");

    assert_eq!(create_response.status(), 201);
    let body_bytes = to_bytes(create_response.into_body(), usize::MAX)
        .await
        .expect("Failed to read body");
    let org_body: ApiResponse<serde_json::Value> =
        serde_json::from_slice(&body_bytes).expect("Failed to parse response");
    let organization_id = org_body.data.unwrap()["id"]
        .as_str()
        .unwrap()
        .parse::<uuid::Uuid>()
        .unwrap();

    // 別のユーザーでアクセス
    let response = app
        .clone()
        .oneshot(common::auth_helper::create_authenticated_request(
            "GET",
            &format!("/organizations/{}/analytics", organization_id),
            &other_user.access_token,
            None,
        ))
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 403);

    let body_bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read body");
    let body: serde_json::Value =
        serde_json::from_slice(&body_bytes).expect("Failed to parse response");

    assert_eq!(body["success"], false);
    let message = body["message"].as_str().unwrap_or("");
    assert!(
        message.contains("member")
            || message.contains("permission")
            || message.contains("forbidden")
            || message.contains("Forbidden")
            || message.contains("access")
    );
}

#[tokio::test]
async fn test_organization_analytics_not_found() {
    let (app, _schema, _db) = setup_full_app().await;
    let user = common::auth_helper::create_and_authenticate_user(&app).await;

    let non_existent_id = uuid::Uuid::new_v4();

    // 存在しない組織の分析データにアクセス
    let response = app
        .oneshot(common::auth_helper::create_authenticated_request(
            "GET",
            &format!("/organizations/{}/analytics", non_existent_id),
            &user.access_token,
            None,
        ))
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 404);

    let body_bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read body");
    let body: serde_json::Value =
        serde_json::from_slice(&body_bytes).expect("Failed to parse response");

    assert_eq!(body["success"], false);
    let message = body["message"].as_str().unwrap_or("");
    assert!(message.contains("not found") || message.contains("Organization"));
}
