// tests/unit/auth/repository/subscription_history_repository_tests.rs

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use task_backend::{
    core::subscription_tier::SubscriptionTier,
    domain::role_model,
    features::auth::repository::user_repository::{CreateUser, UserRepository},
    repository::subscription_history_repository::SubscriptionHistoryRepository,
};
use uuid::Uuid;

use crate::common;

// リポジトリテスト用のセットアップヘルパー関数
async fn setup_test_repository() -> (
    common::db::TestDatabase,
    SubscriptionHistoryRepository,
    UserRepository,
) {
    let db = common::db::TestDatabase::new().await;
    let repo = SubscriptionHistoryRepository::new(db.connection.clone());
    let user_repo = UserRepository::new(db.connection.clone());
    (db, repo, user_repo)
}

// テスト用ユーザーを作成するヘルパー関数
async fn create_test_user(user_repo: &UserRepository, db: &common::db::TestDatabase) -> Uuid {
    // Get the member role ID from the database
    let role_id = get_member_role_id(db).await;

    let create_user = CreateUser {
        email: format!("test{}@example.com", Uuid::new_v4()),
        username: format!("testuser{}", &Uuid::new_v4().to_string()[..8]),
        password_hash: "hashed_password".to_string(),
        role_id,
        subscription_tier: Some(SubscriptionTier::Free.to_string()),
        is_active: Some(true),
        email_verified: Some(false),
    };

    user_repo.create(create_user).await.unwrap().id
}

// メンバーロールのIDを取得するヘルパー関数
async fn get_member_role_id(db: &common::db::TestDatabase) -> Uuid {
    role_model::Entity::find()
        .filter(role_model::Column::Name.eq("member"))
        .one(&db.connection)
        .await
        .unwrap()
        .expect("Member role should exist in test database")
        .id
}

#[tokio::test]
async fn test_create_subscription_history() {
    let (db, repo, user_repo) = setup_test_repository().await;

    let user_id = create_test_user(&user_repo, &db).await;
    let previous_tier = Some(SubscriptionTier::Free.to_string());
    let new_tier = SubscriptionTier::Pro.to_string();
    let changed_by = Some(create_test_user(&user_repo, &db).await);
    let reason = Some("User upgrade".to_string());

    let result = repo
        .create(
            user_id,
            previous_tier.clone(),
            new_tier.clone(),
            changed_by,
            reason.clone(),
        )
        .await;

    assert!(result.is_ok());
    let history = result.unwrap();
    assert_eq!(history.user_id, user_id);
    assert_eq!(history.previous_tier, previous_tier);
    assert_eq!(history.new_tier, new_tier);
    assert_eq!(history.changed_by, changed_by);
    assert_eq!(history.reason, reason);
}

#[tokio::test]
async fn test_subscription_tier_changes() {
    let (db, repo, user_repo) = setup_test_repository().await;

    let user_id = create_test_user(&user_repo, &db).await;

    // Create initial subscription (Free)
    let initial_result = repo
        .create(
            user_id,
            None,
            SubscriptionTier::Free.to_string(),
            None,
            Some("Initial subscription".to_string()),
        )
        .await;
    assert!(initial_result.is_ok());

    // Upgrade to Pro
    let upgrade_result = repo
        .create(
            user_id,
            Some(SubscriptionTier::Free.to_string()),
            SubscriptionTier::Pro.to_string(),
            Some(user_id),
            Some("Upgrade to Pro".to_string()),
        )
        .await;
    assert!(upgrade_result.is_ok());

    // Verify upgrade detection
    let upgrade_history = upgrade_result.unwrap();
    assert!(upgrade_history.is_upgrade());
    assert!(!upgrade_history.is_downgrade());

    // Downgrade to Free
    let downgrade_result = repo
        .create(
            user_id,
            Some(SubscriptionTier::Pro.to_string()),
            SubscriptionTier::Free.to_string(),
            Some(user_id),
            Some("Downgrade to Free".to_string()),
        )
        .await;
    assert!(downgrade_result.is_ok());

    // Verify downgrade detection
    let downgrade_history = downgrade_result.unwrap();
    assert!(!downgrade_history.is_upgrade());
    assert!(downgrade_history.is_downgrade());

    // Test finding by user ID
    let user_histories = repo.find_by_user_id(user_id).await;
    assert!(user_histories.is_ok());
    let histories = user_histories.unwrap();
    assert_eq!(histories.len(), 3);

    // Test finding latest
    let latest = repo.find_latest_by_user_id(user_id).await;
    assert!(latest.is_ok());
    let latest_history = latest.unwrap();
    assert!(latest_history.is_some());
    assert_eq!(
        latest_history.unwrap().new_tier,
        SubscriptionTier::Free.to_string()
    );
}

