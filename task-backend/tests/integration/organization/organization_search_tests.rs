// tests/integration/organization/organization_search_tests.rs

use axum::{
    body::{self, Body},
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;

use crate::common::{app_helper, auth_helper};

#[tokio::test]
async fn test_organization_search_success() {
    // Arrange
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;
    let jwt_token = auth_helper::create_admin_with_jwt(&app).await;

    // 1つ組織を作成
    let org_data = json!({
        "name": "Test Organization for Search",
        "description": "Organization for testing search functionality",
        "subscription_tier": "free"
    });

    let request = Request::builder()
        .uri("/organizations")
        .method("POST")
        .header("Authorization", format!("Bearer {}", jwt_token))
        .header("Content-Type", "application/json")
        .body(Body::from(org_data.to_string()))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    // Act: 統一検索エンドポイントを使用
    let request = Request::builder()
        .uri("/organizations/search?search=Test&page=1&per_page=10")
        .method("GET")
        .header("Authorization", format!("Bearer {}", jwt_token))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);
    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert!(json["success"].as_bool().unwrap());
    assert!(!json["data"]["items"].as_array().unwrap().is_empty());
    assert!(json["data"]["pagination"]["total_count"].as_i64().unwrap() > 0);
}

#[tokio::test]
async fn test_organization_search_invalid_data() {
    // Arrange
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;
    let jwt_token = auth_helper::create_admin_with_jwt(&app).await;

    // Act: 不正なソート順
    let request = Request::builder()
        .uri("/organizations/search?sort_order=invalid")
        .method("GET")
        .header("Authorization", format!("Bearer {}", jwt_token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_organization_search_forbidden() {
    // Arrange
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Act: 認証なしでアクセス
    let request = Request::builder()
        .uri("/organizations/search")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
