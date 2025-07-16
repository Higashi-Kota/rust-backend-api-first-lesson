// tests/integration/auth/signup_tests.rs

use axum::{
    body::{self, Body},
    http::{Request, StatusCode},
};
use serde_json::Value;
use tower::ServiceExt;

use crate::common::{app_helper, test_data};

#[tokio::test]
async fn test_user_signup_success() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    let signup_data = test_data::create_test_signup_data();

    let req = Request::builder()
        .uri("/auth/signup")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&signup_data).unwrap()))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::CREATED);
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let response: Value = serde_json::from_slice(&body).unwrap();

    // レスポンス構造の検証 - New ApiResponse format
    assert!(response["success"].as_bool().unwrap());
    assert!(response["data"].is_object());
    assert!(response["error"].is_null());
    assert!(response["meta"].is_object());

    let data = &response["data"];
    assert!(response["data"]["user"].is_object());
    assert!(response["data"]["tokens"].is_object());
    assert!(response["data"]["tokens"]["access_token"].is_string());
    assert!(response["data"]["tokens"]["refresh_token"].is_string());

    // ユーザー情報の検証
    let user = &data["user"];
    assert_eq!(user["email"], signup_data.email);
    assert_eq!(user["username"], signup_data.username);
    assert!(user["id"].is_string());
    assert!(user["created_at"].is_number());
    assert!(user["updated_at"].is_number());

    // パスワードが含まれていないことを確認
    assert!(user["password"].is_null() || !user.as_object().unwrap().contains_key("password"));

    // トークンが有効な形式であることを確認
    let access_token = data["tokens"]["access_token"].as_str().unwrap();
    let refresh_token = data["tokens"]["refresh_token"].as_str().unwrap();
    assert!(!access_token.is_empty());
    assert!(!refresh_token.is_empty());
}

#[tokio::test]
async fn test_user_signup_duplicate_email() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    let signup_data = test_data::create_signup_data_with_info(
        "duplicate@example.com",
        "user1",
        "MyUniqueP@ssw0rd91",
    );

    // 最初のユーザーを登録
    let req1 = Request::builder()
        .uri("/auth/signup")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&signup_data).unwrap()))
        .unwrap();

    let res1 = app.clone().oneshot(req1).await.unwrap();
    assert_eq!(res1.status(), StatusCode::CREATED);

    // 同じメールアドレスで再度登録を試行
    let duplicate_signup = test_data::create_signup_data_with_info(
        "duplicate@example.com",
        "user2", // 異なるユーザー名
        "MyUniqueP@ssw0rd91",
    );

    let req2 = Request::builder()
        .uri("/auth/signup")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(
            serde_json::to_string(&duplicate_signup).unwrap(),
        ))
        .unwrap();

    let res2 = app.clone().oneshot(req2).await.unwrap();

    assert_eq!(res2.status(), StatusCode::CONFLICT);
    let body = body::to_bytes(res2.into_body(), usize::MAX).await.unwrap();
    let error: Value = serde_json::from_slice(&body).unwrap();

    // Error response format
    assert!(!error["success"].as_bool().unwrap());
    assert!(error["data"].is_null());
    assert!(error["error"].is_object());
    assert_eq!(error["error"]["code"], "CONFLICT");
    assert!(error["error"]["message"]
        .as_str()
        .unwrap()
        .to_lowercase()
        .contains("email"));
}

#[tokio::test]
async fn test_user_signup_duplicate_username() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    let signup_data = test_data::create_signup_data_with_info(
        "user1@example.com",
        "duplicateuser",
        "MyUniqueP@ssw0rd91",
    );

    // 最初のユーザーを登録
    let req1 = Request::builder()
        .uri("/auth/signup")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&signup_data).unwrap()))
        .unwrap();

    let res1 = app.clone().oneshot(req1).await.unwrap();
    assert_eq!(res1.status(), StatusCode::CREATED);

    // 同じユーザー名で再度登録を試行
    let duplicate_signup = test_data::create_signup_data_with_info(
        "user2@example.com", // 異なるメールアドレス
        "duplicateuser",
        "MyUniqueP@ssw0rd91",
    );

    let req2 = Request::builder()
        .uri("/auth/signup")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(
            serde_json::to_string(&duplicate_signup).unwrap(),
        ))
        .unwrap();

    let res2 = app.clone().oneshot(req2).await.unwrap();

    assert_eq!(res2.status(), StatusCode::CONFLICT);
    let body = body::to_bytes(res2.into_body(), usize::MAX).await.unwrap();
    let error: Value = serde_json::from_slice(&body).unwrap();

    // Error response format
    assert!(!error["success"].as_bool().unwrap());
    assert!(error["data"].is_null());
    assert!(error["error"].is_object());
    assert_eq!(error["error"]["code"], "CONFLICT");
    assert!(error["error"]["message"]
        .as_str()
        .unwrap()
        .to_lowercase()
        .contains("username"));
}

