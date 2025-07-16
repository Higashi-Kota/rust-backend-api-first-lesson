// tests/integration/auth/token_timestamp_tests.rs

use axum::{
    body::{self, Body},
    http::{Request, StatusCode},
};
use chrono::Utc;
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
    assert!(json["data"].get("tokens").is_some());
    let tokens = json["data"].get("tokens").unwrap();

    // タイムスタンプフィールドが存在することを確認
    assert!(tokens.get("access_token_expires_at").is_some());
    assert!(tokens.get("should_refresh_at").is_some());

    // Unix timestampであることを確認
    let expires_at_ts = tokens
        .get("access_token_expires_at")
        .unwrap()
        .as_i64()
        .unwrap();
    let refresh_at_ts = tokens.get("should_refresh_at").unwrap().as_i64().unwrap();

    // タイムスタンプが有効な範囲であることを確認
    let now = Utc::now().timestamp();
    assert!(
        expires_at_ts > now,
        "access_token_expires_at should be in the future"
    );
    assert!(
        refresh_at_ts > now,
        "should_refresh_at should be in the future"
    );

    // リフレッシュ時刻が有効期限より前であることを確認
    assert!(refresh_at_ts < expires_at_ts);
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
    let tokens = json["data"].get("tokens").unwrap();

    // タイムスタンプフィールドの検証
    validate_token_timestamps(tokens);
}

// ヘルパー関数
fn validate_token_timestamps(tokens: &Value) {
    assert!(tokens.get("access_token_expires_at").is_some());
    assert!(tokens.get("should_refresh_at").is_some());

    let expires_at_ts = tokens
        .get("access_token_expires_at")
        .unwrap()
        .as_i64()
        .expect("access_token_expires_at should be a number");
    let refresh_at_ts = tokens
        .get("should_refresh_at")
        .unwrap()
        .as_i64()
        .expect("should_refresh_at should be a number");

    let now = Utc::now().timestamp();
    assert!(
        expires_at_ts > now,
        "access_token_expires_at should be in the future"
    );
    assert!(
        refresh_at_ts > now,
        "should_refresh_at should be in the future"
    );
    assert!(
        refresh_at_ts < expires_at_ts,
        "should_refresh_at should be before access_token_expires_at"
    );
}
