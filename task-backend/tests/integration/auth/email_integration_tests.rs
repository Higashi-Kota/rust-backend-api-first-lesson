// tests/integration/auth/email_integration_tests.rs

use crate::common;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use serde_json::json;
use tower::ServiceExt;

/// パスワードリセット機能のメール送信統合テスト
#[tokio::test]
async fn test_password_reset_email_integration() {
    let (app, _schema_name, _db) = common::app_helper::setup_auth_app().await;

    // 1. ユーザーを作成
    let signup_request = Request::builder()
        .method(Method::POST)
        .uri("/auth/signup")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "email": "test@example.com",
                "username": "testuser",
                "password": "MyUniqueP@ssw0rd91"
            })
            .to_string(),
        ))
        .unwrap();

    let signup_response = app.clone().oneshot(signup_request).await.unwrap();
    let status = signup_response.status();
    if status != StatusCode::CREATED {
        let error_body = axum::body::to_bytes(signup_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let error_text = String::from_utf8(error_body.to_vec()).unwrap();
        panic!("Signup failed with status {}: {}", status, error_text);
    }

    // 2. パスワードリセットをリクエスト（メール送信が発生するはず）
    let reset_request = Request::builder()
        .method(Method::POST)
        .uri("/auth/forgot-password")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "email": "test@example.com"
            })
            .to_string(),
        ))
        .unwrap();

    let reset_response = app.clone().oneshot(reset_request).await.unwrap();
    assert_eq!(reset_response.status(), StatusCode::OK);

    // 3. レスポンスの内容を確認
    let response_body = axum::body::to_bytes(reset_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response_text = String::from_utf8(response_body.to_vec()).unwrap();
    assert!(
        response_text.contains("If the email address exists, a password reset link has been sent")
    );
}

/// 存在しないメールアドレスでのパスワードリセット要求テスト
#[tokio::test]
async fn test_password_reset_nonexistent_email() {
    let (app, _schema_name, _db) = common::app_helper::setup_auth_app().await;

    // 存在しないメールアドレスでパスワードリセットをリクエスト
    let reset_request = Request::builder()
        .method(Method::POST)
        .uri("/auth/forgot-password")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "email": "nonexistent@example.com"
            })
            .to_string(),
        ))
        .unwrap();

    let reset_response = app.oneshot(reset_request).await.unwrap();

    // セキュリティ上、存在しないメールでも同じレスポンスを返す
    assert_eq!(reset_response.status(), StatusCode::OK);

    let response_body = axum::body::to_bytes(reset_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response_text = String::from_utf8(response_body.to_vec()).unwrap();
    assert!(
        response_text.contains("If the email address exists, a password reset link has been sent")
    );
}

/// メールアドレス形式のバリデーションテスト
#[tokio::test]
async fn test_password_reset_invalid_email_format() {
    let (app, _schema_name, _db) = common::app_helper::setup_auth_app().await;

    // 無効なメールアドレス形式でリクエスト
    let reset_request = Request::builder()
        .method(Method::POST)
        .uri("/auth/forgot-password")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "email": "invalid-email-format"
            })
            .to_string(),
        ))
        .unwrap();

    let reset_response = app.oneshot(reset_request).await.unwrap();
    assert_eq!(reset_response.status(), StatusCode::BAD_REQUEST);
}

/// メール送信機能の開発モードテスト
#[tokio::test]
async fn test_email_service_development_mode() {
    use task_backend::infrastructure::email::{EmailConfig, EmailMessage, EmailService};

    // 開発モード（デフォルト）のEmailServiceを作成
    let email_service = EmailService::new(EmailConfig {
        development_mode: true,
        ..Default::default()
    })
    .unwrap();

    // テストメッセージを作成
    let test_message = EmailMessage {
        to_email: "test@example.com".to_string(),
        to_name: Some("Test User".to_string()),
        subject: "Test Email".to_string(),
        html_body: "<p>This is a test email</p>".to_string(),
        text_body: "This is a test email".to_string(),
        reply_to: None,
    };

    // 開発モードではエラーが発生せず、ログに出力される
    let result = email_service.send_email(test_message).await;
    assert!(result.is_ok());
}

/// パスワードリセットメール送信の具体的なテスト
#[tokio::test]
async fn test_password_reset_email_content() {
    use task_backend::infrastructure::email::{EmailConfig, EmailService};

    // 開発モードのEmailServiceを作成
    let email_service = EmailService::new(EmailConfig {
        development_mode: true,
        ..Default::default()
    })
    .unwrap();

    // パスワードリセットメールを送信（開発モードなのでログ出力のみ）
    let result = email_service
        .send_password_reset_email(
            "test@example.com",
            "Test User",
            "test_token_12345",
            "http://localhost:5000/reset-password",
        )
        .await;

    assert!(result.is_ok());
}

/// ウェルカムメール送信テスト
#[tokio::test]
async fn test_welcome_email_sending() {
    use task_backend::infrastructure::email::{EmailConfig, EmailService};

    let email_service = EmailService::new(EmailConfig {
        development_mode: true,
        ..Default::default()
    })
    .unwrap();

    let result = email_service
        .send_welcome_email("newuser@example.com", "New User")
        .await;

    assert!(result.is_ok());
}

/// メール認証メール送信テスト
#[tokio::test]
async fn test_email_verification_sending() {
    use task_backend::infrastructure::email::{EmailConfig, EmailService};

    let email_service = EmailService::new(EmailConfig {
        development_mode: true,
        ..Default::default()
    })
    .unwrap();

    let result = email_service
        .send_email_verification_email(
            "verify@example.com",
            "Verify User",
            "verify_token_67890",
            "http://localhost:5000/verify-email",
        )
        .await;

    assert!(result.is_ok());
}

/// セキュリティ通知メール送信テスト
#[tokio::test]
async fn test_security_notification_sending() {
    use task_backend::infrastructure::email::{EmailConfig, EmailService};

    let email_service = EmailService::new(EmailConfig {
        development_mode: true,
        ..Default::default()
    })
    .unwrap();

    let result = email_service
        .send_security_notification_email(
            "security@example.com",
            "Security User",
            "Password Changed",
            "Your password was changed from IP 192.168.1.1",
        )
        .await;

    assert!(result.is_ok());
}

/// EmailConfigのデフォルト設定テスト
#[tokio::test]
async fn test_email_config_from_environment() {
    use task_backend::infrastructure::email::EmailConfig;

    let config = EmailConfig::default();
    assert!(config.development_mode);
    assert_eq!(
        config.provider,
        task_backend::infrastructure::email::EmailProvider::Development
    );
}
