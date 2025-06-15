// task-backend/src/api/mod.rs
use crate::config::AppConfig;
use crate::service::{
    auth_service::AuthService, role_service::RoleService, task_service::TaskService,
    user_service::UserService,
};
use crate::utils::jwt::JwtManager;
use std::sync::Arc;

pub mod dto;
pub mod handlers;

/// 統一されたアプリケーション状態
#[derive(Clone)]
pub struct AppState {
    pub auth_service: Arc<AuthService>,
    pub user_service: Arc<UserService>,
    pub role_service: Arc<RoleService>,
    pub task_service: Arc<TaskService>,
    pub jwt_manager: Arc<JwtManager>,
    pub cookie_config: CookieConfig,
    pub security_headers: SecurityHeaders,
}

/// Cookie設定
#[derive(Clone, Debug)]
pub struct CookieConfig {
    pub access_token_name: String,
    pub refresh_token_name: String,
    pub secure: bool,
    pub http_only: bool,
    pub path: String,
}

impl Default for CookieConfig {
    fn default() -> Self {
        Self {
            access_token_name: "access_token".to_string(),
            refresh_token_name: "refresh_token".to_string(),
            secure: std::env::var("APP_ENV").unwrap_or_default() == "production",
            http_only: true,
            path: "/".to_string(),
        }
    }
}

impl CookieConfig {
    pub fn from_app_config(app_config: &AppConfig) -> Self {
        Self {
            access_token_name: "access_token".to_string(),
            refresh_token_name: "refresh_token".to_string(),
            secure: app_config.security.cookie_secure,
            http_only: true,
            path: "/".to_string(),
        }
    }
}

/// セキュリティヘッダー設定
#[derive(Clone, Debug)]
pub struct SecurityHeaders {
    pub content_security_policy: String,
    pub x_frame_options: String,
    pub x_content_type_options: String,
    pub referrer_policy: String,
    pub permissions_policy: String,
}

impl Default for SecurityHeaders {
    fn default() -> Self {
        Self {
            content_security_policy: "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'".to_string(),
            x_frame_options: "DENY".to_string(),
            x_content_type_options: "nosniff".to_string(),
            referrer_policy: "strict-origin-when-cross-origin".to_string(),
            permissions_policy: "geolocation=(), microphone=(), camera=()".to_string(),
        }
    }
}

impl AppState {
    #[allow(dead_code)]
    pub fn new(
        auth_service: Arc<AuthService>,
        user_service: Arc<UserService>,
        role_service: Arc<RoleService>,
        task_service: Arc<TaskService>,
        jwt_manager: Arc<JwtManager>,
    ) -> Self {
        Self {
            auth_service,
            user_service,
            role_service,
            task_service,
            jwt_manager,
            cookie_config: CookieConfig::default(),
            security_headers: SecurityHeaders::default(),
        }
    }

    pub fn with_config(
        auth_service: Arc<AuthService>,
        user_service: Arc<UserService>,
        role_service: Arc<RoleService>,
        task_service: Arc<TaskService>,
        jwt_manager: Arc<JwtManager>,
        app_config: &AppConfig,
    ) -> Self {
        Self {
            auth_service,
            user_service,
            role_service,
            task_service,
            jwt_manager,
            cookie_config: CookieConfig::from_app_config(app_config),
            security_headers: SecurityHeaders::default(),
        }
    }
}

/// JWT マネージャーを提供するトレイト
pub trait HasJwtManager {
    fn jwt_manager(&self) -> &Arc<JwtManager>;
    fn cookie_config(&self) -> &CookieConfig;
}

impl HasJwtManager for AppState {
    fn jwt_manager(&self) -> &Arc<JwtManager> {
        &self.jwt_manager
    }

    fn cookie_config(&self) -> &CookieConfig {
        &self.cookie_config
    }
}
