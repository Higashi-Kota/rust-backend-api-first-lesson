# タスクアタッチメント機能実装ガイド

このドキュメントでは、タスクに複数のファイルをアップロードできるアタッチメント機能の実装手順を説明します。

## 📋 目次

1. [概要](#概要)
2. [システム設計](#システム設計)
3. [実装手順](#実装手順)
4. [セキュリティ考慮事項](#セキュリティ考慮事項)
5. [テスト方法](#テスト方法)
6. [トラブルシューティング](#トラブルシューティング)

## 概要

### 機能要件

- タスクに複数のファイルを添付可能
- ファイルのアップロード、ダウンロード、削除機能
- サブスクリプションプランに応じたファイルサイズ制限
- セキュアなファイルアクセス制御

### 技術スタック

- **ストレージ**: 
  - 開発環境: MinIO（S3互換）
  - 本番環境: Cloudflare R2
- **ファイルアップロード**: Axumのmultipart機能
- **データベース**: PostgreSQL（メタデータ保存）

## システム設計

### 1. データベース設計

#### 新規テーブル: `task_attachments`

```sql
CREATE TABLE task_attachments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    uploaded_by UUID NOT NULL REFERENCES users(id),
    file_name VARCHAR(255) NOT NULL,
    file_size BIGINT NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    storage_key VARCHAR(500) NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- インデックス
CREATE INDEX idx_task_attachments_task_id ON task_attachments(task_id);
CREATE INDEX idx_task_attachments_uploaded_by ON task_attachments(uploaded_by);
```

### 2. API設計

#### エンドポイント一覧

| メソッド | パス | 説明 | 認証 |
|---------|------|------|------|
| POST | `/tasks/{task_id}/attachments` | ファイルアップロード | 必要 |
| GET | `/tasks/{task_id}/attachments` | アタッチメント一覧取得 | 必要 |
| GET | `/attachments/{attachment_id}` | ファイルダウンロード | 必要 |
| DELETE | `/attachments/{attachment_id}` | ファイル削除 | 必要 |

#### リクエスト/レスポンス例

**ファイルアップロード**
```bash
POST /tasks/{task_id}/attachments
Content-Type: multipart/form-data

file: (binary)
```

**レスポンス**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "task_id": "660e8400-e29b-41d4-a716-446655440000",
  "file_name": "設計書.pdf",
  "file_size": 1048576,
  "mime_type": "application/pdf",
  "created_at": "2024-01-01T00:00:00Z"
}
```

### 3. ファイルサイズ制限

| プラン | 最大ファイルサイズ | 最大ストレージ容量 |
|--------|-------------------|-------------------|
| Free | 5MB | 100MB |
| Pro | 50MB | 10GB |
| Enterprise | 500MB | 無制限 |

## 実装手順

### Step 1: 依存関係の追加

`task-backend/Cargo.toml`に以下を追加:

```toml
[dependencies]
# S3互換ストレージ用（MinIO/Cloudflare R2対応）
aws-config = "1.5"
aws-sdk-s3 = "1.57"

# ファイルタイプ検出
mime = "0.3"

# マルチパート処理（既にaxumに含まれている）
axum = { version = "0.8", features = ["multipart"] }

# UUID生成
uuid = { version = "1.11", features = ["v4"] }
```

**重要**: aws-sdk-s3はAWS S3 APIの標準実装であり、MinIOとCloudflare R2の両方がS3 APIと互換性があるため、このSDKで両方のサービスを利用できます。

### Step 2: マイグレーションファイルの作成

```bash
# 新しいマイグレーションファイルを作成
cd migration
sea-orm-cli migrate generate create_task_attachments_table
```

生成されたファイルに以下の内容を実装:

```rust
// migration/src/mYYYYMMDD_HHMMSS_create_task_attachments_table.rs
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TaskAttachment::Table)
                    .if_not_exists()
                    .col(uuid(TaskAttachment::Id).primary_key())
                    .col(uuid(TaskAttachment::TaskId).not_null())
                    .col(uuid(TaskAttachment::UploadedBy).not_null())
                    .col(string(TaskAttachment::FileName).not_null())
                    .col(big_integer(TaskAttachment::FileSize).not_null())
                    .col(string(TaskAttachment::MimeType).not_null())
                    .col(string(TaskAttachment::StorageKey).not_null().unique_key())
                    .col(timestamp_with_time_zone(TaskAttachment::CreatedAt).not_null())
                    .col(timestamp_with_time_zone(TaskAttachment::UpdatedAt).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_task_attachment_task")
                            .from(TaskAttachment::Table, TaskAttachment::TaskId)
                            .to(Task::Table, Task::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_task_attachment_user")
                            .from(TaskAttachment::Table, TaskAttachment::UploadedBy)
                            .to(User::Table, User::Id),
                    )
                    .to_owned(),
            )
            .await?;

        // インデックスの作成
        manager
            .create_index(
                Index::create()
                    .name("idx_task_attachments_task_id")
                    .table(TaskAttachment::Table)
                    .col(TaskAttachment::TaskId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TaskAttachment::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum TaskAttachment {
    Table,
    Id,
    TaskId,
    UploadedBy,
    FileName,
    FileSize,
    MimeType,
    StorageKey,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Task {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
}
```

### Step 3: ドメインモデルの作成

```rust
// task-backend/src/domain/task_attachment_model.rs
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "task_attachments")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub task_id: Uuid,
    pub uploaded_by: Uuid,
    pub file_name: String,
    pub file_size: i64,
    pub mime_type: String,
    pub storage_key: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::task_model::Entity",
        from = "Column::TaskId",
        to = "super::task_model::Column::Id"
    )]
    Task,
    
    #[sea_orm(
        belongs_to = "super::user_model::Entity",
        from = "Column::UploadedBy",
        to = "super::user_model::Column::Id"
    )]
    User,
}

impl Related<super::task_model::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Task.def()
    }
}

