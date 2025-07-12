use serde::Deserialize;

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
