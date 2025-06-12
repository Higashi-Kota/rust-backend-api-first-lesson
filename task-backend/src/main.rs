// src/main.rs
use std::env;
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
    auth_handler::auth_router_with_service, task_handler::task_router_with_schema,
    user_handler::user_router_with_service,
};
use crate::config::Config;
use crate::db::{create_db_pool, create_db_pool_with_schema, create_schema, schema_exists};
use crate::middleware::auth::{
    cors_layer, jwt_auth_middleware, security_headers_middleware, AuthMiddlewareConfig,
};
use crate::repository::{
    password_reset_token_repository::PasswordResetTokenRepository,
    refresh_token_repository::RefreshTokenRepository, user_repository::UserRepository,
};
use crate::service::{
    auth_service::AuthService, task_service::TaskService, user_service::UserService,
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

    // 設定を読み込む
    let app_config = Config::from_env().expect("Failed to load configuration");
    tracing::info!("📋 Configuration loaded: {:?}", app_config);

    // スキーマ名を環境変数から取得（オプション）
    let schema_name = env::var("DB_SCHEMA").ok();

    // データベース接続を作成
    let db_pool = if let Some(schema) = &schema_name {
        tracing::info!("🗃️  Using schema: {}", schema);

        // まず基本接続を作成
        let base_pool = create_db_pool(&app_config)
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
        create_db_pool_with_schema(&app_config, schema)
            .await
            .expect("Failed to create database pool with schema")
    } else {
        // 通常の接続プールを作成（スキーマ指定なし）
        create_db_pool(&app_config)
            .await
            .expect("Failed to create database pool")
    };

    tracing::info!("✅ Database pool created successfully.");

    // ユーティリティサービスの初期化
    let jwt_manager = Arc::new(JwtManager::from_env().expect("Failed to initialize JWT manager"));
    let password_manager =
        Arc::new(PasswordManager::from_env().expect("Failed to initialize password manager"));
    let _email_service =
        Arc::new(EmailService::from_env().expect("Failed to initialize email service"));

    tracing::info!("🔧 Utility services initialized.");

    // リポジトリの作成
    let user_repo = Arc::new(UserRepository::new(db_pool.clone()));
    let refresh_token_repo = Arc::new(RefreshTokenRepository::new(db_pool.clone()));
    let password_reset_token_repo = Arc::new(PasswordResetTokenRepository::new(db_pool.clone()));

    tracing::info!("📚 Repositories created.");

    // サービスの作成
    let auth_service = Arc::new(AuthService::new(
        user_repo.clone(),
        refresh_token_repo.clone(),
        password_reset_token_repo.clone(),
        password_manager.clone(),
        jwt_manager.clone(),
    ));

    let user_service = Arc::new(UserService::new(user_repo.clone()));

    let task_service = if let Some(schema) = &schema_name {
        Arc::new(TaskService::with_schema(db_pool.clone(), schema.clone()))
    } else {
        Arc::new(TaskService::new(db_pool.clone()))
    };

    tracing::info!("🎯 Business services created.");

    // 認証ミドルウェア設定
    let auth_middleware_config = AuthMiddlewareConfig {
        jwt_manager: jwt_manager.clone(),
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
        require_verified_email: false, // 開発環境では false
        require_active_account: true,
    };

    // ルーターの設定
    let auth_router = auth_router_with_service(auth_service, jwt_manager.clone());
    let user_router = user_router_with_service(user_service, jwt_manager.clone());
    let task_router = task_router_with_schema(task_service, jwt_manager.clone());

    // メインアプリケーションルーターの構築
    let app_router = Router::new()
        .merge(auth_router)
        .merge(user_router)
        .merge(task_router)
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
    tracing::info!("   • Task Management: /tasks/*");
    tracing::info!("   • Health Check: /health");

    // サーバーの起動
    tracing::info!("🌐 Server listening on {}", app_config.server_addr);
    tracing::info!(
        "📚 API Documentation: http://{}/docs",
        app_config.server_addr
    );

    let listener = TcpListener::bind(&app_config.server_addr).await?;

    tracing::info!("🎉 Task Backend server started successfully!");

    axum::serve(listener, app_router.into_make_service()).await?;

    Ok(())
}