impl Related<super::user_model::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
```

### Step 4: ストレージサービスの実装

```rust
// task-backend/src/service/storage_service.rs
use async_trait::async_trait;
use aws_sdk_s3::{Client, Config};
use aws_sdk_s3::config::{Credentials, Region};
use aws_sdk_s3::primitives::ByteStream;
use std::error::Error;
use uuid::Uuid;

#[async_trait]
pub trait StorageService: Send + Sync {
    async fn upload(
        &self,
        file_data: Vec<u8>,
        content_type: &str,
    ) -> Result<String, Box<dyn Error>>;
    
    async fn download(&self, key: &str) -> Result<Vec<u8>, Box<dyn Error>>;
    
    async fn delete(&self, key: &str) -> Result<(), Box<dyn Error>>;
}

pub struct S3StorageService {
    client: Client,
    bucket: String,
}

impl S3StorageService {
    pub async fn new(config: &StorageConfig) -> Result<Self, Box<dyn Error>> {
        let credentials = Credentials::new(
            &config.access_key,
            &config.secret_key,
            None,
            None,
            "storage",
        );

        let s3_config = Config::builder()
            .region(Region::new(config.region.clone()))
            .endpoint_url(&config.endpoint)
            .credentials_provider(credentials)
            .force_path_style(true) // MinIO用（Cloudflare R2も対応）
            .build();

        let client = Client::from_conf(s3_config);

        Ok(Self {
            client,
            bucket: config.bucket.clone(),
        })
    }
}

#[async_trait]
impl StorageService for S3StorageService {
    async fn upload(
        &self,
        file_data: Vec<u8>,
        content_type: &str,
    ) -> Result<String, Box<dyn Error>> {
        let key = format!("attachments/{}", Uuid::new_v4());
        
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .body(ByteStream::from(file_data))
            .content_type(content_type)
            .send()
            .await?;

        Ok(key)
    }

    async fn download(&self, key: &str) -> Result<Vec<u8>, Box<dyn Error>> {
        let response = self.client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await?;

        let data = response.body.collect().await?;
        Ok(data.to_vec())
    }

    async fn delete(&self, key: &str) -> Result<(), Box<dyn Error>> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await?;

        Ok(())
    }
}

#[derive(Clone)]
pub struct StorageConfig {
    pub endpoint: String,
    pub bucket: String,
    pub region: String,
    pub access_key: String,
    pub secret_key: String,
}
```

### Step 5: アタッチメントサービスの実装

```rust
// task-backend/src/service/attachment_service.rs
use crate::domain::task_attachment_model::{self, Entity as TaskAttachment};
use crate::repository::attachment_repository::AttachmentRepository;
use crate::service::storage_service::StorageService;
use crate::error::AppError;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use uuid::Uuid;

pub struct AttachmentService {
    db: Arc<DatabaseConnection>,
    storage: Arc<dyn StorageService>,
    repository: AttachmentRepository,
}

impl AttachmentService {
    pub fn new(
        db: Arc<DatabaseConnection>,
        storage: Arc<dyn StorageService>,
    ) -> Self {
        Self {
            db: db.clone(),
            storage,
            repository: AttachmentRepository::new(db),
        }
    }

