# テスト戦略ベストプラクティス

## 概要

このドキュメントでは、Rust製バックエンドAPIにおける包括的なテスト戦略のベストプラクティスをまとめています。

## テストピラミッド

### 1. テストの階層

```
        /\
       /  \      E2Eテスト（5%）
      /____\     - 完全なユーザーシナリオ
     /      \    - 本番環境に近い設定
    /________\   統合テスト（25%）
   /          \  - APIエンドポイント
  /____________\ - データベース連携
 /              \ ユニットテスト（70%）
/________________\- ビジネスロジック
                  - ユーティリティ関数
```

## ユニットテスト

### 1. ビジネスロジックのテスト

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use mockall::mock;

    // リポジトリのモック定義
    mock! {
        UserRepository {}
        
        #[async_trait]
        impl UserRepository for UserRepository {
            async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, DbErr>;
            async fn update_subscription(&self, id: Uuid, tier: SubscriptionTier) -> Result<User, DbErr>;
        }
    }

    #[tokio::test]
    async fn test_subscription_upgrade_success() {
        // Arrange
        let mut mock_repo = MockUserRepository::new();
        let user_id = Uuid::new_v4();
        let current_user = User {
            id: user_id,
            subscription_tier: SubscriptionTier::Free,
            ..Default::default()
        };
        let upgraded_user = User {
            id: user_id,
            subscription_tier: SubscriptionTier::Pro,
            ..Default::default()
        };

        mock_repo
            .expect_find_by_id()
            .with(eq(user_id))
            .times(1)
            .returning(move |_| Ok(Some(current_user.clone())));

        mock_repo
            .expect_update_subscription()
            .with(eq(user_id), eq(SubscriptionTier::Pro))
            .times(1)
            .returning(move |_, _| Ok(upgraded_user.clone()));

        let service = SubscriptionService::new(Arc::new(mock_repo));

        // Act
        let result = service.upgrade_subscription(user_id, SubscriptionTier::Pro).await;

        // Assert
        assert!(result.is_ok());
        let upgraded = result.unwrap();
        assert_eq!(upgraded.subscription_tier, SubscriptionTier::Pro);
    }

    #[tokio::test]
    async fn test_downgrade_with_constraint_violation() {
        // Arrange
        let service = create_test_service();
        let user_id = create_user_with_tasks(150).await; // Free tier limit is 100

        // Act
        let result = service.downgrade_subscription(user_id, SubscriptionTier::Free).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            SubscriptionError::DowngradeViolations(violations) => {
                assert_eq!(violations.len(), 1);
                assert_eq!(violations[0].resource, "tasks");
                assert_eq!(violations[0].current, 150);
                assert_eq!(violations[0].limit, 100);
            }
            _ => panic!("Expected DowngradeViolations error"),
        }
    }
}
```

### 2. エッジケースのテスト

```rust
#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    fn test_subscription_tier_ordering() {
        assert!(SubscriptionTier::Free < SubscriptionTier::Pro);
        assert!(SubscriptionTier::Pro < SubscriptionTier::Enterprise);
        assert!(SubscriptionTier::Enterprise > SubscriptionTier::Free);
    }

    #[test]
    fn test_feature_limits_boundaries() {
        let free_features = SubscriptionTier::Free.features();
        assert_eq!(free_features.max_tasks, 100);
        assert_eq!(free_features.max_teams, 0); // No team feature

        let enterprise_features = SubscriptionTier::Enterprise.features();
        assert_eq!(enterprise_features.max_tasks, u32::MAX); // Unlimited
    }

    #[tokio::test]
    async fn test_concurrent_quota_updates() {
        let service = create_test_quota_service();
        let user_id = Uuid::new_v4();

        // 並行して複数のクォータ更新を実行
        let handles: Vec<_> = (0..10)
            .map(|_| {
                let service = service.clone();
                tokio::spawn(async move {
                    service.check_and_increment_quota(user_id, ResourceType::Task, 1).await
                })
            })
            .collect();

        let results: Vec<_> = futures::future::join_all(handles).await;

        // 全ての操作が成功し、正しくカウントされることを確認
        let success_count = results.iter().filter(|r| r.as_ref().unwrap().is_ok()).count();
        assert!(success_count <= 100); // Free tier limit
    }
}
```

## 統合テスト

### 1. APIエンドポイントテスト（AAA パターン）

```rust
#[cfg(test)]
mod integration_tests {
    use crate::test_helpers::*;
    use axum::http::StatusCode;
    use serde_json::json;

