use crate::api::dto::common::PaginationQuery;
use crate::domain::task_status::TaskStatus;
use crate::types::{optional_timestamp, SortQuery};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 統一タスク検索クエリ
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct TaskSearchQuery {
    #[serde(flatten)]
    pub pagination: PaginationQuery,
    #[serde(flatten)]
    pub sort: SortQuery,
    pub search: Option<String>,
    pub status: Option<TaskStatus>,
    pub assigned_to: Option<Uuid>,
    pub priority: Option<String>,
    #[serde(default, with = "optional_timestamp")]
    pub due_date_before: Option<DateTime<Utc>>,
    #[serde(default, with = "optional_timestamp")]
    pub due_date_after: Option<DateTime<Utc>>,
    #[serde(default, with = "optional_timestamp")]
    pub created_after: Option<DateTime<Utc>>,
    #[serde(default, with = "optional_timestamp")]
    pub created_before: Option<DateTime<Utc>>,
}

impl TaskSearchQuery {
    /// 許可されたソートフィールド
    pub fn allowed_sort_fields() -> &'static [&'static str] {
        &[
            "title",
            "created_at",
            "updated_at",
            "due_date",
            "priority",
            "status",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SortOrder;

    #[test]
    fn test_task_search_query_defaults() {
        let query = TaskSearchQuery::default();
        assert!(query.search.is_none());
        assert!(query.status.is_none());
        assert!(query.assigned_to.is_none());
        assert!(query.sort.sort_by.is_none());
        assert!(matches!(query.sort.sort_order, SortOrder::Asc));
    }
}
