# メール送信機能ドキュメント

## 概要

Task Backendには包括的なメール送信機能が実装されており、ユーザーの重要なアクションに対して自動的にメール通知を送信します。開発環境ではMailHog、本番環境ではMailgunを使用した柔軟なプロバイダー対応により、環境に応じた最適なメール送信を実現します。

## サポートするメール送信シナリオ

### 1. ユーザー登録時（Welcome Email）
- **タイミング**: 新規ユーザー登録成功後
- **内容**: サービスの歓迎メッセージ、初期設定ガイド
- **実装場所**: `src/service/auth_service.rs` - `signup()` メソッド

### 2. サインイン成功時（Security Notification）
- **タイミング**: ユーザーログイン成功後
- **内容**: セキュリティ通知、ログイン時刻と場所の記録
- **実装場所**: `src/service/auth_service.rs` - `signin()` メソッド

### 3. パスワード変更時（Security Notification）
- **タイミング**: パスワード変更成功後
- **内容**: セキュリティアラート、変更時刻の記録
- **実装場所**: `src/service/auth_service.rs` - `change_password()` メソッド

### 4. パスワードリセット
- **リセット要求時**: パスワードリセットリンク付きメール
- **リセット完了時**: セキュリティ通知メール
- **実装場所**: `src/service/auth_service.rs` - `request_password_reset()`, `reset_password()` メソッド

### 5. チーム招待時（Team Invitation）
- **タイミング**: チームメンバー招待時
- **内容**: チーム名、招待者情報、参加用URL
- **実装場所**: `src/service/team_service.rs` - `invite_team_member()` メソッド

### 6. サブスクリプション変更時（Subscription Change）
- **タイミング**: サブスクリプション階層変更時
- **内容**: 新旧プラン情報、変更時刻、新機能の案内
- **実装場所**: `src/service/subscription_service.rs` - `change_subscription_tier()` メソッド

### 7. アカウント削除時（Account Deletion Confirmation）
- **タイミング**: アカウント削除成功後
- **内容**: 削除確認、削除されたデータの詳細
- **実装場所**: `src/service/auth_service.rs` - `delete_account()` メソッド

## 技術実装

### EmailService の構造

```rust
pub struct EmailService {
    config: EmailConfig,
}

pub struct EmailConfig {
    pub provider: EmailProvider,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub from_email: String,
    pub from_name: String,
    pub mailgun_api_key: Option<String>,
    pub mailgun_domain: Option<String>,
    pub development_mode: bool,
}

pub enum EmailProvider {
    Development,  // コンソール出力のみ
    MailHog,      // 開発環境SMTP
    Mailgun,      // 本番環境API
}
```

### 主要メソッド

```rust
impl EmailService {
    // 基本メール送信
    pub async fn send_email(&self, message: EmailMessage) -> AppResult<()>
    
    // 具体的なメール送信メソッド
    pub async fn send_welcome_email(&self, to_email: &str, to_name: &str) -> AppResult<()>
    pub async fn send_security_notification_email(&self, to_email: &str, to_name: &str, event_type: &str, event_details: &str) -> AppResult<()>
    pub async fn send_password_reset_email(&self, to_email: &str, to_name: &str, reset_token: &str, reset_url: &str) -> AppResult<()>
    pub async fn send_team_invitation_email(&self, to_email: &str, to_name: &str, team_name: &str, inviter_name: &str, invitation_url: &str) -> AppResult<()>
    pub async fn send_subscription_change_email(&self, to_email: &str, to_name: &str, old_tier: &str, new_tier: &str) -> AppResult<()>
    pub async fn send_account_deletion_confirmation_email(&self, to_email: &str, to_name: &str) -> AppResult<()>
}
```

## 設定方法

### 環境変数

#### 開発環境（MailHog）
```bash
# 開発モード設定
EMAIL_DEVELOPMENT_MODE=false
EMAIL_PROVIDER=mailhog

# MailHog設定
MAILHOG_HOST=localhost
MAILHOG_PORT=1025

# 送信者情報
FROM_EMAIL=noreply@taskbackend.local
FROM_NAME=Task Backend Service
```

#### 本番環境（Mailgun）
```bash
# 本番モード設定
EMAIL_DEVELOPMENT_MODE=false
EMAIL_PROVIDER=mailgun

# Mailgun設定
MAILGUN_API_KEY=your-mailgun-api-key
MAILGUN_DOMAIN=mg.yourdomain.com

# 送信者情報
FROM_EMAIL=noreply@yourdomain.com
FROM_NAME=Task Backend
```

#### コンソール出力のみ（Development）
```bash
# 開発モード設定
EMAIL_DEVELOPMENT_MODE=true
# または
EMAIL_PROVIDER=development
```

### MailHog設定手順

1. **MailHogの起動**:
   ```bash
   # Docker Composeで起動
   make dev
   ```

2. **Web UI でメール確認**:
   - URL: http://localhost:8025
   - SMTP: localhost:1025

### Mailgun設定手順

1. **Mailgunアカウント作成**: https://www.mailgun.com/
2. **ドメインの追加・認証**
3. **API Keyの取得**
4. **環境変数の設定**

## 開発モードと本番モード

### 開発モード（EMAIL_DEVELOPMENT_MODE=true）
- **動作**: メールは実際には送信されず、コンソールにログ出力
- **利点**: 
  - 開発中にSMTP設定が不要
  - メール内容の確認が容易
  - 外部サービスへの依存なし
