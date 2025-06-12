// task-backend/src/middleware/auth.rs
#![allow(dead_code)]

use crate::domain::user_model::UserClaims;
use crate::error::AppError;
use crate::utils::jwt::JwtManager;
use axum::{
    extract::{Request, State},
    http::{header, HeaderMap},
    middleware::Next,
    response::Response,
};
use axum_extra::extract::cookie::CookieJar;
use std::sync::Arc;
use tracing::{info, warn};

/// JWT認証ミドルウェアの設定
#[derive(Clone)]
pub struct AuthMiddlewareConfig {
    pub jwt_manager: Arc<JwtManager>,
    pub access_token_cookie_name: String,
    pub skip_auth_paths: Vec<String>,
    pub require_verified_email: bool,
    pub require_active_account: bool,
}

impl Default for AuthMiddlewareConfig {
    fn default() -> Self {
        Self {
            jwt_manager: Arc::new(
                JwtManager::from_env().expect("Failed to initialize JWT manager"),
            ),
            access_token_cookie_name: "access_token".to_string(),
            skip_auth_paths: vec![
                "/auth/signup".to_string(),
                "/auth/signin".to_string(),
                "/auth/refresh".to_string(),
                "/auth/forgot-password".to_string(),
                "/auth/reset-password".to_string(),
                "/health".to_string(),
                "/docs".to_string(),
            ],
            require_verified_email: false, // 開発環境では false
            require_active_account: true,
        }
    }
}

/// 認証済みユーザー情報を格納するエクステンション
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub claims: UserClaims,
    pub access_token: String,
}

impl AuthenticatedUser {
    pub fn new(claims: UserClaims, access_token: String) -> Self {
        Self {
            claims,
            access_token,
        }
    }

    pub fn user_id(&self) -> uuid::Uuid {
        self.claims.user_id
    }

    pub fn username(&self) -> &str {
        &self.claims.username
    }

    pub fn email(&self) -> &str {
        &self.claims.email
    }

    pub fn is_active(&self) -> bool {
        self.claims.is_active
    }

    pub fn is_email_verified(&self) -> bool {
        self.claims.email_verified
    }
}

/// JWT認証ミドルウェア
pub async fn jwt_auth_middleware(
    State(config): State<AuthMiddlewareConfig>,
    headers: HeaderMap,
    cookie_jar: CookieJar,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let path = request.uri().path().to_string();

    // 認証をスキップするパスかチェック
    if should_skip_auth(&path, &config.skip_auth_paths) {
        return Ok(next.run(request).await);
    }

    // トークンを抽出
    let token = extract_token(&headers, &cookie_jar, &config.access_token_cookie_name).ok_or_else(
        || {
            warn!(path = %path, "Missing authentication token");
            AppError::Unauthorized("Authentication required".to_string())
        },
    )?;

    // JWTを検証
    let access_claims = config
        .jwt_manager
        .verify_access_token(&token)
        .map_err(|e| {
            warn!(path = %path, error = %e, "Invalid access token");
            AppError::Unauthorized("Invalid or expired token".to_string())
        })?;

    // ユーザークレームを抽出（クローンして所有権を保持）
    let user_claims = access_claims.user.clone();

    // アカウント状態チェック
    if config.require_active_account && !user_claims.is_active {
        warn!(
            user_id = %user_claims.user_id,
            path = %path,
            "Access attempt with inactive account"
        );
        return Err(AppError::Forbidden("Account is inactive".to_string()));
    }

    // メール認証チェック
    if config.require_verified_email && !user_claims.email_verified {
        warn!(
            user_id = %user_claims.user_id,
            path = %path,
            "Access attempt with unverified email"
        );
        return Err(AppError::Forbidden(
            "Email verification required".to_string(),
        ));
    }

    // トークンの残り有効時間をチェック
    let remaining_minutes = config
        .jwt_manager
        .get_access_token_remaining_minutes(&access_claims);

    if remaining_minutes <= 0 {
        warn!(
            user_id = %user_claims.user_id,
            path = %path,
            "Access attempt with expired token"
        );
        return Err(AppError::Unauthorized("Token has expired".to_string()));
    }

    // 認証済みユーザー情報をリクエストに追加
    let authenticated_user = AuthenticatedUser::new(user_claims.clone(), token);
    request.extensions_mut().insert(authenticated_user);

    info!(
        user_id = %user_claims.user_id,
        username = %user_claims.username,
        path = %path,
        remaining_minutes = %remaining_minutes,
        "Authenticated request"
    );

    Ok(next.run(request).await)
}

