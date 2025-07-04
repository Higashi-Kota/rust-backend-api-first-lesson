# 🗄️ データベースマイグレーション運用手順書

本プロジェクトにおけるデータベーススキーマ変更の安全で再現性のある運用手順を定義します。

---

## 📋 目次

1. [運用の基本原則](#-運用の基本原則)
2. [事前準備](#-事前準備)
3. [本番環境マイグレーション手順](#-本番環境マイグレーション手順)
4. [エッジケース対応](#-エッジケース対応)
5. [ロールバック手順](#-ロールバック手順)
6. [検証とモニタリング](#-検証とモニタリング)
7. [トラブルシューティング](#-トラブルシューティング)

---

## 🎯 運用の基本原則

### 必須ルール

1. **本番環境への変更は必ず事前バックアップを実施**
2. **マイグレーションは段階的に適用し、各段階で検証**
3. **ロールバック計画を事前策定**
4. **メンテナンス時間外での実施を原則とする**
5. **すべての変更操作をログとして記録**

### 冪等性の確保

- すべてのマイグレーションは複数回実行しても同じ結果となるよう設計
- `IF NOT EXISTS` / `IF EXISTS` 句を適切に使用
- 既存データへの影響を最小限に抑制

---

## 🔧 事前準備

### 1. 環境変数の設定

```bash
# 本番環境
export DATABASE_URL="postgresql://user:password@host:port/database?sslmode=require"

# ステージング環境（テスト用）
export STAGING_DATABASE_URL="postgresql://user:password@staging-host:port/database?sslmode=require"
```

**注**: 開発環境では、プロジェクトルートの `.env` ファイルから `DATABASE_URL` が自動的に読み込まれます。

### 2. バックアップの作成

```bash
# 完全バックアップの作成
pg_dump "$DATABASE_URL" > "backup_$(date +%Y%m%d_%H%M%S).sql"

# スキーマのみのバックアップ（構造確認用）
pg_dump --schema-only "$DATABASE_URL" > "schema_backup_$(date +%Y%m%d_%H%M%S).sql"

# データのみのバックアップ（必要に応じて）
pg_dump --data-only "$DATABASE_URL" > "data_backup_$(date +%Y%m%d_%H%M%S).sql"
```

### 3. 現在の状態確認

```bash
# テーブル一覧の確認
psql "$DATABASE_URL" -c "\dt"

# インデックス一覧の確認
psql "$DATABASE_URL" -c "\di"

# マイグレーション状態の確認
sea-orm-cli migrate status

# 制約一覧の確認
psql "$DATABASE_URL" -c "
SELECT tc.table_name, tc.constraint_name, tc.constraint_type
FROM information_schema.table_constraints tc
ORDER BY tc.table_name, tc.constraint_type;
"
```

---

## 🚀 本番環境マイグレーション手順

### Phase 1: 事前検証

#### 1.1 ステージング環境での検証

```bash
# ステージング環境に本番データの最新コピーを作成
pg_dump "$DATABASE_URL" | psql "$STAGING_DATABASE_URL"

# ステージング環境でマイグレーション実行
DATABASE_URL="$STAGING_DATABASE_URL" sea-orm-cli migrate up

# 結果確認
DATABASE_URL="$STAGING_DATABASE_URL" sea-orm-cli migrate status
```

#### 1.2 マイグレーション計画の確認

```bash
# 適用予定のマイグレーション一覧
sea-orm-cli migrate status | grep -E "(Pending|Applied)"

# 新しいマイグレーションファイルの内容確認
find migration/src -name "m*.rs" -type f -exec echo "=== {} ===" \; -exec cat {} \;
```

### Phase 2: 本番環境適用

#### 2.1 メンテナンスモード設定

```bash
# アプリケーションサーバーの停止（必要に応じて）
# systemctl stop your-app-service

# 接続数の確認
psql "$DATABASE_URL" -c "
SELECT count(*) as active_connections 
FROM pg_stat_activity 
WHERE state = 'active' AND datname = 'your_database_name';
"
```

#### 2.2 マイグレーション実行

```bash
# タイムスタンプ付きログファイルの作成
LOG_FILE="migration_$(date +%Y%m%d_%H%M%S).log"

# マイグレーション実行（ログ出力付き）
{
    echo "=== Migration started at $(date) ==="
    echo "Database URL: $DATABASE_URL"
    echo "Git commit: $(git rev-parse HEAD)"
    echo ""
    
    # 現在の状態記録
    echo "--- Before migration ---"
    sea-orm-cli migrate status
    
    # マイグレーション実行
    echo "--- Executing migration ---"
    sea-orm-cli migrate up
    
    # 実行後の状態記録
    echo "--- After migration ---"
    sea-orm-cli migrate status
    
    echo "=== Migration completed at $(date) ==="
} 2>&1 | tee "$LOG_FILE"
```

#### 2.3 即座検証

```bash
# テーブル構造の確認
psql "$DATABASE_URL" -c "\d roles"
psql "$DATABASE_URL" -c "\d users"

# データの整合性確認
psql "$DATABASE_URL" -c "
SELECT 'roles' as table_name, count(*) as record_count FROM roles
UNION ALL
SELECT 'users', count(*) FROM users
UNION ALL
SELECT 'tasks', count(*) FROM tasks;
"

# 外部キー制約の確認
psql "$DATABASE_URL" -c "
SELECT tc.table_name, tc.constraint_name, tc.constraint_type, 
       ccu.table_name AS foreign_table_name,
       ccu.column_name AS foreign_column_name
FROM information_schema.table_constraints tc
JOIN information_schema.constraint_column_usage ccu 
    ON tc.constraint_name = ccu.constraint_name
WHERE tc.constraint_type = 'FOREIGN KEY'
ORDER BY tc.table_name;
"
```

---

## 🛠️ エッジケース対応

### Case 1: マイグレーション履歴の不整合

既存のオブジェクトが存在するがマイグレーション履歴に記録されていない場合：

```bash
# 1. 現在のDB状態とマイグレーション定義の比較
sea-orm-cli migrate status

# 2. 手動でマイグレーション履歴を同期
psql "$DATABASE_URL" -c "
INSERT INTO seaql_migrations (version, applied_at) VALUES 
('m20250511_073638_create_task_table', EXTRACT(epoch FROM NOW())::bigint),
('m20250612_000001_create_users_table', EXTRACT(epoch FROM NOW())::bigint)
ON CONFLICT (version) DO NOTHING;
"

# 3. 状態確認
sea-orm-cli migrate status
```

### Case 2: インデックス重複エラー

```bash
# エラー例: "relation 'idx_users_email' already exists"

# 1. 既存インデックスの確認
psql "$DATABASE_URL" -c "\di+ idx_users_email"

# 2. マイグレーションファイルの修正（.if_not_exists() 追加）
# migration/src/m20250612_000001_create_users_table.rs を編集

# 3. 再実行
sea-orm-cli migrate up
```

### Case 3: 外部キー制約違反

```bash
# 1. 制約違反データの特定
psql "$DATABASE_URL" -c "
SELECT u.id, u.email, u.role_id 
FROM users u 
LEFT JOIN roles r ON u.role_id = r.id 
WHERE u.role_id IS NOT NULL AND r.id IS NULL;
"

# 2. データ修正またはNULL設定
psql "$DATABASE_URL" -c "
UPDATE users 
SET role_id = NULL 
WHERE role_id NOT IN (SELECT id FROM roles);
"

# 3. マイグレーション再実行
sea-orm-cli migrate up
```

### Case 4: データ型変更の必要性

```bash
# 安全なデータ型変更手順

# 1. 新カラム追加
ALTER TABLE users ADD COLUMN new_column_name NEW_DATA_TYPE;

# 2. データ移行
UPDATE users SET new_column_name = CAST(old_column_name AS NEW_DATA_TYPE);

# 3. 制約追加
ALTER TABLE users ALTER COLUMN new_column_name SET NOT NULL;

# 4. 旧カラム削除（別のマイグレーションで実施）
ALTER TABLE users DROP COLUMN old_column_name;

# 5. カラム名変更
ALTER TABLE users RENAME COLUMN new_column_name TO old_column_name;
```

---

## 🔄 ロールバック手順

### 緊急ロールバック（データ復旧）

```bash
# 1. アプリケーション停止
# systemctl stop your-app-service

# 2. 完全データ復旧
psql "$DATABASE_URL" < "backup_YYYYMMDD_HHMMSS.sql"

# 3. 状態確認
psql "$DATABASE_URL" -c "\dt"
sea-orm-cli migrate status
```

### 段階的ロールバック（マイグレーション単位）

```bash
# 1. 特定マイグレーションのロールバック
sea-orm-cli migrate down -n 1

# 2. 複数マイグレーションのロールバック
sea-orm-cli migrate down -n 3

# 3. 全マイグレーションのロールバック
sea-orm-cli migrate reset
```

### カスタムロールバック手順

重要なデータ変更を伴う場合の手動ロールバック：

```bash
# 1. ロールバック前データバックアップ
pg_dump "$DATABASE_URL" > "rollback_backup_$(date +%Y%m%d_%H%M%S).sql"

# 2. 段階的ロールバック
# - 制約削除
# - インデックス削除  
# - カラム削除
# - テーブル削除

# 3. データ整合性確認
psql "$DATABASE_URL" -c "
-- データ整合性確認クエリ
SELECT 'Check passed' WHERE NOT EXISTS (
    -- 孤児レコードチェック等
);
"
```

---

## ✅ 検証とモニタリング

### マイグレーション成功の検証項目

```bash
# 1. テーブル構造確認
psql "$DATABASE_URL" -c "
SELECT table_name, column_name, data_type, is_nullable, column_default
FROM information_schema.columns 
WHERE table_schema = 'public' 
ORDER BY table_name, ordinal_position;
"

# 2. インデックス確認
psql "$DATABASE_URL" -c "
SELECT schemaname, tablename, indexname, indexdef 
FROM pg_indexes 
WHERE schemaname = 'public' 
ORDER BY tablename, indexname;
"

# 3. 制約確認
psql "$DATABASE_URL" -c "
SELECT tc.table_name, tc.constraint_name, tc.constraint_type, cc.check_clause
FROM information_schema.table_constraints tc
LEFT JOIN information_schema.check_constraints cc 
    ON tc.constraint_name = cc.constraint_name
WHERE tc.table_schema = 'public'
ORDER BY tc.table_name, tc.constraint_type;
"

# 4. データ件数確認
psql "$DATABASE_URL" -c "
SELECT 
    schemaname,
    tablename,
    n_tup_ins as inserts,
    n_tup_upd as updates,
    n_tup_del as deletes
FROM pg_stat_user_tables 
ORDER BY tablename;
"
```

### 継続監視項目

```bash
# 1. パフォーマンス監視
psql "$DATABASE_URL" -c "
SELECT schemaname, tablename, seq_scan, seq_tup_read, 
       idx_scan, idx_tup_fetch,
       CASE WHEN seq_scan > 0 THEN seq_tup_read/seq_scan ELSE 0 END as avg_seq_read
FROM pg_stat_user_tables 
ORDER BY seq_scan DESC;
"

# 2. インデックス使用状況
psql "$DATABASE_URL" -c "
SELECT schemaname, tablename, indexname, idx_scan, idx_tup_read, idx_tup_fetch
FROM pg_stat_user_indexes 
ORDER BY idx_scan DESC;
"

# 3. ロック状況監視
psql "$DATABASE_URL" -c "
SELECT pid, state, query_start, state_change, query 
FROM pg_stat_activity 
WHERE state <> 'idle' 
ORDER BY query_start;
"
```

---

## 🚨 トラブルシューティング

### よくある問題と解決方法

#### 問題1: "relation already exists" エラー

```bash
# 原因: オブジェクトが既に存在
# 解決: IF NOT EXISTS の追加

# マイグレーションファイル修正例
manager.create_table(
    Table::create()
        .table(TableName::Table)
        .if_not_exists()  // 追加
        .col(...)
        .to_owned(),
)
```

#### 問題2: 外部キー制約エラー

```bash
# 原因: 参照整合性違反
# 解決: データクリーニング

# 1. 問題データ特定
psql "$DATABASE_URL" -c "
SELECT child.id, child.foreign_key_column
FROM child_table child
LEFT JOIN parent_table parent ON child.foreign_key_column = parent.id
WHERE child.foreign_key_column IS NOT NULL AND parent.id IS NULL;
"

# 2. データ修正
psql "$DATABASE_URL" -c "
-- NULL設定または正しい値への更新
UPDATE child_table SET foreign_key_column = NULL 
WHERE foreign_key_column NOT IN (SELECT id FROM parent_table);
"
```

#### 問題3: マイグレーション実行タイムアウト

```bash
# 原因: 大量データでの処理時間超過
# 解決: バッチ処理への分割

# 例: 大量UPDATE処理
DO $$ 
DECLARE 
    batch_size INT := 10000;
    affected_rows INT;
BEGIN
    LOOP
        UPDATE large_table 
        SET column_name = new_value 
        WHERE condition 
        AND id IN (
            SELECT id FROM large_table 
            WHERE condition AND column_name != new_value 
            LIMIT batch_size
        );
        
        GET DIAGNOSTICS affected_rows = ROW_COUNT;
        EXIT WHEN affected_rows = 0;
        
        COMMIT;
        RAISE NOTICE 'Processed % rows', affected_rows;
    END LOOP;
END $$;
```

#### 問題4: ロールバック失敗

```bash
# 原因: 依存関係による削除順序エラー
# 解決: 手動での段階的削除

# 1. 外部キー制約削除
ALTER TABLE child_table DROP CONSTRAINT fk_constraint_name;

# 2. インデックス削除
DROP INDEX IF EXISTS index_name;

# 3. テーブル削除
DROP TABLE IF EXISTS table_name;
```

---

## 📝 運用チェックリスト

### マイグレーション実行前

- [ ] バックアップ完了確認
- [ ] ステージング環境での検証完了
- [ ] ロールバック手順準備完了
- [ ] メンテナンス時間確保
- [ ] 関係者への通知完了

### マイグレーション実行中

- [ ] ログ出力設定完了
- [ ] 各段階での検証実施
- [ ] 異常時の即座対応準備
- [ ] 進捗状況の定期確認

### マイグレーション実行後

- [ ] 全検証項目の確認完了
- [ ] パフォーマンステスト実施
- [ ] アプリケーション動作確認
- [ ] ログ保存と報告書作成
- [ ] 次回への改善点整理

---

## 🔗 関連ドキュメント

- [SeaORM Migration Documentation](https://www.sea-ql.org/SeaORM/docs/migration/setting-up-migration/)
- [PostgreSQL Documentation](https://www.postgresql.org/docs/)
- [DEPLOY.md](./DEPLOY.md) - デプロイメント手順
- [DEVELOPMENT.md](./DEVELOPMENT.md) - 開発環境セットアップ

---

*最終更新: 2025-06-15*
*作成者: 開発チーム*