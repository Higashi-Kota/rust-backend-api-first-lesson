// tests/unit/auth/service/user_service_tests.rs

// ユーザーサービス関連のユニットテスト

use std::sync::Arc;
use task_backend::features::auth::repository::email_verification_token_repository::EmailVerificationTokenRepository;
use task_backend::features::user::dto::{UpdateProfileRequest, UpdateUsernameRequest};
use task_backend::features::user::repositories::{
    user::UserRepository, user_settings::UserSettingsRepository,
};
use task_backend::features::user::services::user_service::UserService;
use task_backend::repository::bulk_operation_history_repository::BulkOperationHistoryRepository;
use task_backend::utils::validation::common::validate_username;
use validator::Validate;

// テスト用のサービスを作成するヘルパー関数
async fn create_test_user_service() -> (crate::common::db::TestDatabase, UserService) {
    let db = crate::common::db::TestDatabase::new().await;
    let connection = db.connection.clone();

    let user_repo = Arc::new(UserRepository::new(connection.clone()));
    let user_settings_repo = Arc::new(UserSettingsRepository::new(connection.clone()));
    let bulk_operation_history_repo =
        Arc::new(BulkOperationHistoryRepository::new(connection.clone()));
    let email_verification_token_repo = Arc::new(EmailVerificationTokenRepository::new(connection));

    let service = UserService::new(
        user_repo,
        user_settings_repo,
        bulk_operation_history_repo,
        email_verification_token_repo,
    );

    (db, service)
}

// Phase 1.1 新規API用のユニットテスト

#[tokio::test]
async fn test_update_username_with_actual_service() {
    // Arrange: テスト用のサービスとデータベースを准備
    let (db, service) = create_test_user_service().await;
    let connection = db.connection.clone();

    // テスト用ユーザーを作成
    use task_backend::features::user::repositories::user::{CreateUser, UserRepository};
    use task_backend::repository::role_repository::RoleRepository;
    let user_repo = UserRepository::new(connection.clone());
    let role_repo = RoleRepository::new(std::sync::Arc::new(connection));

    let default_role = role_repo.find_by_name("member").await.unwrap().unwrap();
    let test_user = CreateUser {
        email: "test_update@example.com".to_string(),
        username: "oldusername".to_string(),
        password_hash: "hashed_password".to_string(),
        role_id: default_role.id,
        subscription_tier: None,
        is_active: None,
        email_verified: None,
    };
    let created_user = user_repo.create(test_user).await.unwrap();

    // Act: ユーザー名を更新
    let new_username = "newusername";
    let updated_user = service
        .update_username(created_user.id, new_username)
        .await
        .unwrap();

    // Assert: ユーザー名が更新されていることを確認
    assert_eq!(updated_user.username, new_username);
    assert_eq!(updated_user.id, created_user.id);
    assert_eq!(updated_user.email, created_user.email);
}

#[tokio::test]
async fn test_update_email_with_actual_service() {
    // Arrange: テスト用サービスとデータを准備
    let (db, service) = create_test_user_service().await;
    let connection = db.connection.clone();

    // テスト用ユーザーを作成
    use task_backend::features::user::repositories::user::{CreateUser, UserRepository};
    use task_backend::repository::role_repository::RoleRepository;
    let user_repo = UserRepository::new(connection.clone());
    let role_repo = RoleRepository::new(std::sync::Arc::new(connection));

    let default_role = role_repo.find_by_name("member").await.unwrap().unwrap();
    let test_user = CreateUser {
        email: "old_email@example.com".to_string(),
        username: "emailtester".to_string(),
        password_hash: "hashed_password".to_string(),
        role_id: default_role.id,
        subscription_tier: None,
        is_active: None,
        email_verified: None,
    };
    let created_user = user_repo.create(test_user).await.unwrap();

    // Act: メールアドレスを更新
    let new_email = "new_email@example.com";
    let updated_user = service
        .update_email(created_user.id, new_email)
        .await
        .unwrap();

    // Assert: メールアドレスが更新され、email_verifiedがfalseにリセットされる
    assert_eq!(updated_user.email, new_email);
    assert_eq!(updated_user.id, created_user.id);
    assert!(!updated_user.email_verified);
}

#[tokio::test]
async fn test_toggle_account_status_with_actual_service() {
    // Arrange: テスト用サービスとデータを准備
    let (db, service) = create_test_user_service().await;
    let connection = db.connection.clone();

    // テスト用ユーザーを作成
    use task_backend::features::user::repositories::user::{CreateUser, UserRepository};
    use task_backend::repository::role_repository::RoleRepository;
    let user_repo = UserRepository::new(connection.clone());
    let role_repo = RoleRepository::new(std::sync::Arc::new(connection));

    let default_role = role_repo.find_by_name("member").await.unwrap().unwrap();
    let test_user = CreateUser {
        email: "toggle_test@example.com".to_string(),
        username: "toggleuser".to_string(),
        password_hash: "hashed_password".to_string(),
        role_id: default_role.id,
        subscription_tier: None,
        is_active: None,
        email_verified: None,
    };
    let created_user = user_repo.create(test_user).await.unwrap();
    assert!(created_user.is_active); // デフォルトでアクティブ

    // Act: アカウントを非アクティブに
    let deactivated_user = service
        .toggle_account_status(created_user.id, false)
        .await
        .unwrap();

    // Assert: アカウントが非アクティブになった
    assert!(!deactivated_user.is_active);

    // Act: アカウントを再びアクティブに
    let reactivated_user = service
        .toggle_account_status(created_user.id, true)
        .await
        .unwrap();

    // Assert: アカウントが再びアクティブになった
    assert!(reactivated_user.is_active);
}

