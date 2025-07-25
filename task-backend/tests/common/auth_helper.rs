// tests/common/auth_helper.rs

use axum::{
    body::{self, Body},
    http::{Request, StatusCode},
    Router,
};
use serde_json::Value;
use task_backend::api::dto::auth_dto::*;
use tower::ServiceExt;
use uuid::Uuid;

/// テスト用のユーザー情報
#[derive(Debug, Clone)]
pub struct TestUser {
    pub id: Uuid,
    pub user_id: Uuid, // Alias for id to match test expectations
    pub email: String,
    pub username: String,
    pub access_token: String,
    pub token: String, // Alias for access_token to match test expectations
    pub refresh_token: Option<String>,
}

/// テスト用のユーザーデータ
pub fn create_test_user_data() -> SignupRequest {
    SignupRequest {
        email: format!("test{}@example.com", uuid::Uuid::new_v4()),
        username: format!("testuser{}", &uuid::Uuid::new_v4().to_string()[..8]),
        password: "MyUniqueP@ssw0rd91".to_string(),
    }
}

/// 特定の情報でテストユーザーを作成
pub fn create_test_user_with_info(email: &str, username: &str) -> SignupRequest {
    SignupRequest {
        email: email.to_string(),
        username: username.to_string(),
        password: "MyUniqueP@ssw0rd91".to_string(),
    }
}

/// ユーザー登録を実行し、認証情報を返す
pub async fn signup_test_user(
    app: &Router,
    signup_data: SignupRequest,
) -> Result<TestUser, String> {
    let req = Request::builder()
        .uri("/auth/signup")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&signup_data).unwrap()))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();

    if res.status() != StatusCode::CREATED {
        let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
        let error: Value = serde_json::from_slice(&body).unwrap();
        return Err(format!("Signup failed: {:?}", error));
    }

    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let response: Value = serde_json::from_slice(&body).unwrap();

    // Handle the new ApiResponse format
    let data = &response["data"];
    let user_id = Uuid::parse_str(data["user"]["id"].as_str().unwrap()).unwrap();
    let access_token = data["tokens"]["access_token"].as_str().unwrap().to_string();
    let refresh_token = data["tokens"]["refresh_token"]
        .as_str()
        .map(|s| s.to_string());

    Ok(TestUser {
        id: user_id,
        user_id,
        email: signup_data.email.clone(),
        username: signup_data.username.clone(),
        access_token: access_token.clone(),
        token: access_token,
        refresh_token,
    })
}

/// ユーザーログインを実行し、認証情報を返す
pub async fn signin_test_user(
    app: &Router,
    signin_data: SigninRequest,
) -> Result<TestUser, String> {
    let req = Request::builder()
        .uri("/auth/signin")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&signin_data).unwrap()))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();

    if res.status() != StatusCode::OK {
        let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
        let error: Value = serde_json::from_slice(&body).unwrap();
        return Err(format!("Signin failed: {:?}", error));
    }

    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let response: Value = serde_json::from_slice(&body).unwrap();

    // Handle the new ApiResponse format
    let data = &response["data"];
    let user_id = Uuid::parse_str(data["user"]["id"].as_str().unwrap()).unwrap();
    let access_token = data["tokens"]["access_token"].as_str().unwrap().to_string();
    let refresh_token = data["tokens"]["refresh_token"]
        .as_str()
        .map(|s| s.to_string());

    let username = response["data"]["user"]["username"]
        .as_str()
        .unwrap()
        .to_string();
    let email = signin_data.identifier.clone();

    Ok(TestUser {
        id: user_id,
        user_id,
        email: email.clone(),
        username: username.clone(),
        access_token: access_token.clone(),
        token: access_token,
        refresh_token,
    })
}

/// 認証付きのHTTPリクエストを作成するヘルパー
pub fn create_authenticated_request(
    method: &str,
    uri: &str,
    access_token: &str,
    body: Option<String>,
) -> Request<Body> {
    let request_builder = Request::builder()
        .uri(uri)
        .method(method)
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Cookie", format!("access_token={}", access_token))
        .header("Content-Type", "application/json");

    match body {
        Some(body_content) => request_builder.body(Body::from(body_content)).unwrap(),
        None => request_builder.body(Body::empty()).unwrap(),
    }
}

/// ユーザー登録からログインまでの完全なフローを実行
pub async fn setup_authenticated_user(app: &Router) -> Result<TestUser, String> {
    let signup_data = create_test_user_data();
    signup_test_user(app, signup_data).await
}

/// 複数のテストユーザーを作成
pub async fn setup_multiple_users(app: &Router, count: usize) -> Result<Vec<TestUser>, String> {
    let mut users = Vec::new();

    for i in 0..count {
        let signup_data =
            create_test_user_with_info(&format!("user{}@example.com", i), &format!("user{}", i));
        let user = signup_test_user(app, signup_data).await?;
        users.push(user);
    }

    Ok(users)
}

