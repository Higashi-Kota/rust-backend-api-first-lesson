// src/main.rs
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, fmt};

mod config;
mod db;
mod domain; // src/domain/mod.rs または各ファイルへの mod 宣言が必要
mod repository;
mod service;
mod api;
mod error;
// pub mod migration; // 自動マイグレーションを使わない場合は不要

use crate::config::Config;
// use crate::db::{create_db_pool, run_migrations}; // run_migrations のインポートを削除またはコメントアウト
use crate::db::create_db_pool;
use crate::service::task_service::TaskService;
use crate::api::handlers::task_handler::{task_router, AppState};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "task_backend=info,tower_http=info".into()))
        .with(fmt::layer())
        .init();

    tracing::info!("Starting Task Backend server...");

    let app_config = Config::from_env().expect("Failed to load configuration");
    tracing::info!("Configuration loaded: {:?}", app_config);

    let db_pool = create_db_pool(&app_config)
        .await
        .expect("Failed to create database pool");
    tracing::info!("Database pool created successfully.");

    // アプリケーション起動時の自動マイグレーションは無効化
    /*
    tracing::info!("Running database migrations...");
    run_migrations(&db_pool).await.expect("Failed to run database migrations");
    tracing::info!("Database migrations completed.");
    */

    let task_service = Arc::new(TaskService::new(db_pool.clone()));
    let app_state = AppState { task_service };

    let app_router = task_router(app_state)
        .route("/health", axum::routing::get(|| async { "OK" }));

    tracing::info!("Router configured. Server listening on {}", app_config.server_addr);

    let listener = TcpListener::bind(&app_config.server_addr).await?;
    axum::serve(listener, app_router.into_make_service()).await?;

    Ok(())
}