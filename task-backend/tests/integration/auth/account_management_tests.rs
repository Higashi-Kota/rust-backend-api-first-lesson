// tests/integration/auth/account_management_tests.rs

use axum::{
    body::{self, Body},
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;

use crate::common::{app_helper, auth_helper, test_data};

#[tokio::test]
async fn test_get_current_user_success() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // 現在のユーザー情報を取得
    let req =
        auth_helper::create_authenticated_request("GET", "/auth/me", &user.access_token, None);

    let res = app.clone().oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let response: Value = serde_json::from_slice(&body).unwrap();

    // レスポンス構造の検証
    assert!(response["user"].is_object());
    let user_info = &response["user"];

    assert_eq!(user_info["id"], user.id.to_string());
    assert_eq!(user_info["email"], user.email);
    assert_eq!(user_info["username"], user.username);
    assert!(user_info["created_at"].is_string());
    assert!(user_info["updated_at"].is_string());

    // パスワードが含まれていないことを確認
    assert!(
        user_info["password"].is_null() || !user_info.as_object().unwrap().contains_key("password")
    );
}

#[tokio::test]
async fn test_get_current_user_without_token() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    // 認証トークンなしでユーザー情報取得試行
    let req = Request::builder()
        .uri("/auth/me")
        .method("GET")
        .header("Content-Type", "application/json")
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let error: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(error["error_type"], "unauthorized");
}

#[tokio::test]
async fn test_get_current_user_with_invalid_token() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    // 無効なトークンでユーザー情報取得試行
    let req =
        auth_helper::create_authenticated_request("GET", "/auth/me", "invalid.jwt.token", None);

    let res = app.clone().oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let error: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(error["error_type"], "unauthorized");
}

#[tokio::test]
async fn test_delete_account_success() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // アカウント削除
    let delete_request = json!({
        "confirmation": "DELETE MY ACCOUNT",
        "password": "MyUniqueP@ssw0rd91"
    });

    let req = auth_helper::create_authenticated_request(
        "DELETE",
        "/auth/account",
        &user.access_token,
        Some(delete_request.to_string()),
    );

    let res = app.clone().oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let response: Value = serde_json::from_slice(&body).unwrap();

    assert!(response["message"].is_string());
    assert!(
        response["message"].as_str().unwrap().contains("deleted")
            || response["message"].as_str().unwrap().contains("removed")
    );
}

#[tokio::test]
async fn test_delete_account_wrong_confirmation() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // 間違った確認文字列でアカウント削除試行
    let delete_request = json!({
        "confirmation": "WRONG_CONFIRMATION",
        "password": "MyUniqueP@ssw0rd91"
    });

    let req = auth_helper::create_authenticated_request(
        "DELETE",
        "/auth/account",
        &user.access_token,
        Some(delete_request.to_string()),
    );

    let res = app.clone().oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let error: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(error["error_type"], "validation_errors");
    assert!(error["errors"].is_array());
    let errors = error["errors"].as_array().unwrap();
    assert!(!errors.is_empty());

    // 確認文字列関連のエラーが含まれていることを確認
    let error_messages = errors
        .iter()
        .map(|e| e["message"].as_str().unwrap_or(""))
        .collect::<Vec<&str>>();
    assert!(error_messages
        .iter()
        .any(|msg| msg.contains("confirmation") || msg.contains("DELETE MY ACCOUNT")));
}

#[tokio::test]
async fn test_delete_account_wrong_password() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // 間違ったパスワードでアカウント削除試行
    let delete_request = json!({
        "confirmation": "DELETE MY ACCOUNT",
        "password": "WrongPassword4@8!"
    });

    let req = auth_helper::create_authenticated_request(
        "DELETE",
        "/auth/account",
        &user.access_token,
        Some(delete_request.to_string()),
    );

    let res = app.clone().oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let error: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(error["error_type"], "unauthorized");
    assert!(
        error["error"]
            .as_str()
            .unwrap()
            .to_lowercase()
            .contains("password")
            || error["error"].as_str().unwrap().contains("incorrect")
    );
}

