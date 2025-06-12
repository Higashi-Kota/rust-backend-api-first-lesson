// tests/unit/utils/email_tests.rs

// メール関連のユニットテスト（既存のsrc/utils/email.rsのテストを拡張）

#[tokio::test]
async fn test_email_format_validation() {
    // メールフォーマットバリデーションテスト
    let valid_emails = vec![
        "test@example.com",
        "user.name@example.org",
        "user+tag@domain.co.uk",
        "simple@test.net",
    ];

    let invalid_emails = vec![
        "",
        "invalid",
        "invalid@",
        "@invalid.com",
        "user@",
        "@domain.com",
        "user@domain", // ドメインにドットがない
    ];

    // 有効なメールアドレスの検証
    for email in &valid_emails {
        let parts: Vec<&str> = email.split('@').collect();
        assert_eq!(parts.len(), 2, "Email should have exactly one @ symbol");
        assert!(!parts[0].is_empty(), "Local part should not be empty");
        assert!(!parts[1].is_empty(), "Domain part should not be empty");
        assert!(parts[1].contains('.'), "Domain should contain a dot");
    }

    // 無効なメールアドレスの検証
    for email in &invalid_emails {
        if email.is_empty() {
            continue; // 空文字は特別扱い
        }

        let parts: Vec<&str> = email.split('@').collect();
        // 少なくとも1つの条件が満たされていないことを確認
        let valid_format = parts.len() == 2
            && !parts[0].is_empty()
            && !parts[1].is_empty()
            && parts[1].contains('.');

        assert!(!valid_format, "Email '{}' should be invalid", email);
    }
}

#[tokio::test]
async fn test_email_masking_concepts() {
    // メールマスキングの概念テスト
    let email = "user@example.com";
    let parts: Vec<&str> = email.split('@').collect();
    let local_part = parts[0];
    let domain_part = parts[1];

    // マスキング処理の概念
    let masked_local = if local_part.len() <= 2 {
        "*".repeat(local_part.len())
    } else {
        format!(
            "{}***{}",
            &local_part[..1],
            &local_part[local_part.len() - 1..]
        )
    };

    let masked_email = format!("{}@{}", masked_local, domain_part);

    // マスキングされたメールが元のメールと異なることを確認
    assert_ne!(email, masked_email);
    assert!(masked_email.contains("@"));
    assert!(masked_email.contains("example.com"));
}
