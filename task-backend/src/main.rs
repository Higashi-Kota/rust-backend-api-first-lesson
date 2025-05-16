// src/main.rs
use std::env;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod api;
mod config;
mod db;
mod domain;
mod error;
mod repository;
mod service;

use crate::api::handlers::task_handler::task_router_with_schema;
use crate::config::Config;
use crate::db::{create_db_pool, create_db_pool_with_schema, create_schema, schema_exists};
use crate::service::task_service::TaskService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // トレーシングの設定
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "task_backend=info,tower_http=info".into()),
        )
        .with(fmt::layer())
        .init();

    tracing::info!("Starting Task Backend server...");

    // 設定を読み込む
    let app_config = Config::from_env().expect("Failed to load configuration");
    tracing::info!("Configuration loaded: {:?}", app_config);

    // スキーマ名を環境変数から取得（オプション）
    let schema_name = env::var("DB_SCHEMA").ok();

    // データベース接続を作成
    let db_pool = if let Some(schema) = &schema_name {
        tracing::info!("Using schema: {}", schema);

        // まず基本接続を作成
        let base_pool = create_db_pool(&app_config)
            .await
            .expect("Failed to create base database connection");

        // スキーマの存在を確認し、なければ作成
        let schema_exists = schema_exists(&base_pool, schema)
            .await
            .expect("Failed to check schema existence");

        if !schema_exists {
            tracing::info!("Schema does not exist, creating it: {}", schema);
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

    tracing::info!("Database pool created successfully.");

    // TaskServiceの作成
    let task_service = if let Some(schema) = schema_name {
        Arc::new(TaskService::with_schema(db_pool.clone(), schema))
    } else {
        Arc::new(TaskService::new(db_pool.clone()))
    };

    // ルーターの設定
    let app_router = task_router_with_schema(task_service);

    // サーバーの起動
    tracing::info!(
        "Router configured. Server listening on {}",
        app_config.server_addr
    );

    let listener = TcpListener::bind(&app_config.server_addr).await?;
    axum::serve(listener, app_router.into_make_service()).await?;

    Ok(())
}