    pub async fn upload_attachment(
        &self,
        task_id: Uuid,
        user_id: Uuid,
        file_name: String,
        file_data: Vec<u8>,
        mime_type: String,
    ) -> Result<task_attachment_model::Model, AppError> {
        // ファイルサイズチェック
        let file_size = file_data.len() as i64;
        self.validate_file_size(user_id, file_size).await?;

        // ストレージにアップロード
        let storage_key = self.storage
            .upload(file_data, &mime_type)
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        // メタデータをDBに保存
        let attachment = self.repository
            .create(CreateAttachmentDto {
                task_id,
                uploaded_by: user_id,
                file_name,
                file_size,
                mime_type,
                storage_key,
            })
            .await?;

        Ok(attachment)
    }

    pub async fn download_attachment(
        &self,
        attachment_id: Uuid,
        user_id: Uuid,
    ) -> Result<(Vec<u8>, String, String), AppError> {
        // アタッチメント情報を取得
        let attachment = self.repository
            .find_by_id(attachment_id)
            .await?
            .ok_or(AppError::NotFound("Attachment not found".to_string()))?;

        // アクセス権限チェック
        self.check_access_permission(user_id, attachment.task_id).await?;

        // ストレージからダウンロード
        let file_data = self.storage
            .download(&attachment.storage_key)
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        Ok((file_data, attachment.file_name, attachment.mime_type))
    }

    pub async fn delete_attachment(
        &self,
        attachment_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        // アタッチメント情報を取得
        let attachment = self.repository
            .find_by_id(attachment_id)
            .await?
            .ok_or(AppError::NotFound("Attachment not found".to_string()))?;

        // アクセス権限チェック
        self.check_delete_permission(user_id, attachment.task_id).await?;

        // ストレージから削除
        self.storage
            .delete(&attachment.storage_key)
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        // DBから削除
        self.repository.delete(attachment_id).await?;

        Ok(())
    }

    async fn validate_file_size(
        &self,
        user_id: Uuid,
        file_size: i64,
    ) -> Result<(), AppError> {
        // ユーザーのサブスクリプションプランを取得
        let max_size = self.get_max_file_size_for_user(user_id).await?;
        
        if file_size > max_size {
            return Err(AppError::BadRequest(
                format!("File size exceeds limit of {} bytes", max_size)
            ));
        }

        Ok(())
    }

    async fn get_max_file_size_for_user(&self, user_id: Uuid) -> Result<i64, AppError> {
        // TODO: ユーザーのサブスクリプションプランに基づいて制限を返す
        // 仮実装
        Ok(5 * 1024 * 1024) // 5MB
    }

    async fn check_access_permission(
        &self,
        user_id: Uuid,
        task_id: Uuid,
    ) -> Result<(), AppError> {
        // TODO: タスクへのアクセス権限をチェック
        Ok(())
    }

    async fn check_delete_permission(
        &self,
        user_id: Uuid,
        task_id: Uuid,
    ) -> Result<(), AppError> {
        // TODO: 削除権限をチェック（アップロード者またはタスク所有者）
        Ok(())
    }
}

#[derive(Debug)]
pub struct CreateAttachmentDto {
    pub task_id: Uuid,
    pub uploaded_by: Uuid,
    pub file_name: String,
    pub file_size: i64,
    pub mime_type: String,
    pub storage_key: String,
}
```

### Step 6: APIハンドラーの実装

```rust
// task-backend/src/api/handlers/attachment_handler.rs
use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use crate::api::dto::attachment_dto::{AttachmentResponse, AttachmentListResponse};
use crate::auth::AuthenticatedUser;
use crate::service::attachment_service::AttachmentService;
use std::sync::Arc;
use uuid::Uuid;

pub async fn upload_attachment(
    State(service): State<Arc<AttachmentService>>,
    Path(task_id): Path<Uuid>,
    user: AuthenticatedUser,
    mut multipart: Multipart,
) -> Result<Json<AttachmentResponse>, AppError> {
    // マルチパートデータを処理
    while let Some(field) = multipart.next_field().await? {
        let name = field.name().unwrap_or("").to_string();
        
        if name == "file" {
            let file_name = field.file_name()
                .ok_or(AppError::BadRequest("File name is required".to_string()))?
                .to_string();
            
            let content_type = field.content_type()
                .unwrap_or("application/octet-stream")
                .to_string();
            
            let data = field.bytes().await?;
            
            // アップロード処理
            let attachment = service
                .upload_attachment(
                    task_id,
                    user.id,
                    file_name,
                    data.to_vec(),
                    content_type,
                )
                .await?;
            
            return Ok(Json(AttachmentResponse::from(attachment)));
        }
    }
    
    Err(AppError::BadRequest("No file provided".to_string()))
}

