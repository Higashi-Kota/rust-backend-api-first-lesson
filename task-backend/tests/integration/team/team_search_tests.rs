// tests/integration/team/team_search_tests.rs

use axum::{
    body::{self, Body},
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;

use crate::common::{app_helper, auth_helper};

#[tokio::test]
async fn test_team_search_success() {
    // Arrange
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;
    let jwt_token = auth_helper::create_admin_with_jwt(&app).await;

    // まず1つチームを作成
    let team_data = json!({
        "name": "Test Team for Search",
        "description": "Team for testing search functionality"
    });

    let request = Request::builder()
        .uri("/teams")
        .method("POST")
        .header("Authorization", format!("Bearer {}", jwt_token))
        .header("Content-Type", "application/json")
        .body(Body::from(team_data.to_string()))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    // Act: 統一検索エンドポイントを使用
    let request = Request::builder()
        .uri("/teams/search?search=Test&page=1&per_page=10")
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
async fn test_team_search_invalid_data() {
    // Arrange
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;
    let jwt_token = auth_helper::create_admin_with_jwt(&app).await;

    // Act: 不正なページパラメータ
    let request = Request::builder()
        .uri("/teams/search?page=invalid&per_page=10")
        .method("GET")
        .header("Authorization", format!("Bearer {}", jwt_token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_team_search_forbidden() {
    // Arrange
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // Act: 認証なしでアクセス
    let request = Request::builder()
        .uri("/teams/search?search=Test")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
