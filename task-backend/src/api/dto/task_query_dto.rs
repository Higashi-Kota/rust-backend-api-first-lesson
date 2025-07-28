use crate::api::dto::common::PaginationQuery;
use crate::domain::task_status::TaskStatus;
use crate::domain::task_visibility::TaskVisibility;
use crate::types::query::SearchQuery;
use crate::types::{optional_timestamp, SortQuery};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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

    // マルチテナント対応フィルタ
    pub visibility: Option<TaskVisibility>,
    pub team_id: Option<Uuid>,
    pub organization_id: Option<Uuid>,
    pub include_assigned: Option<bool>, // 自分に割り当てられたタスクも含むか
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
            "visibility",
            "assigned_to",
        ]
    }
}

impl SearchQuery for TaskSearchQuery {
    fn search_term(&self) -> Option<&str> {
        self.search.as_deref()
    }

    fn filters(&self) -> HashMap<String, String> {
        let mut filters = HashMap::new();

        if let Some(status) = &self.status {
            filters.insert("status".to_string(), format!("{:?}", status));
        }
        if let Some(id) = &self.assigned_to {
            filters.insert("assigned_to".to_string(), id.to_string());
        }
        if let Some(priority) = &self.priority {
            filters.insert("priority".to_string(), priority.clone());
        }
        if let Some(visibility) = &self.visibility {
            filters.insert("visibility".to_string(), format!("{:?}", visibility));
        }
        if let Some(id) = &self.team_id {
            filters.insert("team_id".to_string(), id.to_string());
        }
        if let Some(id) = &self.organization_id {
            filters.insert("organization_id".to_string(), id.to_string());
        }

        filters
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
