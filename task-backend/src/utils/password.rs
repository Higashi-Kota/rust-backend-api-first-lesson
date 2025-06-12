// task-backend/src/utils/password.rs
#![allow(dead_code)]

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use std::env;
use thiserror::Error;
use validator::Validate;

/// パスワード関連のエラー
#[derive(Error, Debug)]
pub enum PasswordError {
    #[error("Password hashing failed: {0}")]
    HashingError(#[from] argon2::password_hash::Error),

    #[error("Argon2 parameter error: {0}")]
    Argon2Error(#[from] argon2::Error),

    #[error("Password verification failed")]
    VerificationFailed,

    #[error("Password validation failed: {0}")]
    ValidationError(String),

    #[error("Weak password: {0}")]
    WeakPassword(String),

    #[error("Password configuration error: {0}")]
    ConfigurationError(String),
}

/// パスワード強度要件
#[derive(Debug, Clone)]
pub struct PasswordPolicy {
    /// 最小文字数
    pub min_length: usize,
    /// 最大文字数
    pub max_length: usize,
    /// 大文字が必要
    pub require_uppercase: bool,
    /// 小文字が必要
    pub require_lowercase: bool,
    /// 数字が必要
    pub require_digit: bool,
    /// 特殊文字が必要
    pub require_special: bool,
    /// 共通パスワードをチェック
    pub check_common_passwords: bool,
}

impl Default for PasswordPolicy {
    fn default() -> Self {
        Self {
            min_length: 8,
            max_length: 128,
            require_uppercase: true,
            require_lowercase: true,
            require_digit: true,
            require_special: true,
            check_common_passwords: true,
        }
    }
}

impl PasswordPolicy {
    /// 環境変数から設定を読み込み
    pub fn from_env() -> Self {
        let min_length = env::var("PASSWORD_MIN_LENGTH")
            .unwrap_or_else(|_| "8".to_string())
            .parse()
            .unwrap_or(8);

        let max_length = env::var("PASSWORD_MAX_LENGTH")
            .unwrap_or_else(|_| "128".to_string())
            .parse()
            .unwrap_or(128);

        let require_uppercase = env::var("PASSWORD_REQUIRE_UPPERCASE")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);

        let require_lowercase = env::var("PASSWORD_REQUIRE_LOWERCASE")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);

        let require_digit = env::var("PASSWORD_REQUIRE_DIGIT")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);

        let require_special = env::var("PASSWORD_REQUIRE_SPECIAL")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);

        let check_common_passwords = env::var("PASSWORD_CHECK_COMMON")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);

        Self {
            min_length,
            max_length,
            require_uppercase,
            require_lowercase,
            require_digit,
            require_special,
            check_common_passwords,
        }
    }

    /// パスワードポリシーを検証
    pub fn validate(&self) -> Result<(), PasswordError> {
        if self.min_length < 4 {
            return Err(PasswordError::ConfigurationError(
                "Minimum password length must be at least 4".to_string(),
            ));
        }

        if self.max_length < self.min_length {
            return Err(PasswordError::ConfigurationError(
                "Maximum password length must be greater than minimum".to_string(),
            ));
        }

        if self.max_length > 1024 {
            return Err(PasswordError::ConfigurationError(
                "Maximum password length cannot exceed 1024".to_string(),
            ));
        }

        Ok(())
    }
}

/// Argon2 設定
#[derive(Debug, Clone)]
pub struct Argon2Config {
    /// メモリコスト（KB）
    pub memory_cost: u32,
    /// 時間コスト（反復回数）
    pub time_cost: u32,
    /// 並列度
    pub parallelism: u32,
    /// 出力長
    pub output_length: usize,
}

impl Default for Argon2Config {
    fn default() -> Self {
        Self {
            memory_cost: 65536, // 64MB
            time_cost: 3,       // 3回反復
            parallelism: 4,     // 4並列
            output_length: 32,  // 32バイト出力
        }
    }
}

impl Argon2Config {
    /// 環境変数から設定を読み込み
    pub fn from_env() -> Self {
        let memory_cost = env::var("ARGON2_MEMORY_COST")
            .unwrap_or_else(|_| "65536".to_string())
            .parse()
            .unwrap_or(65536);

        let time_cost = env::var("ARGON2_TIME_COST")
            .unwrap_or_else(|_| "3".to_string())
            .parse()
            .unwrap_or(3);

        let parallelism = env::var("ARGON2_PARALLELISM")
            .unwrap_or_else(|_| "4".to_string())
            .parse()
            .unwrap_or(4);

        let output_length = env::var("ARGON2_OUTPUT_LENGTH")
            .unwrap_or_else(|_| "32".to_string())
            .parse()
            .unwrap_or(32);

        Self {
            memory_cost,
            time_cost,
            parallelism,
            output_length,
        }
    }
}

