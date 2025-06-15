# ポストモータム: データベースマイグレーション問題

**インシデント発生日**: 2025-06-15  
**解決完了日**: 2025-06-15  
**影響度**: 中（本番環境のマイグレーション失敗、管理者ログイン不可）  
**ステータス**: 解決済み

---

## 概要

本番環境でのデータベースマイグレーション適用時に複数のエラーが発生し、`roles` テーブルが作成されず、管理者アカウントへのログインができない状態となった。

---

## 発生した問題

### 1. マイグレーション実行エラー

```bash
# 発生したエラー例
Execution Error: error returned from database: relation "idx_users_email" already exists
Execution Error: error returned from database: column "user_id" of relation "tasks" already exists
```

### 2. テーブル不整合

- 本番データベースに `roles` テーブルが存在しない
- `users` テーブルに `role_id` カラムが存在しない
- 管理者アカウントが正常に機能しない

### 3. 認証失敗

```bash
# ログで確認されたエラー
Password verification failed user_id f6459929-eac1-44b3-9227-c76970c4664f
Internal server error: Authentication failed
```

---

## 根本原因分析

### 主要原因: マイグレーション履歴と実際のDB状態の不整合

1. **開発初期の手動DB操作**
   - 開発段階でマイグレーション管理を導入する前に、手動SQLでテーブルやインデックスを作成
   - 本番環境への初回デプロイ時も手動適用を実施

2. **マイグレーション履歴の欠損**
   ```sql
   -- seaql_migrations テーブルが空の状態
   SELECT * FROM seaql_migrations;
   -- (0 rows)
   ```

3. **冪等性の不備**
   - マイグレーションファイルで `IF NOT EXISTS` / `IF EXISTS` 句の使用不足
   - 既存オブジェクトとの衝突回避ができていない

### 副次的原因

1. **パスワードハッシュの不一致**
   - 手動作成時と正式マイグレーション時でArgon2ハッシュが異なる
   - 管理者アカウントでのログイン失敗

2. **環境間での操作手順の差異**
   - 開発環境: マイグレーション使用
   - 本番環境: 手動SQL実行

---

## 実施した対応

### 1. 緊急対応（即時復旧）

#### 1.1 マイグレーション履歴の同期
```sql
-- 既存マイグレーションを履歴に記録
INSERT INTO seaql_migrations (version, applied_at) VALUES 
('m20250511_073638_create_task_table', EXTRACT(epoch FROM NOW())::bigint),
('m20250612_000001_create_users_table', EXTRACT(epoch FROM NOW())::bigint),
('m20250512_000001_add_task_indexes', EXTRACT(epoch FROM NOW())::bigint),
('m20250612_000002_create_refresh_tokens_table', EXTRACT(epoch FROM NOW())::bigint),
('m20250612_000003_create_password_reset_tokens_table', EXTRACT(epoch FROM NOW())::bigint),
('m20250612_000004_add_user_id_to_tasks', EXTRACT(epoch FROM NOW())::bigint)
ON CONFLICT (version) DO NOTHING;
```

#### 1.2 不足テーブル・カラムの手動作成
```sql
-- rolesテーブル作成
CREATE TABLE IF NOT EXISTS roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(50) NOT NULL UNIQUE,
    display_name VARCHAR(100) NOT NULL,
    description TEXT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 初期データ投入
INSERT INTO roles (name, display_name, description, is_active) VALUES 
('admin', 'Administrator', 'System administrator with full access', true),
('member', 'Member', 'Regular user with basic functionality', true)
ON CONFLICT (name) DO NOTHING;

-- usersテーブルにrole_idカラム追加
ALTER TABLE users ADD COLUMN IF NOT EXISTS role_id UUID;
ALTER TABLE users ADD CONSTRAINT fk_users_role_id 
    FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE SET NULL ON UPDATE CASCADE;
```

#### 1.3 管理者アカウントの修正
```sql
-- 正しいパスワードハッシュで更新
UPDATE users 
SET password_hash = '$argon2id$v=19$m=65536,t=3,p=4$rwjnw7itO1QP7YiQLYYPuw$bwYljZ/eNoieCwcPydAbagPt05UT9wcs+n0zH58ZxS4',
    role_id = (SELECT id FROM roles WHERE name = 'admin')
WHERE email = 'admin@example.com';
```

### 2. 恒久対策（再発防止）

