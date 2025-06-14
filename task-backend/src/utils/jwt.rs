// src/utils/jwt.rs
#![allow(dead_code)]

use crate::domain::user_model::UserClaims;
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::env;
use thiserror::Error;
use uuid::Uuid;

/// JWT関連のエラー
#[derive(Error, Debug)]
pub enum JwtError {
    #[error("Failed to encode JWT: {0}")]
    EncodingError(#[from] jsonwebtoken::errors::Error),

    #[error("Failed to decode JWT: {0}")]
    DecodingError(String),

    #[error("JWT token has expired")]
    TokenExpired,

    #[error("Invalid JWT token")]
    InvalidToken,

    #[error("Missing JWT secret key")]
    MissingSecretKey,

    #[error("Invalid JWT configuration: {0}")]
    ConfigurationError(String),
}

/// アクセストークンのClaims
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccessTokenClaims {
    /// Subject (user ID)
    pub sub: String,
    /// Issued at
    pub iat: i64,
    /// Expiration time
    pub exp: i64,
    /// Not before
    pub nbf: i64,
    /// Issuer
    pub iss: String,
    /// Audience
    pub aud: String,
    /// JWT ID
    pub jti: String,
    /// Token type
    pub typ: String,
    /// User information
    pub user: UserClaims,
}

/// リフレッシュトークンのClaims
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RefreshTokenClaims {
    /// Subject (user ID)
    pub sub: String,
    /// Issued at
    pub iat: i64,
    /// Expiration time
    pub exp: i64,
    /// Not before
    pub nbf: i64,
    /// Issuer
    pub iss: String,
    /// Audience
    pub aud: String,
    /// JWT ID
    pub jti: String,
    /// Token type
    pub typ: String,
    /// Token version (for rotation)
    pub ver: u32,
}

/// JWT設定
#[derive(Debug, Clone)]
pub struct JwtConfig {
    /// JWT秘密鍵
    pub secret_key: String,
    /// アクセストークンの有効期限（分）
    pub access_token_expiry_minutes: i64,
    /// リフレッシュトークンの有効期限（日）
    pub refresh_token_expiry_days: i64,
    /// 発行者
    pub issuer: String,
    /// 対象者
    pub audience: String,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret_key: "your-secret-key".to_string(), // 本番では絶対に変更すること
            access_token_expiry_minutes: 15,
            refresh_token_expiry_days: 7,
            issuer: "task-backend".to_string(),
            audience: "task-backend-users".to_string(),
        }
    }
}

impl JwtConfig {
    /// 環境変数から設定を読み込み
    pub fn from_env() -> Result<Self, JwtError> {
        let secret_key = env::var("JWT_SECRET_KEY").map_err(|_| JwtError::MissingSecretKey)?;

        let access_token_expiry_minutes = env::var("JWT_ACCESS_TOKEN_EXPIRY_MINUTES")
            .unwrap_or_else(|_| "15".to_string())
            .parse()
            .map_err(|_| JwtError::ConfigurationError("Invalid access token expiry".to_string()))?;

        let refresh_token_expiry_days = env::var("JWT_REFRESH_TOKEN_EXPIRY_DAYS")
            .unwrap_or_else(|_| "7".to_string())
            .parse()
            .map_err(|_| {
                JwtError::ConfigurationError("Invalid refresh token expiry".to_string())
            })?;

        let issuer = env::var("JWT_ISSUER").unwrap_or_else(|_| "task-backend".to_string());

        let audience =
            env::var("JWT_AUDIENCE").unwrap_or_else(|_| "task-backend-users".to_string());

        Ok(Self {
            secret_key,
            access_token_expiry_minutes,
            refresh_token_expiry_days,
            issuer,
            audience,
        })
    }

    /// 秘密鍵の検証
    pub fn validate(&self) -> Result<(), JwtError> {
        if self.secret_key.len() < 32 {
            return Err(JwtError::ConfigurationError(
                "JWT secret key must be at least 32 characters".to_string(),
            ));
        }

        if self.access_token_expiry_minutes <= 0 {
            return Err(JwtError::ConfigurationError(
                "Access token expiry must be positive".to_string(),
            ));
        }

        if self.refresh_token_expiry_days <= 0 {
            return Err(JwtError::ConfigurationError(
                "Refresh token expiry must be positive".to_string(),
            ));
        }

        Ok(())
    }
}