pub async fn list_attachments(
    State(service): State<Arc<AttachmentService>>,
    Path(task_id): Path<Uuid>,
    user: AuthenticatedUser,
) -> Result<Json<AttachmentListResponse>, AppError> {
    let attachments = service.list_by_task(task_id, user.id).await?;
    
    Ok(Json(AttachmentListResponse {
        attachments: attachments.into_iter().map(AttachmentResponse::from).collect(),
    }))
}

pub async fn download_attachment(
    State(service): State<Arc<AttachmentService>>,
    Path(attachment_id): Path<Uuid>,
    user: AuthenticatedUser,
) -> Result<Response, AppError> {
    let (data, file_name, mime_type) = service
        .download_attachment(attachment_id, user.id)
        .await?;
    
    // ファイルレスポンスを構築
    Ok((
        StatusCode::OK,
        [
            ("Content-Type", mime_type),
            ("Content-Disposition", format!("attachment; filename=\"{}\"", file_name)),
        ],
        data,
    ).into_response())
}

pub async fn delete_attachment(
    State(service): State<Arc<AttachmentService>>,
    Path(attachment_id): Path<Uuid>,
    user: AuthenticatedUser,
) -> Result<StatusCode, AppError> {
    service.delete_attachment(attachment_id, user.id).await?;
    
    Ok(StatusCode::NO_CONTENT)
}
```

### Step 7: ルーティングの設定

```rust
// task-backend/src/api/routes.rs に追加
use crate::api::handlers::attachment_handler;

pub fn attachment_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/tasks/:task_id/attachments", post(attachment_handler::upload_attachment))
        .route("/tasks/:task_id/attachments", get(attachment_handler::list_attachments))
        .route("/attachments/:attachment_id", get(attachment_handler::download_attachment))
        .route("/attachments/:attachment_id", delete(attachment_handler::delete_attachment))
}

// main.rsでルートを登録
app = app.nest("/api", attachment_routes());
```

### Step 8: 環境設定

`.env`ファイルに以下を追加:

```env
# 開発環境（MinIO）
STORAGE_ENDPOINT=http://localhost:9000
STORAGE_BUCKET=task-attachments
STORAGE_REGION=us-east-1
STORAGE_ACCESS_KEY=minioadmin
STORAGE_SECRET_KEY=minioadmin

# 本番環境（Cloudflare R2）
# STORAGE_ENDPOINT=https://YOUR_ACCOUNT_ID.r2.cloudflarestorage.com
# STORAGE_BUCKET=task-attachments
# STORAGE_REGION=auto
# STORAGE_ACCESS_KEY=your_r2_access_key
# STORAGE_SECRET_KEY=your_r2_secret_key
```

## セキュリティ考慮事項

### 1. ファイルタイプの制限

```rust
// 許可するMIMEタイプのホワイトリスト
const ALLOWED_MIME_TYPES: &[&str] = &[
    "image/jpeg",
    "image/png",
    "image/gif",
    "image/webp",
    "application/pdf",
    "application/msword",
    "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    "application/vnd.ms-excel",
    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
    "text/plain",
    "text/csv",
];