/// トークンリフレッシュを実行
pub async fn refresh_token(app: &Router, refresh_token: &str) -> Result<TestUser, String> {
    let refresh_data = RefreshTokenRequest {
        refresh_token: refresh_token.to_string(),
    };

    let req = Request::builder()
        .uri("/auth/refresh")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&refresh_data).unwrap()))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();

    if res.status() != StatusCode::OK {
        let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
        let error: Value = serde_json::from_slice(&body).unwrap();
        return Err(format!("Token refresh failed: {:?}", error));
    }

    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let response: Value = serde_json::from_slice(&body).unwrap();

    // Handle the new ApiResponse format
    let data = &response["data"];
    let user_id = Uuid::parse_str(data["user"]["id"].as_str().unwrap()).unwrap();
    let access_token = data["tokens"]["access_token"].as_str().unwrap().to_string();
    let new_refresh_token = data["tokens"]["refresh_token"]
        .as_str()
        .map(|s| s.to_string());

    let email = data["user"]["email"].as_str().unwrap().to_string();
    let username = data["user"]["username"].as_str().unwrap().to_string();

    Ok(TestUser {
        id: user_id,
        user_id,
        email: email.clone(),
        username: username.clone(),
        access_token: access_token.clone(),
        token: access_token,
        refresh_token: new_refresh_token,
    })
}

/// パスワードリセット要求を送信
pub async fn request_password_reset(app: &Router, email: &str) -> Result<(), String> {
    let reset_request = PasswordResetRequestRequest {
        email: email.to_string(),
    };

    let req = Request::builder()
        .uri("/auth/forgot-password")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&reset_request).unwrap()))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();

    if res.status() != StatusCode::OK {
        let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
        let error: Value = serde_json::from_slice(&body).unwrap();
        return Err(format!("Password reset request failed: {:?}", error));
    }

    Ok(())
}

/// 初期管理者でログインしてJWTトークンを返す
pub async fn create_admin_with_jwt(app: &Router) -> String {
    // 初期管理者でログイン
    let signin_req = SigninRequest {
        identifier: "admin@example.com".to_string(),
        password: "Adm1n$ecurE2024!".to_string(),
    };

    match signin_test_user(app, signin_req).await {
        Ok(admin_user) => {
            println!(
                "Admin login successful, token: {}",
                &admin_user.access_token[..20]
            );
            admin_user.access_token
        }
        Err(e) => {
            println!("Admin login failed: {}", e);
            panic!("Failed to login admin user: {}", e);
        }
    }
}

/// メンバーロールでユーザーを作成してJWTトークンを返す
pub async fn create_member_with_jwt(app: &Router) -> String {
    // メンバーロールはデフォルトで割り当てられるので、通常のサインアップでOK
    let member_signup = create_test_user_data();
    let member_user = signup_test_user(app, member_signup).await.unwrap();
    member_user.access_token
}

/// 管理者ユーザーとして認証されたTestUserを返す
pub async fn get_admin_user(app: &Router) -> TestUser {
    // 初期管理者でログイン
    let signin_req = SigninRequest {
        identifier: "admin@example.com".to_string(),
        password: "Adm1n$ecurE2024!".to_string(),
    };

    match signin_test_user(app, signin_req).await {
        Ok(admin_user) => admin_user,
        Err(e) => {
            panic!("Failed to login admin user: {}", e);
        }
    }
}

/// テスト用のメンバーユーザーを作成して認証済みユーザー情報を返す
pub async fn create_and_authenticate_member(app: &Router) -> TestUser {
    let member_signup = create_test_user_data();
    signup_test_user(app, member_signup).await.unwrap()
}

/// テスト用の管理者でログインして認証トークンを返す
pub async fn create_and_authenticate_admin(app: &Router) -> String {
    create_admin_with_jwt(app).await
}

/// 認証済みユーザーを作成するヘルパー（統合テスト用）
pub async fn create_authenticated_user(app: &Router, _schema_name: &str) -> (TestUser, TestUser) {
    // テストユーザーを作成
    let user = create_and_authenticate_member(app).await;

    // 同じユーザーを2つ返す（既存テストとの互換性のため）
    (user.clone(), user)
}

/// 基本的なHTTPリクエストを作成するヘルパー
pub fn create_request(method: &str, uri: &str, body: Option<String>) -> Request<Body> {
    let request_builder = Request::builder()
        .uri(uri)
        .method(method)
        .header("Content-Type", "application/json");

    match body {
        Some(body_content) => request_builder.body(Body::from(body_content)).unwrap(),
        None => request_builder.body(Body::empty()).unwrap(),
    }
}

/// 通常のユーザーを作成して認証（TestUserの簡易エイリアス）
pub async fn create_and_authenticate_user(app: &Router) -> TestUser {
    create_and_authenticate_member(app).await
}

/// 管理者として認証するヘルパー
pub async fn authenticate_as_admin(app: &Router) -> TestUser {
    let signin_req = SigninRequest {
        identifier: "admin@example.com".to_string(),
        password: "Adm1n$ecurE2024!".to_string(),
    };

    signin_test_user(app, signin_req)
        .await
        .expect("Failed to authenticate as admin")
}

/// 特定の認証情報でユーザーを作成するヘルパー（テスト用）
pub async fn create_user_with_credentials(
    app: &Router,
    email: &str,
    username: &str,
    password: &str,
) -> Result<TestUser, String> {
    let signup_data = SignupRequest {
        email: email.to_string(),
        username: username.to_string(),
        password: password.to_string(),
    };
    signup_test_user(app, signup_data).await
}
