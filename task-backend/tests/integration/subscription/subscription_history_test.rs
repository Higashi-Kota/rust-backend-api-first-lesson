// tests/integration/subscription/subscription_history_test.rs

use crate::common::db::TestDatabase;
use std::sync::Arc;
use task_backend::api::AppState;
use task_backend::config::AppConfig;
use task_backend::repository::{
    activity_log_repository::ActivityLogRepository,
    email_verification_token_repository::EmailVerificationTokenRepository,
    login_attempt_repository::LoginAttemptRepository,
    password_reset_token_repository::PasswordResetTokenRepository,
    refresh_token_repository::RefreshTokenRepository, role_repository::RoleRepository,
    subscription_history_repository::SubscriptionHistoryRepository,
    team_repository::TeamRepository, user_repository::UserRepository,
};
use task_backend::service::{
    auth_service::AuthService, role_service::RoleService,
    subscription_service::SubscriptionService, task_service::TaskService,
    team_service::TeamService, user_service::UserService,
};
use task_backend::utils::{
    email::{EmailConfig, EmailService},
    jwt::JwtManager,
    password::PasswordManager,
};

/// テスト用のAppStateを作成
async fn create_test_app_state() -> (AppState, TestDatabase) {
    let db = TestDatabase::new().await;
    let app_config = AppConfig::for_testing();

    // 必要なサービスを構築
    let user_repo = Arc::new(UserRepository::new(db.connection.clone()));
    let role_repo = Arc::new(RoleRepository::new(Arc::new(db.connection.clone())));
    let refresh_token_repo = Arc::new(RefreshTokenRepository::new(db.connection.clone()));
    let password_reset_token_repo =
        Arc::new(PasswordResetTokenRepository::new(db.connection.clone()));
    let email_verification_token_repo =
        Arc::new(EmailVerificationTokenRepository::new(db.connection.clone()));

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

    let activity_log_repo = Arc::new(ActivityLogRepository::new(db.connection.clone()));
    let login_attempt_repo = Arc::new(LoginAttemptRepository::new(db.connection.clone()));

    let auth_service = Arc::new(AuthService::new(
        user_repo.clone(),
        role_repo.clone(),
        refresh_token_repo,
        password_reset_token_repo,
        email_verification_token_repo,
        activity_log_repo.clone(),
        login_attempt_repo.clone(),
        password_manager,
        jwt_manager.clone(),
        email_service.clone(),
        Arc::new(db.connection.clone()),
    ));

    let user_service = Arc::new(UserService::new(user_repo.clone()));
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
            TeamRepository::new(db.connection.clone()),
            UserRepository::new(db.connection.clone()),
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
        security_service,
        jwt_manager,
        Arc::new(db.connection.clone()),
        &app_config,
    );

    (app_state, db)
}

#[tokio::test]
async fn test_subscription_history_service() {
    let (app_state, _db) = create_test_app_state().await;

    // テストユーザーを作成
    let user_email = "test@example.com";
    let username = "test_user";

    let signup_req = task_backend::api::dto::auth_dto::SignupRequest {
        email: user_email.to_string(),
        username: username.to_string(),
        password: "MyUniqueP@ssw0rd91".to_string(),
    };

    let user = app_state
        .auth_service
        .signup(signup_req)
        .await
        .expect("Failed to create test user");

    // サブスクリプションを変更
    let (updated_user, history) = app_state
        .subscription_service
        .change_subscription_tier(
            user.user.id,
            "pro".to_string(),
            Some(user.user.id),
            Some("Need more features".to_string()),
        )
        .await
        .expect("Failed to change subscription");

    assert_eq!(updated_user.subscription_tier, "pro");
    assert_eq!(history.new_tier, "pro");
    assert_eq!(history.previous_tier, Some("free".to_string()));

    // 履歴を取得
    let (history_list, count) = app_state
        .subscription_service
        .get_user_subscription_history(user.user.id, 1, 10)
        .await
        .expect("Failed to get subscription history");

    assert_eq!(count, 1);
    assert_eq!(history_list.len(), 1);
    assert_eq!(history_list[0].new_tier, "pro");

    // 統計を取得
    let stats = app_state
        .subscription_service
        .get_user_subscription_stats(user.user.id)
        .await
        .expect("Failed to get subscription stats");

    assert_eq!(stats.total_changes, 1);
    assert_eq!(stats.upgrade_count, 1);
    assert_eq!(stats.downgrade_count, 0);
}