/// パスワードハッシュマネージャー
pub struct PasswordManager {
    argon2: Argon2<'static>,
    policy: PasswordPolicy,
}

impl PasswordManager {
    /// 新しいPasswordManagerを作成
    pub fn new(argon2_config: Argon2Config, policy: PasswordPolicy) -> Result<Self, PasswordError> {
        policy.validate()?;

        let argon2 = Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            argon2::Params::new(
                argon2_config.memory_cost,
                argon2_config.time_cost,
                argon2_config.parallelism,
                Some(argon2_config.output_length),
            )
            .map_err(PasswordError::Argon2Error)?,
        );

        Ok(Self { argon2, policy })
    }

    /// デフォルト設定でPasswordManagerを作成
    pub fn new_default() -> Result<Self, PasswordError> {
        Self::new(Argon2Config::default(), PasswordPolicy::default())
    }

    /// 環境変数から設定を読み込んでPasswordManagerを作成
    pub fn from_env() -> Result<Self, PasswordError> {
        Self::new(Argon2Config::from_env(), PasswordPolicy::from_env())
    }

    /// パスワードをハッシュ化
    pub fn hash_password(&self, password: &str) -> Result<String, PasswordError> {
        // パスワード強度チェック
        self.validate_password_strength(password)?;

        // ソルト生成
        let salt = SaltString::generate(&mut OsRng);

        // パスワードハッシュ
        let password_hash = self
            .argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(PasswordError::HashingError)?;

        Ok(password_hash.to_string())
    }

    /// パスワードを検証
    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool, PasswordError> {
        let parsed_hash = PasswordHash::new(hash).map_err(PasswordError::HashingError)?;

        match self
            .argon2
            .verify_password(password.as_bytes(), &parsed_hash)
        {
            Ok(()) => Ok(true),
            Err(argon2::password_hash::Error::Password) => Ok(false),
            Err(e) => Err(PasswordError::HashingError(e)),
        }
    }

    /// パスワード強度をチェック
    pub fn validate_password_strength(&self, password: &str) -> Result<(), PasswordError> {
        let mut errors = Vec::new();

        // 長さチェック
        if password.len() < self.policy.min_length {
            errors.push(format!(
                "Password must be at least {} characters long",
                self.policy.min_length
            ));
        }

        if password.len() > self.policy.max_length {
            errors.push(format!(
                "Password must be no more than {} characters long",
                self.policy.max_length
            ));
        }

        // 大文字チェック
        if self.policy.require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
            errors.push("Password must contain at least one uppercase letter".to_string());
        }

        // 小文字チェック
        if self.policy.require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
            errors.push("Password must contain at least one lowercase letter".to_string());
        }

        // 数字チェック
        if self.policy.require_digit && !password.chars().any(|c| c.is_ascii_digit()) {
            errors.push("Password must contain at least one digit".to_string());
        }

        // 特殊文字チェック
        if self.policy.require_special
            && !password
                .chars()
                .any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c))
        {
            errors.push(
                "Password must contain at least one special character (!@#$%^&*()_+-=[]{}|;:,.<>?)"
                    .to_string(),
            );
        }

        // 共通パスワードチェック
        if self.policy.check_common_passwords && is_common_password(password) {
            errors.push(
                "This password is too common. Please choose a different password".to_string(),
            );
        }

        // 連続する文字チェック
        if has_consecutive_characters(password, 3) {
            errors.push(
                "Password cannot contain 3 or more consecutive identical characters".to_string(),
            );
        }

        // 順次文字チェック（abc, 123など）
        if has_sequential_characters(password, 3) {
            errors.push(
                "Password cannot contain 3 or more sequential characters (e.g., abc, 123)"
                    .to_string(),
            );
        }

        if !errors.is_empty() {
            return Err(PasswordError::WeakPassword(errors.join("; ")));
        }

        Ok(())
    }

    /// パスワードが再ハッシュが必要かチェック
    pub fn needs_rehash(&self, hash: &str) -> Result<bool, PasswordError> {
        let parsed_hash = PasswordHash::new(hash).map_err(PasswordError::HashingError)?;

        // Argon2パラメータが現在の設定と一致するかチェック
        let _current_params = self.argon2.params();

        // 簡略化されたチェック：アルゴリズムIDが一致するかどうかのみチェック
        // より詳細なパラメータ比較が必要な場合は、カスタム実装が必要
        if let Ok(expected_alg) = argon2::password_hash::Ident::new("argon2id") {
            if parsed_hash.algorithm != expected_alg {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// パスワード強度スコアを計算（0-100）
    pub fn calculate_password_score(&self, password: &str) -> u8 {
        let mut score = 0u8;

        // 基本長さスコア（最大25点）
        let length_score = std::cmp::min(password.len() * 2, 25);
        score += length_score as u8;

        // 文字種類スコア（各最大15点）
        if password.chars().any(|c| c.is_lowercase()) {
            score += 15;
        }
        if password.chars().any(|c| c.is_uppercase()) {
            score += 15;
        }
        if password.chars().any(|c| c.is_ascii_digit()) {
            score += 15;
        }
        if password
            .chars()
            .any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c))
        {
            score += 15;
        }

        // 長さボーナス（12文字以上で+10点）
        if password.len() >= 12 {
            score += 10;
        }

        // ペナルティ
        if is_common_password(password) {
            score = score.saturating_sub(30);
        }
        if has_consecutive_characters(password, 3) {
            score = score.saturating_sub(15);
        }
        if has_sequential_characters(password, 3) {
            score = score.saturating_sub(15);
        }

        std::cmp::min(score, 100)
    }

    /// パスワード強度レベルを取得
    pub fn get_password_strength_level(&self, password: &str) -> PasswordStrength {
        let score = self.calculate_password_score(password);

        match score {
            0..=30 => PasswordStrength::VeryWeak,
            31..=50 => PasswordStrength::Weak,
            51..=70 => PasswordStrength::Fair,
            71..=85 => PasswordStrength::Good,
            86..=100 => PasswordStrength::Strong,
            _ => PasswordStrength::Strong, // For scores above 100
        }
    }
}

