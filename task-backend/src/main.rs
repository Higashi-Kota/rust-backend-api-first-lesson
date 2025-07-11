// src/main.rs
use std::sync::Arc;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod api;
mod config;
mod core;
mod db;
mod error;
mod features;
mod infrastructure;
mod middleware;
mod shared;
mod utils;

// Import routers - using new feature modules where available
use crate::api::AppState;
use crate::config::AppConfig;
use crate::db::{create_db_pool, create_db_pool_with_schema, create_schema, schema_exists};
use crate::features::analytics::repositories::activity_log::ActivityLogRepository;
use crate::features::auth::repositories::login_attempt_repository::LoginAttemptRepository;
use crate::features::auth::{
    handler::auth_router_with_state,
    middleware::{
        cors_layer, jwt_auth_middleware, security_headers_middleware, AuthMiddlewareConfig,
    },
    repositories::{
        email_verification_token_repository::EmailVerificationTokenRepository,
        password_reset_token_repository::PasswordResetTokenRepository,
        refresh_token_repository::RefreshTokenRepository,
    },
    service::AuthService,
};
use crate::features::gdpr::handler::gdpr_router_with_state;
use crate::features::organization::handlers::organization_hierarchy_handler::organization_hierarchy_router;
use crate::features::organization::repositories::organization::OrganizationRepository;
use crate::features::organization::services::organization::OrganizationService;
use crate::features::security::repositories::role::RoleRepository;
use crate::features::security::repositories::security_incident::SecurityIncidentRepository;
use crate::features::security::services::permission::PermissionService;
use crate::features::security::services::role::RoleService;
use crate::features::security::services::security::SecurityService;
use crate::features::storage::attachment::handler::attachment_routes;
use crate::features::storage::{
    attachment::service::AttachmentService,
    service::{self as storage_service, StorageConfig},
};
use crate::features::subscription::repositories::history::SubscriptionHistoryRepository;
use crate::features::subscription::services::subscription::SubscriptionService;
use crate::features::system::handlers::system_handler::system_router_with_state;
use crate::features::task::{handlers::task::task_router_with_state, services::task::TaskService};
use crate::features::team::handlers::team_router_with_state;
use crate::features::team::repositories::team::TeamRepository;
use crate::features::team::services::team::TeamService;
use crate::features::user::handlers::user_handler::user_router_with_state;
use crate::features::user::repositories::{
    user::UserRepository, user_settings::UserSettingsRepository,
};
use crate::features::user::services::user_service::UserService;
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
    let email_verification_token_repo =
        Arc::new(EmailVerificationTokenRepository::new(db_pool.clone()));
    let organization_repo = Arc::new(OrganizationRepository::new(db_pool.clone()));
    let team_repo = Arc::new(TeamRepository::new(db_pool.clone()));
    let subscription_history_repo = Arc::new(SubscriptionHistoryRepository::new(db_pool.clone()));
    let daily_activity_summary_repo = Arc::new(
        crate::features::analytics::repositories::daily_activity_summary::DailyActivitySummaryRepository::new(
            db_pool.clone(),
        ),
    );
    let feature_usage_metrics_repo = Arc::new(
        crate::features::analytics::repositories::feature_usage_metrics::FeatureUsageMetricsRepository::new(
            db_pool.clone(),
        ),
    );
    let user_settings_repo = Arc::new(UserSettingsRepository::new(db_pool.clone()));
    let bulk_operation_history_repo = Arc::new(
        crate::features::admin::repositories::bulk_operation_history::BulkOperationHistoryRepository::new(
            db_pool.clone(),
        ),
    );

    // Security repositories
    let activity_log_repo = Arc::new(ActivityLogRepository::new(db_pool.clone()));
    let security_incident_repo = Arc::new(SecurityIncidentRepository::new(db_pool.clone()));
    let login_attempt_repo = Arc::new(LoginAttemptRepository::new(db_pool.clone()));

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

    let payment_service = Arc::new(
        crate::features::payment::services::payment_service::PaymentService::new(
            db_pool.clone(),
            subscription_service.clone(),
        ),
    );

    let team_service = Arc::new(TeamService::new(
        Arc::new(db_pool.clone()),
        TeamRepository::new(db_pool.clone()),
        UserRepository::new(db_pool.clone()),
        email_service.clone(),
    ));

    let organization_service = Arc::new(OrganizationService::new(
        OrganizationRepository::new(db_pool.clone()),
        TeamRepository::new(db_pool.clone()),
        UserRepository::new(db_pool.clone()),
        SubscriptionHistoryRepository::new(db_pool.clone()),
    ));

    let team_invitation_service = Arc::new(
        crate::features::team::services::team_invitation::TeamInvitationService::new(
            crate::features::team::repositories::team_invitation::TeamInvitationRepository::new(
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

    // Task service creation
    let task_service = Arc::new(TaskService::new(db_pool.clone()));

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
        task_service,
        jwt_manager.clone(),
        Arc::new(db_pool.clone()),
        &app_config,
    );

    // ルーターの設定
    let auth_router = auth_router_with_state(app_state.clone());
    let user_router = user_router_with_state(app_state.clone());
    // Use routers from feature modules
    let role_router =
        crate::features::security::handlers::role::role_router_with_state(app_state.clone());
    let team_router = team_router_with_state(app_state.clone());
    let organization_router =
        crate::features::organization::handlers::organization::organization_router_with_state(
            app_state.clone(),
        );
    let subscription_router =
        crate::features::subscription::handlers::subscription::subscription_router_with_state()
            .with_state(app_state.clone());
    let payment_router =
        crate::features::payment::handlers::payment_handler::payment_router_with_state(
            app_state.clone(),
        );
    let permission_router =
        crate::features::security::handlers::permission::permission_router_with_state(
            app_state.clone(),
        );
    let analytics_router =
        crate::features::analytics::handlers::analytics::analytics_router_with_state(
            app_state.clone(),
        );
    let security_router =
        crate::features::security::handlers::security::security_router(app_state.clone());
    let system_router = system_router_with_state(Arc::new(app_state.clone()));
    let admin_router =
        crate::features::admin::handlers::admin_router().with_state(app_state.clone());
    let hierarchy_router = organization_hierarchy_router(app_state.clone());
    let gdpr_router = gdpr_router_with_state(app_state.clone());
    let task_router_inst = task_router_with_state(app_state.clone());

    // メインアプリケーションルーターの構築
    let app_router = Router::new()
        .merge(auth_router)
        .merge(user_router)
        .merge(role_router)
        .merge(team_router)
        .merge(task_router_inst)
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
        .merge(attachment_routes().with_state(app_state.clone()))
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
                .layer(axum_middleware::from_fn(security_headers_middleware))
                .layer(cors_layer()),
        );

    tracing::info!("🛣️  Routers configured:");
    tracing::info!("   • Authentication: /auth/*");
    tracing::info!("   • User Management: /users/*");
    tracing::info!("   • Role Management: /roles/*");
    tracing::info!("   • Task Management: /tasks/*");
    tracing::info!("   • File Attachments: /attachments/*");
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