#[tokio::test]
async fn test_subscription_tier_statistics() {
    let (app_state, _db) = create_test_app_state().await;

    // 複数のユーザーを作成して異なる階層に設定
    for i in 0..3 {
        let signup_req = task_backend::api::dto::auth_dto::SignupRequest {
            email: format!("user{}@example.com", i),
            username: format!("user{}", i),
            password: "MyUniqueP@ssw0rd91".to_string(),
        };

        let user = app_state
            .auth_service
            .signup(signup_req)
            .await
            .expect("Failed to create test user");

        if i == 1 {
            app_state
                .subscription_service
                .change_subscription_tier(user.user.id, "pro".to_string(), None, None)
                .await
                .expect("Failed to change subscription");
        } else if i == 2 {
            app_state
                .subscription_service
                .change_subscription_tier(user.user.id, "enterprise".to_string(), None, None)
                .await
                .expect("Failed to change subscription");
        }
    }

    // 階層統計を取得
    let tier_stats = app_state
        .subscription_service
        .get_tier_change_statistics()
        .await
        .expect("Failed to get tier statistics");

    assert!(!tier_stats.is_empty());

    // アップグレード履歴を取得
    let upgrades = app_state
        .subscription_service
        .get_upgrade_history()
        .await
        .expect("Failed to get upgrade history");

    assert_eq!(upgrades.len(), 2); // pro と enterprise へのアップグレード
}

#[tokio::test]
async fn test_subscription_history_date_range() {
    let (app_state, _db) = create_test_app_state().await;

    // テストユーザーを作成
    let signup_req = task_backend::api::dto::auth_dto::SignupRequest {
        email: "date_test@example.com".to_string(),
        username: "date_test_user".to_string(),
        password: "MyUniqueP@ssw0rd91".to_string(),
    };

    let user = app_state
        .auth_service
        .signup(signup_req)
        .await
        .expect("Failed to create test user");

    // サブスクリプションを変更
    app_state
        .subscription_service
        .change_subscription_tier(user.user.id, "pro".to_string(), None, None)
        .await
        .expect("Failed to change subscription");

    // 日付範囲で履歴を取得
    let end_date = chrono::Utc::now();
    let start_date = end_date - chrono::Duration::days(7);

    let history_in_range = app_state
        .subscription_service
        .get_subscription_history_by_date_range(start_date, end_date)
        .await
        .expect("Failed to get history by date range");

    assert_eq!(history_in_range.len(), 1);
    assert_eq!(history_in_range[0].new_tier, "pro");
}

#[tokio::test]
async fn test_downgrade_history() {
    let (app_state, _db) = create_test_app_state().await;

    // テストユーザーを作成
    let signup_req = task_backend::api::dto::auth_dto::SignupRequest {
        email: "downgrade@example.com".to_string(),
        username: "downgrade_user".to_string(),
        password: "MyUniqueP@ssw0rd91".to_string(),
    };

    let user = app_state
        .auth_service
        .signup(signup_req)
        .await
        .expect("Failed to create test user");

    // まずアップグレード
    app_state
        .subscription_service
        .change_subscription_tier(user.user.id, "enterprise".to_string(), None, None)
        .await
        .expect("Failed to upgrade subscription");

    // 次にダウングレード
    app_state
        .subscription_service
        .change_subscription_tier(
            user.user.id,
            "pro".to_string(),
            None,
            Some("Too expensive".to_string()),
        )
        .await
        .expect("Failed to downgrade subscription");

    // ダウングレード履歴を取得
    let downgrades = app_state
        .subscription_service
        .get_downgrade_history()
        .await
        .expect("Failed to get downgrade history");

    assert_eq!(downgrades.len(), 1);
    assert_eq!(downgrades[0].new_tier, "pro");
    assert_eq!(downgrades[0].previous_tier, Some("enterprise".to_string()));
    assert!(downgrades[0].is_downgrade);
}