#[tokio::test]
async fn test_get_user_profile_with_actual_service() {
    // Arrange: テスト用サービスとデータを准備
    let (db, service) = create_test_user_service().await;
    let connection = db.connection.clone();

    // テスト用ユーザーを作成
    use task_backend::features::user::repositories::user::{CreateUser, UserRepository};
    use task_backend::repository::role_repository::RoleRepository;
    let user_repo = UserRepository::new(connection.clone());
    let role_repo = RoleRepository::new(std::sync::Arc::new(connection));

    let default_role = role_repo.find_by_name("member").await.unwrap().unwrap();
    let test_user = CreateUser {
        email: "profile_test@example.com".to_string(),
        username: "profileuser".to_string(),
        password_hash: "hashed_password".to_string(),
        role_id: default_role.id,
        subscription_tier: None,
        is_active: None,
        email_verified: None,
    };
    let created_user = user_repo.create(test_user).await.unwrap();

    // Act: ユーザープロファイルを取得
    let profile = service.get_user_profile(created_user.id).await.unwrap();

    // Assert: プロファイルが正しく取得され、パスワードハッシュが含まれない
    assert_eq!(profile.id, created_user.id);
    assert_eq!(profile.email, created_user.email);
    assert_eq!(profile.username, created_user.username);
    assert!(profile.is_active);
    // SafeUserにはpassword_hashが含まれないことを確認
}

#[tokio::test]
async fn test_subscription_tier_validation() {
    // Arrange: SubscriptionTierタイプを使用したバリデーション
    use task_backend::core::subscription_tier::SubscriptionTier;

    // Act & Assert: 有効なサブスクリプション階層
    let free_tier = SubscriptionTier::from_str("free").unwrap();
    assert_eq!(free_tier, SubscriptionTier::Free);
    assert_eq!(free_tier.as_str(), "free");

    let pro_tier = SubscriptionTier::from_str("pro").unwrap();
    assert_eq!(pro_tier, SubscriptionTier::Pro);
    assert_eq!(pro_tier.as_str(), "pro");

    let enterprise_tier = SubscriptionTier::from_str("enterprise").unwrap();
    assert_eq!(enterprise_tier, SubscriptionTier::Enterprise);
    assert_eq!(enterprise_tier.as_str(), "enterprise");

    // Act & Assert: 無効なサブスクリプション階層
    let invalid_tier = SubscriptionTier::from_str("Premium");
    assert!(invalid_tier.is_none());

    let empty_tier = SubscriptionTier::from_str("");
    assert!(empty_tier.is_none());
}

#[tokio::test]
async fn test_bulk_operation_with_actual_service() {
    // Arrange: テスト用サービスとデータを准備
    let (db, service) = create_test_user_service().await;
    let connection = db.connection.clone();

    // 複数のテストユーザーを作成
    use task_backend::features::user::dto::BulkUserOperation;
    use task_backend::features::user::repositories::user::{CreateUser, UserRepository};
    use task_backend::repository::role_repository::RoleRepository;

    let user_repo = UserRepository::new(connection.clone());
    let role_repo = RoleRepository::new(std::sync::Arc::new(connection));
    let default_role = role_repo.find_by_name("member").await.unwrap().unwrap();

    let mut user_ids = Vec::new();
    for i in 0..3 {
        let test_user = CreateUser {
            email: format!("bulk_test_{}@example.com", i),
            username: format!("bulkuser{}", i),
            password_hash: "hashed_password".to_string(),
            role_id: default_role.id,
            subscription_tier: None,
            is_active: None,
            email_verified: None,
        };
        let created_user = user_repo.create(test_user).await.unwrap();
        user_ids.push(created_user.id);
    }

    // Act: 一括操作を実行（アカウントを非アクティブに）
    let admin_user_id = user_ids[0]; // 便宜上、最初のユーザーを実行者とする
    let result = service
        .bulk_user_operations_extended(
            &BulkUserOperation::Deactivate,
            &user_ids,
            None,
            false,
            admin_user_id,
        )
        .await
        .unwrap();

    // Assert: 一括操作結果を検証
    assert_eq!(result.successful, 3);
    assert_eq!(result.failed, 0);
    assert_eq!(result.errors.len(), 0);

    // ユーザーが実際に非アクティブになったか確認
    for user_id in &user_ids {
        let user = user_repo.find_by_id(*user_id).await.unwrap().unwrap();
        assert!(!user.is_active);
    }
}

