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
        email_verification_token_repository::EmailVerificationTokenRepository,
        organization_repository::OrganizationRepository,
        password_reset_token_repository::PasswordResetTokenRepository,
        refresh_token_repository::RefreshTokenRepository, role_repository::RoleRepository,
        subscription_history_repository::SubscriptionHistoryRepository,
        team_repository::TeamRepository, user_repository::UserRepository,
    },
    service::{
        auth_service::AuthService, permission_service::PermissionService,
        role_service::RoleService, subscription_service::SubscriptionService,
        task_service::TaskService, team_service::TeamService, user_service::UserService,
    },
    utils::{
        email::{EmailConfig, EmailService},
        jwt::JwtManager,
        password::PasswordManager,
    },
};

use crate::common;

/// 認証機能付きアプリのセットアップ
pub async fn setup_auth_app() -> (Router, String, common::db::TestDatabase) {
    // 新しいテストデータベースを作成
    let db = common::db::TestDatabase::new().await;
    let schema_name = db.schema_name.clone();

    // リポジトリの作成
    let user_repo = Arc::new(UserRepository::new(db.connection.clone()));
    let role_repo = Arc::new(RoleRepository::new(Arc::new(db.connection.clone())));
    let refresh_token_repo = Arc::new(RefreshTokenRepository::new(db.connection.clone()));
    let password_reset_token_repo =
        Arc::new(PasswordResetTokenRepository::new(db.connection.clone()));
    let email_verification_token_repo =
        Arc::new(EmailVerificationTokenRepository::new(db.connection.clone()));
    let user_settings_repo = Arc::new(
        task_backend::repository::user_settings_repository::UserSettingsRepository::new(
            db.connection.clone(),
        ),
    );
    let bulk_operation_history_repo = Arc::new(
        task_backend::repository::bulk_operation_history_repository::BulkOperationHistoryRepository::new(
            db.connection.clone(),
        ),
    );

    // 統合設定を作成
    let mut app_config = AppConfig::for_testing();

    // テスト用にパスワードポリシーを調整（特殊文字要件を無効化）
    app_config.password.policy.require_special = false;

    // 統合設定からユーティリティを作成
    let password_manager = Arc::new(
        PasswordManager::new(
            app_config.password.argon2.clone(),
            app_config.password.policy.clone(),
        )
        .unwrap(),
    );
    let jwt_manager = Arc::new(JwtManager::new(app_config.jwt.clone()).unwrap());
    let email_service = Arc::new(
        EmailService::new(EmailConfig {
            development_mode: true,
            ..Default::default()
        })
        .unwrap(),
    );

    // 追加リポジトリの作成
    let activity_log_repo = Arc::new(
        task_backend::repository::activity_log_repository::ActivityLogRepository::new(
            db.connection.clone(),
        ),
    );
    let login_attempt_repo = Arc::new(
        task_backend::repository::login_attempt_repository::LoginAttemptRepository::new(
            db.connection.clone(),
        ),
    );

    // サービスの作成
    let auth_service = Arc::new(AuthService::new(
        user_repo.clone(),
        role_repo.clone(),
        refresh_token_repo,
        password_reset_token_repo,
        email_verification_token_repo.clone(),
        activity_log_repo.clone(),
        login_attempt_repo.clone(),
        password_manager,
        jwt_manager.clone(),
        email_service.clone(),
        Arc::new(db.connection.clone()),
    ));
    let user_service = Arc::new(UserService::new(
        user_repo.clone(),
        user_settings_repo.clone(),
        bulk_operation_history_repo.clone(),
        email_verification_token_repo.clone(),
    ));

    // 統一されたAppStateの作成

    let role_service = Arc::new(RoleService::new(
        Arc::new(db.connection.clone()),
        role_repo.clone(),
        user_repo.clone(),
    ));
    let task_service = Arc::new(TaskService::new(db.connection.clone()));
    let subscription_service = Arc::new(SubscriptionService::new(
        db.connection.clone(),
        email_service.clone(),
    ));
    let team_service = Arc::new(TeamService::new(
        Arc::new(db.connection.clone()),
        TeamRepository::new(db.connection.clone()),
        UserRepository::new(db.connection.clone()),
        email_service.clone(),
    ));

    let organization_service = Arc::new(
        task_backend::service::organization_service::OrganizationService::new(
            OrganizationRepository::new(db.connection.clone()),
            task_backend::repository::team_repository::TeamRepository::new(db.connection.clone()),
            task_backend::repository::user_repository::UserRepository::new(db.connection.clone()),
            task_backend::repository::subscription_history_repository::SubscriptionHistoryRepository::new(db.connection.clone()),
        ),
    );

    // Security services
    let activity_log_repo_sec = std::sync::Arc::new(
        task_backend::repository::activity_log_repository::ActivityLogRepository::new(
            db.connection.clone(),
        ),
    );
    let login_attempt_repo_sec = std::sync::Arc::new(
        task_backend::repository::login_attempt_repository::LoginAttemptRepository::new(
            db.connection.clone(),
        ),
    );
    let security_incident_repo_sec = std::sync::Arc::new(
        task_backend::repository::security_incident_repository::SecurityIncidentRepository::new(
            db.connection.clone(),
        ),
    );
    let security_service = std::sync::Arc::new(
        task_backend::service::security_service::SecurityService::new(
            std::sync::Arc::new(task_backend::repository::refresh_token_repository::RefreshTokenRepository::new(db.connection.clone())),
            std::sync::Arc::new(task_backend::repository::password_reset_token_repository::PasswordResetTokenRepository::new(db.connection.clone())),
            activity_log_repo_sec,
            security_incident_repo_sec,
            login_attempt_repo_sec,
        )
    );

    // Team invitation service
    let team_invitation_service = Arc::new(
        task_backend::service::team_invitation_service::TeamInvitationService::new(
            task_backend::repository::team_invitation_repository::TeamInvitationRepository::new(
                db.connection.clone(),
            ),
            TeamRepository::new(db.connection.clone()),
            UserRepository::new(db.connection.clone()),
        ),
    );

    let subscription_history_repo =
        Arc::new(SubscriptionHistoryRepository::new(db.connection.clone()));
    let daily_activity_summary_repo = Arc::new(
        task_backend::repository::daily_activity_summary_repository::DailyActivitySummaryRepository::new(
            db.connection.clone(),
        ),
    );
    let feature_usage_metrics_repo = Arc::new(
        task_backend::repository::feature_usage_metrics_repository::FeatureUsageMetricsRepository::new(
            db.connection.clone(),
        ),
    );

    // Create PermissionService
    let permission_service = Arc::new(PermissionService::new(
        role_repo.clone(),
        user_repo.clone(),
        Arc::new(TeamRepository::new(db.connection.clone())),
        Arc::new(OrganizationRepository::new(db.connection.clone())),
    ));

    let bulk_operation_history_repo = Arc::new(
        task_backend::repository::bulk_operation_history_repository::BulkOperationHistoryRepository::new(
            db.connection.clone(),
        ),
    );

    let app_state = AppState::with_config(
        auth_service,
        user_service,
        role_service,
        task_service,
        team_service,
        team_invitation_service,
        organization_service,
        subscription_service,
        subscription_history_repo,
        bulk_operation_history_repo,
        daily_activity_summary_repo,
        feature_usage_metrics_repo,
        permission_service,
        security_service,
        jwt_manager,
        Arc::new(db.connection.clone()),
        &app_config,
    );

    // ルーターを作成して統合
    let app = Router::new()
        .merge(auth_handler::auth_router(app_state.clone()))
        .merge(user_handler::user_router(app_state.clone()))
        .merge(task_backend::api::handlers::security_handler::security_router(app_state.clone()))
        .merge(task_backend::api::handlers::analytics_handler::analytics_router(app_state));

    (app, schema_name, db)
}