fn validate_mime_type(mime_type: &str) -> Result<(), AppError> {
    if !ALLOWED_MIME_TYPES.contains(&mime_type) {
        return Err(AppError::BadRequest("File type not allowed".to_string()));
    }
    Ok(())
}
```

### 2. ファイル名のサニタイゼーション

```rust
fn sanitize_filename(filename: &str) -> String {
    // 危険な文字を除去
    filename
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '-' || *c == '_' || *c == ' ')
        .collect::<String>()
        .trim()
        .to_string()
}
```

### 3. ウイルススキャン（オプション）

```rust
// ClamAVなどのウイルススキャナーとの統合
#[async_trait]
trait VirusScanService {
    async fn scan(&self, data: &[u8]) -> Result<bool, Box<dyn Error>>;
}
```

### 4. アクセス制御

- タスクの閲覧権限を持つユーザーのみアタッチメントにアクセス可能
- アップロード者またはタスク所有者のみ削除可能
- 直接URLアクセスを防ぐため、署名付きURLまたは認証付きダウンロード

## テスト方法

### 1. 統合テストの例

```rust
// task-backend/tests/integration/attachment_tests.rs
#[tokio::test]
async fn test_upload_attachment_success() {
    let (app, _schema, _db) = setup_full_app().await;
    let user = create_and_authenticate_user(&app).await;
    let task = create_test_task(&app, &user).await;
    
    // ファイルアップロード
    let file_data = b"test file content";
    let response = app.oneshot(
        Request::builder()
            .method("POST")
            .uri(&format!("/api/tasks/{}/attachments", task.id))
            .header("Authorization", &format!("Bearer {}", user.token))
            .header("Content-Type", "multipart/form-data; boundary=----boundary")
            .body(Body::from(
                format!(
                    "------boundary\r\n\
                    Content-Disposition: form-data; name=\"file\"; filename=\"test.txt\"\r\n\
                    Content-Type: text/plain\r\n\r\n\
                    {}\r\n\
                    ------boundary--\r\n",
                    String::from_utf8_lossy(file_data)
                )
            ))
            .unwrap()
    ).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_file_size_limit_exceeded() {
    // ファイルサイズ制限のテスト
}

#[tokio::test]
async fn test_unauthorized_access() {
    // 権限のないユーザーのアクセステスト
}
```

### 2. 手動テスト用のcURLコマンド

```bash
# ファイルアップロード
curl -X POST \
  http://localhost:3000/api/tasks/{task_id}/attachments \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -F "file=@/path/to/file.pdf"

# アタッチメント一覧取得
curl -X GET \
  http://localhost:3000/api/tasks/{task_id}/attachments \
  -H "Authorization: Bearer YOUR_TOKEN"

# ファイルダウンロード
curl -X GET \
  http://localhost:3000/api/attachments/{attachment_id} \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -o downloaded_file.pdf

# ファイル削除
curl -X DELETE \
  http://localhost:3000/api/attachments/{attachment_id} \
  -H "Authorization: Bearer YOUR_TOKEN"
```

## SDK互換性情報

### MinIOとの互換性
- **完全対応**: MinIOは最も広くテストされたS3互換ストレージ
- **設定要件**: `force_path_style(true)`を設定（パススタイルURL必須）
- **認証**: AWS Signature Version 4をサポート

### Cloudflare R2との互換性
- **対応確認済み**: aws-sdk-s3での利用が公式にサポート
- **設定要件**:
  - リージョンは`auto`に設定
  - チェックサム関連のエラーが出る場合は、SDKバージョンの調整が必要な場合あり
  - バッチ削除は700件以下に制限

### 共通の注意点
- 両サービスともS3 APIの主要機能をサポート
- IAM APIは非対応（MinIO、R2共通）
- エンドポイントURLの指定が必須

## トラブルシューティング

### よくある問題と解決方法

#### 1. MinIOへの接続エラー

**症状**: `Connection refused`エラー

**解決方法**:
```bash
# MinIOをDockerで起動
docker run -d \
  -p 9000:9000 \
  -p 9001:9001 \
  --name minio \
  -e MINIO_ROOT_USER=minioadmin \
  -e MINIO_ROOT_PASSWORD=minioadmin \
  minio/minio server /data --console-address ":9001"

# バケットを作成
mc alias set local http://localhost:9000 minioadmin minioadmin
mc mb local/task-attachments
```

#### 2. ファイルアップロードが413エラー

**症状**: `Payload Too Large`エラー

**解決方法**:
```rust
// main.rsでボディサイズ制限を設定
use axum::extract::DefaultBodyLimit;

let app = Router::new()
    .layer(DefaultBodyLimit::max(100 * 1024 * 1024)); // 100MB
```

#### 3. CORS エラー

**症状**: ブラウザからのアップロードが失敗

**解決方法**:
```rust
use tower_http::cors::{CorsLayer, Any};

let cors = CorsLayer::new()
    .allow_methods(Any)
    .allow_origin(Any)
    .allow_headers(Any);

let app = app.layer(cors);
```

### デバッグTips

1. **ログレベルを上げる**
   ```bash
   RUST_LOG=debug cargo run
   ```

2. **MinIO管理画面で確認**
   - http://localhost:9001 にアクセス
   - ユーザー名: minioadmin
   - パスワード: minioadmin

3. **データベースの確認**
   ```sql
   SELECT * FROM task_attachments WHERE task_id = 'YOUR_TASK_ID';
   ```

## 次のステップ

### 機能拡張のアイデア

1. **サムネイル生成**
   - 画像ファイルの自動サムネイル生成
   - PDFのプレビュー画像生成

2. **バージョン管理**
   - 同名ファイルのバージョン管理
   - 変更履歴の追跡

3. **共有機能**
   - 外部共有用の期限付きURL生成
   - パスワード保護

4. **検索機能**
   - ファイル名での検索
   - ファイル内容の全文検索（ElasticSearch連携）

5. **圧縮・変換**
   - 自動圧縮によるストレージ節約
   - ファイル形式の変換機能

これで、タスクアタッチメント機能の基本的な実装が完了です。必要に応じて機能を拡張していってください。