#[tokio::test]
async fn test_find_by_user_id_paginated() {
    let (db, repo, user_repo) = setup_test_repository().await;

    let user_id = create_test_user(&user_repo, &db).await;

    // Create multiple subscription history entries
    for i in 0..5 {
        let tier = match i % 3 {
            0 => SubscriptionTier::Free,
            1 => SubscriptionTier::Pro,
            _ => SubscriptionTier::Enterprise,
        };

        repo.create(
            user_id,
            None,
            tier.to_string(),
            Some(user_id),
            Some(format!("Change {}", i)),
        )
        .await
        .unwrap();
    }

    // Test pagination
    let (histories, total_count) = repo.find_by_user_id_paginated(user_id, 1, 3).await.unwrap();

    assert_eq!(histories.len(), 3);
    assert_eq!(total_count, 5);

    // Test second page
    let (histories_page2, _) = repo.find_by_user_id_paginated(user_id, 2, 3).await.unwrap();
    assert_eq!(histories_page2.len(), 2);
}

#[tokio::test]
async fn test_find_by_tier() {
    let (db, repo, user_repo) = setup_test_repository().await;

    let user_id = create_test_user(&user_repo, &db).await;
    let pro_tier = SubscriptionTier::Pro.to_string();

    // Create multiple subscription history entries with different tiers
    repo.create(
        user_id,
        None,
        SubscriptionTier::Free.to_string(),
        None,
        None,
    )
    .await
    .unwrap();
    repo.create(user_id, None, pro_tier.clone(), None, None)
        .await
        .unwrap();
    repo.create(user_id, None, pro_tier.clone(), None, None)
        .await
        .unwrap();
    repo.create(
        user_id,
        None,
        SubscriptionTier::Enterprise.to_string(),
        None,
        None,
    )
    .await
    .unwrap();

    // Find Pro tier changes
    let pro_histories = repo.find_by_tier(&pro_tier).await.unwrap();
    assert_eq!(pro_histories.len(), 2);

    for history in pro_histories {
        assert_eq!(history.new_tier, pro_tier);
    }
}

#[tokio::test]
async fn test_get_tier_change_stats() {
    let (db, repo, user_repo) = setup_test_repository().await;

    let user_id = create_test_user(&user_repo, &db).await;

    // Create multiple subscription history entries
    repo.create(
        user_id,
        None,
        SubscriptionTier::Free.to_string(),
        None,
        None,
    )
    .await
    .unwrap();
    repo.create(user_id, None, SubscriptionTier::Pro.to_string(), None, None)
        .await
        .unwrap();
    repo.create(user_id, None, SubscriptionTier::Pro.to_string(), None, None)
        .await
        .unwrap();
    repo.create(
        user_id,
        None,
        SubscriptionTier::Enterprise.to_string(),
        None,
        None,
    )
    .await
    .unwrap();

    // Get tier change statistics
    let stats = repo.get_tier_change_stats().await.unwrap();

    // Should have stats for all tiers
    assert!(!stats.is_empty());

    // Find Pro tier stats (should have 2 entries)
    let pro_stats = stats
        .iter()
        .find(|(tier, _)| tier == &SubscriptionTier::Pro.to_string());
    assert!(pro_stats.is_some());
    assert_eq!(pro_stats.unwrap().1, 2);
}