#[tokio::test]
async fn test_user_signup_validation_empty_email() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    let invalid_signup = test_data::create_invalid_signup_data_empty_email();

    let req = Request::builder()
        .uri("/auth/signup")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&invalid_signup).unwrap()))
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
        // メール関連のエラーが含まれていることを確認
        let error_messages = details
            .iter()
            .map(|e| e["message"].as_str().unwrap_or(""))
            .collect::<Vec<&str>>();
        assert!(error_messages.iter().any(|msg| msg.contains("email")));
    }
}

#[tokio::test]
async fn test_user_signup_validation_invalid_email() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    let invalid_signup = test_data::create_invalid_signup_data_invalid_email();

    let req = Request::builder()
        .uri("/auth/signup")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&invalid_signup).unwrap()))
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
        // メール形式エラーが含まれていることを確認
        let error_messages = details
            .iter()
            .map(|e| e["message"].as_str().unwrap_or(""))
            .collect::<Vec<&str>>();
        assert!(error_messages
            .iter()
            .any(|msg| msg.contains("email") || msg.contains("format")));
    }
}

#[tokio::test]
async fn test_user_signup_validation_weak_password() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    let invalid_signup = test_data::create_invalid_signup_data_weak_password();

    let req = Request::builder()
        .uri("/auth/signup")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&invalid_signup).unwrap()))
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
        assert!(error_messages
            .iter()
            .any(|msg| msg.contains("password") || msg.contains("characters")));
    }
}

#[tokio::test]
async fn test_user_signup_validation_empty_username() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    let invalid_signup = test_data::create_invalid_signup_data_empty_username();

    let req = Request::builder()
        .uri("/auth/signup")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&invalid_signup).unwrap()))
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
        // ユーザー名関連のエラーが含まれていることを確認
        let error_messages = details
            .iter()
            .map(|e| e["message"].as_str().unwrap_or(""))
            .collect::<Vec<&str>>();
        assert!(error_messages
            .iter()
            .any(|msg| msg.contains("username") || msg.contains("characters")));
    }
}

#[tokio::test]
async fn test_user_signup_malformed_json() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    let malformed_json =
        r#"{"email": "test@example.com", "username": "testuser", "password": invalid}"#;

    let req = Request::builder()
        .uri("/auth/signup")
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
async fn test_user_signup_missing_content_type() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    let signup_data = test_data::create_test_signup_data();

    let req = Request::builder()
        .uri("/auth/signup")
        .method("POST")
        // Content-Type ヘッダーを省略
        .body(Body::from(serde_json::to_string(&signup_data).unwrap()))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();

    // Content-Type が指定されていない場合のレスポンスを検証
    // 実装によっては 400 Bad Request または 415 Unsupported Media Type が返される
    assert!(
        res.status() == StatusCode::BAD_REQUEST
            || res.status() == StatusCode::UNSUPPORTED_MEDIA_TYPE
    );
}

#[tokio::test]
async fn test_user_signup_multiple_validation_errors() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    // 複数のバリデーションエラーを持つデータ
    let invalid_signup = test_data::create_signup_data_with_info(
        "",    // 空のメール
        "",    // 空のユーザー名
        "123", // 弱いパスワード
    );

    let req = Request::builder()
        .uri("/auth/signup")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&invalid_signup).unwrap()))
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

        // 複数のエラーが返されることを確認
        assert!(errors.len() >= 3); // email, username, password のエラー

        let error_messages = errors
            .iter()
            .map(|e| e["message"].as_str().unwrap_or(""))
            .collect::<Vec<&str>>();

        // 各フィールドのエラーが含まれていることを確認
        assert!(error_messages.iter().any(|msg| msg.contains("email")));
        assert!(error_messages.iter().any(|msg| msg.contains("username")));
        assert!(error_messages.iter().any(|msg| msg.contains("password")));
    }
}
