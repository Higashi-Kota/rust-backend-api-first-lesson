// tests/unit/auth/repository/user_repository_tests.rs

// ユーザーリポジトリ関連のユニットテスト

use chrono::Utc;
use sea_orm::Set;
use task_backend::features::auth::dto::SignupRequest;
use task_backend::features::user::models::user::{self, SafeUser};
use task_backend::infrastructure::password::{Argon2Config, PasswordManager, PasswordPolicy};
use task_backend::utils::validation::common::validate_username;
use uuid::Uuid;
use validator::Validate;

#[tokio::test]
async fn test_user_model_creation_and_validation() {
    // AAAパターン: Arrange-Act-Assert

    // Arrange: テストデータを準備
    let email = "test@example.com";
    let username = "testuser";
    let password = "S3cur3P@ss!2024";
    let role_id = Uuid::new_v4();

    // Act: パスワードをハッシュ化
    let password_manager =
        PasswordManager::new(Argon2Config::default(), PasswordPolicy::default()).unwrap();
    let password_hash = password_manager
        .hash_password(password)
        .expect("Password hashing should succeed");

    // Act: ユーザーモデルを作成
    let user_model = user::ActiveModel {
        id: Set(Uuid::new_v4()),
        email: Set(email.to_string()),
        username: Set(username.to_string()),
        password_hash: Set(password_hash.clone()),
        is_active: Set(true),
        email_verified: Set(false),
        role_id: Set(role_id),
        subscription_tier: Set("free".to_string()),
        last_login_at: Set(None),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
        stripe_customer_id: Set(None),
    };

    // Assert: モデルのフィールドを確認
    assert_eq!(user_model.email.as_ref(), email);
    assert_eq!(user_model.username.as_ref(), username);
    assert!(!user_model.password_hash.as_ref().is_empty());
    assert!(*user_model.is_active.as_ref());
    assert!(!*user_model.email_verified.as_ref());

    // Act & Assert: バリデーション関数のテスト
    // Note: Email validation is done by validator crate's email attribute
    assert!(
        validate_username(username).is_ok(),
        "Username validation should pass"
    );

    // 無効なデータのテスト
    // Email validation using SignupRequest
    let mut signup_request = SignupRequest {
        email: "".to_string(),
        username: "validuser".to_string(),
        password: "ValidPass123!".to_string(),
    };
    assert!(
        signup_request.validate().is_err(),
        "Empty email should fail validation"
    );

    signup_request.email = "invalid".to_string();
    assert!(
        signup_request.validate().is_err(),
        "Invalid email should fail validation"
    );
    assert!(
        validate_username("").is_err(),
        "Empty username should fail validation"
    );
    assert!(
        validate_username("ab").is_err(),
        "Too short username should fail validation"
    );
}

#[tokio::test]
async fn test_safe_user_conversion() {
    // AAAパターン: Arrange-Act-Assert

    // Arrange: ユーザーモデルを作成
    let user = user::Model {
        id: Uuid::new_v4(),
        email: "user@example.com".to_string(),
        username: "safeuser".to_string(),
        password_hash: "$2b$12$hash".to_string(),
        is_active: true,
        email_verified: true,
        role_id: Uuid::new_v4(),
        subscription_tier: "pro".to_string(),
        last_login_at: Some(Utc::now()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        stripe_customer_id: None,
    };

    // Act: SafeUserに変換
    let safe_user: SafeUser = user.clone().into();

    // Assert: パスワードハッシュが含まれていないことを確認
    assert_eq!(safe_user.id, user.id);
    assert_eq!(safe_user.email, user.email);
    assert_eq!(safe_user.username, user.username);
    assert_eq!(safe_user.is_active, user.is_active);
    assert_eq!(safe_user.email_verified, user.email_verified);
    assert_eq!(safe_user.role_id, user.role_id);
    assert_eq!(safe_user.subscription_tier, user.subscription_tier);
    assert_eq!(safe_user.last_login_at, user.last_login_at);

    // SafeUserにpassword_hashフィールドが存在しないことを確認
    // これはコンパイル時に保証されるが、テストで明示的にチェック
    let safe_user_json = serde_json::to_value(&safe_user).unwrap();
    assert!(!safe_user_json
        .as_object()
        .unwrap()
        .contains_key("password_hash"));
}

#[tokio::test]
async fn test_register_request_validation() {
    // AAAパターン: Arrange-Act-Assert

    // Arrange: 有効な登録リクエスト
    let valid_request = SignupRequest {
        email: "newuser@example.com".to_string(),
        password: "StrongPass123!".to_string(),
        username: "newuser".to_string(),
    };

    // Act & Assert: バリデーションが成功
    assert!(valid_request.validate().is_ok());

    // Arrange: 無効なリクエストのパターン
    let invalid_requests = vec![
        SignupRequest {
            email: "invalid-email".to_string(),
            password: "StrongPass123!".to_string(),
            username: "validuser".to_string(),
        },
        SignupRequest {
            email: "valid@example.com".to_string(),
            password: "weak".to_string(),
            username: "validuser".to_string(),
        },
        SignupRequest {
            email: "valid@example.com".to_string(),
            password: "StrongPass123!".to_string(),
            username: "a".to_string(), // 短すぎる
        },
    ];

    // Act & Assert: 各無効リクエストがバリデーションに失敗
    for (i, request) in invalid_requests.iter().enumerate() {
        assert!(
            request.validate().is_err(),
            "Invalid request {} should fail validation",
            i
        );
    }
}
