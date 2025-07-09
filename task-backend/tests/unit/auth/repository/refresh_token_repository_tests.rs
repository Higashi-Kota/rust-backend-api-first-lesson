// tests/unit/auth/repository/refresh_token_repository_tests.rs

// リフレッシュトークンリポジトリ関連のユニットテスト

use chrono::{Duration, Utc};
use sea_orm::Set;
use task_backend::domain::refresh_token_model;
use task_backend::infrastructure::jwt::{JwtConfig, JwtManager};
use uuid::Uuid;

#[tokio::test]
async fn test_refresh_token_lifecycle() {
    // AAAパターン: Arrange-Act-Assert

    // Arrange: テスト用のユーザー情報を準備
    let user_id = Uuid::new_v4();
    let token_id = Uuid::new_v4();
    let config = JwtConfig {
        secret_key: "test-secret-key-for-refresh-tokens-long-enough".to_string(),
        access_token_expiry_minutes: 15,
        refresh_token_expiry_days: 7,
        issuer: "test-issuer".to_string(),
        audience: "test-audience".to_string(),
    };
    let jwt_manager = JwtManager::new(config).unwrap();

    // Act: リフレッシュトークンを生成
    let refresh_token_string = jwt_manager
        .generate_refresh_token(user_id, 1)
        .expect("Should encode refresh token");

    // Assert: トークンが生成されたことを確認
    assert!(
        !refresh_token_string.is_empty(),
        "Refresh token should not be empty"
    );

    // Act: リフレッシュトークンモデルを作成
    let now = Utc::now();
    let expires_at = now + Duration::days(7);

    let refresh_token_model = refresh_token_model::ActiveModel {
        id: Set(token_id),
        user_id: Set(user_id),
        token_hash: Set(refresh_token_string.clone()),
        expires_at: Set(expires_at),
        created_at: Set(now),
        updated_at: Set(now),
        is_revoked: Set(false),
        device_type: Set(Some("desktop".to_string())),
        ip_address: Set(Some("127.0.0.1".to_string())),
        user_agent: Set(Some("Test User Agent".to_string())),
        geolocation_country: Set(Some("US".to_string())),
        last_used_at: Set(Some(now)),
        use_count: Set(0),
    };

    // Assert: モデルの各フィールドを確認
    assert_eq!(*refresh_token_model.id.as_ref(), token_id);
    assert_eq!(*refresh_token_model.user_id.as_ref(), user_id);
    assert_eq!(
        refresh_token_model.token_hash.as_ref(),
        &refresh_token_string
    );
    assert!(
        refresh_token_model.expires_at.as_ref() > &now,
        "Token should expire in the future"
    );

    // Act: トークンをデコード
    let decoded_claims = jwt_manager
        .verify_refresh_token(&refresh_token_string)
        .expect("Should decode refresh token");

    // Assert: デコードされた情報を確認
    assert_eq!(
        decoded_claims.sub,
        user_id.to_string(),
        "User ID should match"
    );
    assert_eq!(decoded_claims.ver, 1, "Version should match");
}

#[tokio::test]
async fn test_refresh_token_expiration_validation() {
    // AAAパターン: Arrange-Act-Assert

    // Arrange: トークンの検証をテスト
    let user_id = Uuid::new_v4();
    let config = JwtConfig {
        secret_key: "test-secret-key-for-refresh-tokens-long-enough".to_string(),
        access_token_expiry_minutes: 15,
        refresh_token_expiry_days: 7, // 7日間有効
        issuer: "test-issuer".to_string(),
        audience: "test-audience".to_string(),
    };
    let jwt_manager = JwtManager::new(config).unwrap();

    // 有効なトークンを作成
    let valid_token = jwt_manager
        .generate_refresh_token(user_id, 1)
        .expect("Should encode token");

    // Act: 有効なトークンのデコードを試みる
    let decode_result = jwt_manager.verify_refresh_token(&valid_token);
    assert!(
        decode_result.is_ok(),
        "Valid token should decode successfully"
    );

    // 無効なトークンのテスト
    let invalid_token = "invalid.token.here";
    let invalid_result = jwt_manager.verify_refresh_token(invalid_token);
    assert!(
        invalid_result.is_err(),
        "Invalid token should fail to decode"
    );

    // Arrange: データベースモデルでの期限確認
    let now = Utc::now();
    let expired_model = refresh_token_model::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user_id),
        token_hash: Set("expired_token".to_string()),
        expires_at: Set(now - Duration::hours(1)), // 1時間前に期限切れ
        created_at: Set(now - Duration::days(7)),
        updated_at: Set(now - Duration::days(7)),
        is_revoked: Set(false),
        device_type: Set(Some("mobile".to_string())),
        ip_address: Set(Some("127.0.0.1".to_string())),
        user_agent: Set(Some("Test User Agent".to_string())),
        geolocation_country: Set(Some("US".to_string())),
        last_used_at: Set(Some(now - Duration::hours(2))),
        use_count: Set(5),
    };

    let valid_model = refresh_token_model::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user_id),
        token_hash: Set("valid_token".to_string()),
        expires_at: Set(now + Duration::days(7)), // 7日後に期限切れ
        created_at: Set(now),
        updated_at: Set(now),
        is_revoked: Set(false),
        device_type: Set(Some("tablet".to_string())),
        ip_address: Set(Some("127.0.0.1".to_string())),
        user_agent: Set(Some("Test User Agent".to_string())),
        geolocation_country: Set(Some("US".to_string())),
        last_used_at: Set(Some(now)),
        use_count: Set(1),
    };

    // Assert: 期限の確認
    assert!(
        expired_model.expires_at.as_ref() < &now,
        "Expired model should have past expiration date"
    );
    assert!(
        valid_model.expires_at.as_ref() > &now,
        "Valid model should have future expiration date"
    );
}