#### 2.1 マイグレーションファイルの冪等性確保
```rust
// 修正前
manager.create_index(
    Index::create()
        .name("idx_users_email")
        .col(Users::Email)
        .to_owned(),
)

// 修正後
manager.create_index(
    Index::create()
        .if_not_exists()  // 追加
        .name("idx_users_email")
        .col(Users::Email)
        .to_owned(),
)
```

#### 2.2 包括的運用手順書の作成
- `MIGRATION.md` の作成
- バックアップ・検証・ロールバック手順の標準化
- エッジケース対応の文書化

---

## タイムライン

| 時刻 | イベント | 対応者 | アクション |
|------|----------|--------|------------|
| 04:54 | マイグレーション実行でエラー発生 | 開発チーム | 問題の特定開始 |
| 05:00 | 根本原因特定完了 | 開発チーム | 対応計画策定 |
| 05:15 | マイグレーション履歴同期完了 | 開発チーム | 緊急復旧作業 |
| 05:30 | rolesテーブル作成完了 | 開発チーム | 機能復旧 |
| 05:35 | 管理者ログイン復旧確認 | 開発チーム | 検証完了 |
| 05:45 | 包括的ドキュメント作成開始 | 開発チーム | 再発防止策 |

---

## 影響度評価

### システムへの影響
- **影響範囲**: 本番環境のDBマイグレーション
- **サービス停止**: なし（マイグレーション中の短時間のみ）
- **データ損失**: なし
- **復旧時間**: 約40分

### ユーザーへの影響
- **影響ユーザー数**: 管理者のみ（1アカウント）
- **機能制限**: 管理者機能の一時的な利用不可
- **通知**: 社内向けのみ

---

## 学んだ教訓

### 1. マイグレーション管理の重要性
- **手動DB操作は避ける**: すべてのスキーマ変更をマイグレーションで管理
- **履歴の一貫性**: 開発・ステージング・本番で同一手順を適用
- **冪等性の確保**: 複数回実行しても安全な設計

### 2. 環境間の一貫性
- **開発環境**: 必ずマイグレーションを使用
- **本番環境**: 開発環境と同一手順で適用
- **ドキュメント化**: 手順の標準化と共有

### 3. バックアップとテストの重要性
- **事前バックアップ**: すべてのマイグレーション前に実施
- **ステージング検証**: 本番適用前の必須確認
- **ロールバック計画**: 失敗時の復旧手順を事前準備

### 4. Argon2パスワードハッシュの性質
- **ソルト含有**: 同じパスワードでも実行ごとに異なるハッシュが生成される
- **マイグレーション内容の重要性**: 事前定義されたハッシュ値を使用すべき
- **一貫性の確保**: 開発・本番で同一のハッシュ値を使用

---

## 改善アクション

### 実施済み

1. **マイグレーションファイルの修正**
   - 全インデックス作成に `if_not_exists()` 追加
   - 全インデックス削除に `if_exists()` 追加

2. **包括的運用手順書の作成**
   - `MIGRATION.md` の作成
   - エッジケース対応手順の文書化

3. **管理者アカウントの復旧**
   - マイグレーションファイルと同一のパスワードハッシュで更新
   - 適切なロール割り当て

### 今後の実施計画

1. **CI/CDパイプラインの改善**
   - マイグレーション実行の自動化
   - ステージング環境での自動検証

2. **監視とアラートの設定**
   - マイグレーション失敗時の即座通知
   - データベース状態の継続監視

3. **定期的な整合性チェック**
   - マイグレーション履歴と実際のスキーマの比較
   - 月次での検証作業

---

## 参考資料

- [SeaORM Migration Documentation](https://www.sea-ql.org/SeaORM/docs/migration/setting-up-migration/)
- [PostgreSQL Documentation - DDL](https://www.postgresql.org/docs/current/ddl.html)
- [Argon2 Password Hashing](https://github.com/P-H-C/phc-winner-argon2)

---

## 関連ドキュメント

- [MIGRATION.md](./MIGRATION.md) - データベースマイグレーション運用手順書
- [DEPLOY.md](./DEPLOY.md) - デプロイメント手順
- [DEVELOPMENT.md](./DEVELOPMENT.md) - 開発環境セットアップ

---

**作成者**: 開発チーム  
**最終更新**: 2025-06-15  
**次回レビュー予定**: 2025-07-15