    #[tokio::test]
    async fn test_create_organization_full_flow() {
        // Arrange: テスト環境のセットアップ
        let (app, schema_name, db) = setup_full_app().await;
        let user = create_and_authenticate_user(&app, "test@example.com").await;
        
        let create_request = json!({
            "name": "Test Organization",
            "description": "Test description",
            "subscription_tier": "pro"
        });

        // Act: 組織作成APIを実行
        let response = app
            .oneshot(
                create_authenticated_request(
                    "POST",
                    "/organizations",
                    &user.token,
                    &create_request,
                )
            )
            .await
            .unwrap();

        // Assert: レスポンスとデータベース状態を検証
        assert_eq!(response.status(), StatusCode::CREATED);
        
        let body: serde_json::Value = parse_response_body(response).await;
        assert_eq!(body["data"]["name"], "Test Organization");
        assert_eq!(body["data"]["subscription_tier"], "pro");
        
        // データベースの状態を確認
        let org_count = count_organizations_in_db(&db).await;
        assert_eq!(org_count, 1);
        
        let membership_count = count_organization_members(&db, body["data"]["id"].as_str().unwrap()).await;
        assert_eq!(membership_count, 1); // オーナーが自動的にメンバーになる

        // クリーンアップは自動的に実行される
    }

    #[tokio::test]
    async fn test_team_invitation_workflow() {
        // Arrange: チームとユーザーを準備
        let (app, _, _) = setup_full_app().await;
        let owner = create_and_authenticate_user(&app, "owner@example.com").await;
        let invitee = create_user_without_auth(&app, "invitee@example.com").await;
        let team = create_team(&app, &owner.token, "Test Team").await;

        // Act 1: 招待を送信
        let invite_response = send_team_invitation(
            &app,
            &owner.token,
            &team.id,
            "invitee@example.com",
            "Join our team!",
        ).await;
        
        assert_eq!(invite_response.status(), StatusCode::CREATED);
        let invitation = parse_response_body::<InvitationResponse>(invite_response).await;

        // Act 2: 招待者が招待を確認
        let invitations = get_user_invitations(&app, &invitee.token).await;
        assert_eq!(invitations.len(), 1);
        assert_eq!(invitations[0].team_id, team.id);

        // Act 3: 招待を承認
        let accept_response = accept_team_invitation(
            &app,
            &invitee.token,
            &team.id,
            &invitation.id,
        ).await;
        
        assert_eq!(accept_response.status(), StatusCode::OK);

        // Assert: チームメンバーシップを確認
        let team_members = get_team_members(&app, &owner.token, &team.id).await;
        assert_eq!(team_members.len(), 2);
        assert!(team_members.iter().any(|m| m.email == "invitee@example.com"));
    }
}
```

### 2. データベーストランザクションのテスト

```rust
#[tokio::test]
async fn test_organization_deletion_cascade() {
    // Arrange
    let (app, _, db) = setup_full_app().await;
    let owner = create_and_authenticate_user(&app, "owner@example.com").await;
    
    // 組織、チーム、タスクを作成
    let org = create_organization(&app, &owner.token, "Test Org").await;
    let team = create_team_in_organization(&app, &owner.token, &org.id, "Test Team").await;
    let task = create_task_for_team(&app, &owner.token, &team.id, "Test Task").await;

    // Act: 組織を削除
    let delete_response = delete_organization(&app, &owner.token, &org.id).await;
    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);

    // Assert: カスケード削除を確認
    assert!(get_organization_by_id(&db, &org.id).await.is_none());
    assert!(get_team_by_id(&db, &team.id).await.is_none());
    assert!(get_task_by_id(&db, &task.id).await.is_none());
}

