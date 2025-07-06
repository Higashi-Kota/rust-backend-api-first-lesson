// tests/unit/utils/validation_tests.rs

use task_backend::utils::validation::common::{username, validate_username};

// バリデーション関連のユニットテスト（既存のsrc/utils/validation.rsのテストを拡張）

#[tokio::test]
async fn test_username_validation_with_service() {
    // Arrange: ユーザー名バリデーションをテスト
    let max_length_username = "a".repeat(30);
    let too_long_username = "a".repeat(31);
    let valid_usernames = vec![
        "user",
        "test_user",
        "user123",
        "valid-name",
        &max_length_username, // 最大長
    ];

    let invalid_usernames = vec![
        "ab",               // 短すぎる
        &too_long_username, // 長すぎる
        "user with spaces",
        "user@domain",
        "user#invalid",
    ];

    // Act & Assert: 有効なユーザー名の検証
    for username_str in &valid_usernames {
        // 長さの検証
        assert!(
            username_str.len() >= username::MIN_LENGTH as usize,
            "Username '{}' should be at least {} characters",
            username_str,
            username::MIN_LENGTH
        );
        assert!(
            username_str.len() <= username::MAX_LENGTH as usize,
            "Username '{}' should be at most {} characters",
            username_str,
            username::MAX_LENGTH
        );

        // validate_username関数を使った検証
        let result = validate_username(username_str);
        assert!(
            result.is_ok(),
            "Username '{}' should be valid, but got error: {:?}",
            username_str,
            result
        );
    }

    // Act & Assert: 無効なユーザー名の検証
    for username_str in &invalid_usernames {
        let result = validate_username(username_str);
        assert!(
            result.is_err(),
            "Username '{}' should be invalid",
            username_str
        );
    }
}

#[tokio::test]
async fn test_input_sanitization_with_validation() {
    use task_backend::utils::validation::common::{
        validate_not_empty_or_whitespace, validate_task_title,
    };

    // Arrange: 入力サニタイゼーションをテスト
    let dangerous_inputs = vec![
        "<script>alert('xss')</script>",
        "'; DROP TABLE users; --",
        "Valid but has SQL",
        "  leading and trailing spaces  ",
    ];

    let invalid_for_task_title = vec!["Title with\nnewline", "Title with\0null", "\t\n", "   ", ""];

    // Act & Assert: 危険な入力でも空白でなければvalidate_not_empty_or_whitespaceは通る
    for input in &dangerous_inputs {
        let result = validate_not_empty_or_whitespace(input.trim());
        if input.trim().is_empty() {
            assert!(result.is_err(), "Empty input should fail validation");
        } else {
            assert!(
                result.is_ok(),
                "Non-empty input should pass basic validation"
            );
        }
    }

    // Act & Assert: タスクタイトルの検証（特殊文字のチェック）
    for input in &invalid_for_task_title {
        let result = validate_task_title(input);
        assert!(
            result.is_err(),
            "Input '{}' should fail task title validation",
            input.escape_debug()
        );
    }

    // 有効なタスクタイトルの検証
    let valid_titles = vec![
        "Normal Task Title",
        "Task with symbols: !@#$%^&*()",
        "<script>alert('xss')</script>", // HTMLタグも許可される（エスケープは別レイヤーで）
    ];

    for title in &valid_titles {
        let result = validate_task_title(title);
        assert!(result.is_ok(), "Title '{}' should be valid", title);
    }
}

#[tokio::test]
async fn test_uuid_validation_with_parser() {
    use uuid::Uuid;

    // Arrange: UUID バリデーションをテスト
    let valid_uuids = vec![
        "550e8400-e29b-41d4-a716-446655440000",
        "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
        "123e4567-e89b-12d3-a456-426614174000",
    ];

    let invalid_uuids = vec![
        "",
        "not-a-uuid",
        "550e8400-e29b-41d4-a716",                    // 短すぎる
        "550e8400-e29b-41d4-a716-446655440000-extra", // 長すぎる
        "gggggggg-gggg-gggg-gggg-gggggggggggg",       // 無効な文字
    ];

    // Act & Assert: 有効なUUIDの検証
    for uuid_str in &valid_uuids {
        let result = Uuid::parse_str(uuid_str);
        assert!(result.is_ok(), "UUID '{}' should be valid", uuid_str);

        // パースしたUUIDを文字列に戻して比較
        let parsed_uuid = result.unwrap();
        assert_eq!(
            parsed_uuid.to_string(),
            *uuid_str,
            "Parsed UUID should match original"
        );

        // UUIDがnilでないことを確認
        assert_ne!(parsed_uuid, Uuid::nil(), "UUID should not be nil");
    }

    // Act & Assert: 無効なUUIDの検証
    for uuid_str in &invalid_uuids {
        let result = Uuid::parse_str(uuid_str);
        assert!(result.is_err(), "UUID '{}' should be invalid", uuid_str);
    }

    // 特殊なUUIDのテスト
    let nil_uuid = Uuid::nil();
    assert_eq!(nil_uuid.to_string(), "00000000-0000-0000-0000-000000000000");

    // 新しいUUIDの生成と検証
    let new_uuid = Uuid::new_v4();
    assert_ne!(new_uuid, Uuid::nil(), "New UUID should not be nil");
    assert_eq!(
        new_uuid.to_string().len(),
        36,
        "UUID string should be 36 characters"
    );
}
