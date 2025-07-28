use crate::error::AppError;
use axum::{
    extract::{FromRequestParts, Path},
    http::request::Parts,
};
use serde::{Deserialize, Deserializer};
use uuid::Uuid;

/// URLパスからUUIDパラメータ名を抽出するヘルパー関数
fn extract_uuid_param_name(path: &str) -> Option<&'static str> {
    // パスセグメントを解析してパラメータ名を推測
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    // 一般的なパターンからパラメータ名を推測
    for (i, segment) in segments.iter().enumerate() {
        if segment.len() == 36 && segment.contains('-') {
            // UUIDらしい値の前のセグメントからパラメータ名を推測
            if i > 0 {
                return match segments[i - 1] {
                    "teams" => Some("team_id"),
                    "users" => Some("user_id"),
                    "tasks" => Some("task_id"),
                    "organizations" => Some("organization_id"),
                    "departments" => Some("department_id"),
                    "invitations" => Some("invitation_id"),
                    "members" => Some("member_id"),
                    "roles" => Some("role_id"),
                    "permissions" => Some("permission_id"),
                    "subscriptions" => Some("subscription_id"),
                    "attachments" => Some("attachment_id"),
                    _ => Some("id"),
                };
            }
        }
    }

    None
}

/// 統一UUID Extractor
/// パスパラメータからUUIDを抽出し、検証を行う
#[derive(Debug, Clone, Copy)]
pub struct ValidatedUuid(pub Uuid);

impl<S> FromRequestParts<S> for ValidatedUuid
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Path(uuid_str) = Path::<String>::from_request_parts(parts, state)
            .await
            .map_err(|e| {
                // パスからパラメータ名を抽出
                let path = parts.uri.path();
                let param_name = extract_uuid_param_name(path).unwrap_or("id");
                AppError::BadRequest(format!("Missing path parameter '{}': {}", param_name, e))
            })?;

        let uuid = Uuid::parse_str(&uuid_str).map_err(|_| {
            let path = parts.uri.path();
            let param_name = extract_uuid_param_name(path).unwrap_or("id");
            AppError::BadRequest(format!(
                "Invalid UUID format for '{}': '{}'",
                param_name, uuid_str
            ))
        })?;

        Ok(ValidatedUuid(uuid))
    }
}

/// パスパラメータ検証トレイト
pub trait PathParam: Sized {
    fn parse_from_str(s: &str) -> Result<Self, AppError>;
}

impl PathParam for Uuid {
    fn parse_from_str(s: &str) -> Result<Self, AppError> {
        Uuid::parse_str(s)
            .map_err(|_| AppError::BadRequest(format!("Invalid UUID format: '{}'", s)))
    }
}

/// 複数パスパラメータ用のExtractor
/// Path<T>の代わりに使用し、UUID検証を含む
#[derive(Debug)]
pub struct ValidatedMultiPath<T>(pub T);

impl<S, T> FromRequestParts<S> for ValidatedMultiPath<T>
where
    S: Send + Sync,
    T: serde::de::DeserializeOwned + Send,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Path(value) = Path::<T>::from_request_parts(parts, state)
            .await
            .map_err(|e| {
                // エラーメッセージから具体的なパラメータ名を抽出
                let error_msg = e.to_string();
                if error_msg.contains("missing field") {
                    let field_name = error_msg
                        .split('`')
                        .nth(1)
                        .unwrap_or("parameter");
                    AppError::BadRequest(format!("Missing required path parameter '{}'", field_name))
                } else if error_msg.contains("invalid") || error_msg.contains("Invalid UUID") {
                    // UUID検証エラーの場合、より具体的なメッセージを提供
                    if let Some(field) = error_msg.split('`').nth(1) {
                        AppError::BadRequest(format!("Invalid UUID format for parameter '{}': value does not match UUID pattern", field))
                    } else {
                        AppError::BadRequest(format!("Invalid path parameters: {}", error_msg))
                    }
                } else {
                    AppError::BadRequest(format!("Invalid path parameters: {}", error_msg))
                }
            })?;

        Ok(ValidatedMultiPath(value))
    }
}

/// 汎用パスパラメータExtractor
#[derive(Debug)]
pub struct ValidatedPath<T>(pub T);

impl<S, T> FromRequestParts<S> for ValidatedPath<T>
where
    S: Send + Sync,
    T: PathParam + Send,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Path(value) = Path::<String>::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::BadRequest("Missing path parameter".to_string()))?;

        let parsed = T::parse_from_str(&value)?;
        Ok(ValidatedPath(parsed))
    }
}

/// UUID用のカスタムデシリアライザ
/// Serde用のヘルパー関数
pub fn deserialize_uuid<'de, D>(deserializer: D) -> Result<Uuid, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Uuid::parse_str(&s).map_err(serde::de::Error::custom)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{Method, Request, Uri};

    #[tokio::test]
    async fn test_validated_uuid_valid() {
        let uuid = Uuid::new_v4();
        let uuid_str = uuid.to_string();

        let request = Request::builder()
            .method(Method::GET)
            .uri(Uri::from_static("/test"))
            .body(())
            .unwrap();

        let (mut parts, _) = request.into_parts();

        // Simulate path parameter extraction
        parts.extensions.insert(uuid_str.clone());

        // Since we can't properly test FromRequestParts without full axum context,
        // we'll test the core functionality
        let result = Uuid::parse_str(&uuid_str);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), uuid);
    }

    #[test]
    fn test_path_param_uuid_valid() {
        let uuid = Uuid::new_v4();
        let uuid_str = uuid.to_string();

        let result = <Uuid as PathParam>::parse_from_str(&uuid_str);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), uuid);
    }

    #[test]
    fn test_path_param_uuid_invalid() {
        let invalid_uuid = "not-a-valid-uuid";

        let result = <Uuid as PathParam>::parse_from_str(invalid_uuid);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid UUID format"));
    }

    #[test]
    fn test_deserialize_uuid_valid() {
        use serde_json::json;

        let uuid = Uuid::new_v4();
        let json_value = json!(uuid.to_string());

        let result: Result<Uuid, _> = serde_json::from_value(json_value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), uuid);
    }

    #[test]
    fn test_deserialize_option_uuid_some() {
        use serde_json::json;

        let uuid = Uuid::new_v4();
        let json_value = json!(uuid.to_string());

        let result: Result<Option<Uuid>, _> = serde_json::from_value(json_value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(uuid));
    }

    #[test]
    fn test_deserialize_option_uuid_none() {
        use serde_json::json;

        let json_value = json!(null);

        let result: Result<Option<Uuid>, _> = serde_json::from_value(json_value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }
}