#[tokio::test]
async fn test_concurrent_team_member_additions() {
    // Arrange
    let (app, _, _) = setup_full_app().await;
    let owner = create_and_authenticate_user(&app, "owner@example.com").await;
    let team = create_team(&app, &owner.token, "Test Team").await;
    
    // 10人のユーザーを作成
    let users = futures::future::join_all(
        (0..10).map(|i| create_user_without_auth(&app, &format!("user{}@example.com", i)))
    ).await;

    // Act: 並行してメンバー追加
    let handles: Vec<_> = users
        .into_iter()
        .map(|user| {
            let app = app.clone();
            let token = owner.token.clone();
            let team_id = team.id.clone();
            tokio::spawn(async move {
                add_team_member(&app, &token, &team_id, &user.id).await
            })
        })
        .collect();

    let results = futures::future::join_all(handles).await;

    // Assert: 全員が正しく追加されたことを確認
    let successful_additions = results.iter().filter(|r| r.as_ref().unwrap().is_ok()).count();
    assert_eq!(successful_additions, 10);

    let final_members = get_team_members(&app, &owner.token, &team.id).await;
    assert_eq!(final_members.len(), 11); // オーナー + 10人
}
```

### 3. エラーパスのテスト

```rust
#[tokio::test]
async fn test_subscription_downgrade_with_excess_resources() {
    // Arrange: Pro tier組織を作成し、Free tier制限を超えるリソースを作成
    let (app, _, _) = setup_full_app().await;
    let owner = create_pro_user(&app, "owner@example.com").await;
    let org = create_organization(&app, &owner.token, "Test Org", "pro").await;
    
    // 11人のメンバーを追加（Free tierは10人まで）
    for i in 0..11 {
        add_organization_member(&app, &owner.token, &org.id, &format!("member{}@example.com", i)).await;
    }

    // Act: Free tierへのダウングレードを試行
    let downgrade_response = downgrade_organization_subscription(
        &app,
        &owner.token,
        &org.id,
        "free"
    ).await;

    // Assert: エラーレスポンスを検証
    assert_eq!(downgrade_response.status(), StatusCode::BAD_REQUEST);
    
    let error_body = parse_response_body::<ErrorResponse>(downgrade_response).await;
    assert!(error_body.error.contains("Cannot downgrade"));
    assert!(error_body.error.contains("member count"));
    assert!(error_body.error.contains("exceeds"));
}
```

## テストヘルパーとユーティリティ

### 1. テストデータビルダー

```rust
pub struct TestDataBuilder {
    app: TestApp,
}

impl TestDataBuilder {
    pub fn new(app: TestApp) -> Self {
        Self { app }
    }

    pub async fn create_user(mut self, email: &str) -> TestUserBuilder {
        let user = create_test_user(&self.app, email).await;
        TestUserBuilder {
            app: self.app,
            user,
            teams: vec![],
            tasks: vec![],
        }
    }
}

pub struct TestUserBuilder {
    app: TestApp,
    user: TestUser,
    teams: Vec<TestTeam>,
    tasks: Vec<TestTask>,
}

impl TestUserBuilder {
    pub async fn with_subscription(mut self, tier: SubscriptionTier) -> Self {
        update_user_subscription(&self.app, &self.user.id, tier).await;
        self.user.subscription_tier = tier;
        self
    }

    pub async fn with_team(mut self, name: &str) -> Self {
        let team = create_team(&self.app, &self.user.token, name).await;
        self.teams.push(team);
        self
    }

    pub async fn with_tasks(mut self, count: usize) -> Self {
        for i in 0..count {
            let task = create_task(&self.app, &self.user.token, &format!("Task {}", i)).await;
            self.tasks.push(task);
        }
        self
    }

