use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

impl From<crate::features::task::models::task_attachment_model::Model> for AttachmentDto {
    fn from(model: crate::features::task::models::task_attachment_model::Model) -> Self {
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

/// 署名付きURL生成レスポンス
#[derive(Serialize, Debug)]
pub struct GenerateDownloadUrlResponse {
    pub download_url: String,
    pub expires_in_seconds: u64,
    pub expires_at: DateTime<Utc>,
}

/// 直接アップロード用の署名付きURL生成レスポンス
#[derive(Serialize, Debug)]
pub struct GenerateUploadUrlResponse {
    pub upload_url: String,
    pub upload_key: String,
    pub expires_at: DateTime<Utc>,
}