/// タスク機能付きアプリのセットアップ（認証ミドルウェア付き）
pub async fn setup_full_app() -> (Router, String, common::db::TestDatabase) {
    // 新しいテストデータベースを作成
    let db = common::db::TestDatabase::new().await;
    let schema_name = db.schema_name.clone();

    // リポジトリの作成
    let user_repo = Arc::new(UserRepository::new(db.connection.clone()));
    let role_repo = Arc::new(RoleRepository::new(Arc::new(db.connection.clone())));
    let refresh_token_repo = Arc::new(RefreshTokenRepository::new(db.connection.clone()));
    let password_reset_token_repo =
        Arc::new(PasswordResetTokenRepository::new(db.connection.clone()));
    let email_verification_token_repo =
        Arc::new(EmailVerificationTokenRepository::new(db.connection.clone()));
    let user_settings_repo = Arc::new(
        task_backend::repository::user_settings_repository::UserSettingsRepository::new(
            db.connection.clone(),
        ),
    );
    let bulk_operation_history_repo = Arc::new(
        task_backend::repository::bulk_operation_history_repository::BulkOperationHistoryRepository::new(
            db.connection.clone(),
        ),
    );

    // 統合設定を作成
    let mut app_config = AppConfig::for_testing();

    // テスト用にパスワードポリシーを調整（特殊文字要件を無効化）
    app_config.password.policy.require_special = false;

    // 統合設定からユーティリティを作成
    let password_manager = Arc::new(
        PasswordManager::new(
            app_config.password.argon2.clone(),
            app_config.password.policy.clone(),
        )
        .unwrap(),
    );
    let jwt_manager = Arc::new(JwtManager::new(app_config.jwt.clone()).unwrap());
    let email_service = Arc::new(
        EmailService::new(EmailConfig {
            development_mode: true,
            ..Default::default()
        })
        .unwrap(),
    );

    // 追加リポジトリの作成
    let activity_log_repo = Arc::new(
        task_backend::repository::activity_log_repository::ActivityLogRepository::new(
            db.connection.clone(),
        ),
    );
    let login_attempt_repo = Arc::new(
        task_backend::repository::login_attempt_repository::LoginAttemptRepository::new(
            db.connection.clone(),
        ),
    );

    // サービスの作成
    let auth_service = Arc::new(AuthService::new(
        user_repo.clone(),
        role_repo.clone(),
        refresh_token_repo.clone(),
        password_reset_token_repo.clone(),
        email_verification_token_repo.clone(),
        activity_log_repo.clone(),
        login_attempt_repo.clone(),
        password_manager,
        jwt_manager.clone(),
        email_service.clone(),
        Arc::new(db.connection.clone()),
    ));
    let user_service = Arc::new(UserService::new(
        user_repo.clone(),
        user_settings_repo.clone(),
        bulk_operation_history_repo.clone(),
        email_verification_token_repo.clone(),
    ));
    let role_service = Arc::new(RoleService::new(
        Arc::new(db.connection.clone()),
        role_repo.clone(),
        user_repo.clone(),
    ));
    let task_service = Arc::new(TaskService::new(db.connection.clone()));
    let subscription_service = Arc::new(SubscriptionService::new(
        db.connection.clone(),
        email_service.clone(),
    ));
    let team_service = Arc::new(TeamService::new(
        Arc::new(db.connection.clone()),
        TeamRepository::new(db.connection.clone()),
        UserRepository::new(db.connection.clone()),
        email_service.clone(),
    ));

    let organization_service = Arc::new(
        task_backend::service::organization_service::OrganizationService::new(
            task_backend::repository::organization_repository::OrganizationRepository::new(
                db.connection.clone(),
            ),
            task_backend::repository::team_repository::TeamRepository::new(db.connection.clone()),
            task_backend::repository::user_repository::UserRepository::new(db.connection.clone()),
            task_backend::repository::subscription_history_repository::SubscriptionHistoryRepository::new(db.connection.clone()),
        ),
    );

    // Security services
    let security_incident_repo = Arc::new(
        task_backend::repository::security_incident_repository::SecurityIncidentRepository::new(
            db.connection.clone(),
        ),
    );
    let security_service = Arc::new(
        task_backend::service::security_service::SecurityService::new(
            Arc::new(RefreshTokenRepository::new(db.connection.clone())),
            Arc::new(PasswordResetTokenRepository::new(db.connection.clone())),
            activity_log_repo.clone(),
            security_incident_repo,
            login_attempt_repo.clone(),
        ),
    );

    // Team invitation service
    let team_invitation_service = Arc::new(
        task_backend::service::team_invitation_service::TeamInvitationService::new(
            task_backend::repository::team_invitation_repository::TeamInvitationRepository::new(
                db.connection.clone(),
            ),
            TeamRepository::new(db.connection.clone()),
            UserRepository::new(db.connection.clone()),
        ),
    );

    let subscription_history_repo =
        Arc::new(SubscriptionHistoryRepository::new(db.connection.clone()));
    let daily_activity_summary_repo = Arc::new(
        task_backend::repository::daily_activity_summary_repository::DailyActivitySummaryRepository::new(
            db.connection.clone(),
        ),
    );
    let feature_usage_metrics_repo = Arc::new(
        task_backend::repository::feature_usage_metrics_repository::FeatureUsageMetricsRepository::new(
            db.connection.clone(),
        ),
    );
    let bulk_operation_history_repo = Arc::new(
        task_backend::repository::bulk_operation_history_repository::BulkOperationHistoryRepository::new(
            db.connection.clone(),
        ),
    );

    // Create PermissionService
    let permission_service = Arc::new(PermissionService::new(
        role_repo.clone(),
        user_repo.clone(),
        Arc::new(TeamRepository::new(db.connection.clone())),
        Arc::new(OrganizationRepository::new(db.connection.clone())),
    ));

    // 統一されたAppStateの作成
    let app_state = AppState::with_config(
        auth_service,
        user_service,
        role_service,
        task_service,
        team_service,
        team_invitation_service,
        organization_service,
        subscription_service,
        subscription_history_repo,
        bulk_operation_history_repo,
        daily_activity_summary_repo,
        feature_usage_metrics_repo,
        permission_service,
        security_service,
        jwt_manager.clone(),
        Arc::new(db.connection.clone()),
        &app_config,
    );

    // 認証ミドルウェア設定を作成
    use axum::middleware as axum_middleware;
    use task_backend::middleware::auth::{jwt_auth_middleware, AuthMiddlewareConfig};

    let auth_middleware_config = AuthMiddlewareConfig {
        jwt_manager: jwt_manager.clone(),
        user_repository: user_repo,
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
        .merge(
            task_backend::api::handlers::organization_handler::organization_router_with_state(
                app_state.clone(),
            ),
        )
        .merge(task_backend::api::handlers::admin_handler::admin_router(
            app_state.clone(),
        ))
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
            task_backend::api::handlers::analytics_handler::analytics_router_with_state(
                app_state.clone(),
            ),
        )
        .merge(task_backend::api::handlers::security_handler::security_router(app_state.clone()))
        .merge(task_backend::api::handlers::gdpr_handler::gdpr_router_with_state(app_state))
        .layer(axum_middleware::from_fn_with_state(
            auth_middleware_config,
            jwt_auth_middleware,
        ));

    (app, schema_name, db)
}

