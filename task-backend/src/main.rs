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
mod utils;

use crate::api::handlers::{
    auth_handler::auth_router_with_state,
    role_handler::role_router_with_state,
    task_handler::{admin_task_router, task_router_with_state},
    user_handler::user_router_with_state,
};
use crate::api::AppState;
use crate::config::{AppConfig, Config};
use crate::db::{create_db_pool, create_db_pool_with_schema, create_schema, schema_exists};
use crate::middleware::auth::{
    cors_layer, jwt_auth_middleware, security_headers_middleware, AuthMiddlewareConfig,
};
use crate::repository::{
    password_reset_token_repository::PasswordResetTokenRepository,
    refresh_token_repository::RefreshTokenRepository, role_repository::RoleRepository,
    user_repository::UserRepository,
};
use crate::service::{
    auth_service::AuthService, role_service::RoleService, task_service::TaskService,
    user_service::UserService,
};
use crate::utils::{email::EmailService, jwt::JwtManager, password::PasswordManager};
use axum::{middleware as axum_middleware, Router};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    tracing::info!("   • Environment: {}", app_config.server.environment);
    tracing::info!("   • Server: {}", app_config.server.addr);
    tracing::info!("   • Database: configured");
    tracing::info!("   • JWT: configured");
    tracing::info!(
        "   • Email: configured (dev mode: {})",
        app_config.email.development_mode
    );
    tracing::info!(
        "   • Security: cookie_secure={}",
        app_config.security.cookie_secure
    );

    // 後方互換性のために既存のConfig構造体も作成
    let legacy_config = Config::from_app_config(&app_config);

    // データベース接続を作成
    let db_pool = if let Some(schema) = &app_config.database.schema {
        tracing::info!("🗃️  Using schema: {}", schema);

        // まず基本接続を作成
        let base_pool = create_db_pool(&legacy_config)
            .await
            .expect("Failed to create base database connection");

        // スキーマの存在を確認し、なければ作成
        let schema_exists = schema_exists(&base_pool, schema)
            .await
            .expect("Failed to check schema existence");

        if !schema_exists {
            tracing::info!("📝 Schema does not exist, creating it: {}", schema);
            create_schema(&base_pool, schema)
                .await
                .expect("Failed to create schema");
        }

        // スキーマを指定して接続プールを作成
        create_db_pool_with_schema(&legacy_config, schema)
            .await
            .expect("Failed to create database pool with schema")
    } else {
        // 通常の接続プールを作成（スキーマ指定なし）
        create_db_pool(&legacy_config)
            .await
            .expect("Failed to create database pool")
    };

    tracing::info!("✅ Database pool created successfully.");

    // 統合設定からユーティリティサービスを初期化
    let jwt_manager = Arc::new(
        JwtManager::new(app_config.jwt.clone()).expect("Failed to initialize JWT manager"),
    );
    let password_manager = Arc::new(
        PasswordManager::new(
            app_config.password.argon2.clone(),
            app_config.password.policy.clone(),
        )
        .expect("Failed to initialize password manager"),
    );
    let _email_service = Arc::new(
        EmailService::new(app_config.email.clone()).expect("Failed to initialize email service"),
    );

    tracing::info!("🔧 Utility services initialized.");

    // リポジトリの作成
    let user_repo = Arc::new(UserRepository::new(db_pool.clone()));
    let role_repo = Arc::new(RoleRepository::new(Arc::new(db_pool.clone())));
    let refresh_token_repo = Arc::new(RefreshTokenRepository::new(db_pool.clone()));
    let password_reset_token_repo = Arc::new(PasswordResetTokenRepository::new(db_pool.clone()));

    tracing::info!("📚 Repositories created.");

    // サービスの作成
    let auth_service = Arc::new(AuthService::new(
        user_repo.clone(),
        role_repo.clone(),
        refresh_token_repo.clone(),
        password_reset_token_repo.clone(),
        password_manager.clone(),
        jwt_manager.clone(),
        Arc::new(db_pool.clone()),
    ));

    let user_service = Arc::new(UserService::new(user_repo.clone()));

    let role_service = Arc::new(RoleService::new(role_repo.clone(), user_repo.clone()));

    let task_service = if let Some(schema) = &app_config.database.schema {
        Arc::new(TaskService::with_schema(db_pool.clone(), schema.clone()))
    } else {
        Arc::new(TaskService::new(db_pool.clone()))
    };

    tracing::info!("🎯 Business services created.");

    // 認証ミドルウェア設定
    let auth_middleware_config = AuthMiddlewareConfig {
        jwt_manager: jwt_manager.clone(),
        user_repository: user_repo.clone(),
        role_repository: role_repo.clone(),
        access_token_cookie_name: "access_token".to_string(),
        skip_auth_paths: vec![
            "/auth/signup".to_string(),
            "/auth/signin".to_string(),
            "/auth/refresh".to_string(),
            "/auth/forgot-password".to_string(),
            "/auth/reset-password".to_string(),
            "/health".to_string(),
            "/".to_string(),
        ],
        admin_only_paths: vec!["/admin".to_string(), "/api/admin".to_string()],
        require_verified_email: !app_config.is_development(), // 開発環境では false
        require_active_account: true,
    };

    // 統一されたAppStateを作成（統合設定対応）
    let app_state = AppState::with_config(
        auth_service,
        user_service,
        role_service,
        task_service,
        jwt_manager.clone(),
        &app_config,
    );

    // ルーターの設定
    let auth_router = auth_router_with_state(app_state.clone());
    let user_router = user_router_with_state(app_state.clone());
    let role_router = role_router_with_state(app_state.clone());
    let task_router = task_router_with_state(app_state.clone());
    let admin_router = admin_task_router(app_state.clone());

    // メインアプリケーションルーターの構築
    let app_router = Router::new()
        .merge(auth_router)
        .merge(user_router)
        .merge(role_router)
        .merge(task_router)
        .merge(admin_router)
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
    tracing::info!("   • Admin Management: /admin/*");
    tracing::info!("   • Health Check: /health");

    // サーバーの起動
    tracing::info!("🌐 Server listening on {}", app_config.server.addr);
    tracing::info!(
        "📚 API Documentation: http://{}/docs",
        app_config.server.addr
    );

    let listener = TcpListener::bind(&app_config.server.addr).await?;

    tracing::info!("🎉 Task Backend server started successfully!");

    axum::serve(listener, app_router.into_make_service()).await?;

    Ok(())
}
