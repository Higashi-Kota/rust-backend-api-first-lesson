// task-backend/src/shared/types/pagination.rs

use serde::{Deserialize, Deserializer, Serialize};

/// 文字列または数値からOption<i32>をデシリアライズ
fn deserialize_option_number_from_string<'de, D>(deserializer: D) -> Result<Option<i32>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrNumber {
        String(String),
        Number(i32),
    }

    match Option::<StringOrNumber>::deserialize(deserializer)? {
        Some(StringOrNumber::String(s)) => {
            if s.is_empty() {
                Ok(None)
            } else {
                s.parse::<i32>().map(Some).map_err(serde::de::Error::custom)
            }
        }
        Some(StringOrNumber::Number(n)) => Ok(Some(n)),
        None => Ok(None),
    }
}

/// ページネーション情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationMeta {
    pub page: i32,
    pub per_page: i32,
    pub total_pages: i32,
    pub total_count: i64,
    pub has_next: bool,
    pub has_prev: bool,
}

impl PaginationMeta {
    pub fn new(page: i32, per_page: i32, total_count: i64) -> Self {
        let total_pages = ((total_count as f64) / (per_page as f64)).ceil() as i32;

        Self {
            page,
            per_page,
            total_pages,
            total_count,
            has_next: page < total_pages,
            has_prev: page > 1,
        }
    }
}

/// ページネーションクエリパラメータ
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct PaginationQuery {
    #[serde(deserialize_with = "deserialize_option_number_from_string")]
    pub page: Option<i32>,
    #[serde(deserialize_with = "deserialize_option_number_from_string")]
    pub per_page: Option<i32>,
}

impl PaginationQuery {
    /// デフォルト値を適用してページとper_pageを取得
    pub fn get_pagination(&self) -> (i32, i32) {
        let page = self.page.unwrap_or(1).max(1);
        let per_page = self.per_page.unwrap_or(20).clamp(1, 100);
        (page, per_page)
    }

    /// オフセットを計算
    pub fn get_offset(&self) -> i32 {
        let (page, per_page) = self.get_pagination();
        (page - 1) * per_page
    }
}

/// ページネーション付きレスポンス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub pagination: PaginationMeta,
}

impl<T> PaginatedResponse<T> {
    pub fn new(items: Vec<T>, page: i32, per_page: i32, total_count: i64) -> Self {
        Self {
            items,
            pagination: PaginationMeta::new(page, per_page, total_count),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_meta() {
        let pagination = PaginationMeta::new(2, 10, 25);
        assert_eq!(pagination.page, 2);
        assert_eq!(pagination.per_page, 10);
        assert_eq!(pagination.total_pages, 3);
        assert_eq!(pagination.total_count, 25);
        assert!(pagination.has_next);
        assert!(pagination.has_prev);
    }

    #[test]
    fn test_pagination_query() {
        let query = PaginationQuery {
            page: Some(2),
            per_page: Some(10),
        };
        let (page, per_page) = query.get_pagination();
        assert_eq!(page, 2);
        assert_eq!(per_page, 10);
        assert_eq!(query.get_offset(), 10);
    }

    #[test]
    fn test_pagination_query_defaults() {
        let query = PaginationQuery {
            page: None,
            per_page: None,
        };
        let (page, per_page) = query.get_pagination();
        assert_eq!(page, 1);
        assert_eq!(per_page, 20);
        assert_eq!(query.get_offset(), 0);
    }

    #[test]
    fn test_paginated_response() {
        let items = vec![1, 2, 3];
        let response = PaginatedResponse::new(items, 1, 3, 10);
        assert_eq!(response.items.len(), 3);
        assert_eq!(response.pagination.total_count, 10);
    }
}
