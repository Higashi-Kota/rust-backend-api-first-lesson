// task-backend/src/api/dto/attachment_dto.rs

use crate::domain::task_attachment_model;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// --- Response DTOs ---

/// 添付ファイル情報のレスポンスDTO
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AttachmentDto {
    pub id: Uuid,
    pub task_id: Uuid,
    pub uploaded_by: Uuid,
    pub file_name: String,
    pub file_size: i64,
    pub mime_type: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<task_attachment_model::Model> for AttachmentDto {
    fn from(model: task_attachment_model::Model) -> Self {
        Self {
            id: model.id,
            task_id: model.task_id,
            uploaded_by: model.uploaded_by,
            file_name: model.file_name,
            file_size: model.file_size,
            mime_type: model.mime_type,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

/// 添付ファイルアップロード成功時のレスポンス
#[derive(Serialize, Deserialize, Debug)]
pub struct AttachmentUploadResponse {
    pub attachment: AttachmentDto,
    pub message: String,
}

/// 添付ファイル一覧のレスポンス
#[derive(Serialize, Deserialize, Debug)]
pub struct AttachmentListResponse {
    pub attachments: Vec<AttachmentDto>,
    pub total: u64,
}

/// 添付ファイルダウンロード時のレスポンス情報
/// 実際のファイルデータはバイナリで返すため、ここではメタデータのみ
#[derive(Serialize, Deserialize, Debug)]
pub struct AttachmentDownloadInfo {
    pub file_name: String,
    pub mime_type: String,
    pub file_size: i64,
}

/// ストレージ使用量情報
#[derive(Serialize, Deserialize, Debug)]
pub struct StorageUsageDto {
    pub used_bytes: i64,
    pub limit_bytes: i64,
    pub used_mb: f64,
    pub limit_mb: f64,
    pub usage_percentage: f64,
}

/// エラーレスポンス用の詳細情報
#[derive(Serialize, Deserialize, Debug)]
pub struct AttachmentErrorDetail {
    pub error_type: String,
    pub message: String,
    pub allowed_types: Option<Vec<String>>,
    pub max_size_mb: Option<i64>,
}

// --- Request DTOs ---

/// ファイルアップロード時のフィルタ（将来の拡張用）
#[derive(Deserialize, Debug)]
pub struct AttachmentFilterDto {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub sort_by: Option<AttachmentSortBy>,
    pub sort_order: Option<SortOrder>,
}

impl Default for AttachmentFilterDto {
    fn default() -> Self {
        Self {
            page: Some(1),
            per_page: Some(20),
            sort_by: Some(AttachmentSortBy::CreatedAt),
            sort_order: Some(SortOrder::Desc),
        }
    }
}

/// ソート可能なフィールド
#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum AttachmentSortBy {
    CreatedAt,
    FileName,
    FileSize,
}

/// ソート順
#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum SortOrder {
    Asc,
    Desc,
}

// --- 署名付きURL関連 DTOs ---

/// 署名付きURL生成リクエスト
#[derive(Deserialize, Debug)]
pub struct GenerateDownloadUrlRequest {
    /// URL有効期限（秒）。最小60秒、最大86400秒（24時間）
    pub expires_in_seconds: Option<u64>,
}

impl Default for GenerateDownloadUrlRequest {
    fn default() -> Self {
        Self {
            expires_in_seconds: Some(3600), // デフォルト1時間
        }
    }
}

/// 署名付きURL生成レスポンス
#[derive(Serialize, Debug)]
pub struct GenerateDownloadUrlResponse {
    pub download_url: String,
    pub expires_in_seconds: u64,
    pub expires_at: DateTime<Utc>,
}

// --- 外部共有リンク関連 DTOs ---

/// 共有リンク作成リクエスト
#[derive(Deserialize, Debug)]
pub struct CreateShareLinkRequest {
    /// 共有リンクの説明（任意）
    pub description: Option<String>,
    /// 有効期限（時間）。最小1時間、最大720時間（30日）
    pub expires_in_hours: Option<u32>,
    /// 最大アクセス回数（任意）
    pub max_access_count: Option<i32>,
}

impl Default for CreateShareLinkRequest {
    fn default() -> Self {
        Self {
            description: None,
            expires_in_hours: Some(24), // デフォルト24時間
            max_access_count: None,
        }
    }
}

/// 共有リンク情報
#[derive(Serialize, Debug)]
pub struct ShareLinkDto {
    pub id: Uuid,
    pub attachment_id: Uuid,
    pub created_by: Uuid,
    pub share_token: String,
    pub description: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub max_access_count: Option<i32>,
    pub current_access_count: i32,
    pub is_revoked: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// フルURLを構築して返す
    pub share_url: String,
}

impl ShareLinkDto {
    pub fn from_model(
        model: crate::domain::attachment_share_link_model::Model,
        base_url: &str,
    ) -> Self {
        let share_url = format!("{}/share/{}", base_url, model.share_token);

        Self {
            id: model.id,
            attachment_id: model.attachment_id,
            created_by: model.created_by,
            share_token: model.share_token,
            description: model.description,
            expires_at: model.expires_at,
            max_access_count: model.max_access_count,
            current_access_count: model.current_access_count,
            is_revoked: model.is_revoked,
            created_at: model.created_at,
            updated_at: model.updated_at,
            share_url,
        }
    }
}

/// 共有リンク作成レスポンス
#[derive(Serialize, Debug)]
pub struct CreateShareLinkResponse {
    pub share_link: ShareLinkDto,
    pub message: String,
}

/// 共有リンク一覧レスポンス
#[derive(Serialize, Debug)]
pub struct ShareLinkListResponse {
    pub share_links: Vec<ShareLinkDto>,
    pub total: usize,
}

/// 直接アップロード用の署名付きURL生成リクエスト
#[derive(Deserialize, Debug)]
pub struct GenerateUploadUrlRequest {
    /// ファイル名
    pub file_name: String,
    /// URL有効期限（秒）。最小60秒、最大3600秒（1時間）
    pub expires_in_seconds: Option<u64>,
}

impl Default for GenerateUploadUrlRequest {
    fn default() -> Self {
        Self {
            file_name: String::new(),
            expires_in_seconds: Some(3600), // デフォルト1時間
        }
    }
}

/// 直接アップロード用の署名付きURL生成レスポンス
#[derive(Serialize, Debug)]
pub struct GenerateUploadUrlResponse {
    pub upload_url: String,
    pub upload_key: String,
    pub expires_at: DateTime<Utc>,
}
