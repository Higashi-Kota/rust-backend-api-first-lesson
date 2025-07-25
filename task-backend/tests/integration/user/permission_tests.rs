// tests/integration/user/permission_tests.rs

use crate::common::{app_helper, auth_helper};
use axum::http::StatusCode;
use serde_json::json;
use tower::ServiceExt;

/// 管理者がユーザー一覧を取得（正常系）
#[tokio::test]
async fn test_list_users_as_admin_success() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // 管理者ユーザーとして認証
    let admin_user = auth_helper::authenticate_as_admin(&app).await;

    let req = auth_helper::create_authenticated_request(
        "GET",
        "/admin/users",
        &admin_user.access_token,
        None,
    );

    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

/// 一般ユーザーがユーザー一覧取得を試みる（権限エラー）
#[tokio::test]
async fn test_list_users_as_member_forbidden() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // 一般ユーザーを作成
    let member_data = auth_helper::create_test_user_with_info("member@example.com", "member");
    let member = auth_helper::signup_test_user(&app, member_data)
        .await
        .unwrap();

    let req = auth_helper::create_authenticated_request(
        "GET",
        "/admin/users",
        &member.access_token,
        None,
    );

    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

/// 管理者がユーザー分析情報を取得（正常系）
#[tokio::test]
async fn test_get_user_analytics_as_admin_success() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // 管理者ユーザーとして認証
    let admin_user = auth_helper::authenticate_as_admin(&app).await;

    let req = auth_helper::create_authenticated_request(
        "GET",
        "/admin/users/analytics",
        &admin_user.access_token,
        None,
    );

    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

/// 一般ユーザーがユーザー分析情報取得を試みる（権限エラー）
#[tokio::test]
async fn test_get_user_analytics_as_member_forbidden() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // 一般ユーザーを作成
    let member_data = auth_helper::create_test_user_with_info("member2@example.com", "member2");
    let member = auth_helper::signup_test_user(&app, member_data)
        .await
        .unwrap();

    let req = auth_helper::create_authenticated_request(
        "GET",
        "/admin/users/analytics",
        &member.access_token,
        None,
    );

    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

/// 管理者が一括操作を実行（正常系）
#[tokio::test]
async fn test_bulk_operations_as_admin_success() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // 管理者ユーザーとして認証
    let admin_user = auth_helper::authenticate_as_admin(&app).await;

    // 操作対象のユーザーを作成
    let target_data = auth_helper::create_test_user_with_info("target@example.com", "target");
    let target_user = auth_helper::signup_test_user(&app, target_data)
        .await
        .unwrap();

    let payload = json!({
        "operation": "Deactivate",
        "user_ids": [target_user.user_id],
        "notify_users": false
    });

    let req = auth_helper::create_authenticated_request(
        "POST",
        "/admin/users/bulk-operations",
        &admin_user.access_token,
        Some(serde_json::to_string(&payload).unwrap()),
    );

    let response = app.oneshot(req).await.unwrap();
    let status = response.status();

    if status != StatusCode::OK {
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_string = String::from_utf8_lossy(&body);
        println!("Error response status: {:?}", status);
        println!("Error response body: {}", body_string);
        assert_eq!(status, StatusCode::OK);
    }
}

/// 一般ユーザーが一括操作を試みる（権限エラー）
#[tokio::test]
async fn test_bulk_operations_as_member_forbidden() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // 一般ユーザーを作成
    let member_data = auth_helper::create_test_user_with_info("member3@example.com", "member3");
    let member = auth_helper::signup_test_user(&app, member_data)
        .await
        .unwrap();

    let payload = json!({
        "operation": "Deactivate",
        "user_ids": [member.user_id],
        "notify_users": false
    });

    let req = auth_helper::create_authenticated_request(
        "POST",
        "/admin/users/bulk-operations",
        &member.access_token,
        Some(serde_json::to_string(&payload).unwrap()),
    );

    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

/// 管理者が特定ユーザー情報を取得（正常系）
#[tokio::test]
async fn test_get_user_by_id_as_admin_success() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // 管理者ユーザーとして認証
    let admin_user = auth_helper::authenticate_as_admin(&app).await;

    // 取得対象のユーザーを作成
    let target_data =
        auth_helper::create_test_user_with_info("viewtarget@example.com", "viewtarget");
    let target_user = auth_helper::signup_test_user(&app, target_data)
        .await
        .unwrap();

    let req = auth_helper::create_authenticated_request(
        "GET",
        &format!("/admin/users/{}", target_user.user_id),
        &admin_user.access_token,
        None,
    );

    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

/// 一般ユーザーが他のユーザー情報取得を試みる（権限エラー）
#[tokio::test]
async fn test_get_user_by_id_as_member_forbidden() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // 一般ユーザーを作成
    let member_data = auth_helper::create_test_user_with_info("member4@example.com", "member4");
    let member = auth_helper::signup_test_user(&app, member_data)
        .await
        .unwrap();

    // 取得対象のユーザーを作成
    let target_data =
        auth_helper::create_test_user_with_info("othertarget@example.com", "othertarget");
    let target_user = auth_helper::signup_test_user(&app, target_data)
        .await
        .unwrap();

    let req = auth_helper::create_authenticated_request(
        "GET",
        &format!("/admin/users/{}", target_user.user_id),
        &member.access_token,
        None,
    );

    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

/// 管理者がユーザーステータスを更新（正常系）
#[tokio::test]
async fn test_update_account_status_as_admin_success() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // 管理者ユーザーとして認証
    let admin_user = auth_helper::authenticate_as_admin(&app).await;

    // 更新対象のユーザーを作成
    let target_data = auth_helper::create_test_user_with_info("statustgt@example.com", "statustgt");
    let target_user = auth_helper::signup_test_user(&app, target_data)
        .await
        .unwrap();

    let payload = json!({
        "is_active": false
    });

    let req = auth_helper::create_authenticated_request(
        "PATCH",
        &format!("/admin/users/{}/status", target_user.user_id),
        &admin_user.access_token,
        Some(serde_json::to_string(&payload).unwrap()),
    );

    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

/// 一般ユーザーが他のユーザーステータス更新を試みる（権限エラー）
#[tokio::test]
async fn test_update_account_status_as_member_forbidden() {
    let (app, _schema_name, _db) = app_helper::setup_full_app().await;

    // 一般ユーザーを作成
    let member_data = auth_helper::create_test_user_with_info("member5@example.com", "member5");
    let member = auth_helper::signup_test_user(&app, member_data)
        .await
        .unwrap();

    // 更新対象のユーザーを作成
    let target_data =
        auth_helper::create_test_user_with_info("statustgt2@example.com", "statustgt2");
    let target_user = auth_helper::signup_test_user(&app, target_data)
        .await
        .unwrap();

    let payload = json!({
        "is_active": false
    });

    let req = auth_helper::create_authenticated_request(
        "PATCH",
        &format!("/admin/users/{}/status", target_user.user_id),
        &member.access_token,
        Some(serde_json::to_string(&payload).unwrap()),
    );

    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