- **ログ例**:
  ```
  📧 EMAIL (Development Mode)
  To: John Doe <john@example.com>
  Subject: Welcome to Task Backend!
  --- HTML Body ---
  <html>...</html>
  --- Text Body ---
  Welcome John Doe!...
  --- End Email ---
  ```

### 本番モード（EMAIL_DEVELOPMENT_MODE=false）
- **動作**: プロバイダーに応じた実際のメール送信
  - MailHog: SMTP経由でテスト環境送信
  - Mailgun: API経由で実際の配信
- **要件**: 有効なプロバイダー設定が必要
- **エラーハンドリング**: 送信失敗時もアプリケーション処理は継続

## メールテンプレート

### テンプレート構造

各メールは以下の形式を持ちます：

```rust
pub struct EmailTemplate {
    pub name: String,        // テンプレート名
    pub subject: String,     // 件名
    pub html_body: String,   // HTML形式本文
    pub text_body: String,   // テキスト形式本文
}
```

### テンプレートの特徴

- **HTML + テキスト**: メールクライアントの互換性を確保
- **日本語対応**: UTF-8エンコーディング
- **ブランディング**: 統一されたデザインとメッセージ
- **動的コンテンツ**: ユーザー名、時刻、URLなどを動的挿入

## エラーハンドリング

### メール送信失敗時の動作

```rust
if let Err(e) = self
    .email_service
    .send_welcome_email(&user.email, &user.username)
    .await
{
    // メール送信失敗はログに記録するが、処理は継続
    tracing::warn!("Failed to send welcome email: {}", e);
}
```

### 原則

1. **非ブロッキング**: メール送信失敗がユーザー操作をブロックしない
2. **ログ記録**: 失敗は適切にログに記録
3. **ユーザー体験**: メイン機能は正常に動作し続ける

## テスト

### 単体テスト

- **EmailService**: 開発モードでの動作確認
- **テンプレート**: 各メールテンプレートの内容検証
- **バリデーション**: メールアドレス形式の検証

### 統合テスト

```rust
#[tokio::test]
async fn test_welcome_email_sending() {
    let app = create_test_app().await;
    // ユーザー登録後、ウェルカムメールがログ出力されることを確認
}
```

### テスト実行

```bash
# 1. 開発環境でのメールテスト
make dev  # PostgreSQL + MailHog + アプリ起動

# 2. ユーザー登録によるウェルカムメールテスト
curl -X POST http://localhost:3000/auth/signup \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser",
    "email": "test@example.com",
    "password": "SecurePass123!"
  }'

# 3. MailHog Web UIで受信確認
# http://localhost:8025

# 4. 単体テスト実行
cargo test email

# 5. 全統合テスト実行
cargo test --test main
```

## トラブルシューティング

### よくある問題

1. **MailHog接続エラー**:
   - MailHogが起動しているか確認: `docker ps | grep mailhog`
   - ポート1025が利用可能か確認: `netstat -an | grep 1025`
   - 設定確認: `MAILHOG_HOST=localhost`, `MAILHOG_PORT=1025`

2. **Mailgun API エラー**:
   - API Keyが正しく設定されているか確認
   - ドメインが認証済みか確認
   - 環境変数確認: `MAILGUN_API_KEY`, `MAILGUN_DOMAIN`

3. **開発環境でメールが送信されない**:
   - `EMAIL_DEVELOPMENT_MODE=true`が設定されている場合、コンソールログを確認
   - ログレベルが`info`以上に設定されているか確認（`RUST_LOG=info`）
   - プロバイダー設定確認: `EMAIL_PROVIDER=mailhog`

### デバッグ手順

1. **設定確認**:
   ```bash
   # 環境変数の確認
   env | grep EMAIL
   env | grep MAILHOG
   env | grep MAILGUN
   env | grep FROM_
   ```

2. **ログ確認**:
   ```bash
   # 詳細ログを有効化
   RUST_LOG=debug cargo run
   ```

3. **テスト実行**:
   ```bash
   # メール機能のテスト
   cargo test email -- --nocapture
   ```

## セキュリティ考慮事項

### 認証情報の保護

- **Mailgun API Key**: API Keyは環境変数で管理、ソースコードに含めない
- **環境変数**: 認証情報は環境変数で管理、ソースコードに含めない
- **暗号化通信**: MailgunはHTTPS、MailHogは開発環境のみなので平文通信

### 個人情報の取り扱い

- **メールアドレス**: ログ出力時はマスク処理
- **エラーログ**: 個人情報を含まないよう配慮
- **開発モード**: 実際のメール送信は行わない

## パフォーマンス

### 非同期処理

- 全メール送信は非同期で実行
- メイン処理をブロックしない設計
- 送信失敗時も処理継続

### 制限事項

- 現在のところメール送信の制限やキューイングは実装されていない
- 大量送信時はMailgunの制限に注意が必要
- MailHogは開発環境専用（本番環境での使用は非推奨）

## 今後の拡張予定

- [ ] メール送信キューイング機能
- [ ] メールテンプレートのカスタマイズ機能
- [ ] 複数SMTP プロバイダー対応
- [ ] メール送信統計・分析機能
- [ ] ユーザーによるメール通知設定管理