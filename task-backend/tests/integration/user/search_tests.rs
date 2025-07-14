// task-backend/tests/integration/user/search_tests.rs

use task_backend::api::dto::common::PaginationQuery;
use task_backend::api::dto::user_dto::{SortOrder, UserSearchQuery, UserSortField};
use validator::Validate;

#[test]
fn test_user_search_query_validation() {
    let query = UserSearchQuery {
        q: Some("test".to_string()),
        is_active: Some(true),
        email_verified: Some(true),
        pagination: PaginationQuery {
            page: Some(1),
            per_page: Some(20),
        },
        sort_by: Some(UserSortField::Username),
        sort_order: Some(SortOrder::Ascending),
    };

    // 有効なクエリのバリデーションテスト
    assert!(query.validate().is_ok());

    // ページネーションの境界値テスト（get_pagination()で正規化される）
    let boundary_query = UserSearchQuery {
        pagination: PaginationQuery {
            page: Some(0),
            per_page: Some(200),
        },
        ..query.clone()
    };
    let (page, per_page) = boundary_query.pagination.get_pagination();
    assert_eq!(page, 1); // 0は1にクランプされる
    assert_eq!(per_page, 100); // 200は100にクランプされる

    // 長すぎる検索語のテスト
    let invalid_query = UserSearchQuery {
        q: Some("a".repeat(101)),
        ..query.clone()
    };
    assert!(invalid_query.validate().is_err());
}

#[test]
fn test_user_search_query_with_defaults() {
    let query = UserSearchQuery {
        q: Some("test".to_string()),
        is_active: None,
        email_verified: None,
        pagination: PaginationQuery {
            page: None,
            per_page: None,
        },
        sort_by: None,
        sort_order: None,
    };

    let query_with_defaults = query.with_defaults();

    assert_eq!(query_with_defaults.q, Some("test".to_string()));
    assert_eq!(query_with_defaults.pagination.page, Some(1));
    assert_eq!(query_with_defaults.pagination.per_page, Some(20));
    assert!(query_with_defaults.sort_by.is_some());
    assert!(query_with_defaults.sort_order.is_some());
}
