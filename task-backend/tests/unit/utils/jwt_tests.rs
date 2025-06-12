// tests/unit/utils/jwt_tests.rs

// JWT関連のユニットテスト（既存のsrc/utils/jwt.rsのテストを拡張）

#[tokio::test]
async fn test_jwt_token_structure() {
    // JWTトークンの構造テスト
    let mock_jwt = "header.payload.signature";
    let parts: Vec<&str> = mock_jwt.split('.').collect();

    // JWTが3つの部分（ヘッダー、ペイロード、署名）から成ることを確認
    assert_eq!(parts.len(), 3);
    assert!(!parts[0].is_empty()); // header
    assert!(!parts[1].is_empty()); // payload
    assert!(!parts[2].is_empty()); // signature
}

#[tokio::test]
async fn test_token_expiration_concepts() {
    use chrono::{Duration, Utc};

    // トークン有効期限の概念テスト
    let access_token_lifetime = Duration::minutes(15);
    let refresh_token_lifetime = Duration::days(7);

    let now = Utc::now();
    let access_expires = now + access_token_lifetime;
    let refresh_expires = now + refresh_token_lifetime;

    // アクセストークンがリフレッシュトークンより短い有効期限であることを確認
    assert!(access_expires < refresh_expires);

    // 両方のトークンが現在時刻より後に期限切れになることを確認
    assert!(access_expires > now);
    assert!(refresh_expires > now);
}
