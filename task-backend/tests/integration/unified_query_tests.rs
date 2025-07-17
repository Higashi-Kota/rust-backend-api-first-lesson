// tests/integration/unified_query_tests.rs
// 統一クエリパターンの統合テスト例

use task_backend::api::dto::attachment_query_dto::AttachmentSearchQuery;
use task_backend::api::dto::task_query_dto::TaskSearchQuery;
use task_backend::api::dto::user_dto::UserSearchQuery;
use task_backend::domain::task_status::TaskStatus;
use task_backend::types::{SortOrder, SortQuery};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_search_query_construction() {
        // 直接構造体を構築してテスト
        let query = TaskSearchQuery {
            pagination: task_backend::api::dto::common::PaginationQuery {
                page: 2,
                per_page: 30,
            },
            sort: SortQuery {
                sort_by: Some("created_at".to_string()),
                sort_order: SortOrder::Desc,
            },
            search: Some("bug".to_string()),
            status: Some(TaskStatus::InProgress),
            ..Default::default()
        };

        assert_eq!(query.pagination.page, 2);
        assert_eq!(query.pagination.per_page, 30);
        assert_eq!(query.sort.sort_by, Some("created_at".to_string()));
        assert!(matches!(query.sort.sort_order, SortOrder::Desc));
        assert_eq!(query.search, Some("bug".to_string()));
        assert_eq!(query.status, Some(TaskStatus::InProgress));
    }

    #[test]
    fn test_user_search_query_construction() {
        let query = UserSearchQuery {
            pagination: task_backend::api::dto::common::PaginationQuery {
                page: 1,
                per_page: 20,
            },
            sort: SortQuery {
                sort_by: Some("username".to_string()),
                sort_order: SortOrder::Asc,
            },
            search: Some("john".to_string()),
            is_active: Some(true),
            email_verified: Some(false),
        };

        assert_eq!(query.pagination.page, 1);
        assert_eq!(query.pagination.per_page, 20);
        assert_eq!(query.sort.sort_by, Some("username".to_string()));
        assert!(matches!(query.sort.sort_order, SortOrder::Asc));
        assert_eq!(query.search, Some("john".to_string()));
        assert_eq!(query.is_active, Some(true));
        assert_eq!(query.email_verified, Some(false));
    }

    #[test]
    fn test_attachment_search_query_construction() {
        let query = AttachmentSearchQuery {
            pagination: task_backend::api::dto::common::PaginationQuery {
                page: 3,
                per_page: 10,
            },
            sort: SortQuery {
                sort_by: Some("file_size".to_string()),
                sort_order: SortOrder::Desc,
            },
            search: Some("report".to_string()),
            file_type: Some("application/pdf".to_string()),
            ..Default::default()
        };

        assert_eq!(query.pagination.page, 3);
        assert_eq!(query.pagination.per_page, 10);
        assert_eq!(query.sort.sort_by, Some("file_size".to_string()));
        assert!(matches!(query.sort.sort_order, SortOrder::Desc));
        assert_eq!(query.search, Some("report".to_string()));
        assert_eq!(query.file_type, Some("application/pdf".to_string()));
    }

    #[test]
    fn test_empty_query_defaults() {
        // 空のクエリでデフォルト値が適用されることを確認
        let query = TaskSearchQuery::default();

        // get_pagination()でデフォルト値を取得
        let (page, per_page) = query.pagination.get_pagination();
        assert_eq!(page, 1);
        assert_eq!(per_page, 20);
        assert!(query.sort.sort_by.is_none());
        assert!(matches!(query.sort.sort_order, SortOrder::Asc));
        assert!(query.search.is_none());
        assert!(query.status.is_none());
    }

    #[test]
    fn test_pagination_defaults() {
        let query = TaskSearchQuery {
            search: Some("test".to_string()),
            ..Default::default()
        };

        // get_pagination()でデフォルト値が適用される
        let (page, per_page) = query.pagination.get_pagination();
        assert_eq!(page, 1); // デフォルトページ
        assert_eq!(per_page, 20); // デフォルトページサイズ
    }

    #[test]
    fn test_search_query_trait() {
        let query = TaskSearchQuery {
            search: Some("important task".to_string()),
            status: Some(TaskStatus::Todo),
            priority: Some("high".to_string()),
            ..Default::default()
        };

        // フィールドが正しく設定されていることを確認
        assert_eq!(query.search, Some("important task".to_string()));
        assert_eq!(query.status, Some(TaskStatus::Todo));
        assert_eq!(query.priority, Some("high".to_string()));
    }

    #[test]
    fn test_sort_order_variants() {
        // ソート順序のバリアントをテスト
        let query_desc = TaskSearchQuery {
            sort: SortQuery {
                sort_by: Some("created_at".to_string()),
                sort_order: SortOrder::Desc,
            },
            ..Default::default()
        };

        let query_asc = TaskSearchQuery {
            sort: SortQuery {
                sort_by: Some("created_at".to_string()),
                sort_order: SortOrder::Asc,
            },
            ..Default::default()
        };

        assert!(matches!(query_desc.sort.sort_order, SortOrder::Desc));
        assert!(matches!(query_asc.sort.sort_order, SortOrder::Asc));
    }
}
