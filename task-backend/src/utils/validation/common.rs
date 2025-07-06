// task-backend/src/utils/validation/common.rs

//! 共通バリデーション定数とマクロ
//!
//! DTOファイル間で重複するバリデーションルールを統一管理します。
//! このモジュールにより、バリデーションの一貫性とメンテナンス性が向上します。

use once_cell::sync::Lazy;
use regex::Regex;
use validator::ValidationError;

// =============================================================================
// バリデーション定数
// =============================================================================

/// ユーザー名の制約
pub mod username {
    pub const MIN_LENGTH: u64 = 3;
    pub const MAX_LENGTH: u64 = 30;
}

/// メールアドレスの制約
pub mod email {
    // 定数は削除（直接文字列リテラルを使用）
}

/// パスワードの制約
pub mod password {
    pub const MIN_LENGTH: u64 = 8;
}

/// タスク関連の制約
pub mod task {
    pub const TITLE_MIN_LENGTH: u64 = 1;
    pub const TITLE_MAX_LENGTH: u64 = 200;
    pub const DESCRIPTION_MAX_LENGTH: u64 = 2000;
}

/// 必須フィールドの制約
pub mod required {
    pub const MIN_LENGTH: u64 = 1;
}

// =============================================================================
// バリデーション正規表現
// =============================================================================

/// ユーザー名の正規表現パターン（既存のvalidation.rsから移行）
pub static USERNAME_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[a-zA-Z0-9_-]+$").expect("Invalid username regex"));

// =============================================================================
// カスタムバリデーション関数
// =============================================================================

/// ユーザー名の形式をバリデーション（既存のvalidation.rsから移行）
pub fn validate_username(username: &str) -> Result<(), ValidationError> {
    // 長さチェック
    if username.len() < username::MIN_LENGTH as usize {
        let mut error = ValidationError::new("username_too_short");
        error.message = Some(
            format!(
                "Username must be at least {} characters",
                username::MIN_LENGTH
            )
            .into(),
        );
        return Err(error);
    }

    if username.len() > username::MAX_LENGTH as usize {
        let mut error = ValidationError::new("username_too_long");
        error.message = Some(
            format!(
                "Username must be at most {} characters",
                username::MAX_LENGTH
            )
            .into(),
        );
        return Err(error);
    }

    // 形式チェック
    if !USERNAME_REGEX.is_match(username) {
        let mut error = ValidationError::new("invalid_username_format");
        error.message =
            Some("Username can only contain letters, numbers, underscores, and hyphens".into());
        return Err(error);
    }
    Ok(())
}

/// パスワードの強度をバリデーション（基本チェック）
pub fn validate_password_strength(password: &str) -> Result<(), ValidationError> {
    // 基本的な長さチェックは validator の length で行う
    // ここでは追加の強度チェックを実装

    let has_lowercase = password.chars().any(|c| c.is_ascii_lowercase());
    let has_uppercase = password.chars().any(|c| c.is_ascii_uppercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());

    if !has_lowercase || !has_uppercase || !has_digit {
        let mut error = ValidationError::new("weak_password");
        error.message = Some("Password must contain at least one lowercase letter, one uppercase letter, and one digit".into());
        return Err(error);
    }

    Ok(())
}

/// メールアドレスの詳細バリデーション（TLDを含むドメインの検証）
pub fn validate_email_format(email: &str) -> Result<(), ValidationError> {
    // メールアドレスに@が含まれるかチェック
    if !email.contains('@') {
        let mut error = ValidationError::new("invalid_email_format");
        error.message = Some("Email must contain @ symbol".into());
        return Err(error);
    }

    // @の前にローカル部分があるかチェック
    if email.starts_with('@') {
        let mut error = ValidationError::new("invalid_email_format");
        error.message = Some("Email must have local part before @ symbol".into());
        return Err(error);
    }

    // validatorクレートのemail検証に加えて、ドメインにTLDがあるかチェック
    if !email.contains('.') || email.ends_with('.') {
        let mut error = ValidationError::new("invalid_email_format");
        error.message = Some("Email must have a valid domain with TLD".into());
        return Err(error);
    }

    // ドメイン部分の検証
    if let Some(at_pos) = email.rfind('@') {
        let domain = &email[at_pos + 1..];
        if !domain.contains('.') || domain.starts_with('.') || domain.ends_with('.') {
            let mut error = ValidationError::new("invalid_email_domain");
            error.message = Some("Email must have a valid domain with TLD".into());
            return Err(error);
        }

        // TLD部分の検証（最低2文字）
        if let Some(dot_pos) = domain.rfind('.') {
            let tld = &domain[dot_pos + 1..];
            if tld.len() < 2 {
                let mut error = ValidationError::new("invalid_email_tld");
                error.message = Some("Email TLD must be at least 2 characters".into());
                return Err(error);
            }
        }
    }

    Ok(())
}