/// JWTトークン管理
pub struct JwtManager {
    config: JwtConfig,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
}

impl JwtManager {
    /// 新しいJwtManagerを作成
    pub fn new(config: JwtConfig) -> Result<Self, JwtError> {
        config.validate()?;

        let encoding_key = EncodingKey::from_secret(config.secret_key.as_bytes());
        let decoding_key = DecodingKey::from_secret(config.secret_key.as_bytes());

        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&[&config.issuer]);
        validation.set_audience(&[&config.audience]);
        validation.validate_exp = true;
        validation.validate_nbf = true;

        Ok(Self {
            config,
            encoding_key,
            decoding_key,
            validation,
        })
    }

    /// 環境変数から設定を読み込んでJwtManagerを作成
    pub fn from_env() -> Result<Self, JwtError> {
        let config = JwtConfig::from_env()?;
        Self::new(config)
    }

    /// アクセストークンを生成
    pub fn generate_access_token(&self, user: UserClaims) -> Result<String, JwtError> {
        let now = Utc::now();
        let exp = now + Duration::minutes(self.config.access_token_expiry_minutes);

        let claims = AccessTokenClaims {
            sub: user.user_id.to_string(),
            iat: now.timestamp(),
            exp: exp.timestamp(),
            nbf: now.timestamp(),
            iss: self.config.issuer.clone(),
            aud: self.config.audience.clone(),
            jti: Uuid::new_v4().to_string(),
            typ: "access".to_string(),
            user,
        };

        encode(&Header::default(), &claims, &self.encoding_key).map_err(JwtError::EncodingError)
    }

    /// リフレッシュトークンを生成
    pub fn generate_refresh_token(&self, user_id: Uuid, version: u32) -> Result<String, JwtError> {
        let now = Utc::now();
        let exp = now + Duration::days(self.config.refresh_token_expiry_days);

        let claims = RefreshTokenClaims {
            sub: user_id.to_string(),
            iat: now.timestamp(),
            exp: exp.timestamp(),
            nbf: now.timestamp(),
            iss: self.config.issuer.clone(),
            aud: self.config.audience.clone(),
            jti: Uuid::new_v4().to_string(),
            typ: "refresh".to_string(),
            ver: version,
        };

        encode(&Header::default(), &claims, &self.encoding_key).map_err(JwtError::EncodingError)
    }

    /// アクセストークンを検証・デコード
    pub fn verify_access_token(&self, token: &str) -> Result<AccessTokenClaims, JwtError> {
        let token_data = decode::<AccessTokenClaims>(token, &self.decoding_key, &self.validation)
            .map_err(|e| match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => JwtError::TokenExpired,
            _ => JwtError::DecodingError(e.to_string()),
        })?;

        // トークンタイプの検証
        if token_data.claims.typ != "access" {
            return Err(JwtError::InvalidToken);
        }

        Ok(token_data.claims)
    }

    /// リフレッシュトークンを検証・デコード
    pub fn verify_refresh_token(&self, token: &str) -> Result<RefreshTokenClaims, JwtError> {
        let token_data = decode::<RefreshTokenClaims>(token, &self.decoding_key, &self.validation)
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => JwtError::TokenExpired,
                _ => JwtError::DecodingError(e.to_string()),
            })?;

        // トークンタイプの検証
        if token_data.claims.typ != "refresh" {
            return Err(JwtError::InvalidToken);
        }

        Ok(token_data.claims)
    }

    /// トークンから有効期限を取得（検証せず）
    pub fn get_token_expiry(&self, token: &str) -> Result<DateTime<Utc>, JwtError> {
        // 検証を無効にしたValidationを作成
        let mut no_validation = Validation::new(Algorithm::HS256);
        no_validation.validate_exp = false;
        no_validation.validate_nbf = false;
        no_validation.validate_aud = false;
        no_validation.insecure_disable_signature_validation();

        let token_data = decode::<serde_json::Value>(token, &self.decoding_key, &no_validation)
            .map_err(|e| JwtError::DecodingError(e.to_string()))?;

        let exp = token_data
            .claims
            .get("exp")
            .and_then(|v| v.as_i64())
            .ok_or(JwtError::InvalidToken)?;

        DateTime::from_timestamp(exp, 0).ok_or(JwtError::InvalidToken)
    }

    /// トークンが期限切れかチェック（検証せず）
    pub fn is_token_expired(&self, token: &str) -> Result<bool, JwtError> {
        let expiry = self.get_token_expiry(token)?;
        Ok(expiry <= Utc::now())
    }

    /// アクセストークンの残り有効時間を取得（分）
    pub fn get_access_token_remaining_minutes(&self, claims: &AccessTokenClaims) -> i64 {
        let exp = DateTime::from_timestamp(claims.exp, 0).unwrap_or_else(Utc::now);
        let remaining = exp - Utc::now();
        remaining.num_minutes().max(0)
    }

    /// リフレッシュトークンの残り有効時間を取得（日）
    pub fn get_refresh_token_remaining_days(&self, claims: &RefreshTokenClaims) -> i64 {
        let exp = DateTime::from_timestamp(claims.exp, 0).unwrap_or_else(Utc::now);
        let remaining = exp - Utc::now();
        remaining.num_days().max(0)
    }

    /// アクセストークンの有効期限をISO 8601形式で取得
    pub fn get_access_token_expires_at(&self, claims: &AccessTokenClaims) -> String {
        DateTime::from_timestamp(claims.exp, 0)
            .unwrap_or_else(Utc::now)
            .to_rfc3339()
    }

    /// アクセストークンのリフレッシュ推奨時刻をISO 8601形式で取得（80%時点）
    pub fn get_should_refresh_at(&self, claims: &AccessTokenClaims) -> String {
        let issued_at = DateTime::from_timestamp(claims.iat, 0).unwrap_or_else(Utc::now);
        let expires_at = DateTime::from_timestamp(claims.exp, 0).unwrap_or_else(Utc::now);

        // 有効期限の80%時点を計算
        let total_duration = expires_at - issued_at;
        let refresh_duration_secs = (total_duration.num_seconds() as f64 * 0.8) as i64;
        let refresh_duration = Duration::seconds(refresh_duration_secs);
        let should_refresh_at = issued_at + refresh_duration;

        should_refresh_at.to_rfc3339()
    }

    /// 現在時刻からアクセストークンの有効期限をISO 8601形式で計算
    pub fn calculate_access_token_expires_at(&self) -> String {
        let expires_at = Utc::now() + Duration::minutes(self.config.access_token_expiry_minutes);
        expires_at.to_rfc3339()
    }

    /// 現在時刻からアクセストークンのリフレッシュ推奨時刻をISO 8601形式で計算（80%時点）
    pub fn calculate_should_refresh_at(&self) -> String {
        let now = Utc::now();
        let total_duration = Duration::minutes(self.config.access_token_expiry_minutes);
        let refresh_duration_secs = (total_duration.num_seconds() as f64 * 0.8) as i64;
        let refresh_duration = Duration::seconds(refresh_duration_secs);
        let should_refresh_at = now + refresh_duration;

        should_refresh_at.to_rfc3339()
    }
}

