// tests/unit/utils/password_tests.rs

use task_backend::utils::password::{Argon2Config, PasswordManager, PasswordPolicy};

// パスワード関連のユニットテスト（既存のsrc/utils/password.rsのテストを拡張）

#[tokio::test]
async fn test_password_strength_validation_with_manager() {
    // Arrange: パスワード強度検証をテスト
    let policy = PasswordPolicy::default();
    let argon2_config = Argon2Config::default();
    let password_manager = PasswordManager::new(argon2_config, policy).unwrap();

    let weak_passwords = vec![
        "123",          // 短すぎる
        "password",     // 大文字・数字・特殊文字なし
        "Password",     // 数字・特殊文字なし
        "Password123",  // 特殊文字なし
        "password123!", // 大文字なし
        "PASSWORD123!", // 小文字なし
        "Password!",    // 数字なし
    ];

    let strong_passwords = vec![
        "MyS3cur3!P@ss",
        "C0mpl3x#P@ss9",
        "Str0ng!P@ss7",
        "S3cur3@2024X",
        "T3st!ng@Now5",
    ];

    // Act & Assert: 弱いパスワードの検証
    for password in &weak_passwords {
        let result = password_manager.validate_password_strength(password);
        assert!(
            result.is_err(),
            "Password '{}' should be weak, but validation passed",
            password
        );
    }

    // Act & Assert: 強いパスワードの検証
    for password in &strong_passwords {
        let result = password_manager.validate_password_strength(password);
        assert!(
            result.is_ok(),
            "Password '{}' should be strong, but got error: {:?}",
            password,
            result
        );
    }

    // 特定のエラーメッセージの確認
    let short_password_result = password_manager.validate_password_strength("Abc1!");
    assert!(short_password_result.is_err());
    assert!(short_password_result
        .unwrap_err()
        .contains("at least 8 characters"));

    let common_password_result = password_manager.validate_password_strength("password@2024");
    // passwordという一般的な単語を含む
    assert!(common_password_result.is_err());
    assert!(common_password_result.unwrap_err().contains("too common"));
}

#[tokio::test]
async fn test_password_hashing_and_verification() {
    // Arrange: パスワードハッシュ化と検証をテスト
    let policy = PasswordPolicy::default();
    let argon2_config = Argon2Config::default();
    let password_manager = PasswordManager::new(argon2_config, policy).unwrap();

    let original_password = "S3cur3!T3st@2024";

    // Act: パスワードをハッシュ化
    let hash_result = password_manager.hash_password(original_password);
    assert!(hash_result.is_ok(), "Password hashing should succeed");

    let hash = hash_result.unwrap();

    // Assert: ハッシュの特性を検証
    assert_ne!(
        original_password, hash,
        "Hash should be different from original password"
    );
    assert!(!hash.is_empty(), "Hash should not be empty");
    assert!(
        hash.starts_with("$argon2"),
        "Hash should be in Argon2 format"
    );

    // Act & Assert: 正しいパスワードでの検証
    let verify_result = password_manager.verify_password(original_password, &hash);
    assert!(
        verify_result.is_ok(),
        "Password verification should succeed"
    );
    assert!(
        verify_result.unwrap(),
        "Correct password should verify as true"
    );

    // Act & Assert: 間違ったパスワードでの検証
    let wrong_verify_result = password_manager.verify_password("WrongPassword123!", &hash);
    assert!(
        wrong_verify_result.is_ok(),
        "Wrong password verification should not error"
    );
    assert!(
        !wrong_verify_result.unwrap(),
        "Wrong password should verify as false"
    );

    // Act & Assert: 同じパスワードでも異なるハッシュが生成されることを確認
    let hash2 = password_manager.hash_password(original_password).unwrap();
    assert_ne!(
        hash, hash2,
        "Same password should generate different hashes due to salt"
    );

    // 両方のハッシュが正しく検証されることを確認
    assert!(password_manager
        .verify_password(original_password, &hash2)
        .unwrap());

    // Act & Assert: 再ハッシュ化が必要かのチェック
    let needs_rehash_result = password_manager.needs_rehash(&hash);
    assert!(
        needs_rehash_result.is_ok(),
        "Needs rehash check should succeed"
    );
}
