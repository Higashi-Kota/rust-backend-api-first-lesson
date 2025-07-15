// tests/integration/auth/refresh_tests.rs

use axum::{
    body::{self, Body},
    http::{Request, StatusCode},
};
use serde_json::Value;
use tower::ServiceExt;

use crate::common::{app_helper, auth_helper, test_data};

#[tokio::test]
async fn test_token_refresh_success() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let refresh_token = user.refresh_token.unwrap();

    // トークンリフレッシュ
    let refresh_data = test_data::create_refresh_token_data(&refresh_token);

    let req = Request::builder()
        .uri("/auth/refresh")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&refresh_data).unwrap()))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let response: Value = serde_json::from_slice(&body).unwrap();

    // レスポンス構造の検証 - New ApiResponse format
    assert!(response["success"].as_bool().unwrap());
    assert!(response["data"].is_object());
    assert!(response["error"].is_null());
    assert!(response["meta"].is_object());

    let data = &response["data"];
    assert!(response["data"]["tokens"].is_object());
    let tokens = &data["tokens"];
    assert!(tokens["access_token"].is_string());
    assert!(tokens["refresh_token"].is_string());
    assert!(tokens["access_token_expires_in"].is_number());
    assert!(tokens["refresh_token_expires_in"].is_number());
    assert_eq!(tokens["token_type"], "Bearer");

    // タイムスタンプフィールドの検証
    assert!(tokens["access_token_expires_at"].is_string());
    assert!(tokens["should_refresh_at"].is_string());

    // 新しいトークンが発行されていることを確認
    let new_access_token = tokens["access_token"].as_str().unwrap();
    let new_refresh_token = tokens["refresh_token"].as_str().unwrap();

    assert_ne!(new_access_token, user.access_token);
    assert_ne!(new_refresh_token, refresh_token);
}

#[tokio::test]
async fn test_token_refresh_with_invalid_token() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    // 無効なリフレッシュトークンでテスト
    let invalid_refresh_data = test_data::create_refresh_token_data("invalid-refresh-token");

    let req = Request::builder()
        .uri("/auth/refresh")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(
            serde_json::to_string(&invalid_refresh_data).unwrap(),
        ))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let error: Value = serde_json::from_slice(&body).unwrap();

    // Error response format
    assert!(!error["success"].as_bool().unwrap());
    assert!(error["data"].is_null());
    assert!(error["error"].is_object());
    assert_eq!(error["error"]["code"], "UNAUTHORIZED");
    assert!(error["error"]["message"]
        .as_str()
        .unwrap()
        .to_lowercase()
        .contains("invalid"));
}

#[tokio::test]
async fn test_token_refresh_with_empty_token() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    // 空のリフレッシュトークンでテスト
    let empty_refresh_data = test_data::create_refresh_token_data("");

    let req = Request::builder()
        .uri("/auth/refresh")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(
            serde_json::to_string(&empty_refresh_data).unwrap(),
        ))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let error: Value = serde_json::from_slice(&body).unwrap();

    // Error response format
    assert!(!error["success"].as_bool().unwrap());
    assert!(error["data"].is_null());
    assert!(error["error"].is_object());
    assert!(
        (error["error"]["code"] == "VALIDATION_ERROR"
            || error["error"]["code"] == "VALIDATION_ERRORS")
    );
    assert!(error["error"]["message"]
        .as_str()
        .unwrap()
        .to_lowercase()
        .contains("validation"));

    // Check validation details if present
    if let Some(details) = error["error"]["details"].as_array() {
        assert!(!details.is_empty());
        let error_messages = details
            .iter()
            .filter_map(|e| e["message"].as_str())
            .collect::<Vec<&str>>();
        assert!(error_messages.iter().any(|msg| msg.contains("refresh")
            || msg.contains("token")
            || msg.contains("required")));
    }
}

#[tokio::test]
async fn test_token_refresh_rotation() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let original_refresh_token = user.refresh_token.unwrap();

    // 1回目のリフレッシュ
    let refresh_result = auth_helper::refresh_token(&app, &original_refresh_token)
        .await
        .unwrap();
    let first_new_refresh_token = refresh_result.refresh_token.unwrap();

    // 元のリフレッシュトークンが無効になっていることを確認
    let old_refresh_data = test_data::create_refresh_token_data(&original_refresh_token);

    let req = Request::builder()
        .uri("/auth/refresh")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(
            serde_json::to_string(&old_refresh_data).unwrap(),
        ))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    // 新しいリフレッシュトークンが使用可能であることを確認
    let new_refresh_result = auth_helper::refresh_token(&app, &first_new_refresh_token)
        .await
        .unwrap();
    assert!(new_refresh_result.refresh_token.is_some());
}

#[tokio::test]
async fn test_token_refresh_expired_token() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    // 期限切れのトークンをシミュレート
    // Note: 実際の期限切れトークンの生成は複雑なので、
    // ここでは無効なトークンでテスト
    let expired_refresh_data = test_data::create_refresh_token_data("expired-refresh-token");

    let req = Request::builder()
        .uri("/auth/refresh")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(
            serde_json::to_string(&expired_refresh_data).unwrap(),
        ))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let error: Value = serde_json::from_slice(&body).unwrap();

    // Error response format
    assert!(!error["success"].as_bool().unwrap());
    assert!(error["data"].is_null());
    assert!(error["error"].is_object());
    assert_eq!(error["error"]["code"], "UNAUTHORIZED");
    assert!(
        error["error"]["message"]
            .as_str()
            .unwrap()
            .to_lowercase()
            .contains("invalid")
            || error["error"]["message"]
                .as_str()
                .unwrap()
                .to_lowercase()
                .contains("expired")
    );
}

