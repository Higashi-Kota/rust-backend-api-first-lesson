# メールプロバイダー設定ガイド

## 概要

Task Backend は 3 つのメール送信モードをサポートしています：

1. **Development Mode** - コンソール出力のみ（開発時）
2. **MailHog** - 開発環境用 SMTP テストサーバー
3. **Mailgun** - 本番環境用メール配信サービス

## 設定方法

### 1. Development Mode（開発モード）

コンソールにメール内容を出力するだけで、実際のメール送信は行いません。

```bash
# .env
EMAIL_DEVELOPMENT_MODE=true
# または
EMAIL_PROVIDER=development
FROM_EMAIL=noreply@example.com
FROM_NAME=Task Backend Service
```

### 2. MailHog（開発環境用 SMTP）

MailHog は開発環境でメール送信をテストするための SMTP サーバーです。

#### MailHog の起動

Docker Compose での起動：

```yaml
# docker-compose.yml
version: "3.8"
services:
  mailhog:
    image: mailhog/mailhog:latest
    ports:
      - "1025:1025" # SMTP
      - "8025:8025" # Web UI
    environment:
      - MH_STORAGE=maildir
      - MH_MAILDIR_PATH=/tmp
```

```bash
docker-compose up mailhog -d
```

#### .env 設定

```bash
# .env
EMAIL_DEVELOPMENT_MODE=false
EMAIL_PROVIDER=mailhog

# MailHog Configuration
MAILHOG_HOST=localhost
MAILHOG_PORT=1025

# Email From Information
FROM_EMAIL=noreply@taskbackend.local
FROM_NAME=Task Backend Service
```

#### MailHog Web UI

メール受信確認: http://localhost:8025

### 3. Mailgun（本番環境用）

Mailgun はクラウドベースのメール配信サービスです。

#### Mailgun アカウント設定

1. [Mailgun](https://www.mailgun.com/)でアカウント作成
2. ドメインを追加・認証
3. API Key を取得

#### .env 設定

```bash
# .env
EMAIL_DEVELOPMENT_MODE=false
EMAIL_PROVIDER=mailgun

# Mailgun Configuration
MAILGUN_API_KEY=your-mailgun-api-key-here
MAILGUN_DOMAIN=mg.yourdomain.com

# Email From Information
FROM_EMAIL=noreply@yourdomain.com
FROM_NAME=Task Backend Service
```

## 環境別推奨設定

### 開発環境

```bash
# 開発時はMailHogを使用
EMAIL_DEVELOPMENT_MODE=false
EMAIL_PROVIDER=mailhog
MAILHOG_HOST=localhost
MAILHOG_PORT=1025
FROM_EMAIL=dev@taskbackend.local
FROM_NAME=Task Backend (Dev)
```

### ステージング環境

```bash
# ステージングではMailgunを使用
EMAIL_DEVELOPMENT_MODE=false
EMAIL_PROVIDER=mailgun
MAILGUN_API_KEY=your-staging-api-key
MAILGUN_DOMAIN=staging.yourdomain.com
FROM_EMAIL=staging@yourdomain.com
FROM_NAME=Task Backend (Staging)
```

### 本番環境

```bash
# 本番ではMailgunを使用
EMAIL_DEVELOPMENT_MODE=false
EMAIL_PROVIDER=mailgun
MAILGUN_API_KEY=your-production-api-key
MAILGUN_DOMAIN=mg.yourdomain.com
FROM_EMAIL=noreply@yourdomain.com
FROM_NAME=Task Backend
```

## 自動プロバイダー選択

環境変数`APP_ENV`または`RUST_ENV`に基づく自動選択：

```bash
APP_ENV=development → MailHog
APP_ENV=staging     → Mailgun
APP_ENV=production  → Mailgun
```

明示的に`EMAIL_PROVIDER`を設定すると、自動選択より優先されます。

## フラグによる動作制御

### EMAIL_DEVELOPMENT_MODE フラグ

```bash
# 開発環境でも実際のメール送信を行う場合
EMAIL_DEVELOPMENT_MODE=false
EMAIL_PROVIDER=mailgun
# Mailgun設定...
```

この設定により、開発環境でも Mailgun を使って実際にメール送信できます。

### テスト用設定例

開発環境で各プロバイダーをテストする場合：

```bash
# MailHogテスト
EMAIL_DEVELOPMENT_MODE=false
EMAIL_PROVIDER=mailhog

# Mailgunテスト（開発環境から本番メール送信）
EMAIL_DEVELOPMENT_MODE=false
EMAIL_PROVIDER=mailgun
MAILGUN_API_KEY=your-test-key
MAILGUN_DOMAIN=test.yourdomain.com
```

## 動作確認

### 1. MailHog での確認

```bash
# サーバー起動
make dev

# ユーザー登録（メール送信）
curl -X POST http://localhost:3000/auth/signup \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser",
    "email": "test@example.com",
    "password": "SecurePass123!"
  }'

# MailHog UI でメール確認
open http://localhost:8025
```

### 2. Mailgun での確認

```bash
# Mailgun設定でサーバー起動
EMAIL_PROVIDER=mailgun make run

# ユーザー登録（実際のメール送信）
curl -X POST http://localhost:3000/auth/signup \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser",
    "email": "your-real-email@example.com",
    "password": "SecurePass123!"
  }'

# 実際のメールボックスでメール確認
```

## トラブルシューティング

### MailHog 関連

```bash
# MailHogコンテナの状態確認
docker ps | grep mailhog

# MailHogログ確認
docker logs [mailhog-container-id]

# ポート競合確認
netstat -an | grep 1025
```

### Mailgun 関連

```bash
# API Key確認
curl -s --user 'api:YOUR_API_KEY' \
  https://api.mailgun.net/v3/domains

# ドメイン設定確認
curl -s --user 'api:YOUR_API_KEY' \
  https://api.mailgun.net/v3/YOUR_DOMAIN
```

### 一般的な問題

1. **メール送信されない**:

   - `EMAIL_DEVELOPMENT_MODE=true`になっていないか確認
   - プロバイダー設定が正しいか確認

2. **MailHog 接続エラー**:

   - MailHog が起動しているか確認
   - ポート 1025 が空いているか確認

3. **Mailgun 認証エラー**:
   - API Key が正しいか確認
   - ドメインが認証済みか確認

## Docker Compose での完全設定

```yaml
version: "3.8"
services:
  app:
    build: .
    ports:
      - "3000:3000"
    environment:
      - EMAIL_DEVELOPMENT_MODE=false
      - EMAIL_PROVIDER=mailhog
      - MAILHOG_HOST=mailhog
      - MAILHOG_PORT=1025
    depends_on:
      - postgres
      - mailhog

  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_DB: taskdb
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: password
    ports:
      - "5432:5432"

  mailhog:
    image: mailhog/mailhog:latest
    ports:
      - "1025:1025" # SMTP
      - "8025:8025" # Web UI
```

この設定により、開発環境で MailHog を使ったメール送信のテストができます。
