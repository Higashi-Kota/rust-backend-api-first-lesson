use task_backend::core::task_status::TaskStatus;
// tests/unit/service_tests.rs
use sea_orm::{EntityTrait, Set};
use task_backend::domain::role_model::{ActiveModel as RoleActiveModel, Entity as RoleEntity};
use task_backend::domain::user_model::{ActiveModel as UserActiveModel, Entity as UserEntity};
use task_backend::features::task::{
    dto::{
        BatchCreateTaskDto, BatchDeleteTaskDto, BatchUpdateTaskDto, BatchUpdateTaskItemDto,
        CreateTaskDto, TaskFilterDto, UpdateTaskDto,
    },
    service::TaskService,
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
            priority: None,
            due_date: None,
        })
        .await
        .unwrap();

    service
        .create_task(CreateTaskDto {
            title: "Filter Test Task 2".to_string(),
            description: Some("Low priority".to_string()),
            status: Some(TaskStatus::InProgress),
            priority: None,
            due_date: None,
        })
        .await
        .unwrap();

    service
        .create_task(CreateTaskDto {
            title: "Another Filter Task".to_string(),
            description: Some("Medium priority".to_string()),
            status: Some(TaskStatus::Todo),
            priority: None,
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
                priority: None,
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

#[tokio::test]
async fn test_task_statistics_calculation() {
    // Arrange: タスク統計の計算をテスト
    let (_db, service) = setup_test_service().await;

    // 異なるステータスのタスクを作成
    let mut task_ids = Vec::new();

    // 完了済みタスクを60個作成
    for i in 0..60 {
        let task = service
            .create_task(CreateTaskDto {
                title: format!("Completed Task {}", i),
                description: None,
                status: Some(TaskStatus::Completed),
                priority: None,
                due_date: None,
            })
            .await
            .unwrap();
        task_ids.push(task.id);
    }

    // 保留中タスクを25個作成
    for i in 0..25 {
        let task = service
            .create_task(CreateTaskDto {
                title: format!("Pending Task {}", i),
                description: None,
                status: Some(TaskStatus::Todo),
                priority: None,
                due_date: None,
            })
            .await
            .unwrap();
        task_ids.push(task.id);
    }

    // 進行中タスクを15個作成
    for i in 0..15 {
        let task = service
            .create_task(CreateTaskDto {
                title: format!("In Progress Task {}", i),
                description: None,
                status: Some(TaskStatus::InProgress),
                priority: None,
                due_date: None,
            })
            .await
            .unwrap();
        task_ids.push(task.id);
    }

    // Act: タスク一覧を取得して統計を計算
    let all_tasks = service.list_tasks().await.unwrap();

    // Assert: 統計の検証
    assert_eq!(all_tasks.len(), 100, "Total tasks should be 100");

    let completed_count = all_tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Completed)
        .count();
    let todo_count = all_tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Todo)
        .count();
    let in_progress_count = all_tasks
        .iter()
        .filter(|t| t.status == TaskStatus::InProgress)
        .count();

    assert_eq!(completed_count, 60, "Completed tasks should be 60");
    assert_eq!(todo_count, 25, "Todo tasks should be 25");
    assert_eq!(in_progress_count, 15, "In progress tasks should be 15");

    // 完了率の計算
    let completion_rate = (completed_count as f64 / all_tasks.len() as f64 * 100.0).round();
    assert_eq!(completion_rate, 60.0, "Completion rate should be 60%");
}

#[tokio::test]
async fn test_task_status_distribution() {
    // Arrange: タスクステータス分布をテスト
    let (_db, service) = setup_test_service().await;

    // 異なるステータスのタスクを作成
    let status_distribution = vec![
        (TaskStatus::Todo, 30),
        (TaskStatus::InProgress, 20),
        (TaskStatus::Completed, 45),
    ];

    for (status, count) in &status_distribution {
        for i in 0..*count {
            service
                .create_task(CreateTaskDto {
                    title: format!("{:?} Task {}", status, i),
                    description: None,
                    status: Some(*status),
                    priority: None,
                    due_date: None,
                })
                .await
                .unwrap();
        }
    }

    // Act: タスク一覧を取得して分布を計算
    let all_tasks = service.list_tasks().await.unwrap();

    // Assert: ステータス分布の検証
    let mut status_counts = std::collections::HashMap::new();
    for task in &all_tasks {
        *status_counts.entry(task.status).or_insert(0) += 1;
    }

    assert_eq!(
        status_counts.get(&TaskStatus::Todo).copied().unwrap_or(0),
        30
    );
    assert_eq!(
        status_counts
            .get(&TaskStatus::InProgress)
            .copied()
            .unwrap_or(0),
        20
    );
    assert_eq!(
        status_counts
            .get(&TaskStatus::Completed)
            .copied()
            .unwrap_or(0),
        45
    );

    // 合計の検証
    let total: usize = status_counts.values().sum();
    assert_eq!(total, 95, "Total tasks should match created count");

    // 最も多いステータスの確認
    let max_status = status_counts
        .iter()
        .max_by_key(|(_, count)| *count)
        .map(|(status, _)| status);

    assert_eq!(
        max_status,
        Some(&TaskStatus::Completed),
        "Completed should be the most common status"
    );
}

