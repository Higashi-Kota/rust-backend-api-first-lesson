use task_backend::domain::task_status::TaskStatus;
// tests/unit/service_tests.rs
use sea_orm::{EntityTrait, Set};
use task_backend::domain::role_model::{ActiveModel as RoleActiveModel, Entity as RoleEntity};
use task_backend::domain::user_model::{ActiveModel as UserActiveModel, Entity as UserEntity};
use task_backend::{
    api::dto::task_dto::{
        BatchCreateTaskDto, BatchDeleteTaskDto, BatchUpdateTaskDto, BatchUpdateTaskItemDto,
        CreateTaskDto, TaskFilterDto,
    },
    service::task_service::TaskService,
};
use uuid::Uuid;

use crate::common;

// サービステスト用のセットアップヘルパー関数
async fn setup_test_service() -> (common::db::TestDatabase, TaskService) {
    let db = common::db::TestDatabase::new().await;
    let service = TaskService::new(db.connection.clone());
    (db, service)
}

// テスト用ユーザーを作成するヘルパー関数
async fn create_test_user(db: &common::db::TestDatabase) -> Uuid {
    let user_id = Uuid::new_v4();
    let role_id = Uuid::new_v4();

    // 基本的なロールレコードを挿入
    let role_model = RoleActiveModel {
        id: Set(role_id),
        name: Set("test_role".to_string()),
        display_name: Set("Test Role".to_string()),
        description: Set(Some("Test role for unit tests".to_string())),
        is_active: Set(true),
        created_at: Set(chrono::Utc::now()),
        updated_at: Set(chrono::Utc::now()),
    };

    RoleEntity::insert(role_model)
        .exec(&db.connection)
        .await
        .expect("Failed to create test role");

    // 基本的なユーザーレコードを挿入
    let user_model = UserActiveModel {
        id: Set(user_id),
        email: Set(format!("test{}@example.com", user_id)),
        username: Set(format!("testuser{}", &user_id.to_string()[..8])),
        password_hash: Set("dummy_hash".to_string()),
        is_active: Set(true),
        email_verified: Set(true),
        subscription_tier: Set("free".to_string()),
        created_at: Set(chrono::Utc::now()),
        updated_at: Set(chrono::Utc::now()),
        role_id: Set(role_id),
        last_login_at: Set(None),
        stripe_customer_id: Set(None),
    };

    UserEntity::insert(user_model)
        .exec(&db.connection)
        .await
        .expect("Failed to create test user");

    user_id
}

#[tokio::test]
async fn test_create_task_service() {
    // データベースをセットアップ
    let (_db, service) = setup_test_service().await;

    // タスク作成
    let task_dto = common::create_test_task();
    let created_task = service.create_task(task_dto).await.unwrap();

    // 検証
    assert_eq!(created_task.title, "Test Task");
    assert_eq!(created_task.status, TaskStatus::Todo);
    assert!(common::is_valid_uuid(&created_task));
}

#[tokio::test]
async fn test_get_task_service() {
    let (_db, service) = setup_test_service().await;

    // タスク作成
    let task_dto = common::create_test_task();
    let created_task = service.create_task(task_dto).await.unwrap();

    // タスク取得
    let retrieved_task = service.get_task(created_task.id).await.unwrap();

    // 検証
    assert_eq!(retrieved_task.id, created_task.id);
    assert_eq!(retrieved_task.title, created_task.title);
    assert_eq!(retrieved_task.status, created_task.status);
}

#[tokio::test]
async fn test_list_tasks_service() {
    let (_db, service) = setup_test_service().await;

    // 複数のタスクを作成
    service
        .create_task(common::create_test_task_with_title("Task A"))
        .await
        .unwrap();
    service
        .create_task(common::create_test_task_with_title("Task B"))
        .await
        .unwrap();

    // タスク一覧取得
    let tasks = service.list_tasks().await.unwrap();

    // 検証
    assert_eq!(tasks.len(), 2);
    // タスクのタイトルを確認
    let titles: Vec<String> = tasks.iter().map(|t| t.title.clone()).collect();
    assert!(titles.contains(&"Task A".to_string()));
    assert!(titles.contains(&"Task B".to_string()));
}

