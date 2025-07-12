use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

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
        model: crate::features::storage::models::attachment_share_link::Model,
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
