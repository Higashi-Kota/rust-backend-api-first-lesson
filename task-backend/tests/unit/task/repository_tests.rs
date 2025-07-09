use task_backend::core::task_status::TaskStatus;
// tests/unit/repository_tests.rs
use chrono::Utc;
use task_backend::features::task::{
    dto::{BatchUpdateTaskItemDto, CreateTaskDto, TaskFilterDto, UpdateTaskDto},
    repository::task_repository::TaskRepository,
};

use crate::common;

// リポジトリテスト用のセットアップヘルパー関数
async fn setup_test_repository() -> (common::db::TestDatabase, TaskRepository) {
    let db = common::db::TestDatabase::new().await;
    let repo = TaskRepository::new(db.connection.clone());
    (db, repo)
}

#[tokio::test]
async fn test_create_task() {
    // セットアップ
    let (_db, repo) = setup_test_repository().await;

    // テスト用タスクを作成
    let task_dto = common::create_test_task();
    let created_task = repo.create(task_dto).await.unwrap();

    // 作成されたタスクを検証
    assert_eq!(created_task.title, "Test Task");
    assert_eq!(created_task.status, TaskStatus::Todo.to_string());
    assert!(created_task.description.is_some());
    assert_eq!(created_task.description.unwrap(), "Test Description");
}

#[tokio::test]
async fn test_find_by_id() {
    let (_db, repo) = setup_test_repository().await;

    // タスクを作成してIDを取得
    let task_dto = common::create_test_task();
    let created_task = repo.create(task_dto).await.unwrap();
    let task_id = created_task.id;

    // IDでタスクを検索
    let found_task = repo.find_by_id(task_id).await.unwrap().unwrap();

    // 見つかったタスクを検証
    assert_eq!(found_task.id, task_id);
    assert_eq!(found_task.title, "Test Task");
}

#[tokio::test]
async fn test_find_all() {
    let (_db, repo) = setup_test_repository().await;

    // 複数のタスクを作成
    let task1 = common::create_test_task_with_title("Task 1");
    let task2 = common::create_test_task_with_title("Task 2");

    repo.create(task1).await.unwrap();
    repo.create(task2).await.unwrap();

    // すべてのタスクを取得
    let tasks = repo.find_all().await.unwrap();

    // 正確に2つのタスクがあることを確認
    assert_eq!(tasks.len(), 2);

    // タスクのタイトルを確認
    let titles: Vec<String> = tasks.iter().map(|t| t.title.clone()).collect();
    assert!(titles.contains(&"Task 1".to_string()));
    assert!(titles.contains(&"Task 2".to_string()));
}

#[tokio::test]
async fn test_update_task() {
    let (_db, repo) = setup_test_repository().await;

    // タスクを作成
    let task_dto = common::create_test_task();
    let created_task = repo.create(task_dto).await.unwrap();
    let task_id = created_task.id;

    // 更新データを準備
    let update_dto = UpdateTaskDto {
        title: Some("Updated Title".to_string()),
        status: Some(TaskStatus::InProgress),
        description: None,
        priority: None,
        due_date: None,
    };

    // タスクを更新
    let updated_task = repo.update(task_id, update_dto).await.unwrap().unwrap();

    // 更新されたタスクを検証
    assert_eq!(updated_task.id, task_id);
    assert_eq!(updated_task.title, "Updated Title");
    assert_eq!(updated_task.status, TaskStatus::InProgress.to_string());
    // 指定しなかったフィールドは変更されていないことを確認
    assert_eq!(updated_task.description, created_task.description);
}

#[tokio::test]
async fn test_delete_task() {
    let (_db, repo) = setup_test_repository().await;

    // タスクを作成
    let task_dto = common::create_test_task();
    let created_task = repo.create(task_dto).await.unwrap();
    let task_id = created_task.id;

    // タスクを削除
    let delete_result = repo.delete(task_id).await.unwrap();
    assert_eq!(delete_result.rows_affected, 1);

    // 削除されたことを確認
    let find_result = repo.find_by_id(task_id).await.unwrap();
    assert!(find_result.is_none());
}

