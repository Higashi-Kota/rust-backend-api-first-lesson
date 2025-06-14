// tests/common/app_helper.rs

use axum::Router;
use std::sync::Arc;
use task_backend::{
    api::{
        handlers::{auth_handler, user_handler},
        AppState,
    },
    repository::{
        password_reset_token_repository::PasswordResetTokenRepository,
        refresh_token_repository::RefreshTokenRepository, role_repository::RoleRepository,
        user_repository::UserRepository,
    },
    service::{
        auth_service::AuthService, role_service::RoleService, task_service::TaskService,
        user_service::UserService,
    },
    utils::{jwt::JwtManager, password::PasswordManager},
};

use crate::common;

/// 認証機能付きアプリのセットアップ
pub async fn setup_auth_app() -> (Router, String, common::db::TestDatabase) {
    // 新しいテストデータベースを作成
    let db = common::db::TestDatabase::new().await;
    let schema_name = db.schema_name.clone();

    // リポジトリの作成
    let user_repo = Arc::new(UserRepository::with_schema(
        db.connection.clone(),
        schema_name.clone(),
    ));
    let role_repo = Arc::new(RoleRepository::new(Arc::new(db.connection.clone())));
    let refresh_token_repo = Arc::new(RefreshTokenRepository::with_schema(
        db.connection.clone(),
        schema_name.clone(),
    ));
    let password_reset_token_repo = Arc::new(PasswordResetTokenRepository::with_schema(
        db.connection.clone(),
        schema_name.clone(),
    ));

    // ユーティリティの作成
    use task_backend::utils::jwt::JwtConfig;
    use task_backend::utils::password::{Argon2Config, PasswordPolicy};

    let argon2_config = Argon2Config::default();
    let password_policy = PasswordPolicy::default();
    let password_manager = Arc::new(PasswordManager::new(argon2_config, password_policy).unwrap());

    let jwt_config = JwtConfig {
        secret_key: "test_secret_key_must_be_at_least_32_characters_long_for_testing".to_string(),
        access_token_expiry_minutes: 15,
        refresh_token_expiry_days: 7,
        issuer: "test-task-backend".to_string(),
        audience: "test-users".to_string(),
    };
    let jwt_manager = Arc::new(JwtManager::new(jwt_config).unwrap());

    // サービスの作成
    let auth_service = Arc::new(AuthService::new(
        user_repo.clone(),
        role_repo.clone(),
        refresh_token_repo,
        password_reset_token_repo,
        password_manager,
        jwt_manager.clone(),
    ));
    let user_service = Arc::new(UserService::new(user_repo.clone()));

    // 統一されたAppStateの作成

    let role_service = Arc::new(RoleService::new(role_repo.clone(), user_repo.clone()));
    let task_service = Arc::new(TaskService::with_schema(
        db.connection.clone(),
        schema_name.clone(),
    ));

    let app_state = AppState::new(
        auth_service,
        user_service,
        role_service,
        task_service,
        jwt_manager,
    );

    // ルーターを作成して統合
    let app = Router::new()
        .merge(auth_handler::auth_router(app_state.clone()))
        .merge(user_handler::user_router(app_state));

    (app, schema_name, db)
}

/// タスク機能付きアプリのセットアップ（認証ミドルウェア付き）
pub async fn setup_full_app() -> (Router, String, common::db::TestDatabase) {
    // 新しいテストデータベースを作成
    let db = common::db::TestDatabase::new().await;
    let schema_name = db.schema_name.clone();

    // リポジトリの作成
    let user_repo = Arc::new(UserRepository::with_schema(
        db.connection.clone(),
        schema_name.clone(),
    ));
    let role_repo = Arc::new(RoleRepository::new(Arc::new(db.connection.clone())));
    let refresh_token_repo = Arc::new(RefreshTokenRepository::with_schema(
        db.connection.clone(),
        schema_name.clone(),
    ));
    let password_reset_token_repo = Arc::new(PasswordResetTokenRepository::with_schema(
        db.connection.clone(),
        schema_name.clone(),
    ));

    // ユーティリティの作成
    use task_backend::utils::jwt::JwtConfig;
    use task_backend::utils::password::{Argon2Config, PasswordPolicy};

    let argon2_config = Argon2Config::default();
    let password_policy = PasswordPolicy::default();
    let password_manager = Arc::new(PasswordManager::new(argon2_config, password_policy).unwrap());

    let jwt_config = JwtConfig {
        secret_key: "test_secret_key_must_be_at_least_32_characters_long_for_testing".to_string(),
        access_token_expiry_minutes: 15,
        refresh_token_expiry_days: 7,
        issuer: "test-task-backend".to_string(),
        audience: "test-users".to_string(),
    };
    let jwt_manager = Arc::new(JwtManager::new(jwt_config).unwrap());

    // サービスの作成
    let auth_service = Arc::new(AuthService::new(
        user_repo.clone(),
        role_repo.clone(),
        refresh_token_repo,
        password_reset_token_repo,
        password_manager,
        jwt_manager.clone(),
    ));
    let user_service = Arc::new(UserService::new(user_repo.clone()));
    let role_service = Arc::new(RoleService::new(role_repo.clone(), user_repo.clone()));
    let task_service = Arc::new(TaskService::with_schema(
        db.connection.clone(),
        schema_name.clone(),
    ));

    // 統一されたAppStateの作成
    let app_state = AppState::new(
        auth_service,
        user_service,
        role_service,
        task_service,
        jwt_manager.clone(),
    );

    // 認証ミドルウェア設定を作成
    use axum::middleware as axum_middleware;
    use task_backend::middleware::auth::{jwt_auth_middleware, AuthMiddlewareConfig};

    let auth_middleware_config = AuthMiddlewareConfig {
        jwt_manager: jwt_manager.clone(),
        user_repository: user_repo,
        role_repository: role_repo,
        access_token_cookie_name: "access_token".to_string(),
        skip_auth_paths: vec![
            "/auth/signup".to_string(),
            "/auth/signin".to_string(),
            "/auth/refresh".to_string(),
            "/auth/forgot-password".to_string(),
            "/auth/reset-password".to_string(),
            "/health".to_string(),
            "/".to_string(),
            "/test".to_string(),
        ],
        admin_only_paths: vec!["/admin".to_string(), "/api/admin".to_string()],
        require_verified_email: false,
        require_active_account: true,
    };

    // 全てのルーターを統合
    let app = Router::new()
        .merge(auth_handler::auth_router(app_state.clone()))
        .merge(user_handler::user_router(app_state.clone()))
        .merge(task_backend::api::handlers::role_handler::role_router_with_state(app_state.clone()))
        .merge(task_backend::api::handlers::task_handler::task_router_with_state(app_state.clone()))
        .merge(task_backend::api::handlers::task_handler::admin_task_router(app_state))
        .layer(axum_middleware::from_fn_with_state(
            auth_middleware_config,
            jwt_auth_middleware,
        ));

    (app, schema_name, db)
}
