// tests/common/test_data.rs

use chrono::Utc;
use task_backend::api::dto::task_dto::{CreateTaskDto, UpdateTaskDto};
use task_backend::core::task_status::TaskStatus;
use task_backend::features::auth::dto::*;

// === 認証関連のテストデータ ===

/// テスト用のユーザー登録データを生成
pub fn create_test_signup_data() -> SignupRequest {
    SignupRequest {
        email: format!("test{}@example.com", uuid::Uuid::new_v4()),
        username: format!("testuser{}", &uuid::Uuid::new_v4().to_string()[..8]),
        password: "MyUniqueP@ssw0rd91".to_string(),
    }
}

/// 特定の情報でユーザー登録データを生成
pub fn create_signup_data_with_info(email: &str, username: &str, password: &str) -> SignupRequest {
    SignupRequest {
        email: email.to_string(),
        username: username.to_string(),
        password: password.to_string(),
    }
}

/// メールアドレスとパスワードでログインデータを生成
pub fn create_signin_data_with_email_and_password(
    identifier: &str,
    password: &str,
) -> SigninRequest {
    SigninRequest {
        identifier: identifier.to_string(),
        password: password.to_string(),
    }
}

/// パスワードリセット要求データを生成
pub fn create_forgot_password_data(email: &str) -> PasswordResetRequestRequest {
    PasswordResetRequestRequest {
        email: email.to_string(),
    }
}

/// パスワードリセット実行データを生成
pub fn create_reset_password_data(token: &str, new_password: &str) -> PasswordResetRequest {
    PasswordResetRequest {
        token: token.to_string(),
        new_password: new_password.to_string(),
    }
}

/// リフレッシュトークンデータを生成
pub fn create_refresh_token_data(refresh_token: &str) -> RefreshTokenRequest {
    RefreshTokenRequest {
        refresh_token: refresh_token.to_string(),
    }
}

/// 無効なユーザー登録データのパターン
pub fn create_invalid_signup_data_empty_email() -> SignupRequest {
    SignupRequest {
        email: "".to_string(),
        username: "testuser".to_string(),
        password: "MyUniqueP@ssw0rd91".to_string(),
    }
}

pub fn create_invalid_signup_data_invalid_email() -> SignupRequest {
    SignupRequest {
        email: "invalid-email".to_string(),
        username: "testuser".to_string(),
        password: "MyUniqueP@ssw0rd91".to_string(),
    }
}

pub fn create_invalid_signup_data_weak_password() -> SignupRequest {
    SignupRequest {
        email: "test@example.com".to_string(),
        username: "testuser".to_string(),
        password: "weak".to_string(),
    }
}

pub fn create_invalid_signup_data_empty_username() -> SignupRequest {
    SignupRequest {
        email: "test@example.com".to_string(),
        username: "".to_string(),
        password: "MyUniqueP@ssw0rd91".to_string(),
    }
}

// === タスク関連のテストデータ ===

/// テスト用のタスク作成データを生成
pub fn create_test_task() -> CreateTaskDto {
    CreateTaskDto {
        title: "Test Task".to_string(),
        description: Some("Test Description".to_string()),
        status: Some(TaskStatus::Todo),
        priority: None,
        due_date: Some(Utc::now()),
    }
}

/// 特定のタイトルでタスク作成データを生成
pub fn create_test_task_with_title(title: &str) -> CreateTaskDto {
    CreateTaskDto {
        title: title.to_string(),
        description: Some("Test Description".to_string()),
        status: Some(TaskStatus::Todo),
        priority: None,
        due_date: Some(Utc::now()),
    }
}

/// 完全にカスタマイズされたタスク作成データを生成
pub fn create_custom_task(
    title: &str,
    description: Option<&str>,
    status: Option<&str>,
) -> CreateTaskDto {
    CreateTaskDto {
        title: title.to_string(),
        description: description.map(|d| d.to_string()),
        status: status.and_then(TaskStatus::from_str),
        priority: None,
        due_date: Some(Utc::now()),
    }
}

/// 部分的なタスク更新データを生成（タイトルのみ）
pub fn create_partial_update_task_title(title: &str) -> UpdateTaskDto {
    UpdateTaskDto {
        title: Some(title.to_string()),
        description: None,
        status: None,
        priority: None,
        due_date: None,
    }
}

/// 無効なタスクデータのパターン
pub fn create_invalid_task_empty_title() -> CreateTaskDto {
    CreateTaskDto {
        title: "".to_string(),
        description: Some("Valid description".to_string()),
        status: Some(TaskStatus::Todo),
        priority: None,
        due_date: Some(Utc::now()),
    }
}

// === テスト用ヘルパー関数 ===

use axum::{body, http::Request, Router};
use serde_json::Value;
use task_backend::api::dto::task_dto::TaskDto;
use tower::ServiceExt;

/// テストユーザー用のタスクを作成
pub async fn create_test_task_for_user(
    app: &Router,
    user: &crate::common::auth_helper::TestUser,
) -> TaskDto {
    let task_data = create_test_task();
    let body_content = serde_json::to_string(&task_data).unwrap();

    let req = Request::builder()
        .uri("/tasks")
        .method("POST")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", user.access_token))
        .body(axum::body::Body::from(body_content))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    let status = res.status();
    let body = body::to_bytes(res.into_body(), usize::MAX).await.unwrap();

    if !status.is_success() {
        let body_str = String::from_utf8_lossy(&body);
        panic!(
            "Failed to create task. Status: {}, Body: {}",
            status, body_str
        );
    }

    let response: Value = serde_json::from_slice(&body).unwrap_or_else(|e| {
        let body_str = String::from_utf8_lossy(&body);
        panic!("Failed to parse JSON response: {}, Body: {}", e, body_str);
    });

    // The task creation endpoint returns the task directly, not wrapped
    let task_dto: TaskDto = if response.is_object() && response.get("id").is_some() {
        // Direct task response
        serde_json::from_value(response.clone())
            .unwrap_or_else(|e| panic!("Failed to parse task DTO: {}. Value: {}", e, response))
    } else if let Some(data) = response.get("data") {
        // Wrapped response format (for compatibility)
        if let Some(task) = data.get("task") {
            serde_json::from_value(task.clone())
                .unwrap_or_else(|e| panic!("Failed to parse task DTO: {}. Value: {}", e, task))
        } else {
            serde_json::from_value(data.clone())
                .unwrap_or_else(|e| panic!("Failed to parse task DTO: {}. Value: {}", e, data))
        }
    } else {
        panic!("Unexpected response format. Full response: {}", response);
    };

    task_dto
}
