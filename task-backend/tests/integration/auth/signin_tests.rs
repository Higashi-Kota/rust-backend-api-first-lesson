// tests/integration/auth/signin_tests.rs

use axum::{
    body::{self, Body},
    http::{Request, StatusCode},
};
use serde_json::Value;
use tower::ServiceExt;

use crate::common::{app_helper, auth_helper, test_data};

#[tokio::test]
async fn test_user_signin_success_with_email() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    // ユーザーを事前に登録
    let signup_data = test_data::create_signup_data_with_info(
        "signin_test@example.com",
        "signinuser",
        "MyUniqueP@ssw0rd91",
    );
    let _user = auth_helper::signup_test_user(&app, signup_data.clone())
        .await
        .unwrap();

    // メールアドレスでログイン
    let signin_data = test_data::create_signin_data_with_email_and_password(
        "signin_test@example.com",
        "MyUniqueP@ssw0rd91",
    );

    let req = Request::builder()
        .uri("/auth/signin")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&signin_data).unwrap()))
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
    assert!(response["data"]["user"].is_object());
    assert!(response["data"]["tokens"]["access_token"].is_string());
    assert!(response["data"]["tokens"]["refresh_token"].is_string());

    // ユーザー情報の検証
    let user = &data["user"];
    assert_eq!(user["email"], signup_data.email);
    assert_eq!(user["username"], signup_data.username);
    assert!(user["id"].is_string());

    // パスワードが含まれていないことを確認
    assert!(user["password"].is_null() || !user.as_object().unwrap().contains_key("password"));

    // トークンが有効な形式であることを確認
    let access_token = data["tokens"]["access_token"].as_str().unwrap();
    let refresh_token = data["tokens"]["refresh_token"].as_str().unwrap();
    assert!(!access_token.is_empty());
    assert!(!refresh_token.is_empty());
}

#[tokio::test]
async fn test_user_signin_success_with_username() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    // ユーザーを事前に登録
    let signup_data = test_data::create_signup_data_with_info(
        "signin_test2@example.com",
        "signinuser2",
        "MyUniqueP@ssw0rd91",
    );
    let _user = auth_helper::signup_test_user(&app, signup_data.clone())
        .await
        .unwrap();

    // ユーザー名でログイン
    let signin_data = test_data::create_signin_data_with_email_and_password(
        "signinuser2", // ユーザー名で識別
        "MyUniqueP@ssw0rd91",
    );

    let req = Request::builder()
        .uri("/auth/signin")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&signin_data).unwrap()))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let response: Value = serde_json::from_slice(&body).unwrap();

    // ユーザー情報の検証
    let user = &response["data"]["user"];
    assert_eq!(user["email"], signup_data.email);
    assert_eq!(user["username"], signup_data.username);
}

#[tokio::test]
async fn test_user_signin_invalid_credentials() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    // ユーザーを事前に登録
    let signup_data = test_data::create_signup_data_with_info(
        "signin_test3@example.com",
        "signinuser3",
        "MyUniqueP@ssw0rd91",
    );
    let _user = auth_helper::signup_test_user(&app, signup_data)
        .await
        .unwrap();

    // 間違ったパスワードでログイン試行
    let invalid_signin = test_data::create_signin_data_with_email_and_password(
        "signin_test3@example.com",
        "WrongPassword123!",
    );

    let req = Request::builder()
        .uri("/auth/signin")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&invalid_signin).unwrap()))
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
        .contains("Invalid"));
}

#[tokio::test]
async fn test_user_signin_nonexistent_user() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    // 存在しないユーザーでログイン試行
    let nonexistent_signin = test_data::create_signin_data_with_email_and_password(
        "nonexistent@example.com",
        "MyUniqueP@ssw0rd91",
    );

    let req = Request::builder()
        .uri("/auth/signin")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(
            serde_json::to_string(&nonexistent_signin).unwrap(),
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
        .contains("Invalid"));
}

#[tokio::test]
async fn test_user_signin_validation_empty_identifier() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    let invalid_signin = test_data::create_signin_data_with_email_and_password(
        "", // 空の識別子
        "MyUniqueP@ssw0rd91",
    );

    let req = Request::builder()
        .uri("/auth/signin")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&invalid_signin).unwrap()))
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
    // Check validation details if present
    if let Some(details) = error["error"]["details"].as_array() {
        let errors = details;
        assert!(!errors.is_empty());
    }
    // Check validation details
    if let Some(details) = error["error"]["details"].as_array() {
        assert!(!details.is_empty());
        // 識別子関連のエラーが含まれていることを確認
        let error_messages = details
            .iter()
            .map(|e| e["message"].as_str().unwrap_or(""))
            .collect::<Vec<&str>>();
        assert!(error_messages.iter().any(|msg| msg.contains("identifier")
            || msg.contains("Email")
            || msg.contains("username")
            || msg.contains("required")));
    }
}