/// テストアプリケーションのセットアップ（AppStateも返す）
#[allow(dead_code)]
pub async fn setup_test_app() -> (Router, Arc<AppState>) {
    let (app, _schema_name, db) = setup_full_app().await;
    let mut app_config = AppConfig::for_testing();

    // テスト用にパスワードポリシーを調整（特殊文字要件を無効化）
    app_config.password.policy.require_special = false;

    // 必要なサービスを構築
    let user_repo = Arc::new(UserRepository::new(db.connection.clone()));
    let role_repo = Arc::new(RoleRepository::new(Arc::new(db.connection.clone())));
    let refresh_token_repo = Arc::new(RefreshTokenRepository::new(db.connection.clone()));
    let password_reset_token_repo =
        Arc::new(PasswordResetTokenRepository::new(db.connection.clone()));
    let email_verification_token_repo =
        Arc::new(EmailVerificationTokenRepository::new(db.connection.clone()));
    let user_settings_repo = Arc::new(
        task_backend::repository::user_settings_repository::UserSettingsRepository::new(
            db.connection.clone(),
        ),
    );
    let bulk_operation_history_repo = Arc::new(
        task_backend::repository::bulk_operation_history_repository::BulkOperationHistoryRepository::new(
            db.connection.clone(),
        ),
    );

    let password_manager = Arc::new(
        PasswordManager::new(
            app_config.password.argon2.clone(),
            app_config.password.policy.clone(),
        )
        .unwrap(),
    );
    let jwt_manager = Arc::new(JwtManager::new(app_config.jwt.clone()).unwrap());
    let email_service = Arc::new(
        EmailService::new(EmailConfig {
            development_mode: true,
            ..Default::default()
        })
        .unwrap(),
    );

    let activity_log_repo = Arc::new(
        task_backend::repository::activity_log_repository::ActivityLogRepository::new(
            db.connection.clone(),
        ),
    );
    let login_attempt_repo = Arc::new(
        task_backend::repository::login_attempt_repository::LoginAttemptRepository::new(
            db.connection.clone(),
        ),
    );

    let auth_service = Arc::new(AuthService::new(
        user_repo.clone(),
        role_repo.clone(),
        refresh_token_repo,
        password_reset_token_repo,
        email_verification_token_repo.clone(),
        activity_log_repo.clone(),
        login_attempt_repo.clone(),
        password_manager,
        jwt_manager.clone(),
        email_service.clone(),
        Arc::new(db.connection.clone()),
    ));

    let user_service = Arc::new(UserService::new(
        user_repo.clone(),
        user_settings_repo.clone(),
        bulk_operation_history_repo.clone(),
        email_verification_token_repo.clone(),
    ));
    let role_service = Arc::new(RoleService::new(
        Arc::new(db.connection.clone()),
        role_repo.clone(),
        user_repo.clone(),
    ));
    let task_service = Arc::new(TaskService::new(db.connection.clone()));
    let subscription_service = Arc::new(SubscriptionService::new(
        db.connection.clone(),
        email_service.clone(),
    ));
    let team_service = Arc::new(TeamService::new(
        Arc::new(db.connection.clone()),
        TeamRepository::new(db.connection.clone()),
        UserRepository::new(db.connection.clone()),
        email_service.clone(),
    ));

    let organization_service = Arc::new(
        task_backend::service::organization_service::OrganizationService::new(
            task_backend::repository::organization_repository::OrganizationRepository::new(
                db.connection.clone(),
            ),
            task_backend::repository::team_repository::TeamRepository::new(db.connection.clone()),
            task_backend::repository::user_repository::UserRepository::new(db.connection.clone()),
            task_backend::repository::subscription_history_repository::SubscriptionHistoryRepository::new(db.connection.clone()),
        ),
    );

    let security_incident_repo = Arc::new(
        task_backend::repository::security_incident_repository::SecurityIncidentRepository::new(
            db.connection.clone(),
        ),
    );
    let security_service = Arc::new(
        task_backend::service::security_service::SecurityService::new(
            Arc::new(RefreshTokenRepository::new(db.connection.clone())),
            Arc::new(PasswordResetTokenRepository::new(db.connection.clone())),
            activity_log_repo.clone(),
            security_incident_repo,
            login_attempt_repo.clone(),
        ),
    );

    let team_invitation_service = Arc::new(
        task_backend::service::team_invitation_service::TeamInvitationService::new(
            task_backend::repository::team_invitation_repository::TeamInvitationRepository::new(
                db.connection.clone(),
            ),
            TeamRepository::new(db.connection.clone()),
            UserRepository::new(db.connection.clone()),
        ),
    );

    let subscription_history_repo =
        Arc::new(SubscriptionHistoryRepository::new(db.connection.clone()));
    let daily_activity_summary_repo = Arc::new(
        task_backend::repository::daily_activity_summary_repository::DailyActivitySummaryRepository::new(
            db.connection.clone(),
        ),
    );
    let feature_usage_metrics_repo = Arc::new(
        task_backend::repository::feature_usage_metrics_repository::FeatureUsageMetricsRepository::new(
            db.connection.clone(),
        ),
    );

    // Create PermissionService
    let permission_service = Arc::new(PermissionService::new(
        role_repo.clone(),
        user_repo.clone(),
        Arc::new(TeamRepository::new(db.connection.clone())),
        Arc::new(OrganizationRepository::new(db.connection.clone())),
    ));

    let bulk_operation_history_repo = Arc::new(
        task_backend::repository::bulk_operation_history_repository::BulkOperationHistoryRepository::new(
            db.connection.clone(),
        ),
    );

    let app_state = AppState::with_config(
        auth_service,
        user_service,
        role_service,
        task_service,
        team_service,
        team_invitation_service,
        organization_service,
        subscription_service,
        subscription_history_repo,
        bulk_operation_history_repo,
        daily_activity_summary_repo,
        feature_usage_metrics_repo,
        permission_service,
        security_service,
        jwt_manager,
        Arc::new(db.connection.clone()),
        &app_config,
    );

    (app, Arc::new(app_state))
}
