// src/main.rs
use std::sync::Arc;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod api;
mod config;
mod db;
mod domain;
mod error;
mod middleware;
mod repository;
mod service;
mod shared;
mod types;
mod utils;

use crate::api::handlers::{
    activity_log_handler::activity_log_router,
    admin_handler::admin_router,
    analytics_handler::analytics_router_with_state,
    attachment_handler::attachment_routes,
    audit_log_handler::audit_log_router,
    auth_handler::auth_router_with_state,
    gdpr_handler::gdpr_router_with_state,
    organization_handler::organization_router_with_state,
    organization_hierarchy_handler::organization_hierarchy_router,
    payment_handler::payment_router_with_state,
    permission_handler::permission_router_with_state,
    role_handler::role_router_with_state,
    security_handler::security_router,
    subscription_handler::subscription_router_with_state,
    system_handler::system_router_with_state,
    task_handler::{task_router_with_state, task_router_with_unified_permission},
    task_handler_v2::multi_tenant_task_router,
    team_handler::{team_router_with_state, team_router_with_unified_permission},
    user_handler::user_router_with_state,
};
use crate::api::AppState;
use crate::config::AppConfig;
use crate::db::{create_db_pool, create_db_pool_with_schema, create_schema, schema_exists};
use crate::middleware::auth::{
    cors_layer, jwt_auth_middleware, security_headers_middleware, AuthMiddlewareConfig,
};
use crate::repository::{
    activity_log_repository::ActivityLogRepository,
    login_attempt_repository::LoginAttemptRepository,
    organization_repository::OrganizationRepository,
    password_reset_token_repository::PasswordResetTokenRepository,
    refresh_token_repository::RefreshTokenRepository, role_repository::RoleRepository,
    security_incident_repository::SecurityIncidentRepository,
    subscription_history_repository::SubscriptionHistoryRepository,
    team_repository::TeamRepository, user_repository::UserRepository,
};
use crate::service::{
    attachment_service::AttachmentService,
    audit_log_service::AuditLogService,
    auth_service::AuthService,
    organization_service::OrganizationService,
    payment_service::PaymentService,
    permission_service::PermissionService,
    role_service::RoleService,
    security_service::SecurityService,
    storage_service::{self, StorageConfig},
    subscription_service::SubscriptionService,
    task_service::TaskService,
    team_service::TeamService,
    user_service::UserService,
};
use crate::utils::{
    email::{EmailConfig, EmailService},
    jwt::{JwtConfig, JwtManager},
    password::{Argon2Config, PasswordManager, PasswordPolicy},
};
use axum::{middleware as axum_middleware, Router};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // .envファイルを読み込む
    dotenvy::dotenv().ok();

    // トレーシングの設定
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "task_backend=info,tower_http=info,axum::rejection=trace".into()
            }),
        )
        .with(fmt::layer())
        .init();

    tracing::info!("🚀 Starting Task Backend server...");

    // 統合設定を読み込む
    let app_config = AppConfig::from_env().expect("Failed to load unified configuration");
    tracing::info!("📋 Unified configuration loaded");
    tracing::info!("   • Environment: {}", app_config.environment);
    tracing::info!("   • Server: {}:{}", app_config.host, app_config.port);
    tracing::info!("   • Database: configured");
    tracing::info!("   • JWT: configured");
    tracing::info!(
        "   • Email: configured (dev mode: {})",
        std::env::var("EMAIL_DEVELOPMENT_MODE").unwrap_or_else(|_| "false".to_string())
    );
    tracing::info!(
        "   • Security: cookie_secure={}",
        app_config.security.cookie_secure
    );

    // データベース接続を作成
    let db_pool = if let Ok(schema) = std::env::var("DATABASE_SCHEMA") {
        tracing::info!("🗃️  Using schema: {}", schema);

        // まず基本接続を作成
        let base_pool = create_db_pool(&app_config)
            .await
            .expect("Failed to create base database connection");

        // スキーマの存在を確認し、なければ作成
        let schema_exists = schema_exists(&base_pool, &schema)
            .await
            .expect("Failed to check schema existence");

        if !schema_exists {
            tracing::info!("📝 Schema does not exist, creating it: {}", schema);
            create_schema(&base_pool, &schema)
                .await
                .expect("Failed to create schema");
        }

        // スキーマを指定して接続プールを作成
        create_db_pool_with_schema(&app_config, &schema)
            .await
            .expect("Failed to create database pool with schema")
    } else {
        // 通常の接続プールを作成（スキーマ指定なし）
        create_db_pool(&app_config)
            .await
            .expect("Failed to create database pool")
    };

    tracing::info!("✅ Database pool created successfully.");

    // 統合設定からユーティリティサービスを初期化
    let jwt_config = JwtConfig::from_env().expect("Failed to load JWT configuration");
    let jwt_manager =
        Arc::new(JwtManager::new(jwt_config).expect("Failed to initialize JWT manager"));
    let argon2_config = Argon2Config::from_env();
    let password_policy = PasswordPolicy::from_env();
    let password_manager = Arc::new(
        PasswordManager::new(argon2_config, password_policy)
            .expect("Failed to initialize password manager"),
    );
    let email_config = EmailConfig::from_env().expect("Failed to load email configuration");
    let email_service =
        Arc::new(EmailService::new(email_config).expect("Failed to initialize email service"));

    tracing::info!("🔧 Utility services initialized.");

    // リポジトリの作成
    let user_repo = Arc::new(UserRepository::new(db_pool.clone()));
    let role_repo = Arc::new(RoleRepository::new(Arc::new(db_pool.clone())));
    let refresh_token_repo = Arc::new(RefreshTokenRepository::new(db_pool.clone()));
    let password_reset_token_repo = Arc::new(PasswordResetTokenRepository::new(db_pool.clone()));
    let email_verification_token_repo = Arc::new(crate::repository::email_verification_token_repository::EmailVerificationTokenRepository::new(db_pool.clone()));
    let organization_repo = Arc::new(OrganizationRepository::new(db_pool.clone()));
    let team_repo = Arc::new(TeamRepository::new(db_pool.clone()));
    let subscription_history_repo = Arc::new(SubscriptionHistoryRepository::new(db_pool.clone()));
    let daily_activity_summary_repo = Arc::new(
        crate::repository::daily_activity_summary_repository::DailyActivitySummaryRepository::new(
            db_pool.clone(),
        ),
    );
    let feature_usage_metrics_repo = Arc::new(
        crate::repository::feature_usage_metrics_repository::FeatureUsageMetricsRepository::new(
            db_pool.clone(),
        ),
    );
    let user_settings_repo = Arc::new(
        crate::repository::user_settings_repository::UserSettingsRepository::new(db_pool.clone()),
    );
    let bulk_operation_history_repo = Arc::new(
        crate::repository::bulk_operation_history_repository::BulkOperationHistoryRepository::new(
            db_pool.clone(),
        ),
    );

    // Security repositories
    let activity_log_repo = Arc::new(ActivityLogRepository::new(db_pool.clone()));
    let security_incident_repo = Arc::new(SecurityIncidentRepository::new(db_pool.clone()));
    let login_attempt_repo = Arc::new(LoginAttemptRepository::new(db_pool.clone()));
    let audit_log_repo =
        Arc::new(crate::repository::audit_log_repository::AuditLogRepository::new(db_pool.clone()));

    tracing::info!("📚 Repositories created.");

    // サービスの作成
    let auth_service = Arc::new(AuthService::new(
        user_repo.clone(),
        role_repo.clone(),
        refresh_token_repo.clone(),
        password_reset_token_repo.clone(),
        email_verification_token_repo.clone(),
        activity_log_repo.clone(),
        login_attempt_repo.clone(),
        password_manager.clone(),
        jwt_manager.clone(),
        email_service.clone(),
        Arc::new(db_pool.clone()),
    ));

    let user_service = Arc::new(UserService::new(
        user_repo.clone(),
        user_settings_repo.clone(),
        bulk_operation_history_repo.clone(),
        email_verification_token_repo.clone(),
    ));

    let role_service = Arc::new(RoleService::new(
        Arc::new(db_pool.clone()),
        role_repo.clone(),
        user_repo.clone(),
    ));

    // Audit log service creation
    let audit_log_service = Arc::new(AuditLogService::new(audit_log_repo.clone()));

    // Team service creation (needs to be created before TaskService)
    let team_service = Arc::new(TeamService::new(
        Arc::new(db_pool.clone()),
        TeamRepository::new(db_pool.clone()),
        UserRepository::new(db_pool.clone()),
        email_service.clone(),
    ));

    // Task service creation (depends on team_service and audit_log_service)
    let task_service = Arc::new(TaskService::new(
        db_pool.clone(),
        team_service.clone(),
        audit_log_service.clone(),
    ));

    // ストレージサービスの初期化（必須）
    tracing::info!("📦 Initializing storage service...");
    let storage_config = StorageConfig::from_env().expect(
        "Failed to load storage configuration. Please set STORAGE_* environment variables.",
    );
    let storage_service = storage_service::create_storage_service(storage_config)
        .await
        .expect("Failed to initialize storage service");
    tracing::info!("✅ Storage service initialized successfully");

    // 添付ファイルサービスの初期化
    let attachment_service = Arc::new(AttachmentService::new(
        db_pool.clone(),
        storage_service.clone(),
    ));

    let subscription_service = Arc::new(SubscriptionService::new(
        db_pool.clone(),
        email_service.clone(),
    ));

    let payment_service = Arc::new(PaymentService::new(
        db_pool.clone(),
        subscription_service.clone(),
    ));

    let organization_service = Arc::new(OrganizationService::new(
        OrganizationRepository::new(db_pool.clone()),
        TeamRepository::new(db_pool.clone()),
        UserRepository::new(db_pool.clone()),
        SubscriptionHistoryRepository::new(db_pool.clone()),
    ));

    let team_invitation_service = Arc::new(
        crate::service::team_invitation_service::TeamInvitationService::new(
            crate::repository::team_invitation_repository::TeamInvitationRepository::new(
                db_pool.clone(),
            ),
            TeamRepository::new(db_pool.clone()),
            UserRepository::new(db_pool.clone()),
        ),
    );

    // Security services creation
    let security_service = Arc::new(SecurityService::new(
        refresh_token_repo.clone(),
        password_reset_token_repo.clone(),
        activity_log_repo.clone(),
        security_incident_repo.clone(),
        login_attempt_repo.clone(),
        user_repo.clone(),
    ));

    // Permission service creation
    let permission_service = Arc::new(PermissionService::new(
        role_repo.clone(),
        user_repo.clone(),
        team_repo.clone(),
        organization_repo.clone(),
    ));

    tracing::info!("🎯 Business services created.");

    // 認証ミドルウェア設定
    let auth_middleware_config = AuthMiddlewareConfig {
        jwt_manager: jwt_manager.clone(),
        user_repository: user_repo.clone(),
        access_token_cookie_name: "access_token".to_string(),
        skip_auth_paths: vec![
            "/auth/signup".to_string(),
            "/auth/signin".to_string(),
            "/auth/refresh".to_string(),
            "/auth/forgot-password".to_string(),
            "/auth/reset-password".to_string(),
            "/auth/verify-email".to_string(),
            "/auth/resend-verification".to_string(),
            "/health".to_string(),
            "/".to_string(),
            "/share".to_string(), // 共有リンクのプレフィックス（認証不要）
            "/webhooks/stripe".to_string(), // Stripe Webhook（認証不要）
        ],
        admin_only_paths: vec!["/admin".to_string()],
        require_verified_email: !app_config.is_development(), // 開発環境では false
        require_active_account: true,
    };

    // 統一されたAppStateを作成（統合設定対応）
    let app_state = AppState::with_config(
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
        feature_usage_metrics_repo,
        permission_service,
        security_service,
        attachment_service,
        audit_log_service,
        activity_log_repo.clone(),
        jwt_manager.clone(),
        Arc::new(db_pool.clone()),
        &app_config,
    );

    // ルーターの設定
    let auth_router = auth_router_with_state(app_state.clone());
    let user_router = user_router_with_state(app_state.clone());
    let role_router = role_router_with_state(app_state.clone());
    let task_router = task_router_with_state(app_state.clone());
    let team_router = team_router_with_state(app_state.clone());
    let organization_router = organization_router_with_state(app_state.clone());
    let subscription_router = subscription_router_with_state(app_state.clone());
    let payment_router = payment_router_with_state(app_state.clone());
    let permission_router = permission_router_with_state(app_state.clone());
    let analytics_router = analytics_router_with_state(app_state.clone());
    let security_router = security_router(app_state.clone());
    let system_router = system_router_with_state(Arc::new(app_state.clone()));
    let admin_router = admin_router(app_state.clone());
    let hierarchy_router = organization_hierarchy_router(app_state.clone());
    let gdpr_router = gdpr_router_with_state(app_state.clone());

    // メインアプリケーションルーターの構築
    let app_router = Router::new()
        .merge(auth_router)
        .merge(user_router)
        .merge(role_router)
        .merge(task_router)
        .merge(team_router)
        .merge(organization_router)
        .merge(subscription_router)
        .merge(payment_router)
        .merge(permission_router)
        .merge(analytics_router)
        .merge(security_router)
        .merge(system_router)
        .merge(admin_router)
        .merge(hierarchy_router)
        .merge(gdpr_router)
        .merge(attachment_routes(app_state.clone()))
        .merge(activity_log_router(app_state.clone()))
        .merge(audit_log_router(app_state.clone()))
        .merge(multi_tenant_task_router(app_state.clone()))
        // 統一権限チェックミドルウェアを使用した実験的実装
        .merge(task_router_with_unified_permission(app_state.clone()))
        .merge(team_router_with_unified_permission(app_state.clone()))
        .route(
            "/",
            axum::routing::get(|| async { "Task Backend API v1.0" }),
        )
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(axum_middleware::from_fn_with_state(
                    auth_middleware_config,
                    jwt_auth_middleware,
                ))
                .layer(axum_middleware::from_fn_with_state(
                    middleware::activity_logger::ActivityLogger::new(activity_log_repo.clone()),
                    middleware::activity_logger::log_activity,
                ))
                .layer(axum_middleware::from_fn(security_headers_middleware))
                .layer(cors_layer()),
        );

    tracing::info!("🛣️  Routers configured:");
    tracing::info!("   • Authentication: /auth/*");
    tracing::info!("   • User Management: /users/*");
    tracing::info!("   • Role Management: /roles/*");
    tracing::info!("   • Task Management: /tasks/*");
    tracing::info!("   • File Attachments: /tasks/*/attachments, /attachments/*");
    tracing::info!(
        "   • Share Links: /attachments/*/share-links, /share-links/*, /share/* (public)"
    );
    tracing::info!("   • Team Management: /teams/*");
    tracing::info!("   • Organization Management: /organizations/*");
    tracing::info!("   • Subscription Management: /subscriptions/*");
    tracing::info!("   • Payment Processing: /payments/*, /webhooks/stripe");
    tracing::info!("   • Permission Management: /permissions/*");
    tracing::info!("   • Analytics: /analytics/*");
    tracing::info!("   • Admin Management: /admin/*");
    tracing::info!(
        "   • Organization Hierarchy: /organizations/*/hierarchy, /organizations/*/departments/*"
    );
    tracing::info!("   • GDPR Compliance: /gdpr/*, /admin/gdpr/*");
    tracing::info!("   • Activity Logs: /activity-logs/me, /admin/activity-logs");
    tracing::info!("   • Audit Logs: /audit-logs/me, /teams/*/audit-logs, /admin/audit-logs/*");
    tracing::info!("   • Health Check: /health");

    // サーバーの起動
    let server_addr = if let Ok(addr) = std::env::var("SERVER_ADDR") {
        addr
    } else {
        format!("{}:{}", app_config.host, app_config.port)
    };
    tracing::info!("🌐 Server listening on {}", server_addr);
    tracing::info!("📚 API Documentation: http://{}/docs", server_addr);

    let listener = TcpListener::bind(&server_addr).await?;

    tracing::info!("🎉 Task Backend server started successfully!");

    axum::serve(listener, app_router.into_make_service()).await?;

    Ok(())
}
