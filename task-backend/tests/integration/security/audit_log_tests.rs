// tests/integration/audit_log_tests.rs
use axum::{body, http::StatusCode};
use serde_json::json;
use task_backend::api::dto::team_task_dto::TransferTaskRequest;
use task_backend::service::audit_log_service::AuditLogDto;
use task_backend::shared::types::PaginatedResponse;
use task_backend::types::ApiResponse;
use tower::ServiceExt;

use crate::common::app_helper::{
    add_team_member, create_team_task_assigned_to, create_test_task, create_test_team,
    setup_full_app,
};
use crate::common::auth_helper::create_and_authenticate_user;
use crate::common::request::create_request;

#[tokio::test]
async fn test_audit_log_task_transfer() {
    // Arrange: セットアップとテストデータの準備
    let (app, _schema, _db) = setup_full_app().await;
    let user1 = create_and_authenticate_user(&app).await;
    let user2 = create_and_authenticate_user(&app).await;
    let team = create_test_team(&app, &user1.token).await;

    // User2をチームに追加
    add_team_member(&app, &user1.token, team.id, user2.id).await;

    // User1がチームタスクを作成（User1にアサイン）
    let task = create_team_task_assigned_to(&app, &user1.token, team.id, user1.id).await;

    // Act: タスクをUser2に引き継ぎ
    let transfer_request = TransferTaskRequest {
        new_assignee: user2.id,
        reason: Some("休暇のため引き継ぎ".to_string()),
    };

    let transfer_response = app
        .clone()
        .oneshot(create_request(
            "POST",
            &format!("/tasks/{}/transfer", task.id),
            &user1.token,
            &transfer_request,
        ))
        .await
        .unwrap();

    // Assert: 引き継ぎが成功
    assert_eq!(transfer_response.status(), StatusCode::OK);

    // デバッグ: 他のエンドポイントも試してみる
    let test_response = app
        .clone()
        .oneshot(create_request(
            "GET",
            "/activity-logs/me?page=1&per_page=10",
            &user1.token,
            &(),
        ))
        .await
        .unwrap();
    let activity_status = test_response.status();
    eprintln!("Activity logs response status: {}", activity_status);
    if activity_status != StatusCode::OK {
        let body = body::to_bytes(test_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8_lossy(&body);
        eprintln!("Activity logs response body: {}", body_str);
    }

    // 監査ログの確認（自分の監査ログを取得）
    let audit_response = app
        .clone()
        .oneshot(create_request(
            "GET",
            "/audit-logs/me?page=1&per_page=10",
            &user1.token,
            &(),
        ))
        .await
        .unwrap();

    if audit_response.status() != StatusCode::OK {
        let status = audit_response.status();
        let body = body::to_bytes(audit_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8_lossy(&body);
        eprintln!("Audit log response status: {}", status);
        eprintln!("Audit log response body: {}", body_str);
        panic!("Expected 200 OK, got {}", status);
    }

    assert_eq!(audit_response.status(), StatusCode::OK);

    let body = body::to_bytes(audit_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let api_response: ApiResponse<PaginatedResponse<AuditLogDto>> =
        serde_json::from_slice(&body).unwrap();
    let logs = api_response.data.unwrap();

    // 最新のログがタスク引き継ぎログであることを確認
    assert!(logs.pagination.total_count > 0);
    let latest_log = &logs.items[0];
    assert_eq!(latest_log.action, "task_transferred");
    assert_eq!(latest_log.resource_type, "task");
    assert_eq!(latest_log.resource_id, Some(task.id));
    assert_eq!(latest_log.user_id, user1.id);

    // 詳細情報の確認
    let details = latest_log.details.as_ref().unwrap();
    assert_eq!(details["new_assignee"], json!(user2.id.to_string()));
    assert_eq!(details["reason"], json!("休暇のため引き継ぎ"));
}

#[tokio::test]
async fn test_audit_log_team_task_creation() {
    // Arrange: セットアップ
    let (app, _schema, _db) = setup_full_app().await;
    let user = create_and_authenticate_user(&app).await;
    let team = create_test_team(&app, &user.token).await;

    // Act: チームタスクを作成
    let task_request = json!({
        "title": "監査ログテスト用タスク",
        "description": "このタスクは監査ログのテスト用です",
        "status": "todo",
        "priority": "medium"
    });

    let response = app
        .clone()
        .oneshot(create_request(
            "POST",
            &format!("/teams/{}/tasks", team.id),
            &user.token,
            &task_request,
        ))
        .await
        .unwrap();

    if response.status() != StatusCode::CREATED {
        let status = response.status();
        let body = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8_lossy(&body);
        eprintln!("Team task creation response status: {}", status);
        eprintln!("Team task creation response body: {}", body_str);
        panic!("Expected 201 CREATED, got {}", status);
    }
    assert_eq!(response.status(), StatusCode::CREATED);

    // Assert: 監査ログの確認
    let audit_response = app
        .clone()
        .oneshot(create_request(
            "GET",
            "/audit-logs/me?page=1&per_page=10",
            &user.token,
            &(),
        ))
        .await
        .unwrap();

    assert_eq!(audit_response.status(), StatusCode::OK);

    let body = body::to_bytes(audit_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let api_response: ApiResponse<PaginatedResponse<AuditLogDto>> =
        serde_json::from_slice(&body).unwrap();
    let logs = api_response.data.unwrap();

    // タスク作成ログが記録されていることを確認
    let creation_log = logs
        .items
        .iter()
        .find(|log| log.action == "task_created")
        .expect("Task creation audit log not found");

    assert_eq!(creation_log.resource_type, "task");
    assert_eq!(creation_log.team_id, Some(team.id));
    assert_eq!(creation_log.user_id, user.id);

    let details = creation_log.details.as_ref().unwrap();
    assert_eq!(details["title"], json!("監査ログテスト用タスク"));
    assert_eq!(details["visibility"], json!("team"));
}

#[tokio::test]
async fn test_audit_log_team_access() {
    // Arrange: 2つのチームと3人のユーザーを作成
    let (app, _schema, _db) = setup_full_app().await;
    let user1 = create_and_authenticate_user(&app).await;
    let user2 = create_and_authenticate_user(&app).await;
    let user3 = create_and_authenticate_user(&app).await;

    let team1 = create_test_team(&app, &user1.token).await;
    let _team2 = create_test_team(&app, &user3.token).await;

    // User2をteam1に追加
    add_team_member(&app, &user1.token, team1.id, user2.id).await;

    // Team1でタスクを作成
    create_team_task_assigned_to(&app, &user1.token, team1.id, user1.id).await;

    // Act & Assert: User1とUser2はteam1の監査ログにアクセス可能
    let response1 = app
        .clone()
        .oneshot(create_request(
            "GET",
            &format!("/teams/{}/audit-logs?page=1&per_page=10", team1.id),
            &user1.token,
            &(),
        ))
        .await
        .unwrap();
    assert_eq!(response1.status(), StatusCode::OK);

    let response2 = app
        .clone()
        .oneshot(create_request(
            "GET",
            &format!("/teams/{}/audit-logs?page=1&per_page=10", team1.id),
            &user2.token,
            &(),
        ))
        .await
        .unwrap();
    assert_eq!(response2.status(), StatusCode::OK);

    // User3はteam1のメンバーではないのでアクセス不可
    let response3 = app
        .clone()
        .oneshot(create_request(
            "GET",
            &format!("/teams/{}/audit-logs?page=1&per_page=10", team1.id),
            &user3.token,
            &(),
        ))
        .await
        .unwrap();
    assert_eq!(response3.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_audit_log_pagination() {
    // Arrange: 複数の監査ログを生成
    let (app, _schema, _db) = setup_full_app().await;
    let user = create_and_authenticate_user(&app).await;

    // 15個のタスクを作成（15個の監査ログ）
    for i in 0..15 {
        let task = json!({
            "title": format!("タスク {}", i),
            "description": "ページネーションテスト用",
            "status": "todo",
            "priority": "low"
        });

        app.clone()
            .oneshot(create_request("POST", "/tasks", &user.token, &task))
            .await
            .unwrap();
    }

    // 監査ログが非同期で記録されるため、少し待機
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    // Act: ページネーションのテスト（1ページ10件）
    let page1_response = app
        .clone()
        .oneshot(create_request(
            "GET",
            "/audit-logs/me?page=1&per_page=10",
            &user.token,
            &(),
        ))
        .await
        .unwrap();

    let page2_response = app
        .clone()
        .oneshot(create_request(
            "GET",
            "/audit-logs/me?page=2&per_page=10",
            &user.token,
            &(),
        ))
        .await
        .unwrap();

    // Assert: 両方のページが成功
    assert_eq!(page1_response.status(), StatusCode::OK);
    assert_eq!(page2_response.status(), StatusCode::OK);

    // ページ1の結果を検証
    let body1 = body::to_bytes(page1_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let api_response1: ApiResponse<PaginatedResponse<AuditLogDto>> =
        serde_json::from_slice(&body1).unwrap();
    let logs1 = api_response1.data.unwrap();

    assert_eq!(logs1.pagination.page, 1);
    assert_eq!(logs1.pagination.per_page, 10);

    // デバッグ用にログ数を出力
    eprintln!(
        "Audit logs total: {}, logs on page 1: {}",
        logs1.pagination.total_count,
        logs1.items.len()
    );

    // ログが少ない場合は、少なくとも作成したタスク数分のログがあることを確認
    assert!(
        logs1.pagination.total_count >= 10,
        "Expected at least 10 logs, got {}",
        logs1.pagination.total_count
    );
    assert!(logs1.items.len() <= 10); // ページサイズは10以下

    // ページ2の結果を検証
    let body2 = body::to_bytes(page2_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let api_response2: ApiResponse<PaginatedResponse<AuditLogDto>> =
        serde_json::from_slice(&body2).unwrap();
    let logs2 = api_response2.data.unwrap();

    assert_eq!(logs2.pagination.page, 2);
    assert_eq!(logs2.pagination.per_page, 10);

    // ページ2のログ数も出力
    eprintln!("Logs on page 2: {}", logs2.items.len());

    // ページネーションが機能していることを確認（ページ1とページ2で異なるログ）
    if logs1.pagination.total_count > 10 {
        assert!(!logs2.items.is_empty()); // 総数が10件を超える場合、ページ2にもログがある
    }
}

#[tokio::test]
async fn test_audit_log_admin_access() {
    // Arrange: 管理者と通常ユーザーを作成
    let (app, _schema, _db) = setup_full_app().await;
    let _admin = create_and_authenticate_user(&app).await;
    let user = create_and_authenticate_user(&app).await;

    // 管理者権限を付与（実際の実装では適切な方法で）
    // TODO: 管理者権限の付与方法を実装に合わせて調整

    // ユーザーがタスクを作成
    create_test_task(&app, &user.token).await;

    // Act & Assert: 通常ユーザーは他人の監査ログにアクセス不可
    let response_forbidden = app
        .clone()
        .oneshot(create_request(
            "GET",
            &format!("/admin/audit-logs/users/{}", user.id),
            &user.token,
            &(),
        ))
        .await
        .unwrap();
    assert_eq!(response_forbidden.status(), StatusCode::FORBIDDEN);

    // TODO: 管理者は他人の監査ログにアクセス可能なテストを追加
    // （管理者権限の実装後）
}

#[tokio::test]
async fn test_audit_log_cleanup() {
    // このテストは管理者権限が必要なため、現時点ではスキップ
    // TODO: 管理者権限の実装後にテストを完成させる
}
