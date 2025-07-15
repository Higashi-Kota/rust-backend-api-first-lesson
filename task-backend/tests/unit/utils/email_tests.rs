// tests/unit/utils/email_tests.rs

use task_backend::utils::email::{EmailConfig, EmailMessage, EmailProvider, EmailService};

// メール関連のユニットテスト（既存のsrc/utils/email.rsのテストを拡張）

#[tokio::test]
async fn test_email_format_validation_with_service() {
    // Arrange: メールフォーマットバリデーションをテスト
    let config = EmailConfig {
        provider: EmailProvider::Development,
        ..Default::default()
    };
    let email_service = EmailService::new(config).unwrap();

    let valid_emails = vec![
        "test@example.com",
        "user.name@example.org",
        "user+tag@domain.co.uk",
        "simple@test.net",
    ];

    let invalid_emails = vec![
        "",
        "invalid",
        "invalid@",
        "@invalid.com",
        "user@",
        "@domain.com",
        "user@domain", // ドメインにドットがない
    ];

    // Act & Assert: 有効なメールアドレスでの送信テスト
    for email in &valid_emails {
        let message = EmailMessage {
            to_email: (*email).to_string(),
            to_name: Some("Test User".to_string()),
            subject: "Test Subject".to_string(),
            html_body: "<p>Test HTML</p>".to_string(),
            text_body: "Test Text".to_string(),
            reply_to: None,
        };

        let result = email_service.send_email(message).await;
        assert!(
            result.is_ok(),
            "Sending to valid email '{}' should succeed in development mode",
            email
        );
    }

    // Act & Assert: 無効なメールアドレスでの送信テスト
    for email in &invalid_emails {
        let message = EmailMessage {
            to_email: (*email).to_string(),
            to_name: Some("Test User".to_string()),
            subject: "Test Subject".to_string(),
            html_body: "<p>Test HTML</p>".to_string(),
            text_body: "Test Text".to_string(),
            reply_to: None,
        };

        let result = email_service.send_email(message).await;
        assert!(
            result.is_err(),
            "Sending to invalid email '{}' should fail",
            email
        );
    }
}

#[tokio::test]
async fn test_email_template_generation() {
    // Arrange: メールテンプレート生成をテスト
    let config = EmailConfig {
        provider: EmailProvider::Development,
        from_email: "test@example.com".to_string(),
        from_name: "Test System".to_string(),
        ..Default::default()
    };
    let email_service = EmailService::new(config).unwrap();

    // Act: ウェルカムメールのテスト
    let result = email_service
        .send_welcome_email("user@example.com", "Test User")
        .await;

    assert!(result.is_ok(), "Welcome email should send successfully");

    // Act: パスワードリセットメールのテスト
    let result = email_service
        .send_password_reset_email(
            "user@example.com",
            "Test User",
            "reset-token-12345",
            "https://example.com/reset",
        )
        .await;

    assert!(
        result.is_ok(),
        "Password reset email should send successfully"
    );

    // Act: メール認証メールのテスト
    let result = email_service
        .send_email_verification_email(
            "user@example.com",
            "Test User",
            "verify-token-12345",
            "https://example.com/verify",
        )
        .await;

    assert!(
        result.is_ok(),
        "Email verification should send successfully"
    );

    // Act: セキュリティ通知メールのテスト
    let result = email_service
        .send_security_notification_email(
            "user@example.com",
            "Test User",
            "Login from new device",
            "IP: 192.168.1.1, Location: Tokyo, Japan",
        )
        .await;

    assert!(
        result.is_ok(),
        "Security notification should send successfully"
    );

    // Act: チーム招待メールのテスト
    let result = email_service
        .send_team_invitation_email(
            "user@example.com",
            "Test User",
            "Awesome Team",
            "Team Admin",
            "https://example.com/join/team123",
        )
        .await;

    assert!(result.is_ok(), "Team invitation should send successfully");
}
