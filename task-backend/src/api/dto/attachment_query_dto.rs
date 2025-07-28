use crate::api::dto::common::PaginationQuery;
use crate::types::{query::SearchQuery, SortQuery};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// 統一添付ファイル検索クエリ
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct AttachmentSearchQuery {
    #[serde(flatten)]
    pub pagination: PaginationQuery,
    #[serde(flatten)]
    pub sort: SortQuery,

    pub search: Option<String>,
    pub task_id: Option<Uuid>,
    pub uploaded_by: Option<Uuid>,
    pub file_type: Option<String>,
    pub min_size: Option<i64>,
    pub max_size: Option<i64>,
}

impl AttachmentSearchQuery {
    /// 許可されたソートフィールド
    pub fn allowed_sort_fields() -> &'static [&'static str] {
        &["file_name", "file_size", "uploaded_at", "file_type"]
    }
}

impl SearchQuery for AttachmentSearchQuery {
    fn search_term(&self) -> Option<&str> {
        self.search.as_deref()
    }

    fn filters(&self) -> HashMap<String, String> {
        let mut filters = HashMap::new();

        if let Some(id) = &self.task_id {
            filters.insert("task_id".to_string(), id.to_string());
        }
        if let Some(id) = &self.uploaded_by {
            filters.insert("uploaded_by".to_string(), id.to_string());
        }
        if let Some(file_type) = &self.file_type {
            filters.insert("file_type".to_string(), file_type.clone());
        }
        if let Some(size) = &self.min_size {
            filters.insert("min_size".to_string(), size.to_string());
        }
        if let Some(size) = &self.max_size {
            filters.insert("max_size".to_string(), size.to_string());
        }

        filters
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SortOrder;

    #[test]
    fn test_attachment_search_query_defaults() {
        let query = AttachmentSearchQuery::default();
        assert!(query.search.is_none());
        assert!(query.task_id.is_none());
        assert!(query.uploaded_by.is_none());
        assert!(query.sort.sort_by.is_none());
        assert!(matches!(query.sort.sort_order, SortOrder::Asc));
    }
}
