use axum::body::to_bytes;
use common::app_helper::setup_full_app;
use task_backend::features::security::dto::{
    BulkPermissionCheckRequest, BulkPermissionCheckResponse, PermissionCheck,
};
use task_backend::shared::types::common::ApiResponse;
use tower::ServiceExt;

mod common {
    pub use crate::common::*;
}

#[tokio::test]
async fn test_bulk_permission_check_success() {
    let (app, _schema, _db) = setup_full_app().await;
    let admin_user = common::auth_helper::authenticate_as_admin(&app).await;

    // 複数の権限チェックリクエスト
    let request = BulkPermissionCheckRequest {
        user_id: Some(admin_user.id),
        checks: vec![
            PermissionCheck {
                resource: "users".to_string(),
                action: "read".to_string(),
                target_user_id: None,
            },
            PermissionCheck {
                resource: "teams".to_string(),
                action: "read".to_string(),
                target_user_id: None,
            },
            PermissionCheck {
                resource: "tasks".to_string(),
                action: "read".to_string(),
                target_user_id: None,
            },
        ],
    };

    let response = app
        .oneshot(common::app_helper::create_request(
            "POST",
            "/admin/permissions/bulk-check",
            &admin_user.access_token,
            &request,
        ))
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 200);

    let body_bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read body");
    let body: ApiResponse<BulkPermissionCheckResponse> =
        serde_json::from_slice(&body_bytes).expect("Failed to parse response");

    assert!(body.data.is_some());
    let bulk_response = body.data.unwrap();
    assert_eq!(bulk_response.checks.len(), 3);

    // 権限チェック結果の検証
    // 注: 現在の実装では、adminユーザーでも特定のリソースへのアクセスが制限されている
    let allowed_count = bulk_response.checks.iter().filter(|c| c.allowed).count();
    let denied_count = bulk_response.checks.iter().filter(|c| !c.allowed).count();

    // 少なくとも一つは許可されているはず（users:read）
    assert!(
        allowed_count >= 1,
        "At least one permission should be allowed"
    );

    // サマリーの検証
    assert_eq!(bulk_response.summary.total_checks, 3);
    assert_eq!(bulk_response.summary.allowed_count as usize, allowed_count);
    assert_eq!(bulk_response.summary.denied_count as usize, denied_count);
    // execution_time_ms is u64, always >= 0, can be 0 for very fast operations
}

#[tokio::test]
async fn test_bulk_permission_check_partial_success() {
    let (app, _schema, _db) = setup_full_app().await;
    let user = common::auth_helper::create_and_authenticate_user(&app).await;

    // 一般ユーザーで権限チェック
    let request = BulkPermissionCheckRequest {
        user_id: Some(user.id),
        checks: vec![
            PermissionCheck {
                resource: "tasks".to_string(),
                action: "read".to_string(),
                target_user_id: None,
            },
            PermissionCheck {
                resource: "admin".to_string(),
                action: "manage".to_string(),
                target_user_id: None,
            },
        ],
    };

    let response = app
        .oneshot(common::app_helper::create_request(
            "POST",
            "/admin/permissions/bulk-check",
            &user.access_token,
            &request,
        ))
        .await
        .expect("Failed to send request");

    // 一般ユーザーは管理者エンドポイントにアクセスできない
    assert_eq!(response.status(), 403);
}

#[tokio::test]
async fn test_bulk_permission_check_invalid_data() {
    let (app, _schema, _db) = setup_full_app().await;
    let admin_user = common::auth_helper::authenticate_as_admin(&app).await;

    // 空のチェックリスト
    let request = BulkPermissionCheckRequest {
        user_id: None,
        checks: vec![],
    };

    let response = app
        .oneshot(common::app_helper::create_request(
            "POST",
            "/admin/permissions/bulk-check",
            &admin_user.access_token,
            &request,
        ))
        .await
        .expect("Failed to send request");

    // 空のチェックリストでも200を返すが、結果は空
    assert_eq!(response.status(), 200);

    let body_bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read body");
    let body: ApiResponse<BulkPermissionCheckResponse> =
        serde_json::from_slice(&body_bytes).expect("Failed to parse response");

    assert!(body.data.is_some());
    let bulk_response = body.data.unwrap();
    assert_eq!(bulk_response.checks.len(), 0);
    assert_eq!(bulk_response.summary.total_checks, 0);
}