#[tokio::test]
async fn test_user_signin_validation_empty_password() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    let invalid_signin = test_data::create_signin_data_with_email_and_password(
        "valid@example.com",
        "", // 空のパスワード
    );

    let req = Request::builder()
        .uri("/auth/signin")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&invalid_signin).unwrap()))
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
    // Check validation details if present
    if let Some(details) = error["error"]["details"].as_array() {
        let errors = details;
        assert!(!errors.is_empty());
    }
    // Check validation details
    if let Some(details) = error["error"]["details"].as_array() {
        assert!(!details.is_empty());
        // パスワード関連のエラーが含まれていることを確認
        let error_messages = details
            .iter()
            .map(|e| e["message"].as_str().unwrap_or(""))
            .collect::<Vec<&str>>();
        assert!(error_messages.iter().any(|msg| msg.contains("password")
            || msg.contains("Password")
            || msg.contains("required")));
    }
}

#[tokio::test]
async fn test_user_signin_malformed_json() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    let malformed_json = r#"{"identifier": "test@example.com", "password": invalid}"#;

    let req = Request::builder()
        .uri("/auth/signin")
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
            // JSON パースエラーまたはバリデーションエラーが返されることを確認
            assert!(
                error["error"]["code"] == "PARSE_ERROR"
                    || (error["error"]["code"] == "VALIDATION_ERROR"
                        || error["error"]["code"] == "VALIDATION_ERRORS")
                    || error["error"]["code"] == "BAD_REQUEST"
            );
        }
        Err(_) => {
            // Not JSON, which is also acceptable for malformed requests
            // Just verify we got a 400 status (which we already checked above)
        }
    }
}

#[tokio::test]
async fn test_user_signin_rate_limiting() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    // 複数回の失敗したログイン試行（レート制限のテスト）
    let invalid_signin = test_data::create_signin_data_with_email_and_password(
        "rate_limit_test@example.com",
        "WrongPassword123!",
    );

    // 複数回失敗したログインを実行
    for i in 0..10 {
        let req = Request::builder()
            .uri("/auth/signin")
            .method("POST")
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&invalid_signin).unwrap()))
            .unwrap();

        let res = app.clone().oneshot(req).await.unwrap();

        // 最初の数回は通常の認証エラー
        if i < 5 {
            assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
        }
        // 後半でレート制限がかかる可能性（実装次第）
        // Note: 実際のレート制限の実装により、この部分は調整が必要
    }
}

#[tokio::test]
async fn test_user_signin_refresh_token_generation() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    // ユーザーを事前に登録
    let signup_data = test_data::create_signup_data_with_info(
        "refresh_test@example.com",
        "refreshuser",
        "MyUniqueP@ssw0rd91",
    );
    let _user = auth_helper::signup_test_user(&app, signup_data.clone())
        .await
        .unwrap();

    // ログイン
    let signin_data = test_data::create_signin_data_with_email_and_password(
        "refresh_test@example.com",
        "MyUniqueP@ssw0rd91",
    );

    let req = Request::builder()
        .uri("/auth/signin")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&signin_data).unwrap()))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let response: Value = serde_json::from_slice(&body).unwrap();

    // リフレッシュトークンが生成されていることを確認
    let refresh_token = response["data"]["tokens"]["refresh_token"]
        .as_str()
        .unwrap();
    assert!(!refresh_token.is_empty());

    // リフレッシュトークンのフォーマット検証（UUIDである可能性が高い）
    assert!(refresh_token.len() > 10); // 最低限の長さチェック
}

#[tokio::test]
async fn test_user_signin_multiple_sessions() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    // ユーザーを事前に登録
    let signup_data = test_data::create_signup_data_with_info(
        "multi_session@example.com",
        "multisessionuser",
        "MyUniqueP@ssw0rd91",
    );
    let _user = auth_helper::signup_test_user(&app, signup_data.clone())
        .await
        .unwrap();

    // 複数回ログインして、それぞれ異なるトークンが発行されることを確認
    let signin_data = test_data::create_signin_data_with_email_and_password(
        "multi_session@example.com",
        "MyUniqueP@ssw0rd91",
    );

    let mut tokens = Vec::new();

    for _ in 0..3 {
        let req = Request::builder()
            .uri("/auth/signin")
            .method("POST")
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&signin_data).unwrap()))
            .unwrap();

        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
        let response: Value = serde_json::from_slice(&body).unwrap();

        let access_token = response["data"]["tokens"]["access_token"]
            .as_str()
            .unwrap()
            .to_string();
        let refresh_token = response["data"]["tokens"]["refresh_token"]
            .as_str()
            .unwrap()
            .to_string();

        tokens.push((access_token, refresh_token));
    }

    // すべてのトークンが異なることを確認
    for i in 0..tokens.len() {
        for j in i + 1..tokens.len() {
            assert_ne!(tokens[i].0, tokens[j].0); // アクセストークンが異なる
            assert_ne!(tokens[i].1, tokens[j].1); // リフレッシュトークンが異なる
        }
    }
}
