// task-backend/src/api/dto/common.rs

use serde::{Deserialize, Serialize};

// Re-export pagination types
pub use crate::shared::types::{PaginatedResponse, PaginationMeta};
pub use crate::types::query::PaginationQuery;

/// 操作結果を表すレスポンス（作成・更新・削除用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationResult<T> {
    pub item: T,
    pub changes: Vec<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl<T> OperationResult<T> {
    pub fn new(item: T, changes: Vec<String>) -> Self {
        Self {
            item,
            changes,
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn created(item: T) -> Self {
        Self::new(item, vec!["Created".to_string()])
    }

    pub fn updated(item: T, changes: Vec<String>) -> Self {
        Self::new(item, changes)
    }

    pub fn deleted(item: T) -> Self {
        Self::new(item, vec!["Deleted".to_string()])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_operation_result() {
        let result = OperationResult::created(json!({"id": 1}));
        assert_eq!(result.changes, vec!["Created"]);
        assert!(result.timestamp <= chrono::Utc::now());

        let changes = vec!["Updated name".to_string(), "Updated email".to_string()];
        let result = OperationResult::updated(json!({"id": 1}), changes.clone());
        assert_eq!(result.changes, changes);
    }
}
