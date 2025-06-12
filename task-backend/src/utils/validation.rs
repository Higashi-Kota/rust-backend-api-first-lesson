use once_cell::sync::Lazy;
use regex::Regex;
use validator::ValidationError;

/// ユーザー名用正規表現（文字、数字、アンダースコアのみ）
pub static USERNAME_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[a-zA-Z0-9_]+$").unwrap());

/// ユーザー名バリデーション
pub fn validate_username(username: &str) -> Result<(), ValidationError> {
    if USERNAME_REGEX.is_match(username) {
        Ok(())
    } else {
        Err(ValidationError::new("invalid_username"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_username_regex() {
        assert!(USERNAME_REGEX.is_match("validusername"));
        assert!(USERNAME_REGEX.is_match("valid_username"));
        assert!(USERNAME_REGEX.is_match("valid123"));
        assert!(!USERNAME_REGEX.is_match("invalid-username"));
        assert!(!USERNAME_REGEX.is_match("invalid username"));
        assert!(!USERNAME_REGEX.is_match("invalid!username"));
    }
}
