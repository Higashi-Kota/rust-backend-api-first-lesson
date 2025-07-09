// tests/unit/auth/service/auth_service_tests.rs

use task_backend::features::auth::dto::{SigninRequest, SignupRequest};
use task_backend::infrastructure::password::{Argon2Config, PasswordManager, PasswordPolicy};
use validator::Validate;

#[tokio::test]
async fn test_password_strength_validation() {
    // AAAパターン: Arrange-Act-Assert

    // Arrange: テストデータを準備
    let weak_passwords = ["", "123", "password", "12345678", "qwerty"];
    let strong_passwords = [
        "MyS3cur3!P@ss",
        "Str0ng!P@ss#2",
        "C0mpl3x@W0rd!",
        "S@f3ty1stNow!",
    ];

    // Act & Assert: 弱いパスワードのテスト
    let password_manager =
        PasswordManager::new(Argon2Config::default(), PasswordPolicy::default()).unwrap();

    for password in &weak_passwords {
        let result = password_manager.validate_password_strength(password);
        assert!(
            result.is_err(),
            "Password '{}' should not be considered strong",
            password
        );
    }

    // Act & Assert: 強いパスワードのテスト
    for password in &strong_passwords {
        let result = password_manager.validate_password_strength(password);
        assert!(
            result.is_ok(),
            "Password '{}' should be considered strong. Error: {:?}",
            password,
            result.err()
        );

        // 強いパスワードの要件を確認（文字を直接チェック）
        assert!(
            password.chars().any(|c| c.is_uppercase()),
            "Should have uppercase letters"
        );
        assert!(
            password.chars().any(|c| c.is_lowercase()),
            "Should have lowercase letters"
        );
        assert!(
            password.chars().any(|c| c.is_ascii_digit()),
            "Should have numbers"
        );
        assert!(
            password
                .chars()
                .any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c)),
            "Should have special characters"
        );
    }
}

#[tokio::test]
async fn test_email_validation_with_validator() {
    // AAAパターン: Arrange-Act-Assert

    // Arrange: テストデータを準備
    let invalid_emails = [
        "",
        "invalid",
        "invalid@",
        "@invalid.com",
        "invalid@.com",
        "invalid@domain",
        "user @example.com",
        "user@",
    ];
    let valid_emails = [
        "valid@example.com",
        "user.name@example.com",
        "test+tag@example.co.uk",
        "admin@localhost.localdomain",
    ];

    // Act & Assert: 無効なメールアドレスのテスト
    for email in &invalid_emails {
        // リクエストDTOでのバリデーションテスト
        let signup_request = SignupRequest {
            email: (*email).to_string(),
            password: "Password123!".to_string(),
            username: "testuser".to_string(),
        };
        assert!(
            signup_request.validate().is_err(),
            "SignupRequest with email '{}' should fail validation",
            email
        );
    }

    // Act & Assert: 有効なメールアドレスのテスト
    for email in &valid_emails {
        // リクエストDTOでのバリデーションテスト
        let signup_request = SignupRequest {
            email: (*email).to_string(),
            password: "Password123!".to_string(),
            username: "testuser".to_string(),
        };
        assert!(
            signup_request.validate().is_ok(),
            "SignupRequest with email '{}' should pass validation",
            email
        );
    }
}

#[tokio::test]
async fn test_signin_request_validation() {
    // AAAパターン: Arrange-Act-Assert

    // Arrange: 有効なログインリクエスト
    let valid_request = SigninRequest {
        identifier: "user@example.com".to_string(),
        password: "Password123!".to_string(),
    };

    // Act: バリデーション実行
    let result = valid_request.validate();

    // Assert: 成功を確認
    assert!(
        result.is_ok(),
        "Valid signin request should pass validation"
    );

    // Arrange: 無効なログインリクエスト（空のパスワード）
    let invalid_request = SigninRequest {
        identifier: "user@example.com".to_string(),
        password: "".to_string(),
    };

    // Act: バリデーション実行
    let result = invalid_request.validate();

    // Assert: 失敗を確認
    assert!(
        result.is_err(),
        "Signin request with empty password should fail validation"
    );
}