/// パスワード強度レベル
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PasswordStrength {
    VeryWeak,
    Weak,
    Fair,
    Good,
    Strong,
}

impl PasswordStrength {
    pub fn as_str(&self) -> &'static str {
        match self {
            PasswordStrength::VeryWeak => "very_weak",
            PasswordStrength::Weak => "weak",
            PasswordStrength::Fair => "fair",
            PasswordStrength::Good => "good",
            PasswordStrength::Strong => "strong",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            PasswordStrength::VeryWeak => "#dc2626", // red-600
            PasswordStrength::Weak => "#ea580c",     // orange-600
            PasswordStrength::Fair => "#ca8a04",     // yellow-600
            PasswordStrength::Good => "#16a34a",     // green-600
            PasswordStrength::Strong => "#059669",   // emerald-600
        }
    }
}

/// バリデーション用の構造体
#[derive(Debug, Clone, Validate)]
pub struct PasswordInput {
    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
}

/// パスワード変更用の構造体
#[derive(Debug, Clone, Validate)]
pub struct PasswordChangeInput {
    #[validate(length(min = 1, message = "Current password is required"))]
    pub current_password: String,

    #[validate(length(min = 1, message = "New password is required"))]
    pub new_password: String,

    #[validate(must_match(
        other = "new_password",
        message = "Password confirmation does not match"
    ))]
    pub new_password_confirmation: String,
}

// --- ヘルパー関数 ---

/// 共通パスワードかどうかをチェック
fn is_common_password(password: &str) -> bool {
    const COMMON_PASSWORDS: &[&str] = &[
        "password",
        "123456",
        "123456789",
        "12345678",
        "12345",
        "1234567",
        "1234567890",
        "qwerty",
        "abc123",
        "password123",
        "admin",
        "letmein",
        "welcome",
        "monkey",
        "dragon",
        "111111",
        "666666",
        "123123",
        "654321",
        "superman",
        "qazwsx",
        "michael",
        "Football",
        "baseball",
        "liverpool",
        "jordan",
        "freedom",
        "princess",
        "maggie",
        "131313",
        "sunshine",
        "iloveyou",
        "thomas",
        "michelle",
        "love",
        "jessica",
        "chocolate",
        "fuckyou",
        "hunter",
        "jennifer",
        "buster",
        "johnny",
        "tigger",
        "charlie",
        "robert",
        "arthur",
        "pepper",
        "george",
        "joshua",
        "yamaha",
        "brandon",
        "harley",
    ];

    let lower_password = password.to_lowercase();
    COMMON_PASSWORDS
        .iter()
        .any(|&common| lower_password.contains(common))
}

