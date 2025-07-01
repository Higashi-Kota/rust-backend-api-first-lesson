# 環境変数設定ガイド

このドキュメントは、Task Backend で使用可能なすべての環境変数を網羅的に説明します。

## 📋 目次

1. [必須環境変数](#必須環境変数)
2. [アプリケーション設定](#アプリケーション設定)
3. [サーバー設定](#サーバー設定)
4. [データベース設定](#データベース設定)
5. [JWT認証設定](#jwt認証設定)
6. [メール設定](#メール設定)
7. [パスワードポリシー設定](#パスワードポリシー設定)
8. [パスワードハッシュ設定](#パスワードハッシュ設定argon2)
9. [ロギング設定](#ロギング設定)
10. [環境別設定例](#環境別設定例)

## 必須環境変数

以下の環境変数は必ず設定する必要があります：

| 環境変数 | 説明 | 例 |
|---------|------|-----|
| `DATABASE_URL` | PostgreSQL接続URL | `postgres://user:pass@localhost:5432/db` |
| `JWT_SECRET_KEY` | JWT署名用秘密鍵（32文字以上） | `your-super-secret-key-at-least-32-chars` |

## アプリケーション設定

### 環境モード

| 環境変数 | デフォルト値 | 説明 | 値 |
|---------|------------|------|-----|
| `APP_ENV` | `development` | アプリケーション環境 | `development`, `staging`, `production` |
| `RUST_ENV` | `development` | Rust環境（APP_ENVの代替） | `development`, `staging`, `production` |
| `RUST_TEST` | - | テスト実行時に自動設定 | - |

> **注意**: `APP_ENV` と `RUST_ENV` は同じ目的で使用されます。両方設定されている場合は `APP_ENV` が優先されます。

## サーバー設定

| 環境変数 | デフォルト値 | 説明 | 例 |
|---------|------------|------|-----|
| `SERVER_ADDR` | `0.0.0.0:3000` | サーバーのバインドアドレス | `127.0.0.1:8080` |

## データベース設定

| 環境変数 | デフォルト値 | 説明 | 例 |
|---------|------------|------|-----|
| `DATABASE_URL` | **必須** | PostgreSQL接続URL | `postgres://postgres:password@localhost:5432/taskdb` |
| `DB_SCHEMA` | - | データベーススキーマ名（マルチテナント用） | `tenant_1` |

### DATABASE_URLの形式
```
postgres://[ユーザー名]:[パスワード]@[ホスト]:[ポート]/[データベース名]
```

## JWT認証設定

| 環境変数 | デフォルト値 | 説明 | 制約 |
|---------|------------|------|------|
| `JWT_SECRET_KEY` | **必須** | JWT署名用秘密鍵 | 32文字以上 |
| `JWT_ACCESS_TOKEN_EXPIRY_MINUTES` | `15` | アクセストークン有効期限（分） | 1以上 |
| `JWT_REFRESH_TOKEN_EXPIRY_DAYS` | `7` | リフレッシュトークン有効期限（日） | 1以上 |
| `JWT_ISSUER` | `task-backend` | JWT発行者識別子 | - |
| `JWT_AUDIENCE` | `task-backend-users` | JWT対象者識別子 | - |

### セキュリティ推奨事項
- `JWT_SECRET_KEY` は本番環境では必ず強力なランダム文字列を使用してください
- 定期的に秘密鍵をローテーションすることを推奨します

## メール設定

### 基本設定

| 環境変数 | デフォルト値 | 説明 | 値 |
|---------|------------|------|-----|
| `EMAIL_PROVIDER` | 環境依存* | メールプロバイダー | `development`, `mailhog`, `mailgun` |
| `FROM_EMAIL` | `noreply@example.com` | 送信元メールアドレス | - |
| `FROM_NAME` | `Task Backend` | 送信者名 | - |

*デフォルト値：
- `RUST_ENV=development` → `development`
- それ以外 → `mailhog`

### SMTP設定（MailHog用）

| 環境変数 | デフォルト値 | 説明 | 必要条件 |
|---------|------------|------|---------|
| `SMTP_HOST` | `localhost` | SMTPサーバーホスト | EMAIL_PROVIDER=mailhog |
| `SMTP_PORT` | `1025` | SMTPサーバーポート | EMAIL_PROVIDER=mailhog |

### Mailgun設定

| 環境変数 | デフォルト値 | 説明 | 必要条件 |
|---------|------------|------|---------|
| `MAILGUN_API_KEY` | - | Mailgun APIキー | EMAIL_PROVIDER=mailgun |
| `MAILGUN_DOMAIN` | - | Mailgunドメイン | EMAIL_PROVIDER=mailgun |

### プロバイダー別動作

1. **development**: コンソールにメール内容を出力（実際の送信なし）
2. **mailhog**: ローカルSMTPサーバー経由で送信（開発環境用）
3. **mailgun**: Mailgun API経由で実際にメール送信（本番環境用）

## パスワードポリシー設定

| 環境変数 | デフォルト値 | 説明 | 値 |
|---------|------------|------|-----|
| `PASSWORD_MIN_LENGTH` | `8` | 最小文字数 | 1以上の整数 |
| `PASSWORD_MAX_LENGTH` | `128` | 最大文字数 | MIN_LENGTH以上の整数 |
| `PASSWORD_REQUIRE_UPPERCASE` | `true` | 大文字必須 | `true`, `false` |
| `PASSWORD_REQUIRE_LOWERCASE` | `true` | 小文字必須 | `true`, `false` |
| `PASSWORD_REQUIRE_DIGIT` | `true` | 数字必須 | `true`, `false` |
| `PASSWORD_REQUIRE_SPECIAL` | `true` | 特殊文字必須 | `true`, `false` |
| `PASSWORD_CHECK_COMMON` | `true` | 一般的なパスワードチェック | `true`, `false` |

### 特殊文字の定義
`!@#$%^&*()_+-=[]{}|;:,.<>?`

## パスワードハッシュ設定（Argon2）

| 環境変数 | デフォルト値 | 説明 | 推奨範囲 |
|---------|------------|------|---------|
| `ARGON2_MEMORY_COST` | `65536` | メモリコスト（KB） | 32768-1048576 |
| `ARGON2_TIME_COST` | `3` | 時間コスト（反復回数） | 1-10 |
| `ARGON2_PARALLELISM` | `4` | 並列度 | 1-8 |
| `ARGON2_OUTPUT_LENGTH` | `32` | ハッシュ長（バイト） | 16-64 |

### パフォーマンスとセキュリティのバランス
- **開発環境**: デフォルト値で十分
- **本番環境**: メモリとCPUリソースに応じて調整を推奨

## ロギング設定

| 環境変数 | デフォルト値 | 説明 |
|---------|------------|------|
| `RUST_LOG` | `task_backend=info,tower_http=info,axum::rejection=trace` | ログレベル設定 |

### ログレベル
- `error`: エラーのみ
- `warn`: 警告以上
- `info`: 情報以上（推奨）
- `debug`: デバッグ情報を含む
- `trace`: 詳細なトレース情報

### モジュール別設定例
```bash
RUST_LOG=task_backend=debug,tower_http=warn,sqlx=info
```

## 環境別設定例

### 開発環境 (.env.development)

```bash
# 基本設定
APP_ENV=development
DATABASE_URL=postgres://postgres:password@localhost:5432/taskdb_dev

# JWT設定（開発用の簡単な秘密鍵）
JWT_SECRET_KEY=development-secret-key-32-characters-long
JWT_ACCESS_TOKEN_EXPIRY_MINUTES=60
JWT_REFRESH_TOKEN_EXPIRY_DAYS=30

# メール設定（MailHog使用）
EMAIL_PROVIDER=mailhog
SMTP_HOST=localhost
SMTP_PORT=1025
FROM_EMAIL=dev@taskbackend.local
FROM_NAME=Task Backend (Dev)

# パスワードポリシー（開発用に緩和）
PASSWORD_MIN_LENGTH=6
PASSWORD_REQUIRE_SPECIAL=false
PASSWORD_CHECK_COMMON=false

# ロギング（詳細）
RUST_LOG=task_backend=debug,tower_http=debug
```

### ステージング環境 (.env.staging)

```bash
# 基本設定
APP_ENV=staging
DATABASE_URL=postgres://taskuser:staging-password@staging-db.example.com:5432/taskdb_staging

# JWT設定
JWT_SECRET_KEY=staging-secret-key-please-change-in-production
JWT_ACCESS_TOKEN_EXPIRY_MINUTES=30
JWT_REFRESH_TOKEN_EXPIRY_DAYS=14

# メール設定（Mailgunテスト）
EMAIL_PROVIDER=mailgun
MAILGUN_API_KEY=your-staging-mailgun-api-key
MAILGUN_DOMAIN=staging.yourdomain.com
FROM_EMAIL=noreply@staging.yourdomain.com
FROM_NAME=Task Backend (Staging)

# パスワードポリシー（本番相当）
PASSWORD_MIN_LENGTH=8
PASSWORD_REQUIRE_UPPERCASE=true
PASSWORD_REQUIRE_LOWERCASE=true
PASSWORD_REQUIRE_DIGIT=true
PASSWORD_REQUIRE_SPECIAL=true
PASSWORD_CHECK_COMMON=true

# ロギング
RUST_LOG=task_backend=info,tower_http=info
```

### 本番環境 (.env.production)

```bash
# 基本設定
APP_ENV=production
DATABASE_URL=postgres://taskuser:secure-production-password@prod-db.example.com:5432/taskdb_prod
DB_SCHEMA=public

# JWT設定（強力な秘密鍵）
JWT_SECRET_KEY=${JWT_SECRET_KEY}  # 環境変数から取得
JWT_ACCESS_TOKEN_EXPIRY_MINUTES=15
JWT_REFRESH_TOKEN_EXPIRY_DAYS=7
JWT_ISSUER=task-backend-prod
JWT_AUDIENCE=task-backend-users

# メール設定（Mailgun本番）
EMAIL_PROVIDER=mailgun
MAILGUN_API_KEY=${MAILGUN_API_KEY}  # 環境変数から取得
MAILGUN_DOMAIN=mg.yourdomain.com
FROM_EMAIL=noreply@yourdomain.com
FROM_NAME=Task Backend

# パスワードポリシー（最高セキュリティ）
PASSWORD_MIN_LENGTH=10
PASSWORD_MAX_LENGTH=128
PASSWORD_REQUIRE_UPPERCASE=true
PASSWORD_REQUIRE_LOWERCASE=true
PASSWORD_REQUIRE_DIGIT=true
PASSWORD_REQUIRE_SPECIAL=true
PASSWORD_CHECK_COMMON=true

# Argon2設定（高セキュリティ）
ARGON2_MEMORY_COST=131072
ARGON2_TIME_COST=4
ARGON2_PARALLELISM=4

# ロギング（エラーと警告のみ）
RUST_LOG=task_backend=warn,tower_http=warn

# サーバー設定
SERVER_ADDR=0.0.0.0:8080
```

## Docker Compose での環境変数設定

```yaml
version: '3.8'
services:
  app:
    image: task-backend:latest
    environment:
      - DATABASE_URL=postgres://postgres:password@db:5432/taskdb
      - JWT_SECRET_KEY=${JWT_SECRET_KEY}
      - EMAIL_PROVIDER=mailgun
      - MAILGUN_API_KEY=${MAILGUN_API_KEY}
      - MAILGUN_DOMAIN=${MAILGUN_DOMAIN}
    env_file:
      - .env
```

## セキュリティベストプラクティス

1. **機密情報の管理**
   - `JWT_SECRET_KEY` や `MAILGUN_API_KEY` などの機密情報は、環境変数や秘密管理システムから取得
   - `.env` ファイルは絶対にGitにコミットしない
   - 本番環境では環境変数を直接設定するか、秘密管理ツール（AWS Secrets Manager、HashiCorp Vault等）を使用

2. **環境の分離**
   - 開発、ステージング、本番環境で異なる設定を使用
   - 本番環境のデータベースURLやAPIキーは開発者がアクセスできないように管理

3. **定期的なローテーション**
   - JWT秘密鍵は定期的に更新
   - APIキーも定期的にローテーション

## トラブルシューティング

### よくある問題

1. **環境変数が読み込まれない**
   ```bash
   # .envファイルの存在確認
   ls -la .env
   
   # 環境変数の確認
   echo $DATABASE_URL
   ```

2. **JWTエラー**
   - `JWT_SECRET_KEY` が32文字以上であることを確認
   - 環境間で同じ秘密鍵を使用していないか確認

3. **メール送信エラー**
   - `EMAIL_PROVIDER` の設定を確認
   - Mailgunの場合、APIキーとドメインが正しいか確認
   - MailHogの場合、サービスが起動しているか確認

4. **データベース接続エラー**
   - `DATABASE_URL` の形式が正しいか確認
   - データベースサーバーが起動しているか確認
   - ネットワーク接続を確認

### デバッグ用環境変数

開発時のデバッグに便利な設定：

```bash
# 詳細ログ出力
RUST_LOG=debug

# SQLクエリのログ出力
RUST_LOG=sqlx=debug,task_backend=debug

# 特定モジュールのトレース
RUST_LOG=task_backend::auth=trace
```

## 更新履歴

- 2024-01-XX: 初版作成
- 最新の情報は常にコードベースを参照してください