use serde::Deserialize;

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
