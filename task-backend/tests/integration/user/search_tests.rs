// task-backend/tests/integration/user/search_tests.rs

use task_backend::features::user::dto::requests::{SortOrder, UserSearchQuery, UserSortField};
use validator::Validate;

#[test]
fn test_user_search_query_validation() {
    let query = UserSearchQuery {
        q: Some("test".to_string()),
        is_active: Some(true),
        email_verified: Some(true),
        page: Some(1),
        per_page: Some(20),
        sort_by: Some(UserSortField::Username),
        sort_order: Some(SortOrder::Ascending),
    };

    // 有効なクエリのバリデーションテスト
    assert!(query.validate().is_ok());

    // 不正なページ番号のテスト
    let invalid_query = UserSearchQuery {
        page: Some(0),
        ..query.clone()
    };
    assert!(invalid_query.validate().is_err());

    // 不正なper_page値のテスト
    let invalid_query = UserSearchQuery {
        per_page: Some(101),
        ..query.clone()
    };
    assert!(invalid_query.validate().is_err());

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
        page: None,
        per_page: None,
        sort_by: None,
        sort_order: None,
    };

    let query_with_defaults = query.with_defaults();

    assert_eq!(query_with_defaults.q, Some("test".to_string()));
    assert_eq!(query_with_defaults.page, Some(1));
    assert_eq!(query_with_defaults.per_page, Some(20));
    assert!(query_with_defaults.sort_by.is_some());
    assert!(query_with_defaults.sort_order.is_some());
}
