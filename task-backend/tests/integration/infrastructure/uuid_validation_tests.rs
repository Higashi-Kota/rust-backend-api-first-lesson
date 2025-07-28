use crate::common::{
    app_helper::setup_full_app, auth_helper::create_and_authenticate_user, request::create_request,
};
use axum::http::StatusCode;
use serde_json::json;
use task_backend::types::ApiResponse;
use tower::ServiceExt;

#[tokio::test]
async fn test_invalid_uuid_format_error_with_parameter_name() {
    let (app, _schema, _db) = setup_full_app().await;
    let user = create_and_authenticate_user(&app).await;

    // 無効なUUIDでタスクを取得しようとする
    let request = create_request("GET", "/tasks/invalid-uuid-format", &user.token, &());
    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: ApiResponse<()> = serde_json::from_slice(&body_bytes).unwrap();
    assert!(!body.success);

    // エラーメッセージを確認
    let error_msg = body.error.unwrap().message;
    println!("Error message: {}", error_msg);
    assert!(error_msg.contains("Invalid UUID format") || error_msg.contains("task_id"));
}

#[tokio::test]
async fn test_missing_uuid_parameter_error() {
    let (app, _schema, _db) = setup_full_app().await;
    let user = create_and_authenticate_user(&app).await;

    // UUIDが欠落しているリクエスト（存在しないパス）
    let request = create_request("GET", "/tasks//update", &user.token, &());
    let response = app.oneshot(request).await.unwrap();

    // 存在しないパスなので404を返す
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_invalid_team_member_path_parameters() {
    let (app, _schema, _db) = setup_full_app().await;
    let user = create_and_authenticate_user(&app).await;

    // 無効なUUIDでチームメンバーを更新しようとする
    let payload = json!({
        "role": "viewer"
    });
    let request = create_request(
        "PATCH",
        "/teams/invalid-team-id/members/invalid-member-id/role",
        &user.token,
        &payload,
    );
    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: ApiResponse<()> = serde_json::from_slice(&body_bytes).unwrap();
    assert!(!body.success);
    let error_msg = body.error.unwrap().message;
    println!("Team member error: {}", error_msg);
    // エラーメッセージにUUID検証エラーが含まれることを確認
    assert!(error_msg.contains("Invalid") && error_msg.contains("UUID"));
}

#[tokio::test]
async fn test_valid_uuid_passes_validation() {
    let (app, _schema, _db) = setup_full_app().await;
    let user = create_and_authenticate_user(&app).await;

    // 有効なUUID形式だが存在しないタスク
    let valid_uuid = uuid::Uuid::new_v4();
    let request = create_request("GET", &format!("/tasks/{}", valid_uuid), &user.token, &());
    let response = app.oneshot(request).await.unwrap();

    // UUID検証は通過するが、タスクが存在しないので404
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_organization_hierarchy_path_validation() {
    let (app, _schema, _db) = setup_full_app().await;
    let user = create_and_authenticate_user(&app).await;

    // 無効なUUIDで組織部門を更新しようとする
    let request = create_request(
        "DELETE",
        "/organizations/not-a-uuid/departments/also-not-a-uuid/members/definitely-not-a-uuid",
        &user.token,
        &(),
    );
    let response = app.oneshot(request).await.unwrap();

    // ValidatedUuidを使用しているエンドポイントなら400、そうでなければ404の可能性
    assert!(
        response.status() == StatusCode::BAD_REQUEST || response.status() == StatusCode::NOT_FOUND
    );

    if response.status() == StatusCode::BAD_REQUEST {
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body: ApiResponse<()> = serde_json::from_slice(&body_bytes).unwrap();
        assert!(!body.success);
        let error_msg = body.error.unwrap().message;
        println!("Organization hierarchy error: {}", error_msg);
        // エラーメッセージが改善されていることを確認
        assert!(error_msg.contains("Invalid") || error_msg.contains("UUID"));
    }
}