/// 連続する同じ文字があるかチェック
fn has_consecutive_characters(password: &str, count: usize) -> bool {
    if password.len() < count {
        return false;
    }

    let chars: Vec<char> = password.chars().collect();

    for window in chars.windows(count) {
        if window.iter().all(|&c| c == window[0]) {
            return true;
        }
    }

    false
}

/// 順次文字があるかチェック（abc, 123など）
fn has_sequential_characters(password: &str, count: usize) -> bool {
    if password.len() < count {
        return false;
    }

    let chars: Vec<char> = password.chars().collect();

    for window in chars.windows(count) {
        // 昇順チェック
        let mut is_ascending = true;
        for i in 1..window.len() {
            if (window[i] as u32) != (window[i - 1] as u32) + 1 {
                is_ascending = false;
                break;
            }
        }

        // 降順チェック
        let mut is_descending = true;
        for i in 1..window.len() {
            if (window[i] as u32) != (window[i - 1] as u32) - 1 {
                is_descending = false;
                break;
            }
        }

        if is_ascending || is_descending {
            return true;
        }
    }

    false
}

/// セキュアなランダムパスワードを生成
pub fn generate_secure_password(length: usize) -> String {
    use rand::Rng;

    const CHARSET: &[u8] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*()_+-=[]{}|;:,.<>?";
    let mut rng = rand::thread_rng();

    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// 一時的なパスワードを生成（数字と文字の組み合わせ）
pub fn generate_temporary_password(length: usize) -> String {
    use rand::Rng;

    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNPQRSTUVWXYZ23456789"; // 紛らわしい文字を除外
    let mut rng = rand::thread_rng();

    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing_and_verification() {
        let manager = PasswordManager::new_default().unwrap();
        let password = "MyUniqueP@ssw0rd91";

        let hash = manager.hash_password(password).unwrap();
        assert!(!hash.is_empty());

        assert!(manager.verify_password(password, &hash).unwrap());
        assert!(!manager.verify_password("WrongPassword", &hash).unwrap());
    }

    #[test]
    fn test_password_strength_validation() {
        let manager = PasswordManager::new_default().unwrap();

        // 強いパスワード
        assert!(manager
            .validate_password_strength("MyUniqueP@ssw0rd91")
            .is_ok());

        // 弱いパスワード
        assert!(manager.validate_password_strength("123").is_err());
        assert!(manager.validate_password_strength("password").is_err());
        assert!(manager.validate_password_strength("PASSWORD").is_err());
        assert!(manager.validate_password_strength("12345678").is_err());
    }

    #[test]
    fn test_password_score_calculation() {
        let manager = PasswordManager::new_default().unwrap();

        assert_eq!(
            manager.get_password_strength_level("123"),
            PasswordStrength::VeryWeak
        );
        assert_eq!(
            manager.get_password_strength_level("password"),
            PasswordStrength::VeryWeak
        );
        assert_eq!(
            manager.get_password_strength_level("Password1"),
            PasswordStrength::Weak
        );
        assert_eq!(
            manager.get_password_strength_level("MyUniqueP@ssw0rd91"),
            PasswordStrength::Strong
        );
    }

    #[test]
    fn test_consecutive_characters() {
        assert!(has_consecutive_characters("aaa", 3));
        assert!(has_consecutive_characters("password111", 3));
        assert!(!has_consecutive_characters("password", 3));
    }

    #[test]
    fn test_sequential_characters() {
        assert!(has_sequential_characters("abc", 3));
        assert!(has_sequential_characters("123", 3));
        assert!(has_sequential_characters("xyz", 3));
        assert!(!has_sequential_characters("password", 3));
    }

    #[test]
    fn test_common_password_detection() {
        assert!(is_common_password("password"));
        assert!(is_common_password("Password123"));
        assert!(is_common_password("123456"));
        assert!(!is_common_password("MyUniqueP@ssw0rd"));
    }

    #[test]
    fn test_password_generation() {
        let password = generate_secure_password(16);
        assert_eq!(password.len(), 16);

        let temp_password = generate_temporary_password(8);
        assert_eq!(temp_password.len(), 8);

        // 生成されたパスワードが異なることを確認
        let password2 = generate_secure_password(16);
        assert_ne!(password, password2);
    }
}