/// 文字列が空白のみでないかをチェック
pub fn validate_not_empty_or_whitespace(value: &str) -> Result<(), ValidationError> {
    if value.trim().is_empty() {
        let mut error = ValidationError::new("empty_or_whitespace");
        error.message = Some("Field cannot be empty or contain only whitespace".into());
        return Err(error);
    }
    Ok(())
}

/// タスクタイトルのバリデーション
pub fn validate_task_title(title: &str) -> Result<(), ValidationError> {
    validate_not_empty_or_whitespace(title)?;

    // 特殊文字のチェック（必要に応じて）
    if title.contains('\0') || title.contains('\r') || title.contains('\n') {
        let mut error = ValidationError::new("invalid_characters");
        error.message =
            Some("Title cannot contain null, carriage return, or newline characters".into());
        return Err(error);
    }

    Ok(())
}

// =============================================================================
// テスト
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_username_validation() {
        // 有効なユーザー名
        assert!(validate_username("user123").is_ok());
        assert!(validate_username("test_user").is_ok());
        assert!(validate_username("user-name").is_ok());
        assert!(validate_username("abc").is_ok()); // 最小長3文字

        // 無効なユーザー名
        assert!(validate_username("ab").is_err()); // 短すぎる
        assert!(validate_username("a".repeat(31).as_str()).is_err()); // 長すぎる
        assert!(validate_username("user@123").is_err());
        assert!(validate_username("user 123").is_err());
        assert!(validate_username("user.123").is_err());
    }

    #[test]
    fn test_password_strength_validation() {
        // 強いパスワード
        assert!(validate_password_strength("Password123").is_ok());
        assert!(validate_password_strength("StrongPass1").is_ok());

        // 弱いパスワード
        assert!(validate_password_strength("password").is_err()); // 大文字・数字なし
        assert!(validate_password_strength("PASSWORD").is_err()); // 小文字・数字なし
        assert!(validate_password_strength("Password").is_err()); // 数字なし
        assert!(validate_password_strength("12345678").is_err()); // 文字なし
    }

    #[test]
    fn test_not_empty_or_whitespace() {
        // 有効な値
        assert!(validate_not_empty_or_whitespace("valid text").is_ok());
        assert!(validate_not_empty_or_whitespace("a").is_ok());

        // 無効な値
        assert!(validate_not_empty_or_whitespace("").is_err());
        assert!(validate_not_empty_or_whitespace("   ").is_err());
        assert!(validate_not_empty_or_whitespace("\t\n").is_err());
    }

    #[test]
    fn test_task_title_validation() {
        // 有効なタイトル
        assert!(validate_task_title("Valid Task Title").is_ok());
        assert!(validate_task_title("Task with symbols: !@#$%^&*()").is_ok());

        // 無効なタイトル
        assert!(validate_task_title("").is_err());
        assert!(validate_task_title("   ").is_err());
        assert!(validate_task_title("Title with\nnewline").is_err());
        assert!(validate_task_title("Title with\0null").is_err());
    }

    #[test]
    fn test_email_format_validation() {
        // 有効なメールアドレス
        assert!(validate_email_format("user@example.com").is_ok());
        assert!(validate_email_format("test@domain.co.uk").is_ok());
        assert!(validate_email_format("admin@subdomain.example.org").is_ok());

        // 無効なメールアドレス（TLDなし）
        assert!(validate_email_format("user@domain").is_err());
        assert!(validate_email_format("user@localhost").is_err());
        assert!(validate_email_format("invalid@").is_err());
        assert!(validate_email_format("@example.com").is_err());
        assert!(validate_email_format("user@.com").is_err());
        assert!(validate_email_format("user@domain.").is_err());
        assert!(validate_email_format("user@domain.c").is_err()); // TLDが1文字
    }

    #[test]
    fn test_validation_constants() {
        assert_eq!(username::MIN_LENGTH, 3);
        assert_eq!(username::MAX_LENGTH, 30);
        assert_eq!(password::MIN_LENGTH, 8);
        assert_eq!(task::TITLE_MAX_LENGTH, 200);
        assert_eq!(task::DESCRIPTION_MAX_LENGTH, 2000);
    }
}