#[tokio::test]
async fn test_find_with_filter() {
    let (_db, repo) = setup_test_repository().await;

    // テスト用タスクをいくつか作成
    let task1 = CreateTaskDto {
        title: "Important Task".to_string(),
        description: Some("High priority".to_string()),
        status: Some(TaskStatus::Todo),
        priority: None,
        due_date: Some(Utc::now() + chrono::Duration::days(1)),
    };

    let task2 = CreateTaskDto {
        title: "Normal Task".to_string(),
        description: Some("Medium priority".to_string()),
        status: Some(TaskStatus::InProgress),
        priority: None,
        due_date: Some(Utc::now() + chrono::Duration::days(2)),
    };

    let task3 = CreateTaskDto {
        title: "Another Important Task".to_string(),
        description: Some("Also high priority".to_string()),
        status: Some(TaskStatus::Todo),
        priority: None,
        due_date: Some(Utc::now() + chrono::Duration::days(3)),
    };

    repo.create(task1).await.unwrap();
    repo.create(task2).await.unwrap();
    repo.create(task3).await.unwrap();

    // ステータスでフィルタリング
    let filter = TaskFilterDto {
        status: Some(TaskStatus::Todo),
        ..Default::default()
    };

    let (filtered_tasks, count) = repo.find_with_filter(&filter).await.unwrap();

    // "todo"ステータスのタスクが2つあることを確認
    assert_eq!(filtered_tasks.len(), 2);
    assert_eq!(filtered_tasks.len() as u64, count);

    // タイトルによるフィルタリング
    let filter = TaskFilterDto {
        title_contains: Some("Important".to_string()),
        ..Default::default()
    };

    let (filtered_tasks, count) = repo.find_with_filter(&filter).await.unwrap();

    // "Important"を含むタスクが2つあることを確認
    assert_eq!(filtered_tasks.len(), 2);
    assert_eq!(filtered_tasks.len() as u64, count);

    // 複合フィルタリング
    let filter = TaskFilterDto {
        status: Some(TaskStatus::Todo),
        title_contains: Some("Important".to_string()),
        ..Default::default()
    };

    let (filtered_tasks, count) = repo.find_with_filter(&filter).await.unwrap();

    // "todo"ステータスで"Important"を含むタスクが2つあることを確認
    assert_eq!(filtered_tasks.len(), 2);
    assert_eq!(filtered_tasks.len() as u64, count);

    // すべてのフィルタ条件が一致しないケース
    let filter = TaskFilterDto {
        status: Some(TaskStatus::Completed),
        title_contains: Some("NonExistent".to_string()),
        ..Default::default()
    };

    let (filtered_tasks, _) = repo.find_with_filter(&filter).await.unwrap();

    // 一致するタスクがないことを確認
    assert!(filtered_tasks.is_empty());
}

#[tokio::test]
async fn test_batch_operations() {
    let (_db, repo) = setup_test_repository().await;

    // 複数のタスクを一括作成用に準備
    let batch_tasks = vec![
        common::create_test_task_with_title("Batch Task 1"),
        common::create_test_task_with_title("Batch Task 2"),
        common::create_test_task_with_title("Batch Task 3"),
    ];

    // 個別に作成して、IDを収集
    let mut task_ids = Vec::new();
    for task_dto in batch_tasks {
        let created_task = repo.create(task_dto).await.unwrap();
        task_ids.push(created_task.id);
    }

    // バッチ更新の準備
    let update_items: Vec<BatchUpdateTaskItemDto> = task_ids
        .iter()
        .map(|id| BatchUpdateTaskItemDto {
            id: *id,
            title: Some("Updated Batch Task".to_string()),
            status: Some(TaskStatus::InProgress),
            description: None,
            due_date: None,
        })
        .collect();

    // バッチ更新を実行
    let updated_count = repo.update_many(update_items).await.unwrap();
    assert_eq!(updated_count, 3);

    // 更新されたことを確認
    let all_tasks = repo.find_all().await.unwrap();
    let updated_tasks: Vec<_> = all_tasks
        .iter()
        .filter(|t| task_ids.contains(&t.id))
        .collect();

    for task in updated_tasks {
        assert_eq!(task.title, "Updated Batch Task");
        assert_eq!(task.status, TaskStatus::InProgress.to_string());
    }

    // バッチ削除を実行
    let delete_result = repo.delete_many(task_ids.clone()).await.unwrap();
    assert_eq!(delete_result.rows_affected as usize, task_ids.len());

    // 削除されたことを確認
    for id in task_ids {
        let find_result = repo.find_by_id(id).await.unwrap();
        assert!(find_result.is_none());
    }
}

#[tokio::test]
async fn test_pagination() {
    let (_db, repo) = setup_test_repository().await;

    // 多数のタスクを作成
    for i in 1..=12 {
        let task = CreateTaskDto {
            title: format!("Pagination Task {}", i),
            description: Some("For pagination test".to_string()),
            status: Some(TaskStatus::Todo),
            priority: None,
            due_date: None,
        };
        repo.create(task).await.unwrap();
    }

    // ページネーション付きで取得
    let (page1_tasks, total_count) = repo.find_all_paginated(1, 5).await.unwrap();

    // 1ページ目には5件のタスクがあること
    assert_eq!(page1_tasks.len(), 5);

    // 合計件数は12件であること
    assert_eq!(total_count, 12);

    // 2ページ目を取得
    let (page2_tasks, _) = repo.find_all_paginated(2, 5).await.unwrap();

    // 2ページ目にも5件のタスクがあること
    assert_eq!(page2_tasks.len(), 5);

    // 1ページ目と2ページ目のタスクが異なることを確認
    for p1_task in &page1_tasks {
        for p2_task in &page2_tasks {
            assert_ne!(p1_task.id, p2_task.id);
        }
    }
}