#[tokio::test]
async fn test_get_user_change_stats() {
    let (db, repo, user_repo) = setup_test_repository().await;

    let user_id = create_test_user(&user_repo, &db).await;

    // Create subscription history with upgrades and downgrades
    repo.create(
        user_id,
        None,
        SubscriptionTier::Free.to_string(),
        None,
        Some("Initial".to_string()),
    )
    .await
    .unwrap();
    repo.create(
        user_id,
        Some(SubscriptionTier::Free.to_string()),
        SubscriptionTier::Pro.to_string(),
        None,
        Some("Upgrade".to_string()),
    )
    .await
    .unwrap();
    repo.create(
        user_id,
        Some(SubscriptionTier::Pro.to_string()),
        SubscriptionTier::Enterprise.to_string(),
        None,
        Some("Upgrade".to_string()),
    )
    .await
    .unwrap();
    repo.create(
        user_id,
        Some(SubscriptionTier::Enterprise.to_string()),
        SubscriptionTier::Pro.to_string(),
        None,
        Some("Downgrade".to_string()),
    )
    .await
    .unwrap();

    // Get user statistics
    let stats = repo.get_user_change_stats(user_id).await.unwrap();

    assert_eq!(stats.user_id, user_id);
    assert_eq!(stats.total_changes, 4);
    assert_eq!(stats.upgrade_count, 2);
    assert_eq!(stats.downgrade_count, 1);
    assert_eq!(stats.current_tier, Some(SubscriptionTier::Pro.to_string()));
    assert!(stats.first_subscription_date.is_some());
}

#[tokio::test]
async fn test_find_upgrades_and_downgrades() {
    let (db, repo, user_repo) = setup_test_repository().await;

    let user_id = create_test_user(&user_repo, &db).await;

    // Create various subscription changes
    repo.create(
        user_id,
        None,
        SubscriptionTier::Free.to_string(),
        None,
        None,
    )
    .await
    .unwrap();
    repo.create(
        user_id,
        Some(SubscriptionTier::Free.to_string()),
        SubscriptionTier::Pro.to_string(),
        None,
        None,
    )
    .await
    .unwrap();
    repo.create(
        user_id,
        Some(SubscriptionTier::Pro.to_string()),
        SubscriptionTier::Enterprise.to_string(),
        None,
        None,
    )
    .await
    .unwrap();
    repo.create(
        user_id,
        Some(SubscriptionTier::Enterprise.to_string()),
        SubscriptionTier::Free.to_string(),
        None,
        None,
    )
    .await
    .unwrap();

    // Test finding upgrades
    let upgrades = repo.find_upgrades().await.unwrap();
    assert_eq!(upgrades.len(), 2); // Free->Pro and Pro->Enterprise

    // Test finding downgrades
    let downgrades = repo.find_downgrades().await.unwrap();
    assert_eq!(downgrades.len(), 1); // Enterprise->Free
}

#[tokio::test]
async fn test_delete_operations() {
    let (db, repo, user_repo) = setup_test_repository().await;

    let user_id = create_test_user(&user_repo, &db).await;

    // Create a subscription history entry
    let history = repo
        .create(
            user_id,
            None,
            SubscriptionTier::Pro.to_string(),
            None,
            Some("Test entry".to_string()),
        )
        .await
        .unwrap();

    // Verify it exists
    let found = repo.find_by_id(history.id).await.unwrap();
    assert!(found.is_some());

    // Delete by ID
    let deleted = repo.delete_by_id(history.id).await.unwrap();
    assert!(deleted);

    // Verify it's gone
    let not_found = repo.find_by_id(history.id).await.unwrap();
    assert!(not_found.is_none());

    // Test delete by user ID
    let history1 = repo
        .create(user_id, None, SubscriptionTier::Pro.to_string(), None, None)
        .await
        .unwrap();
    let history2 = repo
        .create(
            user_id,
            None,
            SubscriptionTier::Enterprise.to_string(),
            None,
            None,
        )
        .await
        .unwrap();

    let deleted_count = repo.delete_by_user_id(user_id).await.unwrap();
    assert_eq!(deleted_count, 2);

    // Verify they're gone
    assert!(repo.find_by_id(history1.id).await.unwrap().is_none());
    assert!(repo.find_by_id(history2.id).await.unwrap().is_none());
}
