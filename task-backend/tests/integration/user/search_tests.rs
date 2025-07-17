// task-backend/tests/integration/user/search_tests.rs

use task_backend::api::dto::user_dto::UserSearchQuery;
use task_backend::types::query::{PaginationQuery, SortOrder, SortQuery};
use validator::Validate;

#[test]
fn test_user_search_query_validation() {
    let query = UserSearchQuery {
        search: Some("test".to_string()),
        is_active: Some(true),
        email_verified: Some(true),
        pagination: PaginationQuery {
            page: 1,
            per_page: 20,
        },
        sort: SortQuery {
            sort_by: Some("username".to_string()),
            sort_order: SortOrder::Asc,
        },
    };

    // 有効なクエリのバリデーションテスト
    assert!(query.validate().is_ok());

    // ページネーションの境界値テスト（get_pagination()で正規化される）
    let boundary_query = UserSearchQuery {
        pagination: PaginationQuery {
            page: 0,
            per_page: 200,
        },
        ..query.clone()
    };
    let (page, per_page) = boundary_query.pagination.get_pagination();
    assert_eq!(page, 1); // 0は1にクランプされる
    assert_eq!(per_page, 100); // 200は100にクランプされる

    // 長すぎる検索語のテスト
    let invalid_query = UserSearchQuery {
        search: Some("a".repeat(101)),
        ..query.clone()
    };
    assert!(invalid_query.validate().is_err());
}

#[test]
fn test_user_search_query_with_defaults() {
    let query = UserSearchQuery {
        search: Some("test".to_string()),
        is_active: None,
        email_verified: None,
        pagination: PaginationQuery::default(),
        sort: SortQuery::default(),
    };

    // デフォルト値の検証
    let (page, per_page) = query.pagination.get_pagination();
    assert_eq!(page, 1);
    assert_eq!(per_page, 20);
    assert!(query.sort.sort_by.is_none());
    assert!(matches!(query.sort.sort_order, SortOrder::Asc));
}
