// tests/unit/auth/repository/password_reset_token_repository_tests.rs

// パスワードリセットトークンリポジトリ関連のユニットテスト

use chrono::{Duration, Utc};
use sea_orm::Set;
use task_backend::features::auth::dto::{PasswordResetRequest, PasswordResetRequestRequest};
use task_backend::features::auth::models::password_reset_token;
use uuid::Uuid;
use validator::Validate;

#[tokio::test]
async fn test_password_reset_token_creation_and_validation() {
    // AAAパターン: Arrange-Act-Assert

    // Arrange: テストデータを準備
    let user_id = Uuid::new_v4();
    let valid_token = "abcdef0123456789abcdef0123456789"; // 32文字
    let short_token = "a"; // 1文字（最小長を満たす）
    let empty_token = ""; // 空

    // Act: トークンモデルを作成
    let token_model = password_reset_token::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user_id),
        token_hash: Set(valid_token.to_string()),
        expires_at: Set(Utc::now() + Duration::hours(1)),
        is_used: Set(false),
        ip_address: Set("127.0.0.1".to_string()),
        user_agent: Set(Some("Test User Agent".to_string())),
        requested_from: Set(Some("test_client".to_string())),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };

    // Assert: トークンの基本情報を確認
    assert_eq!(
        token_model.token_hash.as_ref(),
        valid_token,
        "Token should match"
    );
    assert!(
        !token_model.is_used.as_ref(),
        "Token should not be used initially"
    );
    assert!(
        token_model.expires_at.as_ref() > &Utc::now(),
        "Token should not be expired"
    );

    // Act & Assert: パスワードリセットリクエストのバリデーション
    let reset_request = PasswordResetRequestRequest {
        email: "user@example.com".to_string(),
    };
    assert!(
        reset_request.validate().is_ok(),
        "Valid email should pass validation"
    );

    let invalid_reset_request = PasswordResetRequestRequest {
        email: "invalid-email".to_string(),
    };
    assert!(
        invalid_reset_request.validate().is_err(),
        "Invalid email should fail validation"
    );

    // Act & Assert: パスワードリセット確認リクエストのバリデーション
    let confirm_request = PasswordResetRequest {
        token: valid_token.to_string(),
        new_password: "NewPassword123!".to_string(),
    };
    assert!(
        confirm_request.validate().is_ok(),
        "Valid confirm request should pass validation"
    );

    // 短いトークンのテスト（1文字はOK）
    let short_confirm_request = PasswordResetRequest {
        token: short_token.to_string(),
        new_password: "NewPassword123!".to_string(),
    };
    assert!(
        short_confirm_request.validate().is_ok(),
        "1 character token should pass validation (MIN_LENGTH = 1)"
    );

    // 空のトークンのテスト
    let empty_token_request = PasswordResetRequest {
        token: empty_token.to_string(),
        new_password: "NewPassword123!".to_string(),
    };
    assert!(
        empty_token_request.validate().is_err(),
        "Empty token should fail validation"
    );
}

#[tokio::test]
async fn test_password_reset_token_expiration() {
    // AAAパターン: Arrange-Act-Assert

    // Arrange: 期限切れのトークンを作成
    let expired_token = password_reset_token::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(Uuid::new_v4()),
        token_hash: Set("expired_token_12345678901234567890".to_string()),
        expires_at: Set(Utc::now() - Duration::hours(1)), // 1時間前に期限切れ
        is_used: Set(false),
        ip_address: Set("127.0.0.1".to_string()),
        user_agent: Set(Some("Test User Agent".to_string())),
        requested_from: Set(Some("test_client".to_string())),
        created_at: Set(Utc::now() - Duration::hours(2)),
        updated_at: Set(Utc::now() - Duration::hours(2)),
    };

    // Act & Assert: 期限切れを確認
    assert!(
        expired_token.expires_at.as_ref() < &Utc::now(),
        "Token should be expired"
    );

    // Arrange: 使用済みトークンを作成
    let used_token = password_reset_token::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(Uuid::new_v4()),
        token_hash: Set("used_token_1234567890123456789012".to_string()),
        expires_at: Set(Utc::now() + Duration::hours(1)),
        is_used: Set(true), // 使用済み
        ip_address: Set("127.0.0.1".to_string()),
        user_agent: Set(Some("Test User Agent".to_string())),
        requested_from: Set(Some("test_client".to_string())),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };

    // Act & Assert: 使用済みフラグを確認
    assert!(
        *used_token.is_used.as_ref(),
        "Token should be marked as used"
    );
}
