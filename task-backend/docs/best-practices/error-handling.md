# エラーハンドリングベストプラクティス

## 概要

このドキュメントでは、Rust製バックエンドAPIにおけるエラーハンドリングのベストプラクティスをまとめています。

## 実装パターン

### 1. カスタムエラー型の定義（推奨）

```rust
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Authentication required")]
    Unauthorized,
    
    #[error("Forbidden: {0}")]
    Forbidden(String),
    
    #[error("Resource not found: {0}")]
    NotFound(String),
    
    #[error("Bad request: {0}")]
    BadRequest(String),
    
    #[error("Internal server error")]
    InternalServerError,
    
    #[error("Subscription required: {0}")]
    SubscriptionRequired(String),
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error occurred"),
            AppError::Validation(ref e) => (StatusCode::UNPROCESSABLE_ENTITY, e.as_str()),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Authentication required"),
            AppError::Forbidden(ref e) => (StatusCode::FORBIDDEN, e.as_str()),
            AppError::NotFound(ref e) => (StatusCode::NOT_FOUND, e.as_str()),
            AppError::BadRequest(ref e) => (StatusCode::BAD_REQUEST, e.as_str()),
            AppError::InternalServerError => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
            AppError::SubscriptionRequired(ref e) => (StatusCode::PAYMENT_REQUIRED, e.as_str()),
            AppError::RateLimitExceeded => (StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded"),
        };

        let body = Json(json!({
            "error": error_message,
            "error_type": error_type_from_error(&self),
        }));

        (status, body).into_response()
    }
}
```

### 2. Result型エイリアスの活用

```rust
pub type AppResult<T> = Result<T, AppError>;

// 使用例
pub async fn get_user(user_id: Uuid) -> AppResult<User> {
    let user = user_repository
        .find_by_id(user_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("User with id {} not found", user_id)))?;
    
    Ok(user)
}
```

### 3. バリデーションエラーの詳細化

```rust
use validator::ValidationErrors;

impl From<ValidationErrors> for AppError {
    fn from(errors: ValidationErrors) -> Self {
        let error_messages: Vec<String> = errors
            .field_errors()
            .into_iter()
            .flat_map(|(field, errors)| {
                errors.iter().map(move |error| {
                    format!("{}: {}", field, error.message.as_ref().unwrap_or(&error.code))
                })
            })
            .collect();

        AppError::Validation(error_messages.join(", "))
    }
}
```

### 4. 権限エラーの詳細化

```rust
pub enum PermissionError {
    InsufficientRole { required: String, current: String },
    InsufficientSubscription { required: String, current: String },
    ResourceLimitExceeded { resource: String, limit: u32, current: u32 },
    ScopeRestriction { required_scope: String },
}

impl From<PermissionError> for AppError {
    fn from(error: PermissionError) -> Self {
        match error {
            PermissionError::InsufficientRole { required, current } => {
                AppError::Forbidden(format!(
                    "Required role: {}, but you have: {}",
                    required, current
                ))
            }
            PermissionError::InsufficientSubscription { required, current } => {
                AppError::SubscriptionRequired(format!(
                    "This feature requires {} subscription (current: {})",
                    required, current
                ))
            }
            PermissionError::ResourceLimitExceeded { resource, limit, current } => {
                AppError::BadRequest(format!(
                    "{} limit exceeded: {} (limit: {})",
                    resource, current, limit
                ))
            }
            PermissionError::ScopeRestriction { required_scope } => {
                AppError::Forbidden(format!(
                    "Access restricted to {} scope",
                    required_scope
                ))
            }
        }
    }
}
```

## エラー処理のベストプラクティス

### 1. 早期リターンパターン

```rust
pub async fn update_task(
    task_id: Uuid,
    update_data: UpdateTaskRequest,
    user_id: Uuid,
) -> AppResult<Task> {
    // 早期リターンで権限チェック
    let task = task_repository
        .find_by_id(task_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Task {} not found", task_id)))?;

    if task.user_id != user_id {
        return Err(AppError::Forbidden("You can only update your own tasks".to_string()));
    }

    // メイン処理
    let updated_task = task_repository
        .update(task_id, update_data)
        .await?;

    Ok(updated_task)
}
```