/// トークンペア
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub access_token_expires_in: i64,  // 秒
    pub refresh_token_expires_in: i64, // 秒
    pub token_type: String,
    pub access_token_expires_at: String, // ISO 8601 UTC時刻
    pub should_refresh_at: String,       // ISO 8601 UTC時刻（80%時点）
}

impl TokenPair {
    pub fn new(
        access_token: String,
        refresh_token: String,
        access_expiry_minutes: i64,
        refresh_expiry_days: i64,
        access_token_expires_at: String,
        should_refresh_at: String,
    ) -> Self {
        Self {
            access_token,
            refresh_token,
            access_token_expires_in: access_expiry_minutes * 60,
            refresh_token_expires_in: refresh_expiry_days * 24 * 60 * 60,
            token_type: "Bearer".to_string(),
            access_token_expires_at,
            should_refresh_at,
        }
    }

    /// JwtManagerを使って完全なTokenPairを作成
    pub fn create_with_jwt_manager(
        access_token: String,
        refresh_token: String,
        access_expiry_minutes: i64,
        refresh_expiry_days: i64,
        jwt_manager: &JwtManager,
    ) -> Self {
        let access_token_expires_at = jwt_manager.calculate_access_token_expires_at();
        let should_refresh_at = jwt_manager.calculate_should_refresh_at();

        Self::new(
            access_token,
            refresh_token,
            access_expiry_minutes,
            refresh_expiry_days,
            access_token_expires_at,
            should_refresh_at,
        )
    }
}

