// tests/common/app_helper.rs

use axum::Router;
use std::sync::Arc;
use task_backend::{
    api::{
        handlers::{auth_handler, user_handler},
        AppState,
    },
    config::AppConfig,
    repository::{
        password_reset_token_repository::PasswordResetTokenRepository,
        refresh_token_repository::RefreshTokenRepository, role_repository::RoleRepository,
        team_repository::TeamRepository, user_repository::UserRepository,
    },
    service::{
        auth_service::AuthService, role_service::RoleService,
        subscription_service::SubscriptionService, task_service::TaskService,
        team_service::TeamService, user_service::UserService,
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

    // 統合設定を作成
    let app_config = AppConfig::for_testing();

    // 統合設定からユーティリティを作成
    let password_manager = Arc::new(
        PasswordManager::new(
            app_config.password.argon2.clone(),
            app_config.password.policy.clone(),
        )
        .unwrap(),
    );
    let jwt_manager = Arc::new(JwtManager::new(app_config.jwt.clone()).unwrap());

    // サービスの作成
    let auth_service = Arc::new(AuthService::new(
        user_repo.clone(),
        role_repo.clone(),
        refresh_token_repo,
        password_reset_token_repo,
        password_manager,
        jwt_manager.clone(),
        Arc::new(db.connection.clone()),
    ));
    let user_service = Arc::new(UserService::new(user_repo.clone()));

    // 統一されたAppStateの作成

    let role_service = Arc::new(RoleService::new(role_repo.clone(), user_repo.clone()));
    let task_service = Arc::new(TaskService::with_schema(
        db.connection.clone(),
        schema_name.clone(),
    ));
    let subscription_service = Arc::new(SubscriptionService::new(db.connection.clone()));
    let team_service = Arc::new(TeamService::new(
        TeamRepository::new(db.connection.clone()),
        UserRepository::new(db.connection.clone()),
    ));

    let organization_service = Arc::new(
        task_backend::service::organization_service::OrganizationService::new(
            task_backend::repository::organization_repository::OrganizationRepository::new(
                db.connection.clone(),
            ),
            task_backend::repository::team_repository::TeamRepository::new(db.connection.clone()),
            task_backend::repository::user_repository::UserRepository::new(db.connection.clone()),
        ),
    );

    let app_state = AppState::with_config(
        auth_service,
        user_service,
        role_service,
        task_service,
        team_service,
        organization_service,
        subscription_service,
        jwt_manager,
        &app_config,
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

    // 統合設定を作成
    let app_config = AppConfig::for_testing();

    // 統合設定からユーティリティを作成
    let password_manager = Arc::new(
        PasswordManager::new(
            app_config.password.argon2.clone(),
            app_config.password.policy.clone(),
        )
        .unwrap(),
    );
    let jwt_manager = Arc::new(JwtManager::new(app_config.jwt.clone()).unwrap());

    // サービスの作成
    let auth_service = Arc::new(AuthService::new(
        user_repo.clone(),
        role_repo.clone(),
        refresh_token_repo,
        password_reset_token_repo,
        password_manager,
        jwt_manager.clone(),
        Arc::new(db.connection.clone()),
    ));
    let user_service = Arc::new(UserService::new(user_repo.clone()));
    let role_service = Arc::new(RoleService::new(role_repo.clone(), user_repo.clone()));
    let task_service = Arc::new(TaskService::with_schema(
        db.connection.clone(),
        schema_name.clone(),
    ));
    let subscription_service = Arc::new(SubscriptionService::new(db.connection.clone()));
    let team_service = Arc::new(TeamService::new(
        TeamRepository::new(db.connection.clone()),
        UserRepository::new(db.connection.clone()),
    ));

    let organization_service = Arc::new(
        task_backend::service::organization_service::OrganizationService::new(
            task_backend::repository::organization_repository::OrganizationRepository::new(
                db.connection.clone(),
            ),
            task_backend::repository::team_repository::TeamRepository::new(db.connection.clone()),
            task_backend::repository::user_repository::UserRepository::new(db.connection.clone()),
        ),
    );

    // 統一されたAppStateの作成
    let app_state = AppState::with_config(
        auth_service,
        user_service,
        role_service,
        task_service,
        team_service,
        organization_service,
        subscription_service,
        jwt_manager.clone(),
        &app_config,
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
        .merge(task_backend::api::handlers::team_handler::team_router_with_state(app_state.clone()))
        .merge(task_backend::api::handlers::task_handler::admin_task_router(app_state.clone()))
        .merge(
            task_backend::api::handlers::subscription_handler::subscription_router_with_state(
                app_state.clone(),
            ),
        )
        .merge(
            task_backend::api::handlers::permission_handler::permission_router_with_state(
                app_state.clone(),
            ),
        )
        .merge(
            task_backend::api::handlers::analytics_handler::analytics_router_with_state(app_state),
        )
        .layer(axum_middleware::from_fn_with_state(
            auth_middleware_config,
            jwt_auth_middleware,
        ));

    (app, schema_name, db)
}