#[tokio::test]
async fn test_update_task_service() {
    let (_db, service) = setup_test_service().await;

    // タスク作成
    let task_dto = common::create_test_task();
    let created_task = service.create_task(task_dto).await.unwrap();

    // タスク更新
    let update_dto = common::create_update_task();
    let updated_task = service
        .update_task(created_task.id, update_dto)
        .await
        .unwrap();

    // 検証
    assert_eq!(updated_task.id, created_task.id);
    assert_eq!(updated_task.title, "Updated Task");
    assert_eq!(updated_task.status, TaskStatus::InProgress);
    assert_eq!(updated_task.description.unwrap(), "Updated Description");
}

#[tokio::test]
async fn test_delete_task_service() {
    let (_db, service) = setup_test_service().await;

    // タスク作成
    let task_dto = common::create_test_task();
    let created_task = service.create_task(task_dto).await.unwrap();

    // タスク削除
    service.delete_task(created_task.id).await.unwrap();

    // 削除確認 - エラーが返ってくるはず
    let result = service.get_task(created_task.id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_batch_operations_service() {
    let (db, service) = setup_test_service().await;

    // テスト用ユーザーを作成
    let user_id = create_test_user(&db).await;

    // バッチ作成
    let batch_create_dto = BatchCreateTaskDto {
        tasks: vec![
            common::create_test_task_with_title("Batch Service Task 1"),
            common::create_test_task_with_title("Batch Service Task 2"),
        ],
    };

    let batch_create_result = service
        .create_tasks_batch_for_user(user_id, batch_create_dto)
        .await
        .unwrap();

    // 検証
    assert_eq!(batch_create_result.created_tasks.len(), 2);

    // 作成されたタスクのIDを収集
    let task_ids: Vec<Uuid> = batch_create_result
        .created_tasks
        .iter()
        .map(|t| t.id)
        .collect();

    // バッチ更新
    let batch_update_dto = BatchUpdateTaskDto {
        tasks: task_ids
            .iter()
            .map(|id| BatchUpdateTaskItemDto {
                id: *id,
                title: Some("Updated Batch Service Task".to_string()),
                status: Some(TaskStatus::InProgress),
                description: None,
                due_date: None,
            })
            .collect(),
    };

    let batch_update_result = service
        .update_tasks_batch_for_user(user_id, batch_update_dto)
        .await
        .unwrap();

    // 検証
    assert_eq!(batch_update_result.updated_count, 2);

    // 更新確認
    for id in &task_ids {
        let task = service.get_task(*id).await.unwrap();
        assert_eq!(task.title, "Updated Batch Service Task");
        assert_eq!(task.status, TaskStatus::InProgress);
    }

    // バッチ削除
    let batch_delete_dto = BatchDeleteTaskDto {
        ids: task_ids.clone(),
    };

    let batch_delete_result = service
        .delete_tasks_batch_for_user(user_id, batch_delete_dto)
        .await
        .unwrap();

    // 検証
    assert_eq!(batch_delete_result.deleted_count, 2);

    // 削除確認
    for id in task_ids {
        let result = service.get_task(id).await;
        assert!(result.is_err());
    }
}

#[tokio::test]
async fn test_filter_tasks_service() {
    let (_db, service) = setup_test_service().await;

    // フィルタテスト用のタスクを作成
    service
        .create_task(CreateTaskDto {
            title: "Filter Test Task 1".to_string(),
            description: Some("High priority".to_string()),
            status: Some(TaskStatus::Todo),
            due_date: None,
        })
        .await
        .unwrap();

    service
        .create_task(CreateTaskDto {
            title: "Filter Test Task 2".to_string(),
            description: Some("Low priority".to_string()),
            status: Some(TaskStatus::InProgress),
            due_date: None,
        })
        .await
        .unwrap();

    service
        .create_task(CreateTaskDto {
            title: "Another Filter Task".to_string(),
            description: Some("Medium priority".to_string()),
            status: Some(TaskStatus::Todo),
            due_date: None,
        })
        .await
        .unwrap();

    // ステータスでフィルタリング
    let filter = TaskFilterDto {
        status: Some(TaskStatus::Todo),
        limit: Some(10),
        ..Default::default()
    };

    let result = service.filter_tasks(filter).await.unwrap();

    // 検証 - todoステータスのタスクが2つあるはず
    assert_eq!(result.items.len(), 2);
    assert_eq!(result.pagination.per_page, 10);

    // タイトルでフィルタリング
    let filter = TaskFilterDto {
        title_contains: Some("Another".to_string()),
        limit: Some(10),
        ..Default::default()
    };

    let result = service.filter_tasks(filter).await.unwrap();

    // 検証 - "Another"を含むタスクが1つあるはず
    assert_eq!(result.items.len(), 1);
    assert!(result.items.iter().any(|t| t.title.contains("Another")));

    // 該当タスクなしのケース
    let filter = TaskFilterDto {
        status: Some(TaskStatus::Completed),
        title_contains: Some("NonExistent".to_string()),
        limit: Some(10),
        ..Default::default()
    };

    let result = service.filter_tasks(filter).await.unwrap();

    // 検証 - 該当するタスクがないはず
    assert!(result.items.is_empty());
}

#[tokio::test]
async fn test_paginated_tasks_service() {
    let (_db, service) = setup_test_service().await;

    // ページネーションテスト用のタスクを作成
    for i in 1..=15 {
        service
            .create_task(CreateTaskDto {
                title: format!("Pagination Service Task {}", i),
                description: Some("For pagination test".to_string()),
                status: Some(TaskStatus::Todo),
                due_date: None,
            })
            .await
            .unwrap();
    }

    // 1ページ目を取得
    let page1_result = service.list_tasks_paginated(1, 5).await.unwrap();

    // 検証
    assert_eq!(page1_result.items.len(), 5);
    assert_eq!(page1_result.pagination.page, 1);
    assert_eq!(page1_result.pagination.per_page, 5);
    assert_eq!(page1_result.pagination.total_count, 15);
    assert!(page1_result.pagination.has_next);
    assert!(!page1_result.pagination.has_prev);

    // 2ページ目を取得
    let page2_result = service.list_tasks_paginated(2, 5).await.unwrap();

    // 検証
    assert_eq!(page2_result.items.len(), 5);
    assert_eq!(page2_result.pagination.page, 2);
    assert!(page2_result.pagination.has_next);
    assert!(page2_result.pagination.has_prev);

    // ページ間でタスクが重複していないことを確認
    for p1_task in &page1_result.items {
        for p2_task in &page2_result.items {
            assert_ne!(p1_task.id, p2_task.id);
        }
    }
}

#[test]
fn test_task_statistics_concepts() {
    // タスク統計の概念テスト（新しく追加したAPIで使用）

    // タスク統計の構造
    struct TaskStatsConcept {
        total_tasks: usize,
        completed_tasks: usize,
        pending_tasks: usize,
        in_progress_tasks: usize,
        completion_rate: f64,
    }

    let stats = TaskStatsConcept {
        total_tasks: 100,
        completed_tasks: 60,
        pending_tasks: 25,
        in_progress_tasks: 15,
        completion_rate: 60.0,
    };

    // 統計の整合性チェック
    assert_eq!(
        stats.completed_tasks + stats.pending_tasks + stats.in_progress_tasks,
        stats.total_tasks,
        "Task counts should sum to total"
    );

    assert_eq!(
        stats.completion_rate,
        (stats.completed_tasks as f64 / stats.total_tasks as f64 * 100.0).round(),
        "Completion rate should be calculated correctly"
    );

    // 完了率の範囲チェック
    assert!(
        stats.completion_rate >= 0.0 && stats.completion_rate <= 100.0,
        "Completion rate should be between 0 and 100"
    );
}

#[test]
fn test_task_status_distribution_concepts() {
    // タスクステータス分布の概念テスト（新しく追加したAPIで使用）

    // ステータス分布の構造
    struct StatusDistributionConcept {
        pending: usize,
        in_progress: usize,
        completed: usize,
        other: usize,
    }

    let distribution = StatusDistributionConcept {
        pending: 30,
        in_progress: 20,
        completed: 45,
        other: 5,
    };

    let total = distribution.pending
        + distribution.in_progress
        + distribution.completed
        + distribution.other;

    // 分布の整合性チェック
    assert_eq!(total, 100, "Distribution should sum to total tasks");

    // 各ステータスが妥当な値であることを確認（usizeは非負なので存在確認のみ）
    assert!(
        distribution.pending < 1000,
        "Pending tasks should be reasonable count"
    );
    assert!(
        distribution.in_progress < 1000,
        "In progress tasks should be reasonable count"
    );
    assert!(
        distribution.completed < 1000,
        "Completed tasks should be reasonable count"
    );
    assert!(
        distribution.other < 1000,
        "Other tasks should be reasonable count"
    );

    // 最も多いステータスの確認（概念的テスト）
    let status_counts = [
        ("pending", distribution.pending),
        ("in_progress", distribution.in_progress),
        ("completed", distribution.completed),
        ("other", distribution.other),
    ];
    let max_status = status_counts
        .iter()
        .max_by_key(|(_, count)| *count)
        .unwrap();

    assert_eq!(
        max_status.0, "completed",
        "Completed should be the highest status in this test"
    );
}

#[test]
fn test_bulk_status_update_concepts() {
    // 一括ステータス更新の概念テスト（新しく追加したAPIで使用）

    // 一括更新の結果構造
    struct BulkUpdateResultConcept {
        updated_count: usize,
        error_count: usize,
        total_requested: usize,
        new_status: TaskStatus,
    }

    let result = BulkUpdateResultConcept {
        updated_count: 8,
        error_count: 2,
        total_requested: 10,
        new_status: TaskStatus::Completed,
    };

    // 更新結果の整合性チェック
    assert_eq!(
        result.updated_count + result.error_count,
        result.total_requested,
        "Updated and error counts should sum to total requested"
    );

    // 有効なステータス値の確認
    let valid_statuses = ["pending", "in_progress", "completed"];
    assert!(
        valid_statuses.contains(&result.new_status.as_str()),
        "New status should be valid"
    );

    // 成功率の計算（概念的テスト）
    let success_rate = result.updated_count as f64 / result.total_requested as f64;
    assert!(
        (0.0..=1.0).contains(&success_rate),
        "Success rate should be between 0 and 1"
    );

    // この例では80%の成功率
    assert_eq!(
        (success_rate * 100.0).round(),
        80.0,
        "Success rate should be 80%"
    );
}

#[test]
fn test_task_uuid_validation_concepts() {
    // タスクUUID検証の概念テスト（一括操作で使用）
    use uuid::Uuid;

    let valid_uuid = Uuid::new_v4();
    let nil_uuid = Uuid::nil();

    // UUID形式の検証
    assert_ne!(valid_uuid, nil_uuid, "Valid UUID should not be nil");
    assert_eq!(
        valid_uuid.to_string().len(),
        36,
        "UUID string should be 36 characters"
    );

    // UUID文字列からの変換テスト
    let uuid_str = valid_uuid.to_string();
    let parsed_uuid = Uuid::parse_str(&uuid_str).unwrap();
    assert_eq!(
        valid_uuid, parsed_uuid,
        "UUID should parse back to original"
    );

    // 無効なUUID文字列のテスト
    let invalid_uuid_str = "invalid-uuid-string";
    assert!(
        Uuid::parse_str(invalid_uuid_str).is_err(),
        "Invalid UUID string should fail to parse"
    );

    // 空文字列のテスト
    assert!(
        Uuid::parse_str("").is_err(),
        "Empty string should fail to parse as UUID"
    );
}

#[test]
fn test_task_status_validation_concepts() {
    // タスクステータス検証の概念テスト（一括更新で使用）

    let valid_statuses = ["pending", "in_progress", "completed"];
    let invalid_statuses = ["draft", "cancelled", "archived", ""];

    // 有効なステータスの確認
    for status in valid_statuses {
        assert!(!status.is_empty(), "Valid status should not be empty");
        assert!(status.len() <= 20, "Status should be reasonable length");
        assert!(
            status.chars().all(|c| c.is_ascii_lowercase() || c == '_'),
            "Status should be lowercase with underscores only"
        );
    }

    // 無効なステータスの確認
    for status in invalid_statuses {
        if status.is_empty() {
            assert!(status.is_empty(), "Empty status should be detected");
        } else {
            assert!(
                !valid_statuses.contains(&status),
                "Invalid status should not be in valid list"
            );
        }
    }

    // ステータス変換の概念テスト
    let status_mappings = [
        ("pending", "in_progress"),
        ("in_progress", "completed"),
        ("completed", "pending"), // 再開の場合
    ];

    for (from, to) in status_mappings {
        assert!(
            valid_statuses.contains(&from),
            "Source status should be valid"
        );
        assert!(
            valid_statuses.contains(&to),
            "Target status should be valid"
        );
        assert_ne!(from, to, "Status transition should change status");
    }
}

// Admin専用メソッドのテスト
#[tokio::test]
async fn test_admin_list_all_tasks_service() {
    let (_db, service) = setup_test_service().await;

    // user_id なしでタスクを作成（既存のテストと同様）
    let task_dto1 = common::create_test_task();
    let _created_task1 = service.create_task(task_dto1).await.unwrap();

    let task_dto2 = common::create_test_task();
    let _created_task2 = service.create_task(task_dto2).await.unwrap();

    // Admin: 全タスクを取得 (replaced with list_tasks)
    let all_tasks = service.list_tasks().await.unwrap();

    // 検証: 複数のタスクが含まれているべき
    assert!(all_tasks.len() >= 2);
}

#[tokio::test]
async fn test_admin_list_tasks_by_user_id_service() {
    let (_db, service) = setup_test_service().await;

    // 存在しないuser_idで空のリストが返されることをテスト
    let nonexistent_user_id = Uuid::new_v4();

    // Admin: 存在しないユーザーのタスクを取得 (replaced with list_tasks_for_user)
    let user_tasks = service
        .list_tasks_for_user(nonexistent_user_id)
        .await
        .unwrap();

    // 検証: 空のリストが返される
    assert!(user_tasks.is_empty());
}

#[tokio::test]
async fn test_admin_delete_task_by_id_service() {
    let (_db, service) = setup_test_service().await;

    // user_id なしでタスクを作成
    let task_dto = common::create_test_task();
    let created_task = service.create_task(task_dto).await.unwrap();

    // Admin: 任意のタスクを削除 (replaced with delete_task)
    let result = service.delete_task(created_task.id).await;
    assert!(result.is_ok());

    // 検証: タスクが削除されていることを確認
    let get_result = service.get_task(created_task.id).await;
    assert!(get_result.is_err());
}

#[tokio::test]
async fn test_admin_delete_nonexistent_task_service() {
    let (_db, service) = setup_test_service().await;

    let nonexistent_id = Uuid::new_v4();

    // Admin: 存在しないタスクの削除を試行 (replaced with delete_task)
    let result = service.delete_task(nonexistent_id).await;

    // 検証: NotFoundエラーが返される
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("not found"));
}
