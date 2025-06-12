// tests/common/test_data.rs

use chrono::Utc;
use task_backend::api::dto::{
    auth_dto::*,
    task_dto::{CreateTaskDto, UpdateTaskDto},
};

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
        status: Some("todo".to_string()),
        due_date: Some(Utc::now()),
    }
}

/// 特定のタイトルでタスク作成データを生成
pub fn create_test_task_with_title(title: &str) -> CreateTaskDto {
    CreateTaskDto {
        title: title.to_string(),
        description: Some("Test Description".to_string()),
        status: Some("todo".to_string()),
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
        status: status.map(|s| s.to_string()),
        due_date: Some(Utc::now()),
    }
}

/// 部分的なタスク更新データを生成（タイトルのみ）
pub fn create_partial_update_task_title(title: &str) -> UpdateTaskDto {
    UpdateTaskDto {
        title: Some(title.to_string()),
        description: None,
        status: None,
        due_date: None,
    }
}

/// 無効なタスクデータのパターン
pub fn create_invalid_task_empty_title() -> CreateTaskDto {
    CreateTaskDto {
        title: "".to_string(),
        description: Some("Valid description".to_string()),
        status: Some("todo".to_string()),
        due_date: Some(Utc::now()),
    }
}