/// JWTユーティリティ関数
pub mod utils {
    use super::*;
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    use rand::Rng;

    /// ランダムなJWT秘密鍵を生成（64文字）
    pub fn generate_jwt_secret() -> String {
        let mut rng = rand::thread_rng();
        let mut bytes = [0u8; 48];
        for byte in &mut bytes {
            *byte = rng.gen();
        }
        URL_SAFE_NO_PAD.encode(bytes)
    }

    /// トークンからペイロードを安全に抽出（署名検証なし）
    pub fn extract_payload_unsafe(token: &str) -> Result<serde_json::Value, JwtError> {
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err(JwtError::InvalidToken);
        }

        let payload = parts[1];
        let decoded = URL_SAFE_NO_PAD
            .decode(payload)
            .map_err(|_| JwtError::InvalidToken)?;

        serde_json::from_slice(&decoded).map_err(|_| JwtError::InvalidToken)
    }

    /// JTI（JWT ID）をトークンから抽出
    pub fn extract_jti(token: &str) -> Result<String, JwtError> {
        let payload = extract_payload_unsafe(token)?;
        payload
            .get("jti")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or(JwtError::InvalidToken)
    }

    /// ユーザーIDをトークンから抽出
    pub fn extract_user_id(token: &str) -> Result<Uuid, JwtError> {
        let payload = extract_payload_unsafe(token)?;
        let sub = payload
            .get("sub")
            .and_then(|v| v.as_str())
            .ok_or(JwtError::InvalidToken)?;

        Uuid::parse_str(sub).map_err(|_| JwtError::InvalidToken)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::user_model::UserClaims;

    fn create_test_config() -> JwtConfig {
        JwtConfig {
            secret_key: "test-secret-key-must-be-at-least-32-characters-long".to_string(),
            access_token_expiry_minutes: 15,
            refresh_token_expiry_days: 7,
            issuer: "test-issuer".to_string(),
            audience: "test-audience".to_string(),
        }
    }

    fn create_test_user_claims() -> UserClaims {
        UserClaims {
            user_id: Uuid::new_v4(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            is_active: true,
            email_verified: true,
            role_name: "member".to_string(),
        }
    }

    #[test]
    fn test_jwt_generation_and_verification() {
        let config = create_test_config();
        let jwt_manager = JwtManager::new(config).unwrap();
        let user_claims = create_test_user_claims();

        // アクセストークン生成
        let access_token = jwt_manager
            .generate_access_token(user_claims.clone())
            .unwrap();
        assert!(!access_token.is_empty());

        // アクセストークン検証
        let decoded_claims = jwt_manager.verify_access_token(&access_token).unwrap();
        assert_eq!(decoded_claims.user.user_id, user_claims.user_id);
        assert_eq!(decoded_claims.user.username, user_claims.username);

        // リフレッシュトークン生成
        let refresh_token = jwt_manager
            .generate_refresh_token(user_claims.user_id, 1)
            .unwrap();
        assert!(!refresh_token.is_empty());

        // リフレッシュトークン検証
        let refresh_claims = jwt_manager.verify_refresh_token(&refresh_token).unwrap();
        assert_eq!(refresh_claims.sub, user_claims.user_id.to_string());
        assert_eq!(refresh_claims.ver, 1);
    }

    #[test]
    fn test_invalid_secret_key() {
        let mut config = create_test_config();
        config.secret_key = "short".to_string(); // 短すぎる秘密鍵

        let result = JwtManager::new(config);
        assert!(result.is_err());
    }

    #[test]
    fn test_token_utilities() {
        let config = create_test_config();
        let jwt_manager = JwtManager::new(config).unwrap();
        let user_claims = create_test_user_claims();

        let token = jwt_manager
            .generate_access_token(user_claims.clone())
            .unwrap();

        // JTI抽出
        let jti = utils::extract_jti(&token).unwrap();
        assert!(!jti.is_empty());

        // ユーザーID抽出
        let extracted_user_id = utils::extract_user_id(&token).unwrap();
        assert_eq!(extracted_user_id, user_claims.user_id);

        // 有効期限取得
        let expiry = jwt_manager.get_token_expiry(&token).unwrap();
        assert!(expiry > Utc::now());

        // 期限切れチェック
        let is_expired = jwt_manager.is_token_expired(&token).unwrap();
        assert!(!is_expired);
    }

    #[test]
    fn test_timestamp_calculations() {
        let config = create_test_config();
        let jwt_manager = JwtManager::new(config).unwrap();
        let user_claims = create_test_user_claims();

        // アクセストークンを生成
        let access_token = jwt_manager
            .generate_access_token(user_claims.clone())
            .unwrap();

        // トークンから クレームを取得
        let claims = jwt_manager.verify_access_token(&access_token).unwrap();

        // タイムスタンプ計算のテスト
        let expires_at = jwt_manager.get_access_token_expires_at(&claims);
        let should_refresh_at = jwt_manager.get_should_refresh_at(&claims);

        // フォーマットが正しいことを確認（ISO 8601）
        assert!(DateTime::parse_from_rfc3339(&expires_at).is_ok());
        assert!(DateTime::parse_from_rfc3339(&should_refresh_at).is_ok());

        // リフレッシュ時刻が有効期限より前であることを確認
        let expires_dt = DateTime::parse_from_rfc3339(&expires_at).unwrap();
        let refresh_dt = DateTime::parse_from_rfc3339(&should_refresh_at).unwrap();
        assert!(refresh_dt < expires_dt);

        // 現在時刻ベースの計算もテスト
        let calc_expires_at = jwt_manager.calculate_access_token_expires_at();
        let calc_should_refresh_at = jwt_manager.calculate_should_refresh_at();

        assert!(DateTime::parse_from_rfc3339(&calc_expires_at).is_ok());
        assert!(DateTime::parse_from_rfc3339(&calc_should_refresh_at).is_ok());
    }

    #[test]
    fn test_token_pair_with_timestamps() {
        let config = create_test_config();
        let jwt_manager = JwtManager::new(config).unwrap();

        // create_with_jwt_managerメソッドのテスト
        let token_pair = TokenPair::create_with_jwt_manager(
            "test_access_token".to_string(),
            "test_refresh_token".to_string(),
            15, // 15分
            7,  // 7日
            &jwt_manager,
        );

        // フィールドが正しく設定されていることを確認
        assert_eq!(token_pair.access_token, "test_access_token");
        assert_eq!(token_pair.refresh_token, "test_refresh_token");
        assert_eq!(token_pair.access_token_expires_in, 15 * 60);
        assert_eq!(token_pair.refresh_token_expires_in, 7 * 24 * 60 * 60);
        assert_eq!(token_pair.token_type, "Bearer");

        // タイムスタンプフィールドが有効なISO 8601形式であることを確認
        assert!(DateTime::parse_from_rfc3339(&token_pair.access_token_expires_at).is_ok());
        assert!(DateTime::parse_from_rfc3339(&token_pair.should_refresh_at).is_ok());

        // should_refresh_atがaccess_token_expires_atより前であることを確認
        let expires_at = DateTime::parse_from_rfc3339(&token_pair.access_token_expires_at).unwrap();
        let refresh_at = DateTime::parse_from_rfc3339(&token_pair.should_refresh_at).unwrap();
        assert!(refresh_at < expires_at);

        // 手動newメソッドのテスト
        let manual_token_pair = TokenPair::new(
            "manual_access".to_string(),
            "manual_refresh".to_string(),
            10,
            5,
            "2024-01-01T12:10:00Z".to_string(),
            "2024-01-01T12:08:00Z".to_string(),
        );

        assert_eq!(manual_token_pair.access_token_expires_in, 10 * 60);
        assert_eq!(manual_token_pair.refresh_token_expires_in, 5 * 24 * 60 * 60);
        assert_eq!(
            manual_token_pair.access_token_expires_at,
            "2024-01-01T12:10:00Z"
        );
        assert_eq!(manual_token_pair.should_refresh_at, "2024-01-01T12:08:00Z");
    }
}