#[tokio::test]
async fn test_bulk_status_update() {
    // Arrange: 一括ステータス更新をテスト
    let (db, service) = setup_test_service().await;
    let user_id = create_test_user(&db).await;

    // 10個のタスクを作成
    let mut task_ids = Vec::new();
    for i in 0..10 {
        let task = service
            .create_task_for_user(
                user_id,
                CreateTaskDto {
                    title: format!("Bulk Update Task {}", i),
                    description: None,
                    status: Some(TaskStatus::Todo),
                    priority: None,
                    due_date: None,
                },
            )
            .await
            .unwrap();
        task_ids.push(task.id);
    }

    // Act: 一括更新を実行（8個成功、2個失敗をシミュレート）
    let update_items: Vec<BatchUpdateTaskItemDto> = task_ids
        .iter()
        .take(8) // 最初の8個だけを更新対象にする
        .map(|id| BatchUpdateTaskItemDto {
            id: *id,
            title: None,
            status: Some(TaskStatus::Completed),
            description: None,
            due_date: None,
        })
        .collect();

    let batch_update_dto = BatchUpdateTaskDto {
        tasks: update_items,
    };
    let update_result = service
        .update_tasks_batch_for_user(user_id, batch_update_dto)
        .await
        .unwrap();

    // Assert: 更新結果の検証
    assert_eq!(update_result.updated_count, 8, "Should update 8 tasks");

    // 更新されたタスクの状態を確認
    let mut completed_count = 0;
    let mut todo_count = 0;

    for id in &task_ids {
        let task = service.get_task(*id).await.unwrap();
        match task.status {
            TaskStatus::Completed => completed_count += 1,
            TaskStatus::Todo => todo_count += 1,
            _ => {}
        }
    }

    assert_eq!(completed_count, 8, "8 tasks should be completed");
    assert_eq!(todo_count, 2, "2 tasks should remain in todo status");

    // 成功率の計算と検証
    let success_rate = update_result.updated_count as f64 / 10.0;
    assert_eq!(
        (success_rate * 100.0).round(),
        80.0,
        "Success rate should be 80%"
    );
}

#[tokio::test]
async fn test_task_operations_with_invalid_uuid() {
    // Arrange: 無効なUUIDでのタスク操作をテスト
    let (_db, service) = setup_test_service().await;

    // 有効なタスクを作成
    let valid_task = service
        .create_task(CreateTaskDto {
            title: "Valid Task".to_string(),
            description: None,
            status: Some(TaskStatus::Todo),
            priority: None,
            due_date: None,
        })
        .await
        .unwrap();

    // Act & Assert: 存在しないUUIDでのタスク取得
    let non_existent_uuid = Uuid::new_v4();
    let get_result = service.get_task(non_existent_uuid).await;
    assert!(get_result.is_err(), "Getting non-existent task should fail");

    // Act & Assert: 存在しないUUIDでのタスク更新
    let update_result = service
        .update_task(non_existent_uuid, common::create_update_task())
        .await;
    assert!(
        update_result.is_err(),
        "Updating non-existent task should fail"
    );

    // Act & Assert: 存在しないUUIDでのタスク削除
    let delete_result = service.delete_task(non_existent_uuid).await;
    assert!(
        delete_result.is_err(),
        "Deleting non-existent task should fail"
    );

    // Act & Assert: nilのUUIDでの操作もテスト
    let nil_uuid = Uuid::nil();
    let nil_get_result = service.get_task(nil_uuid).await;
    assert!(
        nil_get_result.is_err(),
        "Getting task with nil UUID should fail"
    );

    // 有効なタスクはまだ存在することを確認
    let valid_get_result = service.get_task(valid_task.id).await;
    assert!(valid_get_result.is_ok(), "Valid task should still exist");
}

#[tokio::test]
async fn test_task_status_transitions() {
    // Arrange: タスクステータスの遷移をテスト
    let (_db, service) = setup_test_service().await;

    // Todoステータスでタスクを作成
    let task = service
        .create_task(CreateTaskDto {
            title: "Status Transition Task".to_string(),
            description: None,
            status: Some(TaskStatus::Todo),
            priority: None,
            due_date: None,
        })
        .await
        .unwrap();

    // Act & Assert: Todo -> InProgress への遷移
    let update_to_in_progress = service
        .update_task(
            task.id,
            UpdateTaskDto {
                title: None,
                description: None,
                status: Some(TaskStatus::InProgress),
                priority: None,
                due_date: None,
            },
        )
        .await
        .unwrap();

    assert_eq!(
        update_to_in_progress.status,
        TaskStatus::InProgress,
        "Task should be in progress"
    );

    // Act & Assert: InProgress -> Completed への遷移
    let update_to_completed = service
        .update_task(
            task.id,
            UpdateTaskDto {
                title: None,
                description: None,
                status: Some(TaskStatus::Completed),
                priority: None,
                due_date: None,
            },
        )
        .await
        .unwrap();

    assert_eq!(
        update_to_completed.status,
        TaskStatus::Completed,
        "Task should be completed"
    );

    // Act & Assert: Completed -> Todo への遷移（再開）
    let update_to_todo = service
        .update_task(
            task.id,
            UpdateTaskDto {
                title: None,
                description: None,
                status: Some(TaskStatus::Todo),
                priority: None,
                due_date: None,
            },
        )
        .await
        .unwrap();

    assert_eq!(
        update_to_todo.status,
        TaskStatus::Todo,
        "Task should be back to todo"
    );

    // タスクの履歴を確認（更新日時が変わっていることを確認）
    assert!(
        update_to_todo.updated_at > task.updated_at,
        "Updated timestamp should be newer"
    );
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
