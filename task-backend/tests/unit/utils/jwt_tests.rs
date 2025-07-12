// tests/unit/utils/jwt_tests.rs

use task_backend::core::subscription_tier::SubscriptionTier;
use task_backend::features::user::models::user::UserClaims;
use task_backend::infrastructure::jwt::{JwtConfig, JwtManager};
use uuid::Uuid;

// JWT関連のユニットテスト（既存のsrc/utils/jwt.rsのテストを拡張）

#[tokio::test]
async fn test_jwt_token_generation_and_structure() {
    // Arrange: JWTトークン生成と構造をテスト
    let config = JwtConfig {
        secret_key: "test-secret-key-must-be-at-least-32-characters-long".to_string(),
        access_token_expiry_minutes: 15,
        refresh_token_expiry_days: 7,
        issuer: "test-issuer".to_string(),
        audience: "test-audience".to_string(),
    };

    let jwt_manager = JwtManager::new(config).unwrap();

    let user_claims = UserClaims {
        user_id: Uuid::new_v4(),
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        is_active: true,
        email_verified: true,
        role_name: "member".to_string(),
        role: None,
        subscription_tier: SubscriptionTier::Free,
    };

    // Act: アクセストークンを生成
    let access_token = jwt_manager
        .generate_access_token(user_claims.clone())
        .unwrap();

    // Assert: JWTの構造を検証
    let parts: Vec<&str> = access_token.split('.').collect();
    assert_eq!(parts.len(), 3, "JWT should have 3 parts separated by dots");
    assert!(!parts[0].is_empty(), "Header should not be empty");
    assert!(!parts[1].is_empty(), "Payload should not be empty");
    assert!(!parts[2].is_empty(), "Signature should not be empty");

    // Act & Assert: トークンの検証
    let decoded_claims = jwt_manager.verify_access_token(&access_token).unwrap();
    assert_eq!(decoded_claims.user.user_id, user_claims.user_id);
    assert_eq!(decoded_claims.user.username, user_claims.username);
    assert_eq!(decoded_claims.user.email, user_claims.email);
    assert_eq!(decoded_claims.typ, "access");

    // Act: リフレッシュトークンも生成
    let refresh_token = jwt_manager
        .generate_refresh_token(user_claims.user_id, 1)
        .unwrap();
    let refresh_parts: Vec<&str> = refresh_token.split('.').collect();
    assert_eq!(
        refresh_parts.len(),
        3,
        "Refresh token should also have 3 parts"
    );
}

#[tokio::test]
async fn test_token_expiration_and_validation() {
    use chrono::{DateTime, Utc};
    use task_backend::infrastructure::jwt::TokenPair;

    // Arrange: トークン有効期限の検証をテスト
    let config = JwtConfig {
        secret_key: "test-secret-key-must-be-at-least-32-characters-long".to_string(),
        access_token_expiry_minutes: 15,
        refresh_token_expiry_days: 7,
        issuer: "test-issuer".to_string(),
        audience: "test-audience".to_string(),
    };

    let jwt_manager = JwtManager::new(config).unwrap();

    let user_claims = UserClaims {
        user_id: Uuid::new_v4(),
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        is_active: true,
        email_verified: true,
        role_name: "member".to_string(),
        role: None,
        subscription_tier: SubscriptionTier::Free,
    };

    // Act: トークンを生成
    let access_token = jwt_manager
        .generate_access_token(user_claims.clone())
        .unwrap();
    let refresh_token = jwt_manager
        .generate_refresh_token(user_claims.user_id, 1)
        .unwrap();

    // Act: TokenPairを作成
    let token_pair = TokenPair::create_with_jwt_manager(
        access_token.clone(),
        refresh_token.clone(),
        15, // 15分
        7,  // 7日
        &jwt_manager,
    );

    // Assert: 有効期限の検証
    assert_eq!(
        token_pair.access_token_expires_in,
        15 * 60,
        "Access token expires in 15 minutes"
    );
    assert_eq!(
        token_pair.refresh_token_expires_in,
        7 * 24 * 60 * 60,
        "Refresh token expires in 7 days"
    );
    assert_eq!(token_pair.token_type, "Bearer");

    // タイムスタンプが有効なISO 8601形式であることを確認
    let expires_at = DateTime::parse_from_rfc3339(&token_pair.access_token_expires_at).unwrap();
    let should_refresh_at = DateTime::parse_from_rfc3339(&token_pair.should_refresh_at).unwrap();

    // should_refresh_atが有効期限より前であることを確認（80%時点）
    assert!(
        should_refresh_at < expires_at,
        "Should refresh before expiration"
    );

    // 現在時刻より後であることを確認
    let now = Utc::now();
    assert!(expires_at > now, "Access token should expire in the future");
    assert!(
        should_refresh_at > now,
        "Should refresh time should be in the future"
    );

    // Act & Assert: 無効なトークンの検証
    let invalid_token = "invalid.token.here";
    let verify_result = jwt_manager.verify_access_token(invalid_token);
    assert!(
        verify_result.is_err(),
        "Invalid token should fail verification"
    );
}
