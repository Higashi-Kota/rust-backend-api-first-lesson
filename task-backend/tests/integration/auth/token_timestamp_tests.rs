// tests/integration/auth/token_timestamp_tests.rs

use axum::{
    body::{self, Body},
    http::{Request, StatusCode},
};
use chrono::DateTime;
use serde_json::Value;
use tower::ServiceExt;

use crate::common::{app_helper, test_data};

#[tokio::test]
async fn test_signup_response_contains_token_timestamps() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    let signup_data = test_data::create_test_signup_data();

    let req = Request::builder()
        .uri("/auth/signup")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&signup_data).unwrap()))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // tokensオブジェクトが存在することを確認
    assert!(json.get("tokens").is_some());
    let tokens = json.get("tokens").unwrap();

    // タイムスタンプフィールドが存在することを確認
    assert!(tokens.get("access_token_expires_at").is_some());
    assert!(tokens.get("should_refresh_at").is_some());

    // ISO 8601形式であることを確認
    let expires_at_str = tokens
        .get("access_token_expires_at")
        .unwrap()
        .as_str()
        .unwrap();
    let refresh_at_str = tokens.get("should_refresh_at").unwrap().as_str().unwrap();

    let expires_at = DateTime::parse_from_rfc3339(expires_at_str);
    let refresh_at = DateTime::parse_from_rfc3339(refresh_at_str);

    assert!(
        expires_at.is_ok(),
        "access_token_expires_at should be valid ISO 8601"
    );
    assert!(
        refresh_at.is_ok(),
        "should_refresh_at should be valid ISO 8601"
    );

    // リフレッシュ時刻が有効期限より前であることを確認
    assert!(refresh_at.unwrap() < expires_at.unwrap());
}

#[tokio::test]
async fn test_signin_response_contains_token_timestamps() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    // ユーザー登録
    let signup_data = test_data::create_test_signup_data();
    let req = Request::builder()
        .uri("/auth/signup")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&signup_data).unwrap()))
        .unwrap();

    let _ = app.clone().oneshot(req).await.unwrap();

    // ログイン
    let signin_data = serde_json::json!({
        "identifier": signup_data.email,
        "password": signup_data.password
    });

    let req = Request::builder()
        .uri("/auth/signin")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(signin_data.to_string()))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    let tokens = json.get("tokens").unwrap();

    // タイムスタンプフィールドの検証
    validate_token_timestamps(tokens);
}

// ヘルパー関数
fn validate_token_timestamps(tokens: &Value) {
    assert!(tokens.get("access_token_expires_at").is_some());
    assert!(tokens.get("should_refresh_at").is_some());

    let expires_at_str = tokens
        .get("access_token_expires_at")
        .unwrap()
        .as_str()
        .unwrap();
    let refresh_at_str = tokens.get("should_refresh_at").unwrap().as_str().unwrap();

    let expires_at = DateTime::parse_from_rfc3339(expires_at_str)
        .expect("access_token_expires_at should be valid ISO 8601");
    let refresh_at = DateTime::parse_from_rfc3339(refresh_at_str)
        .expect("should_refresh_at should be valid ISO 8601");

    assert!(
        refresh_at < expires_at,
        "should_refresh_at should be before access_token_expires_at"
    );
}