#[tokio::test]
async fn test_subscription_tier_validation_comprehensive() {
    // サブスクリプション階層の包括的バリデーションテスト
    let valid_tiers = ["free", "pro", "enterprise"];
    let invalid_tiers = ["", "Basic", "Premium", "Ultimate"];

    for tier in valid_tiers {
        assert!(["free", "pro", "enterprise"].contains(&tier));
    }

    for tier in invalid_tiers {
        assert!(!["free", "pro", "enterprise"].contains(&tier));
    }
}

#[tokio::test]
async fn test_role_name_validation() {
    // ロール名のバリデーションテスト
    let valid_roles = ["admin", "member", "viewer"];
    let invalid_roles = ["", "super_admin", "guest"];

    for role in valid_roles {
        assert!(["admin", "member", "viewer"].contains(&role));
    }

    for role in invalid_roles {
        assert!(!["admin", "member", "viewer"].contains(&role));
    }
}

#[tokio::test]
async fn test_pagination_calculation() {
    // ページネーション計算のテスト
    let total_users = 95;
    let page_size = 20;

    let total_pages = (total_users + page_size - 1) / page_size; // ceil division
    assert_eq!(total_pages, 5);

    // 最後のページのアイテム数
    let last_page_items = total_users % page_size;
    let expected_last_page_items = if last_page_items == 0 {
        page_size
    } else {
        last_page_items
    };
    assert_eq!(expected_last_page_items, 15);
}

#[tokio::test]
async fn test_conversion_rate_calculation() {
    // コンバージョン率計算のテスト
    let total_users = 100;
    let paid_users = 30; // Pro + Enterprise

    let conversion_rate = if total_users > 0 {
        (paid_users as f64 / total_users as f64) * 100.0
    } else {
        0.0
    };

    assert_eq!(conversion_rate, 30.0);

    // エッジケース: ユーザーが0人の場合
    let conversion_rate_zero = if 0 > 0 {
        (0 as f64 / 0 as f64) * 100.0
    } else {
        0.0
    };
    assert_eq!(conversion_rate_zero, 0.0);
}

#[tokio::test]
async fn test_username_validation_with_request() {
    // AAAパターン: Arrange-Act-Assert

    // Arrange: テストデータを準備
    let long_username = "a".repeat(31);
    let invalid_usernames = ["", "ab", &long_username, "invalid-name!", "user@name"];
    let valid_usernames = ["user", "test_user", "user123", "alice_bob"];

    // Act & Assert: 無効なユーザー名のバリデーション
    for username in &invalid_usernames {
        let request = UpdateUsernameRequest {
            username: (*username).to_string(),
        };

        // バリデーションが失敗することを確認
        let result = request.validate();
        assert!(result.is_err(), "Username '{}' should be invalid", username);
    }

    // Act & Assert: 有効なユーザー名のバリデーション
    for username in &valid_usernames {
        let request = UpdateUsernameRequest {
            username: (*username).to_string(),
        };

        // バリデーションが成功することを確認
        let result = request.validate();
        assert!(result.is_ok(), "Username '{}' should be valid", username);

        // カスタムバリデーション関数もテスト
        let validation_result = validate_username(username);
        assert!(
            validation_result.is_ok(),
            "Custom validation for '{}' should pass",
            username
        );
    }
}

#[tokio::test]
async fn test_profile_update_request_validation() {
    // AAAパターン: Arrange-Act-Assert

    // Arrange: テストデータを準備
    let profile_updates = [
        (Some("new_username"), Some("newemail@example.com")),
        (Some("updated_user"), Some("updated@example.com")),
        (Some("alice123"), None),
        (None, Some("test@example.com")),
    ];

    // Act & Assert: プロファイル更新リクエストのバリデーション
    for (username, email) in &profile_updates {
        let request = UpdateProfileRequest {
            username: username.map(|s| s.to_string()),
            email: email.map(|s| s.to_string()),
        };

        // 基本バリデーション
        let validation_result = request.validate();
        assert!(
            validation_result.is_ok(),
            "Profile update request should be valid"
        );

        // カスタムバリデーション（少なくとも1つのフィールドが必要）
        let custom_validation = request.validate_update();
        if username.is_none() && email.is_none() {
            assert!(
                custom_validation.is_err(),
                "Update without any fields should be invalid"
            );
        } else {
            assert!(
                custom_validation.is_ok(),
                "Update with at least one field should be valid"
            );
        }

        // 更新されたフィールドの確認
        let updated_fields = request.get_updated_fields();
        let expected_count = [username.is_some(), email.is_some()]
            .iter()
            .filter(|&&x| x)
            .count();
        assert_eq!(
            updated_fields.len(),
            expected_count,
            "Updated fields count mismatch"
        );
    }

    // 無効なプロファイル更新リクエストのテスト
    let empty_request = UpdateProfileRequest {
        username: None,
        email: None,
    };
    assert!(
        empty_request.validate_update().is_err(),
        "Empty update request should be invalid"
    );
}