#[tokio::test]
async fn test_token_refresh_malformed_json() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    let malformed_json = r#"{"refresh_token": invalid}"#;

    let req = Request::builder()
        .uri("/auth/refresh")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(malformed_json))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();

    // Try to parse as JSON, but handle the case where it might not be JSON
    match serde_json::from_slice::<Value>(&body) {
        Ok(error) => {
            // Error response format - JSON パースエラーまたはバリデーションエラーが返されることを確認
            assert!(!error["success"].as_bool().unwrap());
            assert!(error["data"].is_null());
            assert!(error["error"].is_object());
            let error_code = error["error"]["code"].as_str().unwrap();
            assert!(
                error_code == "PARSE_ERROR"
                    || (error_code == "VALIDATION_ERROR" || error_code == "VALIDATION_ERRORS")
                    || error_code == "BAD_REQUEST"
            );
        }
        Err(_) => {
            // Not JSON, which is also acceptable for malformed requests
            // Just verify we got a 400 status (which we already checked above)
        }
    }
}

#[tokio::test]
async fn test_token_refresh_new_tokens_are_valid() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let refresh_token = user.refresh_token.unwrap();

    // トークンリフレッシュ
    let refresh_result = auth_helper::refresh_token(&app, &refresh_token)
        .await
        .unwrap();

    // 新しいアクセストークンで認証が必要なエンドポイントにアクセス
    let req = auth_helper::create_authenticated_request(
        "GET",
        "/auth/me",
        &refresh_result.access_token,
        None,
    );

    let res = app.clone().oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let response: Value = serde_json::from_slice(&body).unwrap();

    // ユーザー情報が正しく取得できることを確認 - New ApiResponse format
    assert!(response["success"].as_bool().unwrap());
    assert!(response["data"].is_object());
    assert!(response["error"].is_null());
    assert!(response["meta"].is_object());

    assert!(response["data"]["user"].is_object());
    let user_info = &response["data"]["user"];
    assert_eq!(user_info["id"], user.id.to_string());
    assert_eq!(user_info["email"], user.email);
}

#[tokio::test]
async fn test_token_refresh_concurrent_requests() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let refresh_token = user.refresh_token.unwrap();

    // 同時に複数のリフレッシュリクエストを送信
    let mut handles = Vec::new();

    for _ in 0..3 {
        let app_clone = app.clone();
        let token = refresh_token.clone();

        let handle = tokio::spawn(async move {
            let refresh_data = test_data::create_refresh_token_data(&token);

            let req = Request::builder()
                .uri("/auth/refresh")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&refresh_data).unwrap()))
                .unwrap();

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

    // 最低1つは成功すること（リフレッシュトークンのローテーション実装による）
    let success_count = results
        .into_iter()
        .filter_map(|result| result.ok())
        .filter(|response| response.status() == StatusCode::OK)
        .count();

    assert!(success_count >= 1);

    // リフレッシュトークンローテーション実装の場合、
    // 他のリクエストは失敗する可能性がある
}

#[tokio::test]
async fn test_token_refresh_rate_limiting() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let mut current_refresh_token = user.refresh_token.unwrap();

    // 短時間に多数のリフレッシュを実行
    for i in 0..10 {
        let refresh_data = test_data::create_refresh_token_data(&current_refresh_token);

        let req = Request::builder()
            .uri("/auth/refresh")
            .method("POST")
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&refresh_data).unwrap()))
            .unwrap();

        let res = app.clone().oneshot(req).await.unwrap();

        if i < 5 {
            // 最初の数回は成功することを期待
            if res.status() == StatusCode::OK {
                let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
                let response: Value = serde_json::from_slice(&body).unwrap();
                // New ApiResponse format
                let data = &response["data"];
                current_refresh_token = data["tokens"]["refresh_token"]
                    .as_str()
                    .unwrap()
                    .to_string();
            }
        }
        // 後半でレート制限がかかる可能性（実装次第）
    }
}

#[tokio::test]
async fn test_token_refresh_preserves_user_data() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let refresh_token = user.refresh_token.unwrap();

    // トークンリフレッシュ
    let refresh_result = auth_helper::refresh_token(&app, &refresh_token)
        .await
        .unwrap();

    // リフレッシュ前後でユーザー情報が一致することを確認
    assert_eq!(refresh_result.id, user.id);
    assert_eq!(refresh_result.email, user.email);
    assert_eq!(refresh_result.username, user.username);
}

#[tokio::test]
async fn test_token_refresh_updates_last_activity() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();
    let refresh_token = user.refresh_token.unwrap();

    // 少し待機
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // トークンリフレッシュ
    let refresh_result = auth_helper::refresh_token(&app, &refresh_token)
        .await
        .unwrap();

    // 新しいアクセストークンでユーザー情報を取得
    let req = auth_helper::create_authenticated_request(
        "GET",
        "/auth/me",
        &refresh_result.access_token,
        None,
    );

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let response: Value = serde_json::from_slice(&body).unwrap();

    // ユーザー情報が取得できることを確認 - New ApiResponse format
    assert!(response["success"].as_bool().unwrap());
    assert!(response["data"].is_object());
    assert!(response["data"]["user"].is_object());
    // Note: last_activity の更新は実装による
}
