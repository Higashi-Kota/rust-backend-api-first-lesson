// tests/common/app_helper.rs

use axum::{
    body::{self, Body},
    http::{Request, StatusCode},
    Router,
};
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
        attachment_service::AttachmentService, auth_service::AuthService,
        payment_service::PaymentService, permission_service::PermissionService,
        role_service::RoleService, storage_service::StorageService,
        subscription_service::SubscriptionService, task_service::TaskService,
        team_service::TeamService, user_service::UserService,
    },
    utils::{
        email::{EmailConfig, EmailProvider, EmailService},
        jwt::JwtManager,
        password::PasswordManager,
    },
};
use tower::ServiceExt;

use crate::common;

/// 認証機能付きアプリのセットアップ
pub async fn setup_auth_app() -> (Router, String, common::db::TestDatabase) {
    // テスト環境を初期化
    common::init_test_env();

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
    let app_config = AppConfig::for_testing();

    // テスト用にパスワードポリシーを調整（特殊文字要件を無効化）
    let password_policy = task_backend::utils::password::PasswordPolicy {
        require_special: false,
        ..Default::default()
    };
    let argon2_config = task_backend::utils::password::Argon2Config::default();

    // 統合設定からユーティリティを作成
    let password_manager = Arc::new(PasswordManager::new(argon2_config, password_policy).unwrap());
    let jwt_config = task_backend::utils::jwt::JwtConfig {
        secret_key: std::env::var("JWT_SECRET")
            .or_else(|_| std::env::var("JWT_SECRET_KEY"))
            .unwrap_or_else(|_| "test-secret-key-that-is-at-least-32-characters-long".to_string()),
        access_token_expiry_minutes: std::env::var("JWT_ACCESS_TOKEN_EXPIRY_MINUTES")
            .unwrap_or_else(|_| "15".to_string())
            .parse()
            .unwrap_or(15),
        refresh_token_expiry_days: std::env::var("JWT_REFRESH_TOKEN_EXPIRY_DAYS")
            .unwrap_or_else(|_| "7".to_string())
            .parse()
            .unwrap_or(7),
        issuer: "task-backend".to_string(),
        audience: "task-backend-users".to_string(),
    };
    let jwt_manager = Arc::new(JwtManager::new(jwt_config).unwrap());
    let email_service = Arc::new(
        EmailService::new(EmailConfig {
            provider: EmailProvider::Development,
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
    // Audit log service for task service
    let audit_log_repo_local = Arc::new(
        task_backend::repository::audit_log_repository::AuditLogRepository::new(
            db.connection.clone(),
        ),
    );
    let audit_log_service_local = Arc::new(
        task_backend::service::audit_log_service::AuditLogService::new(
            audit_log_repo_local.clone(),
        ),
    );

    let team_service_for_task = Arc::new(TeamService::new(
        Arc::new(db.connection.clone()),
        TeamRepository::new(db.connection.clone()),
        UserRepository::new(db.connection.clone()),
        OrganizationRepository::new(db.connection.clone()),
        email_service.clone(),
    ));

    let task_service = Arc::new(TaskService::new(
        db.connection.clone(),
        team_service_for_task,
        audit_log_service_local,
    ));
    let subscription_service = Arc::new(SubscriptionService::new(
        db.connection.clone(),
        email_service.clone(),
    ));
    let team_service = Arc::new(TeamService::new(
        Arc::new(db.connection.clone()),
        TeamRepository::new(db.connection.clone()),
        UserRepository::new(db.connection.clone()),
        OrganizationRepository::new(db.connection.clone()),
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
            user_repo.clone(),
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
    let permission_service = Arc::new(PermissionService::new(role_repo.clone(), user_repo.clone()));

    let bulk_operation_history_repo = Arc::new(
        task_backend::repository::bulk_operation_history_repository::BulkOperationHistoryRepository::new(
            db.connection.clone(),
        ),
    );

    // テスト用のモックストレージサービスを作成
    let storage_service: Arc<dyn StorageService> =
        Arc::new(crate::common::mock_storage::MockStorageService::new());

    // 添付ファイルサービスの作成
    let attachment_service = Arc::new(AttachmentService::new(
        db.connection.clone(),
        storage_service.clone(),
    ));

    // 決済サービスの作成
    let payment_service = Arc::new(PaymentService::new(
        db.connection.clone(),
        subscription_service.clone(),
    ));

    // Global audit log service for AppState
    let audit_log_service = Arc::new(
        task_backend::service::audit_log_service::AuditLogService::new(
            audit_log_repo_local.clone(),
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
        jwt_manager,
        Arc::new(db.connection.clone()),
        &app_config,
    );

    // ルーターを作成して統合
    let app = Router::new()
        .merge(auth_handler::auth_router(app_state.clone()))
        .merge(user_handler::user_router(app_state.clone()))
        .merge(task_backend::api::handlers::security_handler::security_router(app_state.clone()))
        .merge(task_backend::api::handlers::analytics_handler::analytics_router(app_state.clone()))
        .merge(task_backend::api::handlers::task_handler::task_router_with_state(app_state.clone()))
        .merge(
            task_backend::api::handlers::attachment_handler::attachment_routes(app_state.clone()),
        )
        .merge(
            task_backend::api::handlers::activity_log_handler::activity_log_router(
                app_state.clone(),
            ),
        )
        .layer(axum::middleware::from_fn_with_state(
            task_backend::middleware::activity_logger::ActivityLogger::new(
                activity_log_repo.clone(),
            ),
            task_backend::middleware::activity_logger::log_activity,
        ));

    (app, schema_name, db)
}

/// タスク機能付きアプリのセットアップ（認証ミドルウェア付き）
pub async fn setup_full_app() -> (Router, String, common::db::TestDatabase) {
    // テスト環境を初期化
    common::init_test_env();

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
    let app_config = AppConfig::for_testing();

    // テスト用にパスワードポリシーを調整（特殊文字要件を無効化）
    let password_policy = task_backend::utils::password::PasswordPolicy {
        require_special: false,
        ..Default::default()
    };
    let argon2_config = task_backend::utils::password::Argon2Config::default();

    // 統合設定からユーティリティを作成
    let password_manager = Arc::new(PasswordManager::new(argon2_config, password_policy).unwrap());
    let jwt_config = task_backend::utils::jwt::JwtConfig {
        secret_key: std::env::var("JWT_SECRET")
            .or_else(|_| std::env::var("JWT_SECRET_KEY"))
            .unwrap_or_else(|_| "test-secret-key-that-is-at-least-32-characters-long".to_string()),
        access_token_expiry_minutes: std::env::var("JWT_ACCESS_TOKEN_EXPIRY_MINUTES")
            .unwrap_or_else(|_| "15".to_string())
            .parse()
            .unwrap_or(15),
        refresh_token_expiry_days: std::env::var("JWT_REFRESH_TOKEN_EXPIRY_DAYS")
            .unwrap_or_else(|_| "7".to_string())
            .parse()
            .unwrap_or(7),
        issuer: "task-backend".to_string(),
        audience: "task-backend-users".to_string(),
    };
    let jwt_manager = Arc::new(JwtManager::new(jwt_config).unwrap());
    let email_service = Arc::new(
        EmailService::new(EmailConfig {
            provider: EmailProvider::Development,
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
    // Audit log service for task service
    let audit_log_repo_local = Arc::new(
        task_backend::repository::audit_log_repository::AuditLogRepository::new(
            db.connection.clone(),
        ),
    );
    let audit_log_service_local = Arc::new(
        task_backend::service::audit_log_service::AuditLogService::new(
            audit_log_repo_local.clone(),
        ),
    );

    let team_service_for_task = Arc::new(TeamService::new(
        Arc::new(db.connection.clone()),
        TeamRepository::new(db.connection.clone()),
        UserRepository::new(db.connection.clone()),
        OrganizationRepository::new(db.connection.clone()),
        email_service.clone(),
    ));

    let task_service = Arc::new(TaskService::new(
        db.connection.clone(),
        team_service_for_task,
        audit_log_service_local,
    ));
    let subscription_service = Arc::new(SubscriptionService::new(
        db.connection.clone(),
        email_service.clone(),
    ));
    let team_service = Arc::new(TeamService::new(
        Arc::new(db.connection.clone()),
        TeamRepository::new(db.connection.clone()),
        UserRepository::new(db.connection.clone()),
        OrganizationRepository::new(db.connection.clone()),
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
            user_repo.clone(),
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
    let permission_service = Arc::new(PermissionService::new(role_repo.clone(), user_repo.clone()));

    // テスト用のモックストレージサービスを作成
    let storage_service: Arc<dyn StorageService> =
        Arc::new(crate::common::mock_storage::MockStorageService::new());

    // 添付ファイルサービスの作成
    let attachment_service = Arc::new(AttachmentService::new(
        db.connection.clone(),
        storage_service.clone(),
    ));

    // 決済サービスの作成
    let payment_service = Arc::new(PaymentService::new(
        db.connection.clone(),
        subscription_service.clone(),
    ));

    // Global audit log service for AppState
    let audit_log_service = Arc::new(
        task_backend::service::audit_log_service::AuditLogService::new(
            audit_log_repo_local.clone(),
        ),
    );

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
            "/share".to_string(), // 共有リンクのプレフィックス（認証不要）
            "/webhooks/stripe".to_string(), // Stripe Webhook（認証不要）
        ],
        admin_only_paths: vec!["/admin".to_string()],
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
            task_backend::api::handlers::activity_log_handler::activity_log_router(
                app_state.clone(),
            ),
        )
        .layer(axum_middleware::from_fn_with_state(
            task_backend::middleware::activity_logger::ActivityLogger::new(
                activity_log_repo.clone(),
            ),
            task_backend::middleware::activity_logger::log_activity,
        ))
        .merge(
            task_backend::api::handlers::subscription_handler::subscription_router_with_state(
                app_state.clone(),
            ),
        )
        .merge(
            task_backend::api::handlers::payment_handler::payment_router_with_state(
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
        .merge(
            task_backend::api::handlers::system_handler::system_router_with_state(Arc::new(
                app_state.clone(),
            )),
        )
        .merge(task_backend::api::handlers::gdpr_handler::gdpr_router_with_state(app_state.clone()))
        .merge(
            task_backend::api::handlers::attachment_handler::attachment_routes(app_state.clone()),
        )
        .merge(task_backend::api::handlers::audit_log_handler::audit_log_router(app_state.clone()))
        .layer(axum_middleware::from_fn_with_state(
            auth_middleware_config,
            jwt_auth_middleware,
        ));

    (app, schema_name, db)
}

/// タスク機能付きアプリのセットアップ（ストレージ機能付き）
pub async fn setup_full_app_with_storage() -> (Router, String, common::db::TestDatabase) {
    // テスト環境を初期化
    common::init_test_env();

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
    let app_config = AppConfig::for_testing();

    // テスト用にパスワードポリシーを調整（特殊文字要件を無効化）
    let password_policy = task_backend::utils::password::PasswordPolicy {
        require_special: false,
        ..Default::default()
    };
    let argon2_config = task_backend::utils::password::Argon2Config::default();

    // 統合設定からユーティリティを作成
    let password_manager = Arc::new(PasswordManager::new(argon2_config, password_policy).unwrap());
    let jwt_config = task_backend::utils::jwt::JwtConfig {
        secret_key: std::env::var("JWT_SECRET")
            .or_else(|_| std::env::var("JWT_SECRET_KEY"))
            .unwrap_or_else(|_| "test-secret-key-that-is-at-least-32-characters-long".to_string()),
        access_token_expiry_minutes: std::env::var("JWT_ACCESS_TOKEN_EXPIRY_MINUTES")
            .unwrap_or_else(|_| "15".to_string())
            .parse()
            .unwrap_or(15),
        refresh_token_expiry_days: std::env::var("JWT_REFRESH_TOKEN_EXPIRY_DAYS")
            .unwrap_or_else(|_| "7".to_string())
            .parse()
            .unwrap_or(7),
        issuer: "task-backend".to_string(),
        audience: "task-backend-users".to_string(),
    };
    let jwt_manager = Arc::new(JwtManager::new(jwt_config).unwrap());
    let email_service = Arc::new(
        EmailService::new(EmailConfig {
            provider: EmailProvider::Development,
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
    // Audit log service for task service
    let audit_log_repo_local = Arc::new(
        task_backend::repository::audit_log_repository::AuditLogRepository::new(
            db.connection.clone(),
        ),
    );
    let audit_log_service_local = Arc::new(
        task_backend::service::audit_log_service::AuditLogService::new(
            audit_log_repo_local.clone(),
        ),
    );

    let team_service_for_task = Arc::new(TeamService::new(
        Arc::new(db.connection.clone()),
        TeamRepository::new(db.connection.clone()),
        UserRepository::new(db.connection.clone()),
        OrganizationRepository::new(db.connection.clone()),
        email_service.clone(),
    ));

    let task_service = Arc::new(TaskService::new(
        db.connection.clone(),
        team_service_for_task,
        audit_log_service_local,
    ));
    let subscription_service = Arc::new(SubscriptionService::new(
        db.connection.clone(),
        email_service.clone(),
    ));
    let team_service = Arc::new(TeamService::new(
        Arc::new(db.connection.clone()),
        TeamRepository::new(db.connection.clone()),
        UserRepository::new(db.connection.clone()),
        OrganizationRepository::new(db.connection.clone()),
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
            user_repo.clone(),
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
    let permission_service = Arc::new(PermissionService::new(role_repo.clone(), user_repo.clone()));

    // テスト用のモックストレージサービスを作成
    let storage_service: Arc<dyn StorageService> =
        Arc::new(crate::common::mock_storage::MockStorageService::new());

    // 添付ファイルサービスの作成
    let attachment_service = Arc::new(AttachmentService::new(
        db.connection.clone(),
        storage_service.clone(),
    ));

    // 決済サービスの作成
    let payment_service = Arc::new(PaymentService::new(
        db.connection.clone(),
        subscription_service.clone(),
    ));

    // Global audit log service for AppState
    let audit_log_service = Arc::new(
        task_backend::service::audit_log_service::AuditLogService::new(
            audit_log_repo_local.clone(),
        ),
    );

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
            "/share".to_string(), // 共有リンクのプレフィックス（認証不要）
            "/webhooks/stripe".to_string(), // Stripe Webhook（認証不要）
        ],
        admin_only_paths: vec!["/admin".to_string()],
        require_verified_email: false,
        require_active_account: true,
    };

    // 全てのルーターを統合
    let app = Router::new()
        .merge(auth_handler::auth_router(app_state.clone()))
        .merge(user_handler::user_router(app_state.clone()))
        .merge(task_backend::api::handlers::task_handler::task_router(
            app_state.clone(),
        ))
        .merge(task_backend::api::handlers::role_handler::role_router(
            app_state.clone(),
        ))
        .merge(task_backend::api::handlers::team_handler::team_router(
            app_state.clone(),
        ))
        .merge(
            task_backend::api::handlers::organization_handler::organization_router_with_state(
                app_state.clone(),
            ),
        )
        .merge(
            task_backend::api::handlers::subscription_handler::subscription_router(
                app_state.clone(),
            ),
        )
        .merge(
            task_backend::api::handlers::permission_handler::permission_router(app_state.clone()),
        )
        .merge(task_backend::api::handlers::analytics_handler::analytics_router(app_state.clone()))
        .merge(task_backend::api::handlers::security_handler::security_router(app_state.clone()))
        .merge(task_backend::api::handlers::gdpr_handler::gdpr_router_with_state(app_state.clone()))
        .merge(
            task_backend::api::handlers::attachment_handler::attachment_routes(app_state.clone()),
        )
        .merge(
            task_backend::api::handlers::activity_log_handler::activity_log_router(
                app_state.clone(),
            ),
        )
        .merge(task_backend::api::handlers::audit_log_handler::audit_log_router(app_state.clone()))
        .layer(axum_middleware::from_fn_with_state(
            task_backend::middleware::activity_logger::ActivityLogger::new(
                activity_log_repo.clone(),
            ),
            task_backend::middleware::activity_logger::log_activity,
        ))
        .layer(axum_middleware::from_fn_with_state(
            auth_middleware_config,
            jwt_auth_middleware,
        ));

    (app, schema_name, db)
}

/// 認証付きHTTPリクエストを作成するヘルパー関数
pub fn create_request<T: serde::Serialize>(
    method: &str,
    uri: &str,
    token: &str,
    body: &T,
) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .method(method)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(serde_json::to_string(body).unwrap()))
        .unwrap()
}

/// レスポンスボディをパースするヘルパー関数
pub async fn parse_response_body<T: serde::de::DeserializeOwned>(
    response: axum::http::Response<Body>,
) -> T {
    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // ApiResponseラッパーを考慮
    if let Some(data) = response_json.get("data") {
        serde_json::from_value(data.clone()).unwrap()
    } else {
        serde_json::from_value(response_json).unwrap()
    }
}

/// テスト用タスクを作成するヘルパー関数
pub async fn create_test_task(
    app: &Router,
    token: &str,
) -> task_backend::api::dto::task_dto::TaskDto {
    use task_backend::api::dto::task_dto::CreateTaskDto;
    use task_backend::domain::task_status::TaskStatus;

    let create_task_dto = CreateTaskDto {
        title: "Test Task".to_string(),
        description: Some("Test Description".to_string()),
        status: Some(TaskStatus::Todo),
        priority: None,
        due_date: Some(chrono::Utc::now()),
    };

    let response = app
        .clone()
        .oneshot(create_request("POST", "/tasks", token, &create_task_dto))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    parse_response_body(response).await
}

/// テスト用チームを作成するヘルパー関数
pub async fn create_test_team(
    app: &Router,
    token: &str,
) -> task_backend::api::dto::team_dto::TeamResponse {
    let team_data = serde_json::json!({
        "name": format!("Test Team {}", uuid::Uuid::new_v4()),
        "description": "Test team for integration tests"
    });

    let response = app
        .clone()
        .oneshot(create_request("POST", "/teams", token, &team_data))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    parse_response_body(response).await
}

/// チームメンバーを追加するヘルパー関数
pub async fn add_team_member(app: &Router, token: &str, team_id: uuid::Uuid, user_id: uuid::Uuid) {
    let member_data = serde_json::json!({
        "user_id": user_id,
        "role": "Member"
    });

    let response = app
        .clone()
        .oneshot(create_request(
            "POST",
            &format!("/teams/{}/members", team_id),
            token,
            &member_data,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
}

/// チームタスクを作成するヘルパー関数
pub async fn create_team_task(
    app: &Router,
    token: &str,
    team_id: uuid::Uuid,
) -> task_backend::api::dto::task_dto::TaskDto {
    let task_data = serde_json::json!({
        "title": "Team Task",
        "description": "Test team task",
        "visibility": "team"
    });

    let response = app
        .clone()
        .oneshot(create_request(
            "POST",
            &format!("/teams/{}/tasks", team_id),
            token,
            &task_data,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    parse_response_body(response).await
}

/// 特定ユーザーに割り当てられたチームタスクを作成するヘルパー関数
pub async fn create_team_task_assigned_to(
    app: &Router,
    token: &str,
    team_id: uuid::Uuid,
    assigned_to: uuid::Uuid,
) -> task_backend::api::dto::task_dto::TaskDto {
    let task_data = serde_json::json!({
        "title": "Assigned Team Task",
        "description": "Test team task with assignment",
        "visibility": "team",
        "assigned_to": assigned_to
    });

    let response = app
        .clone()
        .oneshot(create_request(
            "POST",
            &format!("/teams/{}/tasks", team_id),
            token,
            &task_data,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    parse_response_body(response).await
}

/// メールアドレスからユーザーを作成するヘルパー関数
pub async fn create_user(app: &Router, email: &str) -> crate::common::auth_helper::TestUser {
    use crate::common::auth_helper::create_user_with_credentials;

    let username = email.split('@').next().unwrap();
    create_user_with_credentials(app, email, username, "MyUniqueP@ssw0rd91")
        .await
        .unwrap()
}
