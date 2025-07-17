use crate::api::dto::common::PaginationQuery;
use crate::types::SortQuery;
use serde::{Deserialize, Serialize};
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