/// 管理者権限チェックミドルウェア
pub async fn admin_auth_middleware(
    State(config): State<AuthMiddlewareConfig>,
    headers: HeaderMap,
    cookie_jar: CookieJar,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // まず基本の認証を実行
    let auth_result =
        jwt_auth_middleware(State(config), headers, cookie_jar, request, next.clone()).await;

    match auth_result {
        Ok(response) => Ok(response),
        Err(e) => Err(e),
    }

    // TODO: 管理者権限の具体的なチェックを実装
    // 現在はプレースホルダー
    // let authenticated_user = request
    //     .extensions()
    //     .get::<AuthenticatedUser>()
    //     .ok_or_else(|| AppError::Unauthorized("Authentication required".to_string()))?;

    // if !authenticated_user.claims.is_admin {
    //     return Err(AppError::Forbidden("Admin access required".to_string()));
    // }

    // Ok(next.run(request).await)
}

/// オプショナル認証ミドルウェア（認証なしでもアクセス可能だが、認証情報があれば追加）
pub async fn optional_auth_middleware(
    State(config): State<AuthMiddlewareConfig>,
    headers: HeaderMap,
    cookie_jar: CookieJar,
    mut request: Request,
    next: Next,
) -> Response {
    // トークンを抽出（なくてもエラーにしない）
    if let Some(token) = extract_token(&headers, &cookie_jar, &config.access_token_cookie_name) {
        // JWTを検証（失敗してもエラーにしない）
        if let Ok(access_claims) = config.jwt_manager.verify_access_token(&token) {
            let user_claims = access_claims.user;

            // アカウントがアクティブで有効な場合のみ認証情報を追加
            if user_claims.is_active {
                let authenticated_user = AuthenticatedUser::new(user_claims.clone(), token);
                request.extensions_mut().insert(authenticated_user);

                info!(
                    user_id = %user_claims.user_id,
                    username = %user_claims.username,
                    path = %request.uri().path(),
                    "Optional authentication successful"
                );
            }
        }
    }

    next.run(request).await
}

/// レート制限ミドルウェア（基本実装）
pub async fn rate_limit_middleware(headers: HeaderMap, request: Request, next: Next) -> Response {
    // TODO: 実際のレート制限ロジックを実装
    // 現在はプレースホルダー

    let client_ip = extract_client_ip(&headers);
    let path = request.uri().path();

    // 認証関連のエンドポイントは厳しくレート制限
    if is_auth_endpoint(path) {
        // TODO: Redis やインメモリストアを使用したレート制限
        info!(
            client_ip = ?client_ip,
            path = %path,
            "Rate limit check for auth endpoint"
        );
    }

    next.run(request).await
}

/// CORS ミドルウェア設定
pub fn cors_layer() -> tower_http::cors::CorsLayer {
    tower_http::cors::CorsLayer::new()
        .allow_origin(
            "http://localhost:3000"
                .parse::<axum::http::HeaderValue>()
                .unwrap(),
        ) // フロントエンドのURL
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PATCH,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
            axum::http::header::ACCEPT,
        ])
        .allow_credentials(true) // Cookie送信を許可
        .max_age(std::time::Duration::from_secs(3600)) // プリフライトリクエストのキャッシュ時間
}

