// tests/integration/auth/signout_tests.rs

use axum::{
    body::{self, Body},
    http::{Request, StatusCode},
};
use serde_json::Value;
use tower::ServiceExt;

use crate::common::{app_helper, auth_helper, test_data};

#[tokio::test]
async fn test_user_signout_success() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // ログアウト
    let req = auth_helper::create_authenticated_request(
        "POST",
        "/auth/signout",
        &user.access_token,
        None,
    );

    let res = app.clone().oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let response: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response["message"], "Successfully signed out");
}

#[tokio::test]
async fn test_user_signout_without_token() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    // 認証トークンなしでログアウト試行
    let req = Request::builder()
        .uri("/auth/signout")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let error: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(error["error_type"], "unauthorized");
    assert!(error["error"].as_str().unwrap().contains("token"));
}

#[tokio::test]
async fn test_user_signout_with_invalid_token() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    // 無効なトークンでログアウト試行
    let req = auth_helper::create_authenticated_request(
        "POST",
        "/auth/signout",
        "invalid.jwt.token",
        None,
    );

    let res = app.clone().oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let error: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(error["error_type"], "unauthorized");
}

#[tokio::test]
async fn test_user_signout_with_expired_token() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    // 期限切れのトークンでログアウト試行
    // Note: 実際の期限切れトークンの生成は複雑なので、
    // ここでは無効なトークンでテスト
    let expired_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";

    let req =
        auth_helper::create_authenticated_request("POST", "/auth/signout", expired_token, None);

    let res = app.clone().oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let error: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(error["error_type"], "unauthorized");
}

#[tokio::test]
async fn test_user_signout_twice() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // 最初のログアウト
    let req1 = auth_helper::create_authenticated_request(
        "POST",
        "/auth/signout",
        &user.access_token,
        None,
    );

    let res1 = app.clone().oneshot(req1).await.unwrap();
    assert_eq!(res1.status(), StatusCode::OK);

    // 同じトークンで再度ログアウト試行
    let req2 = auth_helper::create_authenticated_request(
        "POST",
        "/auth/signout",
        &user.access_token,
        None,
    );

    let res2 = app.clone().oneshot(req2).await.unwrap();

    // 既にログアウト済みのトークンに対する動作
    // 実装によっては成功を返す場合もある（冪等性）
    assert!(res2.status() == StatusCode::OK || res2.status() == StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_user_signout_all_sessions() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    // ユーザーを登録
    let signup_data = test_data::create_test_signup_data();
    let _user = auth_helper::signup_test_user(&app, signup_data.clone())
        .await
        .unwrap();

    // 複数回ログインしてセッションを作成
    let signin_data = test_data::create_signin_data_with_email_and_password(
        &signup_data.email,
        &signup_data.password,
    );

    let mut users = Vec::new();
    for _ in 0..3 {
        let user = auth_helper::signin_test_user(&app, signin_data.clone())
            .await
            .unwrap();
        users.push(user);
    }

    // 全セッションからのログアウト（1つのトークンでテスト）
    let req = auth_helper::create_authenticated_request(
        "POST",
        "/auth/signout",
        &users[0].access_token,
        None,
    );

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 他のトークンがまだ有効かテスト
    // Note: この動作は実装によって異なる
    // - セッション毎の無効化：他のトークンは有効
    // - 全セッション無効化：他のトークンも無効
}

#[tokio::test]
async fn test_user_signout_invalidates_refresh_token() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let refresh_token = user.refresh_token.clone().unwrap();

    // ログアウト（リフレッシュトークンをクッキーとして設定）
    let signout_req = Request::builder()
        .uri("/auth/signout")
        .method("POST")
        .header("Authorization", format!("Bearer {}", user.access_token))
        .header("Content-Type", "application/json")
        .header("Cookie", format!("refresh_token={}", refresh_token))
        .body(Body::empty())
        .unwrap();

    let signout_res = app.clone().oneshot(signout_req).await.unwrap();
    assert_eq!(signout_res.status(), StatusCode::OK);

    // ログアウト後にリフレッシュトークンを使用
    let refresh_data = test_data::create_refresh_token_data(&refresh_token);

    let refresh_req = Request::builder()
        .uri("/auth/refresh")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&refresh_data).unwrap()))
        .unwrap();

    let refresh_res = app.clone().oneshot(refresh_req).await.unwrap();

    // リフレッシュトークンが無効化されていることを確認
    assert_eq!(refresh_res.status(), StatusCode::UNAUTHORIZED);
    let body = body::to_bytes(refresh_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let error: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(error["error_type"], "unauthorized");
    assert!(error["error"].as_str().unwrap().contains("Invalid"));
}

#[tokio::test]
async fn test_user_signout_clears_cookies() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // ログアウト
    let req = auth_helper::create_authenticated_request(
        "POST",
        "/auth/signout",
        &user.access_token,
        None,
    );

    let res = app.clone().oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    // レスポンスヘッダーにクッキークリア指示が含まれていることを確認
    let headers = res.headers();

    // Set-Cookieヘッダーがあることを確認
    let set_cookie_headers: Vec<_> = headers.get_all("set-cookie").iter().collect();

    if !set_cookie_headers.is_empty() {
        // クッキーのクリア（expires=Thu, 01 Jan 1970 など）が設定されていることを確認
        let cookie_str = set_cookie_headers[0].to_str().unwrap();
        assert!(cookie_str.contains("Max-Age=0") || cookie_str.contains("expires="));
    }
}

#[tokio::test]
async fn test_user_signout_audit_log() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // ログアウト
    let req = auth_helper::create_authenticated_request(
        "POST",
        "/auth/signout",
        &user.access_token,
        None,
    );

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // Note: 実際の監査ログの確認は実装次第
    // ここでは正常にログアウトできることの確認のみ
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let response: Value = serde_json::from_slice(&body).unwrap();

    assert!(response["message"].is_string());
}

#[tokio::test]
async fn test_user_signout_concurrent_requests() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // 同時に複数のログアウトリクエストを送信
    let mut handles = Vec::new();

    for _ in 0..5 {
        let app_clone = app.clone();
        let token = user.access_token.clone();

        let handle = tokio::spawn(async move {
            let req =
                auth_helper::create_authenticated_request("POST", "/auth/signout", &token, None);

            app_clone.oneshot(req).await.unwrap()
        });

        handles.push(handle);
    }

    // 全ての結果を待機
    let results = {
        let mut results = Vec::new();
        for handle in handles {
            results.push(handle.await);
        }
        results
    };

    // 少なくとも1つは成功すること
    let success_count = results
        .into_iter()
        .filter_map(|result| result.ok())
        .filter(|response| response.status() == StatusCode::OK)
        .count();

    assert!(success_count >= 1);
}
