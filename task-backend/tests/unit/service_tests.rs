// tests/unit/service_tests.rs
use task_backend::{
    api::dto::task_dto::{
        BatchCreateTaskDto, BatchDeleteTaskDto, BatchUpdateTaskDto, BatchUpdateTaskItemDto,
        CreateTaskDto, TaskFilterDto, UpdateTaskDto,
    },
    service::task_service::TaskService,
};
use uuid::Uuid;

use crate::common;

#[tokio::test]
async fn test_create_task_service() {
    // データベースをセットアップ
    let db = common::db::TestDatabase::new().await;
    let service = TaskService::new(db.connection);

    // タスク作成
    let task_dto = common::create_test_task();
    let created_task = service.create_task(task_dto).await.unwrap();

    // 検証
    assert_eq!(created_task.title, "Test Task");
    assert_eq!(created_task.status, "todo");
    assert!(common::is_valid_uuid(&created_task));
}

#[tokio::test]
async fn test_get_task_service() {
    let db = common::db::TestDatabase::new().await;
    let service = TaskService::new(db.connection);

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
    let db = common::db::TestDatabase::new().await;
    let service = TaskService::new(db.connection);

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
    assert!(tasks.len() >= 2);
}

#[tokio::test]
async fn test_update_task_service() {
    let db = common::db::TestDatabase::new().await;
    let service = TaskService::new(db.connection);

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
    assert_eq!(updated_task.status, "in_progress");
    assert_eq!(updated_task.description.unwrap(), "Updated Description");
}

#[tokio::test]
async fn test_delete_task_service() {
    let db = common::db::TestDatabase::new().await;
    let service = TaskService::new(db.connection);

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
    let db = common::db::TestDatabase::new().await;
    let service = TaskService::new(db.connection);

    // バッチ作成
    let batch_create_dto = BatchCreateTaskDto {
        tasks: vec![
            common::create_test_task_with_title("Batch Service Task 1"),
            common::create_test_task_with_title("Batch Service Task 2"),
        ],
    };

    let batch_create_result = service.create_tasks_batch(batch_create_dto).await.unwrap();

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
                status: Some("in_progress".to_string()),
                description: None,
                due_date: None,
            })
            .collect(),
    };

    let batch_update_result = service.update_tasks_batch(batch_update_dto).await.unwrap();

    // 検証
    assert_eq!(batch_update_result.updated_count, 2);

    // 更新確認
    for id in &task_ids {
        let task = service.get_task(*id).await.unwrap();
        assert_eq!(task.title, "Updated Batch Service Task");
        assert_eq!(task.status, "in_progress");
    }

    // バッチ削除
    let batch_delete_dto = BatchDeleteTaskDto {
        ids: task_ids.clone(),
    };

    let batch_delete_result = service.delete_tasks_batch(batch_delete_dto).await.unwrap();

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
    let db = common::db::TestDatabase::new().await;
    let service = TaskService::new(db.connection);

    // フィルタテスト用のタスクを作成
    service
        .create_task(CreateTaskDto {
            title: "Filter Test Task 1".to_string(),
            description: Some("High priority".to_string()),
            status: Some("todo".to_string()),
            due_date: None,
        })
        .await
        .unwrap();

    service
        .create_task(CreateTaskDto {
            title: "Filter Test Task 2".to_string(),
            description: Some("Low priority".to_string()),
            status: Some("in_progress".to_string()),
            due_date: None,
        })
        .await
        .unwrap();

    service
        .create_task(CreateTaskDto {
            title: "Another Filter Task".to_string(),
            description: Some("Medium priority".to_string()),
            status: Some("todo".to_string()),
            due_date: None,
        })
        .await
        .unwrap();

    // ステータスでフィルタリング
    let filter = TaskFilterDto {
        status: Some("todo".to_string()),
        limit: Some(10),
        ..Default::default()
    };

    let result = service.filter_tasks(filter).await.unwrap();

    // 検証 - todoステータスのタスクが少なくとも2つあるはず
    assert!(result.tasks.len() >= 2);
    assert_eq!(result.pagination.page_size, 10);

    // タイトルでフィルタリング
    let filter = TaskFilterDto {
        title_contains: Some("Another".to_string()),
        limit: Some(10),
        ..Default::default()
    };

    let result = service.filter_tasks(filter).await.unwrap();

    // 検証 - "Another"を含むタスクが少なくとも1つあるはず
    assert!(!result.tasks.is_empty());
    assert!(result.tasks.iter().any(|t| t.title.contains("Another")));

    // 該当タスクなしのケース
    let filter = TaskFilterDto {
        status: Some("completed".to_string()),
        title_contains: Some("NonExistent".to_string()),
        limit: Some(10),
        ..Default::default()
    };

    let result = service.filter_tasks(filter).await.unwrap();

    // 検証 - 該当するタスクがないはず
    assert!(result.tasks.is_empty());
}

#[tokio::test]
async fn test_paginated_tasks_service() {
    let db = common::db::TestDatabase::new().await;
    let service = TaskService::new(db.connection);

    // ページネーションテスト用のタスクを作成
    for i in 1..=15 {
        service
            .create_task(CreateTaskDto {
                title: format!("Pagination Service Task {}", i),
                description: Some("For pagination test".to_string()),
                status: Some("todo".to_string()),
                due_date: None,
            })
            .await
            .unwrap();
    }

    // 1ページ目を取得
    let page1_result = service.list_tasks_paginated(1, 5).await.unwrap();

    // 検証
    assert_eq!(page1_result.tasks.len(), 5);
    assert_eq!(page1_result.pagination.current_page, 1);
    assert_eq!(page1_result.pagination.page_size, 5);
    assert!(page1_result.pagination.total_items >= 15);
    assert!(page1_result.pagination.has_next_page);
    assert!(!page1_result.pagination.has_previous_page);

    // 2ページ目を取得
    let page2_result = service.list_tasks_paginated(2, 5).await.unwrap();

    // 検証
    assert_eq!(page2_result.tasks.len(), 5);
    assert_eq!(page2_result.pagination.current_page, 2);
    assert!(page2_result.pagination.has_next_page);
    assert!(page2_result.pagination.has_previous_page);

    // ページ間でタスクが重複していないことを確認
    for p1_task in &page1_result.tasks {
        for p2_task in &page2_result.tasks {
            assert_ne!(p1_task.id, p2_task.id);
        }
    }
}
