use serde::{Deserialize, Deserializer, Serialize};

// Re-export pagination constants
pub use crate::shared::types::{DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE};

/// デフォルトページ番号
fn default_page() -> u32 {
    1
}

/// デフォルトページサイズ
fn default_per_page() -> u32 {
    DEFAULT_PAGE_SIZE
}

/// 文字列または数値からu32をデシリアライズ
fn deserialize_u32_from_string<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrNumber {
        String(String),
        Number(u32),
    }

    match StringOrNumber::deserialize(deserializer)? {
        StringOrNumber::String(s) => s.parse::<u32>().map_err(serde::de::Error::custom),
        StringOrNumber::Number(n) => Ok(n),
    }
}

/// 統一ページネーションクエリパラメータ
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PaginationQuery {
    #[serde(
        default = "default_page",
        deserialize_with = "deserialize_u32_from_string"
    )]
    pub page: u32,
    #[serde(
        default = "default_per_page",
        deserialize_with = "deserialize_u32_from_string"
    )]
    pub per_page: u32,
}

impl Default for PaginationQuery {
    fn default() -> Self {
        Self {
            page: default_page(),
            per_page: default_per_page(),
        }
    }
}

impl PaginationQuery {
    /// デフォルト値を適用してページとper_pageを取得
    pub fn get_pagination(&self) -> (i32, i32) {
        let page = self.page.max(1) as i32;
        let per_page = self.per_page.clamp(1, MAX_PAGE_SIZE) as i32;
        (page, per_page)
    }

    /// オフセットを計算
    pub fn get_offset(&self) -> i32 {
        let page = self.page.max(1);
        ((page - 1) * self.per_page) as i32
    }
}

/// 統一ソートクエリパラメータ
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SortQuery {
    pub sort_by: Option<String>,
    #[serde(default)]
    pub sort_order: SortOrder,
}

/// ソート順序
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    #[default]
    Asc,
    Desc,
}

// SearchQuery trait removed - methods were never used in production code
// Individual query structs implement their own search logic directly

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_order_default() {
        let sort = SortQuery::default();
        assert!(sort.sort_by.is_none());
        assert!(matches!(sort.sort_order, SortOrder::Asc));
    }
}