#[tokio::test]
async fn test_delete_account_without_authentication() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    // 認証なしでアカウント削除試行
    let delete_request = json!({
        "confirmation": "CONFIRM_DELETE",
        "password": "MyUniqueP@ssw0rd91"
    });

    let req = Request::builder()
        .uri("/auth/account")
        .method("DELETE")
        .header("Content-Type", "application/json")
        .body(Body::from(delete_request.to_string()))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let error: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(error["error_type"], "unauthorized");
}

#[tokio::test]
async fn test_change_password_success() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // パスワード変更
    let change_request = json!({
        "current_password": "MyUniqueP@ssw0rd91",
        "new_password": "NewMyUniqueP@ssw0rd91",
        "new_password_confirmation": "NewMyUniqueP@ssw0rd91"
    });

    let req = auth_helper::create_authenticated_request(
        "PUT",
        "/auth/change-password",
        &user.access_token,
        Some(change_request.to_string()),
    );

    let res = app.clone().oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let response: Value = serde_json::from_slice(&body).unwrap();

    assert!(response["message"].is_string());
    assert!(
        response["message"].as_str().unwrap().contains("changed")
            || response["message"].as_str().unwrap().contains("updated")
    );
}

#[tokio::test]
async fn test_change_password_wrong_current_password() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // 間違った現在のパスワードで変更試行
    let change_request = json!({
        "current_password": "WrongCurrentPass4@8!",
        "new_password": "NewMyUniqueP@ssw0rd91",
        "new_password_confirmation": "NewMyUniqueP@ssw0rd91"
    });

    let req = auth_helper::create_authenticated_request(
        "PUT",
        "/auth/change-password",
        &user.access_token,
        Some(change_request.to_string()),
    );

    let res = app.clone().oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let error: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(error["error_type"], "unauthorized");
    assert!(
        error["error"].as_str().unwrap().contains("current")
            || error["error"].as_str().unwrap().contains("password")
    );
}

#[tokio::test]
async fn test_change_password_mismatch_confirmation() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // パスワード確認が一致しない変更試行
    let change_request = json!({
        "current_password": "MyUniqueP@ssw0rd91",
        "new_password": "NewMyUniqueP@ssw0rd91",
        "new_password_confirmation": "DifferentPass4@8!"
    });

    let req = auth_helper::create_authenticated_request(
        "PUT",
        "/auth/change-password",
        &user.access_token,
        Some(change_request.to_string()),
    );

    let res = app.clone().oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let error: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(error["error_type"], "validation_errors");
    assert!(error["errors"].is_array());
    let errors = error["errors"].as_array().unwrap();
    assert!(!errors.is_empty());

    // パスワード確認関連のエラーが含まれていることを確認
    let error_messages = errors
        .iter()
        .map(|e| e["message"].as_str().unwrap_or(""))
        .collect::<Vec<&str>>();
    assert!(error_messages
        .iter()
        .any(|msg| msg.contains("confirmation") || msg.contains("match")));
}

#[tokio::test]
async fn test_change_password_weak_new_password() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // 弱い新しいパスワードで変更試行
    let change_request = json!({
        "current_password": "MyUniqueP@ssw0rd91",
        "new_password": "weak",
        "new_password_confirmation": "weak"
    });

    let req = auth_helper::create_authenticated_request(
        "PUT",
        "/auth/change-password",
        &user.access_token,
        Some(change_request.to_string()),
    );

    let res = app.clone().oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let error: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(error["error_type"], "validation_errors");
    assert!(error["errors"].is_array());
    let errors = error["errors"].as_array().unwrap();
    assert!(!errors.is_empty());

    // パスワード強度関連のエラーが含まれていることを確認
    let error_messages = errors
        .iter()
        .map(|e| e["message"].as_str().unwrap_or(""))
        .collect::<Vec<&str>>();
    assert!(error_messages
        .iter()
        .any(|msg| msg.contains("password") && (msg.contains("8") || msg.contains("characters"))));
}