/// セキュリティヘッダーミドルウェア
pub async fn security_headers_middleware(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;

    let headers = response.headers_mut();

    // セキュリティヘッダーを追加
    headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
    headers.insert("X-Frame-Options", "DENY".parse().unwrap());
    headers.insert("X-XSS-Protection", "1; mode=block".parse().unwrap());
    headers.insert(
        "Referrer-Policy",
        "strict-origin-when-cross-origin".parse().unwrap(),
    );
    headers.insert(
        "Content-Security-Policy",
        "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self'; connect-src 'self'; frame-ancestors 'none';"
            .parse()
            .unwrap(),
    );
    headers.insert(
        "Permissions-Policy",
        "camera=(), microphone=(), geolocation=()".parse().unwrap(),
    );

    response
}

// --- ヘルパー関数 ---

/// リクエストからトークンを抽出
fn extract_token(headers: &HeaderMap, cookie_jar: &CookieJar, cookie_name: &str) -> Option<String> {
    // Authorization ヘッダーからトークンを取得
    let auth_header = headers
        .get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|auth_str| auth_str.strip_prefix("Bearer ").map(|s| s.to_string()));

    // Cookieからトークンを取得（フォールバック）
    let cookie_token = cookie_jar
        .get(cookie_name)
        .map(|cookie| cookie.value().to_string());

    auth_header.or(cookie_token)
}

/// 認証をスキップするパスかチェック
fn should_skip_auth(path: &str, skip_paths: &[String]) -> bool {
    skip_paths
        .iter()
        .any(|skip_path| path.starts_with(skip_path) || path == skip_path)
}

/// 認証エンドポイントかチェック
fn is_auth_endpoint(path: &str) -> bool {
    path.starts_with("/auth/")
}

/// クライアントIPを抽出
fn extract_client_ip(headers: &HeaderMap) -> Option<String> {
    // X-Forwarded-For ヘッダーをチェック（プロキシ経由の場合）
    if let Some(forwarded_for) = headers.get("X-Forwarded-For") {
        if let Ok(forwarded_str) = forwarded_for.to_str() {
            // 最初のIPアドレスを取得
            return forwarded_str
                .split(',')
                .next()
                .map(|ip| ip.trim().to_string());
        }
    }

    // X-Real-IP ヘッダーをチェック
    if let Some(real_ip) = headers.get("X-Real-IP") {
        if let Ok(ip_str) = real_ip.to_str() {
            return Some(ip_str.to_string());
        }
    }

    None
}

/// リクエスト拡張からユーザー情報を取得するヘルパー
pub fn get_authenticated_user(request: &Request) -> Option<&AuthenticatedUser> {
    request.extensions().get::<AuthenticatedUser>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_skip_auth() {
        let skip_paths = vec![
            "/auth/signup".to_string(),
            "/auth/signin".to_string(),
            "/health".to_string(),
        ];

        assert!(should_skip_auth("/auth/signup", &skip_paths));
        assert!(should_skip_auth("/auth/signin", &skip_paths));
        assert!(should_skip_auth("/health", &skip_paths));
        assert!(!should_skip_auth("/users/profile", &skip_paths));
        assert!(!should_skip_auth("/tasks", &skip_paths));
    }

    #[test]
    fn test_is_auth_endpoint() {
        assert!(is_auth_endpoint("/auth/signup"));
        assert!(is_auth_endpoint("/auth/signin"));
        assert!(is_auth_endpoint("/auth/me"));
        assert!(!is_auth_endpoint("/users/profile"));
        assert!(!is_auth_endpoint("/tasks"));
    }

    #[test]
    fn test_extract_client_ip() {
        let mut headers = HeaderMap::new();

        // X-Forwarded-For ヘッダーのテスト
        headers.insert("X-Forwarded-For", "192.168.1.1, 10.0.0.1".parse().unwrap());
        assert_eq!(extract_client_ip(&headers), Some("192.168.1.1".to_string()));

        // X-Real-IP ヘッダーのテスト
        headers.clear();
        headers.insert("X-Real-IP", "203.0.113.195".parse().unwrap());
        assert_eq!(
            extract_client_ip(&headers),
            Some("203.0.113.195".to_string())
        );

        // ヘッダーがない場合
        headers.clear();
        assert_eq!(extract_client_ip(&headers), None);
    }
}