    pub fn build(self) -> TestScenario {
        TestScenario {
            app: self.app,
            user: self.user,
            teams: self.teams,
            tasks: self.tasks,
        }
    }
}
```

### 2. カスタムアサーション

```rust
pub trait CustomAssertions {
    async fn assert_user_has_subscription(&self, user_id: &Uuid, expected_tier: SubscriptionTier);
    async fn assert_organization_member_count(&self, org_id: &Uuid, expected_count: usize);
    async fn assert_task_ownership(&self, task_id: &Uuid, expected_owner_id: &Uuid);
}

impl CustomAssertions for TestApp {
    async fn assert_user_has_subscription(&self, user_id: &Uuid, expected_tier: SubscriptionTier) {
        let user = get_user_by_id(&self.db, user_id).await
            .expect("User not found");
        assert_eq!(
            user.subscription_tier, expected_tier,
            "Expected user {} to have {} subscription, but found {}",
            user_id, expected_tier, user.subscription_tier
        );
    }

    async fn assert_organization_member_count(&self, org_id: &Uuid, expected_count: usize) {
        let count = count_organization_members(&self.db, org_id).await;
        assert_eq!(
            count, expected_count,
            "Expected organization {} to have {} members, but found {}",
            org_id, expected_count, count
        );
    }

    async fn assert_task_ownership(&self, task_id: &Uuid, expected_owner_id: &Uuid) {
        let task = get_task_by_id(&self.db, task_id).await
            .expect("Task not found");
        assert_eq!(
            task.user_id, *expected_owner_id,
            "Expected task {} to be owned by user {}, but found owner {}",
            task_id, expected_owner_id, task.user_id
        );
    }
}
```

### 3. テストデータのクリーンアップ

```rust
pub struct TestCleanup {
    db: DatabaseConnection,
    schema_name: String,
}

impl Drop for TestCleanup {
    fn drop(&mut self) {
        // テスト終了時に自動的にスキーマを削除
        let db = self.db.clone();
        let schema_name = self.schema_name.clone();
        
        tokio::task::spawn_blocking(move || {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(async {
                    let _ = db
                        .execute(Statement::from_string(
                            DatabaseBackend::Postgres,
                            format!("DROP SCHEMA IF EXISTS {} CASCADE", schema_name),
                        ))
                        .await;
                });
        });
    }
}
```

## パフォーマンステスト

### 1. 負荷テストの実装

```rust
#[tokio::test]
#[ignore] // 手動実行のみ
async fn test_api_performance_under_load() {
    let (app, _, _) = setup_full_app().await;
    let users = create_test_users(&app, 100).await;
    
    let start = Instant::now();
    
    // 1000リクエストを並行実行
    let handles: Vec<_> = (0..1000)
        .map(|i| {
            let app = app.clone();
            let user = users[i % users.len()].clone();
            tokio::spawn(async move {
                let start = Instant::now();
                let response = get_tasks(&app, &user.token).await;
                let duration = start.elapsed();
                (response.status(), duration)
            })
        })
        .collect();
    
    let results = futures::future::join_all(handles).await;
    let total_duration = start.elapsed();
    
    // パフォーマンス統計
    let successful = results.iter().filter(|r| r.as_ref().unwrap().0 == StatusCode::OK).count();
    let avg_duration = results
        .iter()
        .map(|r| r.as_ref().unwrap().1.as_millis())
        .sum::<u128>() / results.len() as u128;
    
    println!("Total requests: 1000");
    println!("Successful: {}", successful);
    println!("Total duration: {:?}", total_duration);
    println!("Average response time: {}ms", avg_duration);
    println!("Requests per second: {:.2}", 1000.0 / total_duration.as_secs_f64());
    
    // アサーション
    assert!(successful > 990); // 99%以上の成功率
    assert!(avg_duration < 100); // 平均100ms未満
}
```

## まとめ

包括的なテスト戦略は、信頼性の高いAPIの基盤です。ユニットテスト、統合テスト、E2Eテストを適切に組み合わせ、AAAパターンに従い、実際のシナリオをカバーすることで、高品質なソフトウェアを維持できます。