### 2. エラーのコンテキスト追加

```rust
use anyhow::Context;

pub async fn complex_operation(user_id: Uuid) -> AppResult<ComplexResult> {
    let user = user_repository
        .find_by_id(user_id)
        .await
        .context("Failed to fetch user")?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    let subscription = subscription_service
        .get_user_subscription(user_id)
        .await
        .context("Failed to fetch subscription")?;

    // ... 複雑な処理
    
    Ok(result)
}
```

### 3. トランザクションでのエラーハンドリング

```rust
use sea_orm::TransactionTrait;

pub async fn create_organization_with_owner(
    create_data: CreateOrganizationData,
    owner_id: Uuid,
) -> AppResult<Organization> {
    let txn = db.begin().await?;

    let result = async {
        // 組織作成
        let organization = organization_repository
            .create(&txn, create_data)
            .await?;

        // オーナー設定
        let membership = organization_member_repository
            .add_member(&txn, organization.id, owner_id, "Owner")
            .await?;

        Ok(organization)
    }
    .await;

    match result {
        Ok(org) => {
            txn.commit().await?;
            Ok(org)
        }
        Err(e) => {
            txn.rollback().await?;
            Err(e)
        }
    }
}
```

## アンチパターン

### ❌ パニックの使用（避けるべき）

```rust
// 悪い例
pub async fn get_user(user_id: Uuid) -> User {
    let user = user_repository
        .find_by_id(user_id)
        .await
        .unwrap(); // パニックの可能性
    
    user.unwrap() // パニックの可能性
}

// 良い例
pub async fn get_user(user_id: Uuid) -> AppResult<User> {
    let user = user_repository
        .find_by_id(user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
    
    Ok(user)
}
```

### ❌ エラー情報の漏洩（避けるべき）

```rust
// 悪い例：内部エラーをそのまま露出
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = Json(json!({
            "error": format!("{:?}", self), // デバッグ情報を露出
        }));
        
        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}

// 良い例：適切なエラーメッセージ
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::Database(_) => {
                // ログには詳細を記録
                tracing::error!("Database error: {:?}", self);
                // ユーザーには一般的なメッセージ
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error occurred")
            }
            // ...
        };
        
        (status, Json(json!({"error": message}))).into_response()
    }
}
```

## ロギングとモニタリング

### 1. 構造化ログの活用

```rust
use tracing::{error, warn, info};

pub async fn handle_request(request: Request) -> AppResult<Response> {
    info!(
        request_id = %request.id,
        user_id = %request.user_id,
        "Processing request"
    );

    match process_request(request).await {
        Ok(response) => {
            info!(
                request_id = %request.id,
                "Request processed successfully"
            );
            Ok(response)
        }
        Err(e) => {
            error!(
                request_id = %request.id,
                error = %e,
                "Request processing failed"
            );
            Err(e)
        }
    }
}
```

### 2. エラーメトリクスの収集

```rust
use prometheus::{Counter, register_counter};

lazy_static! {
    static ref ERROR_COUNTER: Counter = register_counter!(
        "api_errors_total",
        "Total number of API errors"
    ).unwrap();
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // エラーカウンターを増加
        ERROR_COUNTER.inc();
        
        // エラータイプ別のメトリクス
        match &self {
            AppError::Validation(_) => VALIDATION_ERROR_COUNTER.inc(),
            AppError::Unauthorized => AUTH_ERROR_COUNTER.inc(),
            AppError::RateLimitExceeded => RATE_LIMIT_COUNTER.inc(),
            _ => {}
        }
        
        // レスポンス生成...
    }
}
```

## まとめ

効果的なエラーハンドリングは、信頼性の高いAPIの基盤です。カスタムエラー型、適切なエラーメッセージ、構造化ログ、メトリクスを組み合わせることで、デバッグが容易で、ユーザーフレンドリーなエラー処理を実現できます。