// task-backend/src/api/mod.rs
use crate::config::AppConfig;
use crate::features::auth::service::AuthService;
use crate::features::storage::attachment::service::AttachmentService;
use crate::repository::{
    bulk_operation_history_repository::BulkOperationHistoryRepository,
    daily_activity_summary_repository::DailyActivitySummaryRepository,
    feature_usage_metrics_repository::FeatureUsageMetricsRepository,
    subscription_history_repository::SubscriptionHistoryRepository,
};
use crate::service::{
    feature_tracking_service::FeatureTrackingService, organization_service::OrganizationService,
    payment_service::PaymentService, permission_service::PermissionService,
    role_service::RoleService, security_service::SecurityService,
    subscription_service::SubscriptionService, task_service::TaskService,
    team_invitation_service::TeamInvitationService, team_service::TeamService,
    user_service::UserService,
};
use crate::utils::jwt::JwtManager;
use sea_orm::DatabaseConnection;
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
    pub team_service: Arc<TeamService>,
    pub team_invitation_service: Arc<TeamInvitationService>,
    pub organization_service: Arc<OrganizationService>,
    pub subscription_service: Arc<SubscriptionService>,
    pub payment_service: Arc<PaymentService>,
    pub subscription_history_repo: Arc<SubscriptionHistoryRepository>,
    pub bulk_operation_history_repo: Arc<BulkOperationHistoryRepository>,
    pub daily_activity_summary_repo: Arc<DailyActivitySummaryRepository>,
    pub feature_usage_metrics_repo: Arc<FeatureUsageMetricsRepository>,
    pub feature_tracking_service: Arc<FeatureTrackingService>,
    pub permission_service: Arc<PermissionService>,
    pub security_service: Arc<SecurityService>,
    pub attachment_service: Arc<AttachmentService>,
    pub jwt_manager: Arc<JwtManager>,
    pub db: Arc<DatabaseConnection>,
    pub db_pool: Arc<DatabaseConnection>,
    pub cookie_config: CookieConfig,
    pub security_headers: SecurityHeaders,
    pub server_addr: String,
    pub config: Arc<AppConfig>,
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
    #[allow(clippy::too_many_arguments)]
    pub fn with_config(
        auth_service: Arc<AuthService>,
        user_service: Arc<UserService>,
        role_service: Arc<RoleService>,
        task_service: Arc<TaskService>,
        team_service: Arc<TeamService>,
        team_invitation_service: Arc<TeamInvitationService>,
        organization_service: Arc<OrganizationService>,
        subscription_service: Arc<SubscriptionService>,
        payment_service: Arc<PaymentService>,
        subscription_history_repo: Arc<SubscriptionHistoryRepository>,
        bulk_operation_history_repo: Arc<BulkOperationHistoryRepository>,
        daily_activity_summary_repo: Arc<DailyActivitySummaryRepository>,
        feature_usage_metrics_repo: Arc<FeatureUsageMetricsRepository>,
        permission_service: Arc<PermissionService>,
        security_service: Arc<SecurityService>,
        attachment_service: Arc<AttachmentService>,
        jwt_manager: Arc<JwtManager>,
        db_pool: Arc<DatabaseConnection>,
        app_config: &AppConfig,
    ) -> Self {
        Self {
            auth_service,
            user_service,
            role_service,
            task_service,
            team_service,
            team_invitation_service,
            organization_service,
            subscription_service,
            payment_service,
            subscription_history_repo,
            bulk_operation_history_repo,
            daily_activity_summary_repo,
            feature_usage_metrics_repo: feature_usage_metrics_repo.clone(),
            feature_tracking_service: Arc::new(FeatureTrackingService::new(
                feature_usage_metrics_repo,
            )),
            permission_service,
            security_service,
            attachment_service,
            jwt_manager,
            db: db_pool.clone(),
            db_pool,
            cookie_config: CookieConfig::from_app_config(app_config),
            security_headers: SecurityHeaders::default(),
            server_addr: format!("{}:{}", app_config.host, app_config.port),
            config: Arc::new(app_config.clone()),
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