#[tokio::test]
async fn test_change_password_same_as_current() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    // ユーザー登録とログイン
    let user = auth_helper::setup_authenticated_user(&app).await.unwrap();

    // 現在と同じパスワードで変更試行
    let change_request = json!({
        "current_password": "MyUniqueP@ssw0rd91",
        "new_password": "MyUniqueP@ssw0rd91",
        "new_password_confirmation": "MyUniqueP@ssw0rd91"
    });

    let req = auth_helper::create_authenticated_request(
        "PUT",
        "/auth/change-password",
        &user.access_token,
        Some(change_request.to_string()),
    );

    let res = app.clone().oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let error: Value = serde_json::from_slice(&body).unwrap();

    // The error should indicate validation error
    assert!(error["error_type"]
        .as_str()
        .unwrap_or("")
        .contains("validation"));

    // Check for error message about same password - could be in different formats
    let error_text = serde_json::to_string(&error).unwrap().to_lowercase();
    assert!(
        error_text.contains("different")
            || error_text.contains("same")
            || error_text.contains("current")
            || error_text.contains("must not match"),
        "Error should indicate password must be different from current. Got: {:?}",
        error
    );
}

#[tokio::test]
async fn test_user_can_login_with_new_password() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    // 特定の情報でユーザー登録
    let signup_data = test_data::create_signup_data_with_info(
        "changepass@example.com",
        "changepassuser",
        "OldPass4@8!",
    );
    let user = auth_helper::signup_test_user(&app, signup_data.clone())
        .await
        .unwrap();

    // パスワード変更
    let change_request = json!({
        "current_password": "OldPass4@8!",
        "new_password": "NewPass4@8!",
        "new_password_confirmation": "NewPass4@8!"
    });

    let req = auth_helper::create_authenticated_request(
        "PUT",
        "/auth/change-password",
        &user.access_token,
        Some(change_request.to_string()),
    );

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 新しいパスワードでログイン
    let signin_data = test_data::create_signin_data_with_email_and_password(
        "changepass@example.com",
        "NewPass4@8!",
    );

    let signin_req = Request::builder()
        .uri("/auth/signin")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&signin_data).unwrap()))
        .unwrap();

    let signin_res = app.clone().oneshot(signin_req).await.unwrap();

    assert_eq!(signin_res.status(), StatusCode::OK);
    let body = body::to_bytes(signin_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: Value = serde_json::from_slice(&body).unwrap();

    assert!(response["tokens"]["access_token"].is_string());
    assert!(response["user"].is_object());
}

#[tokio::test]
async fn test_old_password_no_longer_works() {
    let (app, _schema_name, _db) = app_helper::setup_auth_app().await;

    // 特定の情報でユーザー登録
    let signup_data = test_data::create_signup_data_with_info(
        "oldpass@example.com",
        "oldpassuser",
        "OldPass4@8!",
    );
    let user = auth_helper::signup_test_user(&app, signup_data.clone())
        .await
        .unwrap();

    // パスワード変更
    let change_request = json!({
        "current_password": "OldPass4@8!",
        "new_password": "NewPass4@8!",
        "new_password_confirmation": "NewPass4@8!"
    });

    let req = auth_helper::create_authenticated_request(
        "PUT",
        "/auth/change-password",
        &user.access_token,
        Some(change_request.to_string()),
    );

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 古いパスワードでのログイン試行
    let signin_data =
        test_data::create_signin_data_with_email_and_password("oldpass@example.com", "OldPass4@8!");

    let signin_req = Request::builder()
        .uri("/auth/signin")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&signin_data).unwrap()))
        .unwrap();

    let signin_res = app.clone().oneshot(signin_req).await.unwrap();

    // 古いパスワードでのログインは失敗すること
    assert_eq!(signin_res.status(), StatusCode::UNAUTHORIZED);
    let body = body::to_bytes(signin_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let error: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(error["error_type"], "unauthorized");
}
