# Stripe決済統合ベストプラクティス

## 概要

このドキュメントは、現在のRust製タスク管理バックエンドにStripe決済機能を統合するための包括的なベストプラクティスをまとめたものです。既存のサブスクリプション階層（Free/Pro/Enterprise）に実際の決済機能を追加し、堅牢で拡張性のあるシステムを構築するためのガイドラインです。

## 🚀 最短実装フロー（MVP版）

既存コードベースにStripe決済を最速で組み込むための最小限の実装手順です。

### Phase 0: 事前準備（30分）

#### 1. Stripeアカウント作成
```bash
# 1. https://stripe.com/jp にアクセス
# 2. 「今すぐ始める」をクリックしてアカウント作成
# 3. メールアドレス確認後、ダッシュボードにログイン

# 重要: 最初は「テストモード」で開発します
# ダッシュボード左上の「テストモード」スイッチがONになっていることを確認
```

#### 2. APIキーの取得
```bash
# Stripeダッシュボード → 開発者 → APIキー
# 以下の2つをメモ：
# - 公開可能キー: pk_test_... （フロントエンドで使用）
# - シークレットキー: sk_test_... （バックエンドで使用）

# .envファイルに保存
echo "STRIPE_SECRET_KEY=sk_test_..." >> .env
echo "STRIPE_PUBLISHABLE_KEY=pk_test_..." >> .env
```

#### 3. 環境変数の一覧（.env.example）
```bash
# Stripe設定
STRIPE_SECRET_KEY=sk_test_xxx
STRIPE_PUBLISHABLE_KEY=pk_test_xxx
STRIPE_WEBHOOK_SECRET=whsec_test_xxx  # stripe listenの出力から取得
STRIPE_PRO_PRICE_ID=price_xxx         # Stripe商品作成後に設定
STRIPE_ENTERPRISE_PRICE_ID=price_yyy  # Stripe商品作成後に設定

# アプリケーション設定
DATABASE_URL=postgresql://user:password@localhost/dbname
JWT_SECRET=your-jwt-secret
FRONTEND_URL=http://localhost:3001    # フロントエンドのURL

# 既存の環境変数はそのまま維持
```

### Phase 1: 基本セットアップ（1-2日）

#### 1. 環境準備
```bash
# Stripe CLIのインストール（Webhookのローカルテスト用）
curl -s https://packages.stripe.dev/api/security/keypair/stripe-cli-gpg/public | gpg --dearmor | sudo tee /usr/share/keyrings/stripe.gpg
echo "deb [signed-by=/usr/share/keyrings/stripe.gpg] https://packages.stripe.dev/stripe-cli-debian-local stable main" | sudo tee -a /etc/apt/sources.list.d/stripe.list
sudo apt update && sudo apt install stripe

# Stripe CLIでログイン
stripe login

# 必要な依存関係を追加
# Cargo.toml
[dependencies]
stripe-rust = "0.15"
# 既存の依存関係に追加（既にあるはず）
sea-orm = { version = "0.12", features = ["runtime-tokio-native-tls", "sqlx-postgres"] }
sqlx = { version = "0.7", features = ["runtime-tokio-native-tls", "postgres", "uuid", "chrono"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

#### 2. 最小限のDB拡張
```sql
-- migration/src/m20250703_000001_add_stripe_support.rs として作成
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 既存のusersテーブルにカラム追加
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(
                        ColumnDef::new(Users::StripeCustomerId)
                            .string()
                            .unique_key()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Stripeサブスクリプション管理テーブル
        manager
            .create_table(
                Table::create()
                    .table(StripeSubscriptions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(StripeSubscriptions::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(StripeSubscriptions::UserId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(StripeSubscriptions::StripeSubscriptionId)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(StripeSubscriptions::Status)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(StripeSubscriptions::CurrentPeriodEnd)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(StripeSubscriptions::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(StripeSubscriptions::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_stripe_subscriptions_user_id")
                            .from(StripeSubscriptions::Table, StripeSubscriptions::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // インデックス作成
        manager
            .create_index(
                Index::create()
                    .name("idx_stripe_subscriptions_user_id")
                    .table(StripeSubscriptions::Table)
                    .col(StripeSubscriptions::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_stripe_subscriptions_status")
                    .table(StripeSubscriptions::Table)
                    .col(StripeSubscriptions::Status)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(StripeSubscriptions::Table).to_owned())
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::StripeCustomerId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
    StripeCustomerId,
}

#[derive(DeriveIden)]
enum StripeSubscriptions {
    Table,
    Id,
    UserId,
    StripeSubscriptionId,
    Status,
    CurrentPeriodEnd,
    CreatedAt,
    UpdatedAt,
}
```

```bash
# マイグレーションの実行
# migration/src/lib.rsに追加
mod m20250703_000001_add_stripe_support;
pub use m20250703_000001_add_stripe_support::Migration as AddStripeSupport;

# Migratorに追加
vec![
    // ... 既存のマイグレーション
    Box::new(AddStripeSupport),
]

# 実行
sea-orm-cli migrate up
```

### Phase 2: コア機能実装（2-3日）

#### 1. Stripe設定とサービス
```rust
// src/service/stripe_service.rs
use stripe::{Client, Customer, CheckoutSession, Webhook};

pub struct StripeService {
    client: Client,
    webhook_secret: String,
}

impl StripeService {
    pub fn new() -> Self {
        Self {
            client: Client::new(env::var("STRIPE_SECRET_KEY").unwrap()),
            webhook_secret: env::var("STRIPE_WEBHOOK_SECRET").unwrap(),
        }
    }

    // 最小限の実装：Checkout Session作成
    pub async fn create_checkout_session(
        &self,
        user_id: Uuid,
        price_id: &str,
    ) -> Result<String, AppError> {
        let params = CreateCheckoutSession {
            mode: Some(CheckoutSessionMode::Subscription),
            line_items: Some(vec![
                CreateCheckoutSessionLineItems {
                    price: Some(price_id.to_string()),
                    quantity: Some(1),
                    ..Default::default()
                }
            ]),
            success_url: Some(format!("{}/subscription/success", env::var("FRONTEND_URL").unwrap_or("http://localhost:3001".to_string()))),
            cancel_url: Some(format!("{}/subscription/cancel", env::var("FRONTEND_URL").unwrap_or("http://localhost:3001".to_string()))),
            client_reference_id: Some(user_id.to_string()),
            ..Default::default()
        };

        let session = CheckoutSession::create(&self.client, params).await?;
        Ok(session.url.unwrap())
    }
}
```

#### 2. 必須DTOの定義
```rust
// src/api/dto/subscription_dto.rs に追加
use serde::{Deserialize, Serialize};
use crate::domain::subscription_tier::SubscriptionTier;

#[derive(Debug, Deserialize)]
pub struct CreateCheckoutRequest {
    pub tier: SubscriptionTier,
}

#[derive(Debug, Serialize)]
pub struct CheckoutResponse {
    pub checkout_url: String,
}
```

#### 3. 必須APIエンドポイント（2つだけ）
```rust
// src/api/handlers/subscription_handler.rs
use axum::{
    extract::{State, Json},
    http::{StatusCode, HeaderMap},
    response::IntoResponse,
};
use bytes::Bytes;
use stripe::{EventType, Webhook, CheckoutSession, Invoice};
use std::env;
use uuid::Uuid;
use chrono::{Utc, Duration};
use crate::{
    api::{AppState, dto::subscription_dto::{CreateCheckoutRequest, CheckoutResponse}},
    middleware::auth::AuthenticatedUser,
    domain::subscription_tier::SubscriptionTier,
    error::AppError,
};

// 1. Checkout開始エンドポイント
pub async fn create_checkout_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(req): Json<CreateCheckoutRequest>,
) -> AppResult<Json<CheckoutResponse>> {
    // 価格IDマッピング（環境変数から取得）
    let price_id = match req.tier {
        SubscriptionTier::Pro => env::var("STRIPE_PRO_PRICE_ID")?,
        SubscriptionTier::Enterprise => env::var("STRIPE_ENTERPRISE_PRICE_ID")?,
        _ => return Err(AppError::BadRequest("Invalid tier")),
    };

    let checkout_url = state.stripe_service
        .create_checkout_session(user.user_id, &price_id)
        .await?;

    Ok(Json(CheckoutResponse { checkout_url }))
}

// 2. Webhook受信エンドポイント（最重要）
pub async fn stripe_webhook_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> AppResult<StatusCode> {
    // 署名検証
    let event = Webhook::construct_event(
        &String::from_utf8(body.to_vec())?,
        headers.get("stripe-signature").unwrap().to_str()?,
        &state.stripe_service.webhook_secret,
    )?;

    // 最小限のイベント処理
    match event.type_ {
        EventType::CheckoutSessionCompleted => {
            let session: CheckoutSession = event.data.object.into();
            let user_id = Uuid::parse_str(&session.client_reference_id.unwrap())?;
            
            // DB更新（トランザクション内で）
            state.db.transaction::<_, _, AppError>(|txn| {
                Box::pin(async move {
                    // 1. stripe_customer_id更新
                    sqlx::query!(
                        "UPDATE users SET stripe_customer_id = $1 WHERE id = $2",
                        session.customer.unwrap(),
                        user_id
                    ).execute(txn).await?;

                    // 2. subscription_tier更新
                    sqlx::query!(
                        "UPDATE users SET subscription_tier = $1 WHERE id = $2",
                        determine_tier_from_session(&session)?,
                        user_id
                    ).execute(txn).await?;

                    // 3. stripe_subscriptions挿入
                    sqlx::query!(
                        "INSERT INTO stripe_subscriptions (user_id, stripe_subscription_id, status, current_period_end) 
                         VALUES ($1, $2, $3, $4)",
                        user_id,
                        session.subscription.unwrap(),
                        "active",
                        Utc::now() + Duration::days(30)
                    ).execute(txn).await?;

                    Ok(())
                })
            }).await?;
        }
        EventType::InvoicePaymentSucceeded => {
            // 継続課金の成功処理
            let invoice: Invoice = event.data.object.into();
            sqlx::query!(
                "UPDATE stripe_subscriptions 
                 SET current_period_end = $1, updated_at = NOW() 
                 WHERE stripe_subscription_id = $2",
                invoice.period_end,
                invoice.subscription.unwrap()
            ).execute(&state.db).await?;
        }
        _ => {} // 他のイベントは後で実装
    }

    Ok(StatusCode::OK)
}

// ヘルパー関数：セッションから階層を判定
fn determine_tier_from_session(session: &CheckoutSession) -> Result<String, AppError> {
    // Stripeの商品メタデータまたは価格IDから階層を判定
    let price_id = session.line_items
        .as_ref()
        .and_then(|items| items.data.first())
        .and_then(|item| item.price.as_ref())
        .and_then(|price| price.id.as_ref())
        .ok_or(AppError::InternalServerError("Price ID not found"))?;
    
    // 環境変数と照合
    if price_id == &env::var("STRIPE_PRO_PRICE_ID")? {
        Ok("pro".to_string())
    } else if price_id == &env::var("STRIPE_ENTERPRISE_PRICE_ID")? {
        Ok("enterprise".to_string())
    } else {
        Err(AppError::BadRequest("Unknown price ID"))
    }
}
```

#### 3. エラーハンドリングの追加
```rust
// src/error.rs に追加（既存のAppErrorに統合）
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    // ... 既存のエラー
    
    #[error("Stripe error: {0}")]
    StripeError(#[from] stripe::StripeError),
    
    #[error("Environment variable error: {0}")]
    EnvError(#[from] std::env::VarError),
}

// Axumのレスポンスへの変換
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::StripeError(_) => (StatusCode::BAD_REQUEST, "Payment processing error"),
            AppError::EnvError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Configuration error"),
            // ... 他のエラー処理
        };
        
        (status, Json(json!({ "error": error_message }))).into_response()
    }
}
```

#### 4. ルーティング設定
```rust
// src/api/mod.rs
pub fn subscription_routes() -> Router<AppState> {
    Router::new()
        .route("/checkout", post(create_checkout_handler))
        .route("/webhook", post(stripe_webhook_handler))
}

// main.rsでの統合
let app = Router::new()
    .nest("/api/tasks", task_routes())
    .nest("/api/users", user_routes())
    .nest("/api/subscriptions", subscription_routes()) // 追加
    .layer(cors)
    .layer(trace_layer)
    .with_state(app_state);

// 重要: Webhookエンドポイントは認証ミドルウェアをスキップする必要がある
// src/api/mod.rs で調整
pub fn subscription_routes() -> Router<AppState> {
    Router::new()
        .route("/checkout", post(create_checkout_handler)
            .layer(middleware::from_fn(auth::require_auth))) // 認証必要
        .route("/webhook", post(stripe_webhook_handler)) // 認証不要（Stripeからの呼び出し）
}
```

### Phase 2.5: ローカルテスト（重要）

#### 1. Webhookのローカルテスト設定
```bash
# ターミナル1: アプリケーションを起動
cargo run

# ターミナル2: Stripe CLIでWebhookを転送
stripe listen --forward-to localhost:3000/api/subscriptions/webhook

# 出力例:
# Ready! Your webhook signing secret is whsec_test_xxx (^C to quit)
# このwhsec_test_xxxを.envファイルに追加
echo "STRIPE_WEBHOOK_SECRET=whsec_test_xxx" >> .env

# アプリケーションを再起動して新しい環境変数を読み込む
```

#### 2. テスト用の価格作成
```bash
# Stripeダッシュボード → 商品 → 新規作成
# または、Stripe CLIで作成：

# Pro月額プラン
stripe products create \
  --name="Pro Plan" \
  --description="Professional features"

stripe prices create \
  --product=prod_xxx \
  --unit-amount=3000 \
  --currency=jpy \
  --recurring[interval]=month

# Enterprise月額プラン  
stripe products create \
  --name="Enterprise Plan" \
  --description="All features included"

stripe prices create \
  --product=prod_yyy \
  --unit-amount=10000 \
  --currency=jpy \
  --recurring[interval]=month

# 作成された価格IDを.envに追加
echo "STRIPE_PRO_PRICE_ID=price_xxx" >> .env
echo "STRIPE_ENTERPRISE_PRICE_ID=price_yyy" >> .env
```

#### 3. テスト実行
```bash
# テスト用のカード番号
# 成功: 4242 4242 4242 4242
# 失敗: 4000 0000 0000 9995

# curlでCheckout Sessionを作成
curl -X POST http://localhost:3000/api/subscriptions/checkout \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"tier": "pro"}'

# レスポンスのcheckout_urlをブラウザで開いてテスト決済
```

### Phase 3: 既存システムとの統合（1日）

#### 0. AppStateの更新
```rust
// src/api/mod.rs のAppStateに追加
use crate::service::stripe_service::StripeService;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub user_service: Arc<UserService>,
    pub task_service: Arc<TaskService>,
    // ... 既存のサービス
    pub stripe_service: Arc<StripeService>, // 追加
}

// main.rsでの初期化
let stripe_service = Arc::new(StripeService::new());
let app_state = AppState {
    db: db.clone(),
    user_service,
    task_service,
    // ...
    stripe_service, // 追加
};
```

#### 1. 既存の権限チェックを活用
```rust
// 既存のsubscription_tierを使った権限チェックはそのまま動作
impl User {
    pub fn can_access_feature(&self, feature: Feature) -> bool {
        match feature {
            Feature::AdvancedAnalytics => 
                self.subscription_tier.is_at_least(&SubscriptionTier::Pro),
            Feature::TeamManagement => 
                self.subscription_tier.is_at_least(&SubscriptionTier::Enterprise),
            _ => true,
        }
    }
}
```

#### 2. 移行スクリプト（既存ユーザー対応）
```rust
// scripts/migrate_existing_users.rs
async fn migrate_existing_paid_users(db: &DatabaseConnection) -> Result<()> {
    // 既存のPro/Enterpriseユーザーに対してStripe Customerを作成
    let users = sqlx::query!(
        "SELECT id, email, subscription_tier FROM users 
         WHERE subscription_tier != 'free' AND stripe_customer_id IS NULL"
    ).fetch_all(db).await?;

    for user in users {
        // Stripe Customer作成（課金はまだしない）
        let customer = Customer::create(&stripe_client, CreateCustomer {
            email: Some(user.email),
            metadata: Some(HashMap::from([
                ("user_id", user.id.to_string()),
                ("legacy_tier", user.subscription_tier),
            ])),
            ..Default::default()
        }).await?;

        // stripe_customer_id更新
        sqlx::query!(
            "UPDATE users SET stripe_customer_id = $1 WHERE id = $2",
            customer.id,
            user.id
        ).execute(db).await?;
    }
    Ok(())
}
```

### Phase 4: 本番デプロイ（1日）

#### 1. Stripeダッシュボード設定
```yaml
# 必須設定項目
1. 商品とPrice作成:
   - Pro月額: price_xxx_pro_monthly
   - Pro年額: price_xxx_pro_yearly
   - Enterprise月額: price_xxx_enterprise_monthly
   - Enterprise年額: price_xxx_enterprise_yearly

2. Webhookエンドポイント登録:
   - URL: https://api.yourdomain.com/api/subscriptions/webhook
   - イベント: 
     - checkout.session.completed
     - invoice.payment_succeeded
     - invoice.payment_failed

3. 環境変数設定:
   STRIPE_SECRET_KEY=sk_live_xxx
   STRIPE_PUBLISHABLE_KEY=pk_live_xxx
   STRIPE_WEBHOOK_SECRET=whsec_xxx
   STRIPE_PRO_PRICE_ID=price_xxx
   STRIPE_ENTERPRISE_PRICE_ID=price_xxx
```

#### 2. 監視設定
```rust
// 最小限のメトリクス
- Webhook受信成功率
- Checkout Session作成数
- 決済成功/失敗数
```

### Phase 5: フロントエンドとの連携（最小限）

#### 1. React/Next.jsでの実装例
```typescript
// components/UpgradeButton.tsx
import { useState } from 'react';

export function UpgradeButton({ tier }: { tier: 'pro' | 'enterprise' }) {
  const [loading, setLoading] = useState(false);

  const handleUpgrade = async () => {
    setLoading(true);
    try {
      // バックエンドAPIを呼び出し
      const response = await fetch('/api/subscriptions/checkout', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${localStorage.getItem('jwt_token')}`,
        },
        body: JSON.stringify({ tier }),
      });

      const data = await response.json();
      
      if (data.checkout_url) {
        // Stripeのチェックアウトページにリダイレクト
        window.location.href = data.checkout_url;
      }
    } catch (error) {
      console.error('Upgrade failed:', error);
      alert('アップグレードに失敗しました');
    } finally {
      setLoading(false);
    }
  };

  return (
    <button onClick={handleUpgrade} disabled={loading}>
      {loading ? '処理中...' : `${tier}プランにアップグレード`}
    </button>
  );
}
```

#### 2. 成功/キャンセルページ
```typescript
// pages/subscription/success.tsx
export default function SubscessPage() {
  useEffect(() => {
    // 成功後の処理（ユーザー情報の再取得など）
    fetchUserProfile();
  }, []);

  return <div>決済が完了しました！プランがアップグレードされました。</div>;
}

// pages/subscription/cancel.tsx
export default function CancelPage() {
  return <div>決済をキャンセルしました。</div>;
}
```

### 実装完了チェックリスト

- [ ] Stripeアカウント作成・テストモード確認
- [ ] Stripe CLIインストール・ログイン完了
- [ ] 環境変数（.env）設定完了
- [ ] DBマイグレーション実行完了
- [ ] Stripe CLIでローカルWebhookテスト成功
- [ ] Checkout Sessionが作成できる
- [ ] Webhookが正しく処理される
- [ ] DBのsubscription_tierが更新される
- [ ] 既存の権限チェックが動作する
- [ ] フロントエンドから決済フロー確認
- [ ] 本番環境の環境変数設定完了

### この最短実装で実現できること

1. **新規ユーザー**: Stripe Checkoutで決済 → 自動的にPro/Enterpriseへ
2. **既存ユーザー**: 現状維持しつつ、段階的にStripe移行
3. **継続課金**: Stripeが自動処理、Webhookで状態更新
4. **権限管理**: 既存のロジックがそのまま使える

### よくある問題と解決方法

#### 1. Webhook署名検証エラー
```bash
# エラー: "Invalid webhook signature"
# 原因: webhook_secretが間違っている

# 解決方法:
# 1. stripe listenコマンドの出力を確認
stripe listen --forward-to localhost:3000/api/subscriptions/webhook
# 出力されたwhsec_test_xxxを.envに正確にコピー

# 2. アプリケーションを再起動
```

#### 2. 価格IDが見つからない
```bash
# エラー: "Unknown price ID"
# 原因: 環境変数の価格IDが間違っている

# 解決方法:
# Stripeダッシュボードで価格IDを確認
# 商品 → 該当商品 → 価格セクション → price_xxxをコピー
```

#### 3. DBトランザクションエラー
```bash
# エラー: "Transaction rollback"
# 原因: スキーマの不一致

# 解決方法:
sea-orm-cli migrate fresh  # 開発環境のみ
# または
sea-orm-cli migrate down
sea-orm-cli migrate up
```

#### 4. 本番環境への切り替え時の注意
```bash
# テストモードから本番モードへ
# 1. Stripeダッシュボードで本番モードに切り替え
# 2. 本番用のAPIキーを取得
# 3. 本番環境の.envを更新
STRIPE_SECRET_KEY=sk_live_xxx  # sk_test_xxxから変更
STRIPE_PUBLISHABLE_KEY=pk_live_xxx  # pk_test_xxxから変更

# 4. 本番用の商品・価格を再作成（テストと本番は別）
# 5. Webhookエンドポイントを本番用に再登録
```

### 次のステップ（優先度順）

1. **エラーハンドリング強化**（1週間後）
   - 支払い失敗時の処理
   - リトライロジック

2. **ユーザー向け機能**（2週間後）
   - 支払い方法の更新
   - 請求履歴の表示
   - プランダウングレード

3. **管理機能**（1ヶ月後）
   - 返金処理
   - クーポン機能
   - 使用量ベース課金

---

## 現在のアーキテクチャと決済統合方針

### 既存システムの特徴

1. **サブスクリプション階層**: ユーザーと組織の両方にFree/Pro/Enterpriseの階層が存在
2. **履歴管理**: subscription_historiesテーブルで階層変更履歴を管理
3. **権限システム**: ロールベースのアクセス制御（Admin/Member）
4. **SeaORM**: 非同期ORMとしてSeaORMを利用
5. **Axum**: Webフレームワークとして使用

### Stripe統合の基本方針

1. **決済情報の非保持**: PCI DSS準拠のため、カード情報は一切保存せず、Stripeトークンのみ管理
2. **Webhook駆動**: Stripeイベントをwebhook経由で受信し、システム状態を更新
3. **冪等性の保証**: 決済処理の重複実行を防ぐため、冪等性キーを使用
4. **非同期処理**: 決済関連の処理は非同期タスクとして実行

## バックエンドシナリオフロー

### 1. 初回サブスクリプション登録フロー

```mermaid
sequenceDiagram
    participant User as ユーザー
    participant API as Backend API
    participant DB as Database
    participant Stripe as Stripe API
    participant Queue as Job Queue
    
    User->>API: POST /api/subscriptions/checkout
    Note over API: 認証確認
    API->>Stripe: Create Checkout Session
    Stripe-->>API: Session URL
    API-->>User: Redirect to Stripe Checkout
    
    User->>Stripe: 支払い情報入力
    Stripe->>Stripe: 決済処理
    
    alt 決済成功
        Stripe->>API: Webhook: checkout.session.completed
        API->>API: Webhook署名検証
        API->>DB: トランザクション開始
        API->>DB: users.subscription_tier更新
        API->>DB: subscription_histories挿入
        API->>DB: stripe_customers挿入
        API->>DB: stripe_subscriptions挿入
        API->>DB: コミット
        API->>Queue: 成功通知メール送信ジョブ
        API-->>Stripe: 200 OK
    else 決済失敗
        Stripe->>API: Webhook: checkout.session.expired
        API->>Queue: 失敗通知メール送信ジョブ
        API-->>Stripe: 200 OK
    end
```

### 2. 定期課金処理フロー

```mermaid
sequenceDiagram
    participant Stripe as Stripe
    participant API as Backend API  
    participant DB as Database
    participant Queue as Job Queue
    participant Worker as Background Worker
    
    Note over Stripe: 月次課金日
    Stripe->>Stripe: 自動課金実行
    
    alt 課金成功
        Stripe->>API: Webhook: invoice.payment_succeeded
        API->>API: Webhook署名検証
        API->>DB: payments挿入（支払い記録）
        API->>DB: subscription期限延長
        API->>Queue: 領収書メール送信ジョブ
        API-->>Stripe: 200 OK
    else 課金失敗
        Stripe->>API: Webhook: invoice.payment_failed
        API->>API: Webhook署名検証
        API->>DB: payments挿入（失敗記録）
        API->>DB: subscription.status = 'past_due'
        API->>Queue: 支払い失敗通知ジョブ
        API-->>Stripe: 200 OK
        
        Note over Worker: リトライ戦略実行
        Worker->>DB: past_due契約を検索
        Worker->>Queue: リトライ通知メール送信
        
        Note over Stripe: Stripeの自動リトライ
        Stripe->>Stripe: 数日後に再課金試行
    end
```

### 3. プラン変更（アップグレード/ダウングレード）フロー

```mermaid
sequenceDiagram
    participant User as ユーザー
    participant API as Backend API
    participant DB as Database
    participant Stripe as Stripe API
    participant Queue as Job Queue
    
    User->>API: PUT /api/subscriptions/upgrade
    Note over API: 認証・権限確認
    API->>DB: 現在のsubscription取得
    API->>Stripe: Update Subscription
    Note over Stripe: 即時プロレーション計算
    Stripe-->>API: 更新結果＋請求額
    
    API->>DB: トランザクション開始
    API->>DB: users.subscription_tier更新
    API->>DB: subscription_histories挿入
    API->>DB: stripe_subscriptions更新
    API->>DB: コミット
    
    API->>Queue: プラン変更通知メール
    API-->>User: 200 OK
    
    Note over Stripe: 差額請求処理
    Stripe->>API: Webhook: invoice.created
    Stripe->>API: Webhook: invoice.payment_succeeded
```

### 4. サブスクリプションキャンセルフロー

```mermaid
sequenceDiagram
    participant User as ユーザー
    participant API as Backend API
    participant DB as Database  
    participant Stripe as Stripe API
    participant Queue as Job Queue
    
    User->>API: DELETE /api/subscriptions
    Note over API: 認証確認
    API->>Stripe: Cancel Subscription at Period End
    Stripe-->>API: 確認
    
    API->>DB: subscription.cancel_at_period_end = true
    API->>DB: subscription_histories挿入
    API->>Queue: キャンセル確認メール
    API-->>User: 200 OK
    
    Note over Stripe: 期間終了時
    Stripe->>API: Webhook: customer.subscription.deleted
    API->>DB: users.subscription_tier = 'Free'
    API->>DB: subscription.status = 'canceled'
    API->>Queue: キャンセル完了通知
```

## データモデル拡張設計

### 新規テーブル

```sql
-- Stripe顧客情報
CREATE TABLE stripe_customers (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID UNIQUE NOT NULL REFERENCES users(id),
    stripe_customer_id VARCHAR(255) UNIQUE NOT NULL,
    default_payment_method_id VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Stripeサブスクリプション情報
CREATE TABLE stripe_subscriptions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id),
    stripe_subscription_id VARCHAR(255) UNIQUE NOT NULL,
    stripe_price_id VARCHAR(255) NOT NULL,
    status VARCHAR(50) NOT NULL, -- active, past_due, canceled, etc
    current_period_start TIMESTAMPTZ NOT NULL,
    current_period_end TIMESTAMPTZ NOT NULL,
    cancel_at_period_end BOOLEAN NOT NULL DEFAULT FALSE,
    canceled_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 支払い履歴
CREATE TABLE payments (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id),
    stripe_payment_intent_id VARCHAR(255) UNIQUE,
    stripe_invoice_id VARCHAR(255),
    amount_cents INTEGER NOT NULL,
    currency VARCHAR(3) NOT NULL DEFAULT 'USD',
    status VARCHAR(50) NOT NULL, -- succeeded, failed, pending
    description TEXT,
    failure_reason TEXT,
    paid_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Webhook処理の冪等性保証
CREATE TABLE webhook_events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    stripe_event_id VARCHAR(255) UNIQUE NOT NULL,
    event_type VARCHAR(100) NOT NULL,
    processed BOOLEAN NOT NULL DEFAULT FALSE,
    processed_at TIMESTAMPTZ,
    error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- インデックス
CREATE INDEX idx_stripe_customers_user_id ON stripe_customers(user_id);
CREATE INDEX idx_stripe_subscriptions_user_id ON stripe_subscriptions(user_id);
CREATE INDEX idx_stripe_subscriptions_status ON stripe_subscriptions(status);
CREATE INDEX idx_payments_user_id ON payments(user_id);
CREATE INDEX idx_payments_created_at ON payments(created_at);
CREATE INDEX idx_webhook_events_created_at ON webhook_events(created_at);
```

## 実装アーキテクチャ

### レイヤー構造

```mermaid
graph TB
    subgraph "API Layer"
        A[Webhook Handler]
        B[Subscription Handler]
        C[Payment Handler]
    end
    
    subgraph "Service Layer"
        D[Stripe Service]
        E[Subscription Service]
        F[Payment Service]
        G[Notification Service]
    end
    
    subgraph "Domain Layer"
        H[Stripe Customer Model]
        I[Stripe Subscription Model]
        J[Payment Model]
        K[Webhook Event Model]
    end
    
    subgraph "Infrastructure Layer"
        L[Stripe Client]
        M[Database]
        N[Job Queue]
        O[Email Service]
    end
    
    A --> D
    B --> E
    C --> F
    
    D --> L
    E --> D
    E --> M
    F --> M
    G --> O
    G --> N
    
    D --> H
    E --> I
    F --> J
    A --> K
```

## 詳細実装手順

### Phase 1: 基盤構築

#### 1.1 Stripe接続設定

**Cargo.toml**
```toml
[dependencies]
# Stripe API Client
stripe = { version = "0.16", features = ["async", "runtime-tokio-hyper"] }
tokio-retry = "0.3"
backoff = { version = "0.4", features = ["tokio"] }
```

**環境設定**
```env
# Stripe設定
STRIPE_SECRET_KEY=sk_test_...
STRIPE_WEBHOOK_SECRET=whsec_...
STRIPE_PUBLISHABLE_KEY=pk_test_...

# 価格ID（Stripeダッシュボードで作成）
# 日本円価格
STRIPE_PRICE_ID_PRO_MONTHLY_JPY=price_1234567890abcdefJPY
STRIPE_PRICE_ID_PRO_YEARLY_JPY=price_0987654321zyxwvuJPY
STRIPE_PRICE_ID_ENTERPRISE_MONTHLY_JPY=price_abcdefghijk12345JPY
STRIPE_PRICE_ID_ENTERPRISE_YEARLY_JPY=price_zyxwvutsrqp09876JPY

# USD価格
STRIPE_PRICE_ID_PRO_MONTHLY_USD=price_1234567890abcdefUSD
STRIPE_PRICE_ID_PRO_YEARLY_USD=price_0987654321zyxwvuUSD
STRIPE_PRICE_ID_ENTERPRISE_MONTHLY_USD=price_abcdefghijk12345USD
STRIPE_PRICE_ID_ENTERPRISE_YEARLY_USD=price_zyxwvutsrqp09876USD

# EUR価格
STRIPE_PRICE_ID_PRO_MONTHLY_EUR=price_1234567890abcdefEUR
STRIPE_PRICE_ID_PRO_YEARLY_EUR=price_0987654321zyxwvuEUR
STRIPE_PRICE_ID_ENTERPRISE_MONTHLY_EUR=price_abcdefghijk12345EUR
STRIPE_PRICE_ID_ENTERPRISE_YEARLY_EUR=price_zyxwvutsrqp09876EUR

# 決済設定
STRIPE_SUCCESS_URL=https://app.example.com/subscription/success
STRIPE_CANCEL_URL=https://app.example.com/subscription/cancel
STRIPE_TRIAL_DAYS=14
```

#### 1.2 Stripe Serviceの実装

```rust
// src/service/stripe_service.rs
use stripe::{Client, Customer, CheckoutSession, Subscription};
use crate::error::{AppError, StripeError};

#[derive(Clone)]
pub struct StripeService {
    client: Client,
    config: StripeConfig,
}

impl StripeService {
    pub fn new(config: StripeConfig) -> Result<Self, AppError> {
        let client = Client::new(&config.secret_key);
        Ok(Self { client, config })
    }

    /// Checkout Session作成
    pub async fn create_checkout_session(
        &self,
        customer_id: &str,
        price_id: &str,
        user_id: Uuid,
    ) -> Result<CheckoutSession, StripeError> {
        let mut params = CreateCheckoutSession::new();
        params.customer = Some(customer_id.to_string());
        params.mode = Some(CheckoutSessionMode::Subscription);
        params.line_items = Some(vec![
            CreateCheckoutSessionLineItems {
                price: Some(price_id.to_string()),
                quantity: Some(1),
                ..Default::default()
            },
        ]);
        params.success_url = Some(&self.config.urls.success);
        params.cancel_url = Some(&self.config.urls.cancel);
        
        // メタデータに user_id を設定（後でWebhookで使用）
        params.metadata = Some([
            ("user_id".to_string(), user_id.to_string()),
        ].iter().cloned().collect());

        // 試用期間設定
        if let Some(trial_days) = self.config.trial_days {
            params.subscription_data = Some(CreateCheckoutSessionSubscriptionData {
                trial_period_days: Some(trial_days),
                ..Default::default()
            });
        }

        CheckoutSession::create(&self.client, params).await
            .map_err(|e| StripeError::ApiError(e.to_string()))
    }
    
    /// 多通貨対応のCheckout Session作成
    pub async fn create_checkout_session_with_currency(
        &self,
        customer_id: &str,
        price_id: &str,
        user_id: Uuid,
        enable_tax: bool,
    ) -> Result<CheckoutSession, StripeError> {
        let mut params = CreateCheckoutSession::new();
        params.customer = Some(customer_id.to_string());
        params.mode = Some(CheckoutSessionMode::Subscription);
        
        // 価格IDから自動的に通貨が決定される
        params.line_items = Some(vec![
            CreateCheckoutSessionLineItems {
                price: Some(price_id.to_string()),
                quantity: Some(1),
                ..Default::default()
            },
        ]);
        
        // 税金の自動計算を有効化（オプション）
        if enable_tax {
            params.automatic_tax = Some(CreateCheckoutSessionAutomaticTax {
                enabled: true,
                ..Default::default()
            });
            
            // 請求先住所の収集（税金計算に必要）
            params.billing_address_collection = Some(
                CheckoutSessionBillingAddressCollection::Required
            );
        }
        
        params.success_url = Some(&self.config.urls.success);
        params.cancel_url = Some(&self.config.urls.cancel);
        params.metadata = Some([
            ("user_id".to_string(), user_id.to_string()),
        ].iter().cloned().collect());

        CheckoutSession::create(&self.client, params).await
            .map_err(|e| StripeError::ApiError(e.to_string()))
    }

    /// Webhook署名検証
    pub fn verify_webhook_signature(
        &self,
        payload: &[u8],
        signature: &str,
    ) -> Result<Event, StripeError> {
        ConstructEvent::construct_event(
            std::str::from_utf8(payload)
                .map_err(|_| StripeError::InvalidSignature)?,
            signature,
            &self.config.webhook_secret,
        )
        .map_err(|_| StripeError::InvalidSignature)
    }
}
```

### Phase 2: Webhook処理とトランザクション管理

#### 2.1 Webhook Handler実装

```rust
// src/api/handlers/webhook_handler.rs
use axum::{extract::State, http::StatusCode, response::IntoResponse};
use sea_orm::{DatabaseConnection, TransactionTrait};

pub async fn handle_stripe_webhook(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<impl IntoResponse, AppError> {
    // 署名検証
    let signature = headers
        .get("stripe-signature")
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::BadRequest("Missing stripe signature".into()))?;

    let event = state
        .stripe_service
        .verify_webhook_signature(&body, signature)?;

    // 冪等性チェック（重複処理防止）
    let event_id = event.id.to_string();
    if WebhookEventRepository::exists(&state.db, &event_id).await? {
        return Ok(StatusCode::OK);
    }

    // イベント記録
    WebhookEventRepository::create(
        &state.db,
        &event_id,
        &event.event_type.to_string(),
    )
    .await?;

    // イベントタイプに応じた処理
    match event.event_type {
        EventType::CheckoutSessionCompleted => {
            handle_checkout_completed(&state, event).await?;
        }
        EventType::InvoicePaymentSucceeded => {
            handle_payment_succeeded(&state, event).await?;
        }
        EventType::InvoicePaymentFailed => {
            handle_payment_failed(&state, event).await?;
        }
        EventType::CustomerSubscriptionUpdated => {
            handle_subscription_updated(&state, event).await?;
        }
        EventType::CustomerSubscriptionDeleted => {
            handle_subscription_canceled(&state, event).await?;
        }
        _ => {
            tracing::info!("Unhandled webhook event: {:?}", event.event_type);
        }
    }

    Ok(StatusCode::OK)
}

/// Checkout完了処理（SeaORMトランザクション使用）
async fn handle_checkout_completed(
    state: &Arc<AppState>,
    event: Event,
) -> Result<(), AppError> {
    let session = match &event.data.object {
        EventObject::CheckoutSession(session) => session,
        _ => return Err(AppError::BadRequest("Invalid event object".into())),
    };

    let user_id = session
        .metadata
        .get("user_id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or(AppError::BadRequest("Missing user_id in metadata".into()))?;

    // SeaORMのトランザクション管理
    state.db.transaction::<_, _, AppError>(|txn| {
        Box::pin(async move {
            // 1. Stripe顧客情報を保存
            StripeCustomerRepository::create_or_update(
                txn,
                user_id,
                &session.customer.clone().unwrap(),
            )
            .await?;

            // 2. サブスクリプション情報を保存
            if let Some(subscription_id) = &session.subscription {
                let tier = determine_tier_from_price(&session.line_items);
                
                StripeSubscriptionRepository::create(
                    txn,
                    user_id,
                    subscription_id,
                    &tier,
                )
                .await?;

                // 3. ユーザーのsubscription_tierを更新
                UserRepository::update_subscription_tier(
                    txn,
                    user_id,
                    tier,
                )
                .await?;

                // 4. 履歴を記録
                SubscriptionHistoryRepository::create(
                    txn,
                    user_id,
                    SubscriptionTier::Free,
                    tier,
                    Some(user_id),
                    "Stripe checkout completed".to_string(),
                )
                .await?;
            }

            Ok(())
        })
    })
    .await?;

    // 通知メール送信（非同期ジョブ）
    state.job_queue.enqueue(NotificationJob::SubscriptionStarted {
        user_id,
        tier: SubscriptionTier::Pro,
    })?;

    Ok(())
}
```

### Phase 3: 非同期処理とリトライ戦略

#### 3.1 リトライ機構の実装

```rust
// src/worker/payment_retry_worker.rs
use tokio_retry::{Retry, strategy::{ExponentialBackoff, jitter}};

pub struct PaymentRetryWorker {
    db: DatabaseConnection,
    stripe: StripeService,
    notification: NotificationService,
}

impl PaymentRetryWorker {
    /// 失敗した支払いのリトライ処理
    pub async fn retry_failed_payments(&self) -> Result<(), AppError> {
        let failed_payments = PaymentRepository::get_retriable_failures(&self.db).await?;

        for payment in failed_payments {
            // エクスポネンシャルバックオフ戦略
            let retry_strategy = ExponentialBackoff::from_millis(500)
                .factor(2)              // バックオフ倍率
                .max_delay(Duration::from_secs(3600)) // 最大1時間
                .map(jitter)            // ジッタ適用
                .take(payment.retry_count as usize + 1);

            let payment_id = payment.id;
            let result = Retry::spawn(retry_strategy, || async {
                self.process_payment_retry(&payment_id).await
            })
            .await;

            match result {
                Ok(_) => {
                    tracing::info!("Payment retry succeeded: {}", payment_id);
                    self.notification.send_payment_success(payment.user_id).await?;
                }
                Err(e) => {
                    // 恒久的失敗の判定
                    if self.is_permanent_failure(&e) {
                        self.handle_permanent_failure(&payment).await?;
                    } else {
                        self.schedule_next_retry(&payment).await?;
                    }
                }
            }
        }

        Ok(())
    }

    /// エラータイプに基づく恒久的失敗の判定
    fn is_permanent_failure(&self, error: &AppError) -> bool {
        match error {
            AppError::StripeError(StripeError::CardDeclined(reason)) => {
                matches!(reason.as_str(), "invalid_card" | "card_not_supported")
            }
            AppError::StripeError(StripeError::CustomerDeleted) => true,
            _ => false,
        }
    }
}
```

#### 3.2 ダニング管理（段階的対応）

```rust
// src/domain/subscription_state.rs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SubscriptionState {
    Active,
    Trialing,
    PastDue,
    Limited,     // 機能制限状態
    Suspended,   // 一時停止
    Canceled,
    Expired,
}

impl SubscriptionState {
    pub fn allowed_features(&self) -> Vec<Feature> {
        match self {
            Self::Active | Self::Trialing => Feature::all(),
            Self::PastDue => Feature::all(), // 猶予期間は全機能
            Self::Limited => Feature::read_only(), // 読み取りのみ
            Self::Suspended => Feature::none(),
            _ => Feature::none(),
        }
    }
}

// src/worker/dunning_worker.rs
async fn handle_dunning_process(
    &self,
    subscription: &stripe_subscription::Model,
) -> Result<(), AppError> {
    let days_since_failure = (Utc::now() - subscription.last_payment_failed_at.unwrap())
        .num_days();

    match days_since_failure {
        0..=3 => {
            // 初期段階：優しいリマインダー
            self.notification.send_payment_reminder_gentle(
                subscription.user_id,
            ).await?;
        }
        4..=7 => {
            // 中期段階：サービス制限の警告
            self.notification.send_service_limitation_warning(
                subscription.user_id,
            ).await?;
        }
        8..=14 => {
            // 後期段階：サービス一時停止
            self.suspend_subscription(subscription).await?;
            self.notification.send_suspension_notice(
                subscription.user_id,
            ).await?;
        }
        _ => {
            // 最終段階：自動キャンセル
            self.cancel_subscription(subscription).await?;
            self.notification.send_cancellation_notice(
                subscription.user_id,
            ).await?;
        }
    }

    Ok(())
}
```

## 業務シナリオの網羅的検討事項

### 1. 初回登録シナリオ

#### 1.1 新規ユーザーの無料トライアル

```mermaid
flowchart TD
    A[ユーザー登録] --> B{プラン選択}
    B -->|Free選択| C[即時有効化]
    B -->|Pro/Enterprise選択| D[支払い情報入力画面へ]
    D --> E{支払い方法}
    E -->|クレジットカード| F[Stripe Checkout]
    E -->|請求書払い| G[営業チーム連絡]
    F --> H{決済結果}
    H -->|成功| I[14日間トライアル開始]
    H -->|失敗| J[エラー表示・再試行]
    I --> K[トライアル終了3日前通知]
    K --> L{継続意思}
    L -->|継続| M[自動課金開始]
    L -->|キャンセル| N[Freeプランへ移行]
```

**考慮事項：**
- トライアル期間中の機能制限有無
- 支払い情報なしでのトライアル許可の可否
- 学生・非営利団体向け割引の適用方法
- 複数ユーザーの組織登録時の取り扱い

#### 1.2 既存ユーザーのアップグレード検証

```rust
pub async fn validate_upgrade_eligibility(
    user: &User,
    target_tier: SubscriptionTier,
) -> Result<(), UpgradeError> {
    // 組織の制限チェック
    if let Some(org_id) = user.organization_id {
        let org = OrganizationRepository::find_by_id(org_id).await?;
        if org.subscription_tier < target_tier {
            return Err(UpgradeError::OrganizationTierInsufficient);
        }
    }

    // 未払い請求の確認
    let unpaid_invoices = PaymentRepository::count_unpaid_by_user(user.id).await?;
    if unpaid_invoices > 0 {
        return Err(UpgradeError::UnpaidInvoicesExist);
    }

    // 使用量制限の確認
    let usage = UsageRepository::get_current_month(user.id).await?;
    if usage.api_calls > target_tier.api_call_limit() {
        return Err(UpgradeError::UsageExceedsNewPlanLimit);
    }

    Ok(())
}
```

### 2. 定期課金シナリオ

#### 2.1 支払い失敗時の段階的対応

```mermaid
stateDiagram-v2
    [*] --> Active: サブスクリプション有効
    Active --> PaymentDue: 課金日到来
    PaymentDue --> PaymentProcessing: 課金実行
    PaymentProcessing --> Active: 成功
    PaymentProcessing --> GracePeriod: 失敗（猶予期間）
    
    GracePeriod --> RetryAttempt1: 1日後
    RetryAttempt1 --> Active: 成功
    RetryAttempt1 --> RetryAttempt2: 失敗
    
    RetryAttempt2 --> RetryAttempt3: 3日後
    RetryAttempt3 --> Active: 成功
    RetryAttempt3 --> PastDue: 失敗
    
    PastDue --> Limited: 7日後（機能制限）
    Limited --> Active: 支払い完了
    Limited --> Suspended: 14日後
    
    Suspended --> Active: 支払い完了
    Suspended --> Canceled: 30日後
    Canceled --> [*]
```

### 3. プラン変更シナリオ

#### 3.1 アップグレード時のプロレーション計算

```rust
pub async fn handle_plan_upgrade(
    user_id: Uuid,
    new_tier: SubscriptionTier,
) -> Result<UpgradeResult, AppError> {
    let db = get_db_connection();
    
    db.transaction::<_, _, AppError>(|txn| {
        Box::pin(async move {
            // 現在のサブスクリプション取得
            let current_sub = StripeSubscriptionRepository::find_active_by_user(txn, user_id)
                .await?
                .ok_or(AppError::NotFound)?;

            // プロレーション計算
            let proration = calculate_proration(
                &current_sub,
                new_tier.monthly_price(),
                Utc::now(),
            );

            // Stripeでプラン更新
            let updated_sub = stripe_service.update_subscription_plan(
                &current_sub.stripe_subscription_id,
                &new_tier.stripe_price_id(),
            ).await?;

            // DB更新
            StripeSubscriptionRepository::update(
                txn,
                current_sub.id,
                updated_sub,
            ).await?;

            // 即時請求作成
            if proration.amount > 0 {
                PaymentRepository::create_proration_invoice(
                    txn,
                    user_id,
                    proration,
                ).await?;
            }

            Ok(UpgradeResult {
                new_tier,
                proration_amount: proration.amount,
                effective_date: Utc::now(),
            })
        })
    }).await
}
```

#### 3.2 ダウングレード時の考慮事項

- 現在の請求期間は維持
- 次回更新時に新プラン適用
- 使用量が新プラン上限を超える場合の対応
- データ保持期間の調整
- 機能制限の段階的適用
- クレジット付与の計算

```mermaid
stateDiagram-v2
    [*] --> EnterpriseActive: Enterprise契約中
    [*] --> ProActive: Pro契約中
    
    EnterpriseActive --> DowngradeToPro: Proへダウングレード申請
    ProActive --> DowngradeToFree: Freeへダウングレード申請
    
    DowngradeToPro --> ProPending: 期間終了待ち
    ProPending --> ProActive: 期間終了時
    Note right of ProPending: Enterprise機能は期間終了まで利用可
    
    DowngradeToFree --> FreePending: キャンセル待ち
    FreePending --> DataRetention: 期間終了
    DataRetention --> FreeActive: 30日後
    Note right of DataRetention: データ保持期間中
    
    FreeActive --> [*]
```

### 4. キャンセルシナリオ

#### 4.1 キャンセル理由の収集と分析

```rust
#[derive(Debug, Deserialize)]
pub struct CancelSubscriptionRequest {
    pub reason: CancellationReason,
    pub feedback: Option<String>,
    pub cancel_immediately: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CancellationReason {
    TooExpensive,
    NotUsingEnough,
    MissingFeatures,
    FoundAlternative,
    TemporaryBreak,
    Other,
}

pub async fn handle_cancellation(
    user_id: Uuid,
    request: CancelSubscriptionRequest,
) -> Result<(), AppError> {
    // アンケート保存
    CancellationSurveyRepository::create(
        user_id,
        request.reason,
        request.feedback,
    ).await?;

    if request.cancel_immediately {
        // 即時キャンセル（返金計算あり）
        let refund_amount = calculate_prorated_refund(user_id).await?;
        process_immediate_cancellation(user_id, refund_amount).await?;
    } else {
        // 期間終了時キャンセル
        schedule_end_of_period_cancellation(user_id).await?;
    }

    // Win-backキャンペーンのスケジュール
    schedule_win_back_campaign(user_id, request.reason).await?;

    Ok(())
}
```

#### 4.2 Win-backキャンペーン戦略

```mermaid
sequenceDiagram
    participant User
    participant System
    participant Email
    participant Analytics
    
    User->>System: キャンセル実行
    System->>Analytics: キャンセル理由記録
    System->>Email: 即時: キャンセル確認メール
    
    Note over System: 7日後
    System->>Email: フィードバック依頼
    
    Note over System: 30日後
    Analytics->>System: ユーザーセグメント分析
    System->>Email: 20%割引オファー
    
    Note over System: 60日後
    System->>Email: 新機能案内
    
    Note over System: 90日後
    System->>Email: 最終オファー（初月無料）
```

### 5. 特殊ケースの対応

#### 5.1 支払い方法の問題対応

```rust
pub enum PaymentMethodIssue {
    CardExpired {
        last_four: String,
        expiry_date: NaiveDate,
    },
    CardDeclined {
        reason: DeclineReason,
        attempted_at: DateTime<Utc>,
    },
    InsufficientFunds,
    BankAccountClosed,
}

impl PaymentMethodIssue {
    pub fn get_resolution_steps(&self) -> Vec<ResolutionStep> {
        match self {
            Self::CardExpired { .. } => vec![
                ResolutionStep::SendUpdateCardEmail,
                ResolutionStep::ShowInAppNotification,
                ResolutionStep::EnableGracePeriod(7),
            ],
            Self::CardDeclined { reason, .. } => match reason {
                DeclineReason::Fraud => vec![
                    ResolutionStep::ContactSupport,
                    ResolutionStep::TemporarilySuspend,
                ],
                DeclineReason::InsufficientFunds => vec![
                    ResolutionStep::RetryIn(Duration::days(3)),
                    ResolutionStep::SendPaymentReminderEmail,
                ],
                _ => vec![ResolutionStep::RequestAlternativePayment],
            },
            _ => vec![ResolutionStep::ContactSupport],
        }
    }
}
```

#### 5.2 多通貨対応と地域別価格設定

##### Stripeの多通貨対応フロー

```mermaid
flowchart TD
    A[ユーザーアクセス] --> B{地域判定}
    B -->|日本| C[JPY価格表示]
    B -->|米国| D[USD価格表示]
    B -->|EU| E[EUR価格表示]
    
    C --> F[Stripe Checkout]
    D --> F
    E --> F
    
    F --> G{決済処理}
    G --> H[各通貨で課金]
    
    H --> I[自動為替換算]
    I --> J[アカウント通貨で入金]
```

##### 価格設定の構造

```mermaid
graph TB
    subgraph "Stripeダッシュボード"
        P1[Product作成<br/>例: Pro Plan]
        P1 --> PR1[Price作成 - JPY<br/>¥3,000/月]
        P1 --> PR2[Price作成 - USD<br/>$20/月]
        P1 --> PR3[Price作成 - EUR<br/>€18/月]
    end
    
    subgraph "環境変数設定"
        PR1 --> E1[STRIPE_PRICE_ID_PRO_MONTHLY_JPY=price_xxx]
        PR2 --> E2[STRIPE_PRICE_ID_PRO_MONTHLY_USD=price_yyy]
        PR3 --> E3[STRIPE_PRICE_ID_PRO_MONTHLY_EUR=price_zzz]
    end
```

##### 多通貨対応の実装

```rust
// src/service/pricing_service.rs
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Currency {
    JPY,
    USD,
    EUR,
    GBP,
}

impl Currency {
    /// 地域から通貨を判定
    pub fn from_country_code(country: &str) -> Self {
        match country {
            "JP" => Self::JPY,
            "US" | "CA" => Self::USD,
            "GB" => Self::GBP,
            "DE" | "FR" | "IT" | "ES" | "NL" => Self::EUR,
            _ => Self::USD, // デフォルト
        }
    }
    
    /// 通貨記号を取得
    pub fn symbol(&self) -> &str {
        match self {
            Self::JPY => "¥",
            Self::USD => "$",
            Self::EUR => "€",
            Self::GBP => "£",
        }
    }
    
    /// 小数点以下桁数（Stripeの仕様）
    pub fn decimal_places(&self) -> u8 {
        match self {
            Self::JPY => 0, // 日本円は小数点なし
            _ => 2,
        }
    }
}

// 価格取得サービス
pub struct PricingService {
    prices: HashMap<Currency, PriceConfig>,
}

impl PricingService {
    /// ユーザーの地域に基づいて価格IDを取得
    pub fn get_price_id(
        &self,
        tier: SubscriptionTier,
        period: BillingPeriod,
        currency: Currency,
    ) -> Result<String, PricingError> {
        self.prices
            .get(&currency)
            .and_then(|config| config.price_ids.get(&tier))
            .and_then(|tier_prices| tier_prices.get(&period))
            .cloned()
            .ok_or(PricingError::PriceNotFound)
    }
    
    /// 表示用の価格フォーマット
    pub fn format_price(&self, amount: i64, currency: &Currency) -> String {
        match currency {
            Currency::JPY => format!("¥{:,}", amount),
            Currency::USD => format!("${:.2}", amount as f64 / 100.0),
            Currency::EUR => format!("€{:.2}", amount as f64 / 100.0),
            Currency::GBP => format!("£{:.2}", amount as f64 / 100.0),
        }
    }
}
```

##### 地域判定とCheckout Session作成

```rust
// src/api/handlers/subscription_handler.rs
pub async fn create_checkout_session(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<CreateCheckoutRequest>,
) -> Result<impl IntoResponse, AppError> {
    // IPアドレスから地域を判定
    let country = detect_country_from_ip(&headers)?;
    let currency = Currency::from_country_code(&country);
    
    // 適切な価格IDを取得
    let price_id = state.pricing_service.get_price_id(
        req.tier,
        req.billing_period,
        currency,
    )?;
    
    // Checkout Session作成（税金自動計算付き）
    let mut params = CreateCheckoutSession::new();
    params.mode = Some(CheckoutSessionMode::Subscription);
    params.line_items = Some(vec![
        CreateCheckoutSessionLineItems {
            price: Some(price_id),
            quantity: Some(1),
            ..Default::default()
        },
    ]);
    
    // 税金の自動計算を有効化
    params.automatic_tax = Some(CreateCheckoutSessionAutomaticTax {
        enabled: true,
        ..Default::default()
    });
    
    // 請求先住所の収集（税金計算に必要）
    params.billing_address_collection = Some(
        CheckoutSessionBillingAddressCollection::Required
    );
    
    let session = CheckoutSession::create(&state.stripe, params).await?;
    
    Ok(Json(CheckoutResponse {
        url: session.url.unwrap(),
        currency: currency.to_string(),
    }))
}
```

##### 為替レートと決済の仕組み

```mermaid
sequenceDiagram
    participant User as ユーザー（日本）
    participant App as アプリケーション
    participant Stripe as Stripe
    participant Bank as 銀行口座（USD）
    
    User->>App: プラン選択
    App->>App: 地域判定→JPY
    App->>User: ¥3,000/月 表示
    
    User->>Stripe: 決済（JPY）
    Stripe->>Stripe: JPYで課金処理
    
    Note over Stripe: 自動為替換算
    Stripe->>Bank: USD入金
    Note over Bank: 為替手数料2%控除後
```

##### 地域別税金計算

```rust
pub struct TaxCalculator {
    tax_rates: HashMap<String, TaxRate>,
}

impl TaxCalculator {
    pub async fn calculate_tax(
        &self,
        amount: Decimal,
        customer_location: &CustomerLocation,
    ) -> Result<TaxCalculation, AppError> {
        let tax_rate = match customer_location {
            CustomerLocation::Japan { prefecture } => {
                // 日本の消費税（10%）
                TaxRate::Fixed(Decimal::from_str("0.10")?)
            }
            CustomerLocation::US { state } => {
                // 州別売上税
                self.tax_rates.get(state)
                    .cloned()
                    .unwrap_or(TaxRate::Fixed(Decimal::ZERO))
            }
            CustomerLocation::EU { country, vat_number } => {
                // EU VAT（B2Bの場合はリバースチャージ）
                if vat_number.is_some() {
                    TaxRate::ReverseCharge
                } else {
                    self.get_eu_vat_rate(country)?
                }
            }
            _ => TaxRate::Fixed(Decimal::ZERO),
        };

        Ok(TaxCalculation {
            subtotal: amount,
            tax_rate,
            tax_amount: tax_rate.calculate(amount),
            total: amount + tax_rate.calculate(amount),
        })
    }
}
```

### 6. 年間・月間契約の切り替え

```rust
pub async fn switch_billing_period(
    user_id: Uuid,
    new_period: BillingPeriod,
) -> Result<(), AppError> {
    let current_sub = get_current_subscription(user_id).await?;
    
    match (current_sub.billing_period, new_period) {
        (BillingPeriod::Monthly, BillingPeriod::Yearly) => {
            // 月次→年次：残り月数分の割引適用
            let months_remaining = calculate_months_until_renewal(&current_sub);
            let discount = months_remaining as f64 * 0.17; // 年次は約17%割引
            apply_yearly_upgrade_credit(user_id, discount).await?;
        }
        (BillingPeriod::Yearly, BillingPeriod::Monthly) => {
            // 年次→月次：残り期間のクレジット付与
            let credit = calculate_unused_yearly_credit(&current_sub);
            apply_account_credit(user_id, credit).await?;
        }
        _ => {}
    }
    
    update_subscription_period(user_id, new_period).await
}
```

## スケーラビリティとパフォーマンス最適化

### 1. Webhook処理の水平スケーリング

```rust
// src/infrastructure/webhook_processor.rs
pub struct WebhookProcessor {
    redis: RedisPool,
    db: DatabaseConnection,
    workers: usize,
}

impl WebhookProcessor {
    /// 分散ロックを使用した重複処理防止
    pub async fn process_with_lock(
        &self,
        event_id: &str,
        processor: impl Future<Output = Result<(), AppError>>,
    ) -> Result<(), AppError> {
        let lock_key = format!("webhook:lock:{}", event_id);
        let lock_duration = Duration::seconds(300); // 5分

        // Redisで分散ロック取得
        let lock = self.redis
            .set_nx_ex(&lock_key, "1", lock_duration.num_seconds() as u64)
            .await?;

        if !lock {
            // 他のインスタンスが処理中
            return Ok(());
        }

        // 処理実行
        let result = processor.await;

        // ロック解放
        self.redis.del(&lock_key).await?;

        result
    }
}
```

### 2. 期限切れサブスクリプションのバッチ処理

```rust
// src/repository/stripe_subscription_repository.rs
impl StripeSubscriptionRepository {
    /// 複数サブスクリプションの一括処理（SKIP LOCKED使用）
    pub async fn process_expired_subscriptions_batch(
        db: &DatabaseConnection,
        batch_size: u64,
    ) -> Result<Vec<stripe_subscription::Model>, DbErr> {
        // PostgreSQL特有のSKIP LOCKED構文を使用
        let sql = r#"
            SELECT * FROM stripe_subscriptions
            WHERE status = 'active' 
              AND current_period_end < NOW()
            ORDER BY current_period_end
            LIMIT $1
            FOR UPDATE SKIP LOCKED
        "#;

        let subscriptions = stripe_subscription::Entity::find()
            .from_raw_sql(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                sql,
                vec![batch_size.into()],
            ))
            .all(db)
            .await?;

        Ok(subscriptions)
    }
}
```

### 3. 並列処理の最適化

```rust
// src/worker/batch_processor.rs
pub struct BatchProcessor {
    db: DatabaseConnection,
    batch_size: usize,
    parallel_workers: usize,
}

impl BatchProcessor {
    /// 期限切れサブスクリプションの並列処理
    pub async fn process_expired_subscriptions(&self) -> Result<(), AppError> {
        let (tx, mut rx) = mpsc::channel(self.batch_size);
        let semaphore = Arc::new(Semaphore::new(self.parallel_workers));

        // プロデューサー: バッチ読み込み
        let producer = tokio::spawn(async move {
            loop {
                let batch = StripeSubscriptionRepository::get_expired_batch(
                    &self.db,
                    self.batch_size,
                ).await?;

                if batch.is_empty() {
                    break;
                }

                for subscription in batch {
                    tx.send(subscription).await?;
                }
            }
            Ok::<(), AppError>(())
        });

        // コンシューマー: 並列処理
        let consumers = (0..self.parallel_workers)
            .map(|_| {
                let rx = rx.clone();
                let semaphore = semaphore.clone();
                
                tokio::spawn(async move {
                    while let Some(subscription) = rx.recv().await {
                        let _permit = semaphore.acquire().await?;
                        self.process_single_subscription(subscription).await?;
                    }
                    Ok::<(), AppError>(())
                })
            })
            .collect::<Vec<_>>();

        // 完了待機
        producer.await??;
        for consumer in consumers {
            consumer.await??;
        }

        Ok(())
    }
}
```

## セキュリティベストプラクティス

### 1. Webhook署名検証

```rust
// src/middleware/stripe_webhook_auth.rs
pub async fn verify_stripe_webhook(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // タイムスタンプ検証（リプレイ攻撃対策）
    let timestamp = extract_timestamp(&request)?;
    let current_time = Utc::now().timestamp();
    
    if (current_time - timestamp).abs() > 300 {
        return Err(AppError::Unauthorized("Webhook timestamp too old".into()));
    }

    // 署名検証
    let signature = extract_signature(&request)?;
    let body = extract_body(&request).await?;
    
    state.stripe_service.verify_webhook_signature(&body, &signature)?;

    // リクエストに検証済みフラグを付与
    let mut request = request;
    request.extensions_mut().insert(WebhookVerified);

    Ok(next.run(request).await)
}
```

### 2. 機密情報の暗号化

```rust
// src/security/encryption.rs
pub struct EncryptionService {
    key: Key<Aes256Gcm>,
}

impl EncryptionService {
    /// 支払い方法のメタデータを暗号化
    pub fn encrypt_payment_metadata(
        &self,
        metadata: &PaymentMethodMetadata,
    ) -> Result<EncryptedData, AppError> {
        let plaintext = serde_json::to_vec(metadata)?;
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        
        let ciphertext = self.cipher
            .encrypt(&nonce, plaintext.as_ref())
            .map_err(|_| AppError::EncryptionError)?;

        Ok(EncryptedData {
            nonce: nonce.to_vec(),
            ciphertext,
        })
    }

    /// 監査ログのための選択的復号
    pub fn decrypt_for_audit(
        &self,
        encrypted: &EncryptedData,
        audit_context: &AuditContext,
    ) -> Result<String, AppError> {
        // 監査権限の確認
        if !audit_context.has_permission(Permission::ViewSensitiveData) {
            return Err(AppError::Forbidden);
        }

        // 監査ログ記録
        AuditLogger::log_sensitive_data_access(
            audit_context.user_id,
            "payment_metadata_decryption",
            audit_context.reason,
        ).await?;

        // 復号実行
        let plaintext = self.decrypt(encrypted)?;
        Ok(String::from_utf8(plaintext)?)
    }
}
```

### 3. エラーハンドリング

```rust
#[derive(Debug, thiserror::Error)]
pub enum StripeError {
    #[error("Stripe API error: {0}")]
    ApiError(String),
    
    #[error("Invalid webhook signature")]
    InvalidSignature,
    
    #[error("Duplicate webhook event")]
    DuplicateEvent,
    
    #[error("Payment failed: {0}")]
    PaymentFailed(String),
}

// HTTPレスポンス変換
impl IntoResponse for StripeError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::ApiError(_) => (StatusCode::BAD_GATEWAY, "Payment service error"),
            Self::InvalidSignature => (StatusCode::UNAUTHORIZED, "Invalid webhook signature"),
            Self::DuplicateEvent => (StatusCode::OK, "Already processed"),
            Self::PaymentFailed(ref reason) => {
                if reason.contains("insufficient_funds") {
                    (StatusCode::PAYMENT_REQUIRED, "Insufficient funds")
                } else {
                    (StatusCode::PAYMENT_REQUIRED, "Payment failed")
                }
            }
        };

        Json(json!({
            "error": message,
            "code": status.as_u16()
        }))
        .into_response()
    }
}
```

## 監視とアラート

### 1. メトリクス収集

```rust
// src/monitoring/subscription_metrics.rs
pub struct SubscriptionMetrics {
    prometheus: PrometheusRegistry,
}

impl SubscriptionMetrics {
    pub fn record_payment_result(&self, success: bool, amount: f64, reason: Option<&str>) {
        self.payment_attempts
            .with_label_values(&[if success { "success" } else { "failure" }])
            .inc();

        if success {
            self.successful_payment_amount.observe(amount);
        } else {
            self.failed_payments_by_reason
                .with_label_values(&[reason.unwrap_or("unknown")])
                .inc();
        }
    }

    pub fn record_subscription_change(
        &self,
        from_tier: &str,
        to_tier: &str,
        change_type: &str,
    ) {
        self.subscription_changes
            .with_label_values(&[from_tier, to_tier, change_type])
            .inc();
    }
}
```

### 2. アラート設定

```yaml
# prometheus/alerts.yml
groups:
  - name: subscription_alerts
    rules:
      - alert: HighPaymentFailureRate
        expr: rate(payment_failures[5m]) > 0.1
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "Payment failure rate is above 10%"
          description: "{{ $value | humanizePercentage }} of payments are failing"

      - alert: UnusualCancellationRate
        expr: rate(subscription_cancellations[1h]) > 0.05
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "High cancellation rate detected"
          description: "{{ $value | humanizePercentage }} cancellation rate in the last hour"

      - alert: WebhookProcessingDelay
        expr: webhook_processing_duration_seconds > 30
        for: 5m
        labels:
          severity: error
        annotations:
          summary: "Webhook processing is taking too long"
          description: "Average processing time: {{ $value }}s"
```

## テスト戦略

### 1. 単体テスト

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockito;

    #[tokio::test]
    async fn test_create_checkout_session() {
        let mut server = mockito::Server::new();
        let mock = server.mock("POST", "/v1/checkout/sessions")
            .with_status(200)
            .with_body(r#"{
                "id": "cs_test_123",
                "url": "https://checkout.stripe.com/pay/cs_test_123"
            }"#)
            .create();

        let service = StripeService::new_with_url(
            "sk_test_123".into(),
            &server.url(),
        );

        let result = service.create_checkout_session(
            "cus_123",
            "price_123",
            Uuid::new_v4(),
        ).await;

        assert!(result.is_ok());
        mock.assert();
    }
}
```

### 2. 統合テスト

```rust
// tests/integration/subscription_test.rs
#[tokio::test]
async fn test_subscription_lifecycle() {
    let app = create_test_app().await;
    let user = create_test_user(&app).await;

    // 1. Checkout session作成
    let response = app.post("/api/subscriptions/checkout")
        .json(&json!({
            "price_id": "price_pro_monthly",
            "success_url": "http://test.com/success",
            "cancel_url": "http://test.com/cancel"
        }))
        .send()
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let checkout_url = response.json::<CheckoutResponse>().await.url;

    // 2. Webhook処理シミュレーション
    let webhook_payload = create_checkout_completed_event(&user);
    let signature = sign_webhook(&webhook_payload);

    let response = app.post("/api/webhooks/stripe")
        .header("stripe-signature", signature)
        .body(webhook_payload)
        .send()
        .await;

    assert_eq!(response.status(), StatusCode::OK);

    // 3. サブスクリプション状態確認
    let subscription = get_user_subscription(&app, &user).await;
    assert_eq!(subscription.tier, SubscriptionTier::Pro);
    assert_eq!(subscription.status, "active");
}
```

### 3. E2Eテスト with Stripe CLI

```bash
# Stripe CLIでWebhookをローカルにフォワード
stripe listen --forward-to localhost:3000/api/webhooks/stripe

# テストイベントの送信
stripe trigger checkout.session.completed
stripe trigger invoice.payment_succeeded
stripe trigger customer.subscription.deleted
```

## 運用上の考慮事項

### 1. 移行戦略

#### 移行戦略の概要

「移行戦略」とは、既にシステム内でPro/Enterprise階層を持っているが、まだStripe決済と連携していないユーザーを、Stripe決済と連携させるプロセスを指します。

```mermaid
stateDiagram-v2
    state "既存システム（Stripe未連携）" as OldSystem {
        [*] --> ManualPro: 手動でPro設定
        [*] --> ManualEnterprise: 手動でEnterprise設定
        ManualPro --> ManualPro: 管理者が手動管理
        ManualEnterprise --> ManualEnterprise: 管理者が手動管理
    }
    
    state "移行プロセス" as Migration {
        CheckUser --> CreateStripeCustomer: Stripe顧客作成
        CreateStripeCustomer --> CreateSubscription: サブスクリプション作成（請求なし）
        CreateSubscription --> UpdateDB: DB記録
        UpdateDB --> NextUser: 次のユーザーへ
    }
    
    state "新システム（Stripe連携済み）" as NewSystem {
        StripePro --> StripePro: Stripe自動課金
        StripeEnterprise --> StripeEnterprise: Stripe自動課金
        [*] --> StripePro
        [*] --> StripeEnterprise
    }
    
    OldSystem --> Migration: 移行バッチ実行
    Migration --> NewSystem: 移行完了
```

#### 詳細な移行フロー

```mermaid
sequenceDiagram
    participant Admin as 管理者
    participant Batch as 移行バッチ
    participant DB as Database
    participant Stripe as Stripe API
    participant User as ユーザー
    
    Admin->>Batch: 移行処理開始
    
    loop 各既存課金ユーザー
        Batch->>DB: Pro/Enterpriseユーザー取得
        DB-->>Batch: ユーザー情報
        
        Note over Batch: レート制限考慮（100ms待機）
        
        Batch->>Stripe: Customer作成（メール、名前）
        Stripe-->>Batch: customer_id
        
        Batch->>Stripe: Subscription作成
        Note over Stripe: trial_from_plan=true
        Note over Stripe: 初回請求スキップ
        Stripe-->>Batch: subscription_id
        
        Batch->>DB: トランザクション開始
        Batch->>DB: stripe_customers挿入
        Batch->>DB: stripe_subscriptions挿入
        Batch->>DB: コミット
        
        Batch->>Batch: 成功カウント++
    end
    
    Batch->>Admin: 移行レポート表示
    Note over Admin: 成功: 1250件<br/>失敗: 3件<br/>エラー詳細...
    
    Note over User: この時点でユーザーは<br/>何も気づかない
    
    Note over Stripe: 次回請求日から<br/>自動課金開始
```

#### 移行シナリオの種類

```mermaid
flowchart TD
    subgraph "シナリオ1: 既存課金ユーザーの移行"
        A1[手動管理のPro/Enterprise] --> B1[Stripeに顧客作成]
        B1 --> C1[請求なしでサブスク作成]
        C1 --> D1[次回更新から自動課金]
    end
    
    subgraph "シナリオ2: 新規ユーザーフロー"
        A2[新規ユーザー登録] --> B2{プラン選択}
        B2 -->|Free| C2[そのまま利用]
        B2 -->|Pro/Enterprise| D2[Stripe Checkout]
        D2 --> E2[決済情報入力]
        E2 --> F2[即時課金開始]
    end
    
    subgraph "シナリオ3: 無料→有料アップグレード"
        A3[既存Freeユーザー] --> B3[アップグレード選択]
        B3 --> C3[Stripe Checkout]
        C3 --> D3[決済情報入力]
        D3 --> E3[プロレーション課金]
    end
    
    subgraph "シナリオ4: ダウングレード移行"
        A4[Pro/Enterpriseユーザー] --> B4{ダウングレード種別}
        B4 -->|別プランへ| C4[期間終了時に変更]
        B4 -->|Freeへ| D4[サブスクキャンセル]
        C4 --> E4[差額はクレジット付与]
        D4 --> F4[期間終了までPro機能利用可]
    end
```

#### ダウングレード移行の詳細フロー

```mermaid
sequenceDiagram
    participant User as ユーザー
    participant App as アプリケーション
    participant Stripe as Stripe API
    participant DB as Database
    
    alt Enterprise→Proダウングレード
        User->>App: Proプランへダウングレード選択
        App->>Stripe: Subscription Update (at period end)
        Stripe-->>App: 次回更新時に変更確認
        App->>DB: subscription_histories挿入
        App->>User: 現在の期間はEnterprise継続通知
        
        Note over Stripe: 次回更新日
        Stripe->>App: Webhook: subscription.updated
        App->>DB: users.subscription_tier = 'Pro'
        App->>User: ダウングレード完了通知
    else Pro/Enterprise→Freeダウングレード
        User->>App: 無料プランへ変更選択
        App->>App: データ保持ポリシー確認
        App->>User: 機能制限の警告表示
        User->>App: ダウングレード確定
        
        App->>Stripe: Cancel Subscription at Period End
        Stripe-->>App: キャンセル確認
        App->>DB: subscription.cancel_at_period_end = true
        
        Note over User: 期間終了まで有料機能利用可能
        
        Note over Stripe: 期間終了時
        Stripe->>App: Webhook: subscription.deleted
        App->>DB: users.subscription_tier = 'Free'
        App->>App: データクォータ適用
        App->>User: Free移行完了通知
    end
```

#### ダウングレード時の考慮事項

```rust
// src/service/downgrade_service.rs
pub struct DowngradeService {
    db: DatabaseConnection,
    stripe: StripeService,
}

impl DowngradeService {
    /// ダウングレード可能性の検証
    pub async fn validate_downgrade(
        &self,
        user: &User,
        target_tier: SubscriptionTier,
    ) -> Result<DowngradeValidation, DowngradeError> {
        let current_usage = self.get_current_usage(user.id).await?;
        let target_limits = target_tier.get_limits();
        
        let validation = DowngradeValidation {
            can_downgrade: true,
            warnings: vec![],
            required_actions: vec![],
        };
        
        // データ使用量チェック
        if current_usage.storage_gb > target_limits.storage_gb {
            validation.warnings.push(format!(
                "ストレージ使用量({:.1}GB)が新プラン上限({:.1}GB)を超過",
                current_usage.storage_gb,
                target_limits.storage_gb
            ));
            validation.required_actions.push(RequiredAction::ReduceStorage {
                current: current_usage.storage_gb,
                target: target_limits.storage_gb,
            });
        }
        
        // チームメンバー数チェック
        if current_usage.team_members > target_limits.max_team_members {
            validation.warnings.push(format!(
                "チームメンバー数({})が新プラン上限({})を超過",
                current_usage.team_members,
                target_limits.max_team_members
            ));
            validation.required_actions.push(RequiredAction::RemoveTeamMembers {
                current: current_usage.team_members,
                target: target_limits.max_team_members,
            });
        }
        
        // API使用量チェック（月間）
        if current_usage.api_calls_this_month > target_limits.monthly_api_calls {
            validation.warnings.push(format!(
                "今月のAPI使用量({})が新プラン上限({})を超過",
                current_usage.api_calls_this_month,
                target_limits.monthly_api_calls
            ));
            // APIは即座に制限されるため、ダウングレード不可
            validation.can_downgrade = false;
        }
        
        Ok(validation)
    }
    
    /// ダウングレード実行
    pub async fn execute_downgrade(
        &self,
        user_id: Uuid,
        target_tier: SubscriptionTier,
    ) -> Result<DowngradeResult, AppError> {
        // 現在のサブスクリプション取得
        let current_sub = StripeSubscriptionRepository::find_active_by_user(
            &self.db,
            user_id
        ).await?.ok_or(AppError::NotFound)?;
        
        match target_tier {
            SubscriptionTier::Free => {
                // 無料プランへのダウングレード = キャンセル
                self.stripe.cancel_subscription_at_period_end(
                    &current_sub.stripe_subscription_id
                ).await?;
                
                // データ保持期間の設定
                DataRetentionRepository::schedule_cleanup(
                    &self.db,
                    user_id,
                    current_sub.current_period_end + Duration::days(30),
                ).await?;
                
                Ok(DowngradeResult::CancelScheduled {
                    effective_date: current_sub.current_period_end,
                    data_retention_days: 30,
                })
            }
            _ => {
                // 別の有料プランへのダウングレード
                let new_price_id = self.get_price_id_for_tier(target_tier)?;
                
                self.stripe.update_subscription_at_period_end(
                    &current_sub.stripe_subscription_id,
                    &new_price_id,
                ).await?;
                
                // 差額計算（クレジット付与の可能性）
                let credit = self.calculate_downgrade_credit(
                    &current_sub,
                    target_tier,
                ).await?;
                
                if credit > 0 {
                    AccountCreditRepository::add_credit(
                        &self.db,
                        user_id,
                        credit,
                        "ダウングレードによるクレジット",
                    ).await?;
                }
                
                Ok(DowngradeResult::DowngradeScheduled {
                    new_tier: target_tier,
                    effective_date: current_sub.current_period_end,
                    credit_amount: credit,
                })
            }
        }
    }
    
    /// データ移行とクリーンアップ
    pub async fn prepare_for_downgrade(
        &self,
        user_id: Uuid,
        target_tier: SubscriptionTier,
    ) -> Result<(), AppError> {
        let target_limits = target_tier.get_limits();
        
        // 1. 古いデータのアーカイブ
        if target_tier == SubscriptionTier::Free {
            ArchiveService::archive_user_data(user_id).await?;
        }
        
        // 2. 機能の無効化
        FeatureToggleRepository::disable_premium_features(
            &self.db,
            user_id,
            target_tier,
        ).await?;
        
        // 3. 通知設定の調整
        NotificationSettingsRepository::adjust_for_tier(
            &self.db,
            user_id,
            target_tier,
        ).await?;
        
        Ok(())
    }
}
```

#### 移行実装コード

```rust
// src/migration/stripe_migration.rs
pub async fn migrate_existing_subscriptions(
    db: &DatabaseConnection,
    stripe: &StripeService,
) -> Result<MigrationReport, AppError> {
    let mut report = MigrationReport::default();
    
    // 既存のPro/Enterpriseユーザーを取得
    let users = UserRepository::find_paid_users(db).await?;
    
    for user in users {
        match migrate_single_user(db, stripe, &user).await {
            Ok(_) => report.successful += 1,
            Err(e) => {
                report.failed += 1;
                report.errors.push(format!("User {}: {}", user.id, e));
                // エラーは記録するが処理は継続
            }
        }
        
        // レート制限を考慮
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    report
}

async fn migrate_single_user(
    db: &DatabaseConnection,
    stripe: &StripeService,
    user: &User,
) -> Result<(), AppError> {
    // 1. Stripe顧客作成
    let customer = stripe.create_customer(
        &user.email,
        user.username.as_deref(),
    ).await?;
    
    // 2. サブスクリプション作成（初回請求なし）
    let subscription = stripe.create_subscription_without_charge(
        &customer.id,
        &get_price_id_for_tier(user.subscription_tier),
    ).await?;
    
    // 3. DBに記録
    db.transaction(|txn| {
        Box::pin(async move {
            StripeCustomerRepository::create(txn, user.id, &customer).await?;
            StripeSubscriptionRepository::create(txn, user.id, &subscription).await?;
            Ok(())
        })
    }).await
}
```

#### 追加の移行パターン

##### シナリオ5: 支払い失敗からの復旧

```mermaid
sequenceDiagram
    participant User as ユーザー
    participant App as アプリケーション
    participant Stripe as Stripe API
    participant DB as Database
    participant Queue as Job Queue
    
    Note over Stripe: 支払い失敗発生
    Stripe->>App: Webhook: invoice.payment_failed
    App->>DB: subscription.status = 'past_due'
    App->>Queue: 支払い失敗通知メール送信
    
    loop 復旧試行（最大3回）
        Note over Stripe: Stripeの自動リトライ
        alt 支払い成功
            Stripe->>App: Webhook: invoice.payment_succeeded
            App->>DB: subscription.status = 'active'
            App->>User: 復旧成功通知
            Note over User: サービス継続
        else 支払い失敗継続
            Note over App: 次回リトライまで待機
        end
    end
    
    alt 最終的に失敗
        Stripe->>App: Webhook: subscription.deleted
        App->>DB: users.subscription_tier = 'Free'
        App->>User: サービスダウングレード通知
    end
```

##### シナリオ6: 組織間の移行

```mermaid
flowchart TD
    subgraph "個人→組織移行"
        A1[個人Pro契約] --> B1[組織に参加]
        B1 --> C1{組織の契約確認}
        C1 -->|組織がEnterprise| D1[個人契約キャンセル]
        C1 -->|組織がFree| E1[個人契約維持]
        D1 --> F1[組織の権限で利用]
    end
    
    subgraph "組織→個人移行"
        A2[組織Enterprise利用] --> B2[組織から離脱]
        B2 --> C2{個人契約の選択}
        C2 -->|継続希望| D2[新規個人契約作成]
        C2 -->|継続不要| E2[Freeプランへ]
        D2 --> F2[プロレーション調整]
    end
```

##### シナリオ7: 請求周期の変更

```rust
// src/service/billing_cycle_service.rs
pub async fn change_billing_cycle(
    &self,
    user_id: Uuid,
    new_period: BillingPeriod,
) -> Result<BillingCycleChangeResult, AppError> {
    let current_sub = self.get_active_subscription(user_id).await?;
    
    // プロレーション計算
    let proration = match (current_sub.billing_period, new_period) {
        (BillingPeriod::Monthly, BillingPeriod::Yearly) => {
            // 月次→年次: 残り日数分の返金と年額の請求
            let days_remaining = (current_sub.current_period_end - Utc::now()).num_days();
            let monthly_rate = current_sub.amount / 30;
            let credit = monthly_rate * days_remaining;
            let annual_cost = current_sub.amount * 12 * 0.8; // 20%割引
            ProrationResult::UpgradeToYearly {
                credit,
                charge: annual_cost - credit,
            }
        }
        (BillingPeriod::Yearly, BillingPeriod::Monthly) => {
            // 年次→月次: 未使用分をクレジットとして保持
            let months_remaining = (current_sub.current_period_end - Utc::now()).num_days() / 30;
            let monthly_value = current_sub.amount / 12;
            ProrationResult::DowngradeToMonthly {
                credit: monthly_value * months_remaining,
                next_charge_date: current_sub.current_period_end,
            }
        }
        _ => ProrationResult::NoChange,
    };
    
    // Stripeサブスクリプション更新
    self.stripe.update_subscription_schedule(
        &current_sub.stripe_subscription_id,
        &new_period,
        &proration,
    ).await?;
    
    Ok(BillingCycleChangeResult {
        new_period,
        proration,
        effective_date: Utc::now(),
    })
}
```

##### トライアル期間の移行パターン

```mermaid
stateDiagram-v2
    [*] --> Trial: 無料トライアル開始（14日間）
    
    Trial --> ConvertBeforeEnd: トライアル中に有料化
    Trial --> TrialExpired: トライアル期限切れ
    
    ConvertBeforeEnd --> ActivePaid: 即時課金開始
    
    TrialExpired --> GracePeriod: 猶予期間（3日間）
    GracePeriod --> ConvertInGrace: 猶予期間中に有料化
    GracePeriod --> DowngradeToFree: 無料プランへ自動移行
    
    ConvertInGrace --> ActivePaid: 課金開始
    DowngradeToFree --> [*]
    
    ActivePaid --> [*]
    
    note right of Trial
        機能制限なし
        決済情報不要
    end note
    
    note right of GracePeriod
        機能制限あり
        データは保持
    end note
```

##### 特殊な移行ケースの処理

```rust
// src/service/special_migration_service.rs
pub struct SpecialMigrationService {
    db: DatabaseConnection,
    stripe: StripeService,
    notification: NotificationService,
}

impl SpecialMigrationService {
    /// プロモーション移行（特別料金での移行）
    pub async fn promotional_migration(
        &self,
        user_id: Uuid,
        promo_code: &str,
    ) -> Result<(), AppError> {
        // プロモーションコード検証
        let promotion = PromotionRepository::validate_code(
            &self.db,
            promo_code,
        ).await?;
        
        // Stripeクーポン適用
        let coupon = self.stripe.apply_coupon(
            user_id,
            &promotion.stripe_coupon_id,
        ).await?;
        
        // 履歴記録
        SubscriptionHistoryRepository::create_with_metadata(
            &self.db,
            user_id,
            json!({
                "migration_type": "promotional",
                "promo_code": promo_code,
                "discount_percent": promotion.discount_percent,
                "valid_months": promotion.valid_months,
            }),
        ).await?;
        
        Ok(())
    }
    
    /// 緊急移行（システム障害やビジネス判断による）
    pub async fn emergency_migration(
        &self,
        user_ids: Vec<Uuid>,
        target_tier: SubscriptionTier,
        reason: &str,
    ) -> Result<EmergencyMigrationReport, AppError> {
        let mut report = EmergencyMigrationReport::default();
        
        // バッチ処理で一括移行
        for chunk in user_ids.chunks(100) {
            let results = futures::future::join_all(
                chunk.iter().map(|user_id| {
                    self.migrate_single_emergency(*user_id, target_tier, reason)
                })
            ).await;
            
            for result in results {
                match result {
                    Ok(_) => report.successful += 1,
                    Err(e) => {
                        report.failed += 1;
                        report.errors.push(e.to_string());
                    }
                }
            }
            
            // Stripeのレート制限対策
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
        
        // 管理者への報告
        self.notification.send_admin_report(
            "緊急移行完了",
            &report,
        ).await?;
        
        Ok(report)
    }
}
```

##### シナリオ8: サブスクリプションの一時停止・再開

```mermaid
sequenceDiagram
    participant User as ユーザー
    participant App as アプリケーション
    participant Stripe as Stripe API
    participant DB as Database
    
    User->>App: 一時停止リクエスト
    App->>App: 一時停止理由の確認
    
    alt 短期停止（1-3ヶ月）
        App->>Stripe: Pause Subscription
        Stripe-->>App: 停止確認
        App->>DB: subscription.status = 'paused'
        App->>DB: pause_end_date設定
        App->>User: 再開予定日通知
        
        Note over User: 期間中アクセス制限
        
        Note over App: 再開予定日到達
        App->>Stripe: Resume Subscription
        App->>DB: subscription.status = 'active'
        App->>User: 再開通知
    else 長期停止（3ヶ月以上）
        App->>User: キャンセル推奨の提案
        User->>App: 停止継続を選択
        App->>Stripe: Cancel with Grace Period
        App->>DB: 長期停止フラグ設定
    end
```

##### シナリオ9: 返金・払い戻し処理

```rust
// src/service/refund_service.rs
pub async fn process_refund(
    &self,
    user_id: Uuid,
    refund_request: RefundRequest,
) -> Result<RefundResult, AppError> {
    // 返金ポリシーの確認
    let policy_check = self.validate_refund_policy(
        &refund_request,
        user_id,
    ).await?;
    
    match refund_request.reason {
        RefundReason::ServiceIssue => {
            // サービス側の問題：全額返金
            self.stripe.create_refund(
                &refund_request.payment_id,
                None, // 全額
            ).await?
        }
        RefundReason::UserRequest => {
            // ユーザー都合：日割り計算
            let refund_amount = self.calculate_prorated_refund(
                &refund_request,
            ).await?;
            
            self.stripe.create_refund(
                &refund_request.payment_id,
                Some(refund_amount),
            ).await?
        }
        RefundReason::Duplicate => {
            // 重複課金：全額返金
            self.stripe.create_refund(
                &refund_request.payment_id,
                None,
            ).await?
        }
    }
    
    // サブスクリプション調整
    if refund_request.cancel_subscription {
        self.cancel_subscription_immediately(user_id).await?;
    }
    
    Ok(RefundResult {
        refund_id: refund.id,
        amount: refund.amount,
        status: RefundStatus::Completed,
    })
}
```

##### シナリオ10: 地域/通貨変更に伴う移行

```mermaid
flowchart TD
    subgraph "地域変更フロー"
        A[日本在住・JPY課金] --> B[海外転居通知]
        B --> C{新地域の確認}
        C -->|米国| D[USD価格への切替]
        C -->|EU| E[EUR価格への切替]
        D --> F[次回請求から新通貨]
        E --> F
        F --> G[税金設定の更新]
    end
    
    subgraph "価格調整"
        H[現在の契約期間] --> I{調整方法}
        I -->|即時変更| J[プロレーション計算]
        I -->|期間終了時| K[次回更新時に適用]
        J --> L[差額調整]
    end
```

##### シナリオ11: グランドファザリング（旧プラン維持）

```rust
// src/service/grandfathering_service.rs
pub struct GrandfatheringService {
    legacy_plans: HashMap<String, LegacyPlan>,
}

impl GrandfatheringService {
    /// 旧プランから新プランへの移行オファー
    pub async fn offer_migration(
        &self,
        user: &User,
    ) -> Option<MigrationOffer> {
        if let Some(legacy_plan) = self.is_on_legacy_plan(user) {
            // 特別オファーの生成
            let offer = MigrationOffer {
                current_plan: legacy_plan.name.clone(),
                current_price: legacy_plan.price,
                new_plan: self.find_equivalent_plan(&legacy_plan),
                special_price: legacy_plan.price, // 同価格保証
                benefits: vec![
                    "現在の価格を永続的に維持",
                    "新機能へのアクセス",
                    "優先サポート",
                ],
                expiry_date: Utc::now() + Duration::days(30),
            };
            
            Some(offer)
        } else {
            None
        }
    }
    
    /// 強制移行（サービス終了時）
    pub async fn force_migration(
        &self,
        legacy_users: Vec<User>,
    ) -> Result<MigrationReport, AppError> {
        let mut report = MigrationReport::default();
        
        for user in legacy_users {
            // 最も近い新プランへマッピング
            let new_plan = self.map_to_new_plan(&user.legacy_plan);
            
            // 特別価格の適用（最大12ヶ月）
            let discount = self.calculate_transition_discount(&user);
            
            match self.migrate_with_benefits(user, new_plan, discount).await {
                Ok(_) => {
                    report.successful += 1;
                    // 移行完了通知（特典説明付き）
                    self.send_migration_notice(user.id).await?;
                }
                Err(e) => {
                    report.failed += 1;
                    report.require_manual_review.push(user.id);
                }
            }
        }
        
        Ok(report)
    }
}
```

##### シナリオ12: 複数アカウントの統合

```mermaid
sequenceDiagram
    participant User as ユーザー
    participant App as アプリケーション
    participant Stripe as Stripe API
    participant DB as Database
    
    User->>App: アカウント統合リクエスト
    App->>DB: 複数アカウント検証
    DB-->>App: アカウントA(Pro), アカウントB(Free)
    
    App->>App: 統合プランの決定
    Note over App: 最上位プランを採用
    
    App->>DB: トランザクション開始
    App->>DB: データ移行（タスク、設定等）
    App->>DB: 権限の統合
    
    App->>Stripe: 重複サブスクリプション確認
    alt 重複課金あり
        App->>Stripe: 古いサブスクリプションキャンセル
        App->>App: 返金額の計算
        App->>Stripe: 部分返金処理
    end
    
    App->>DB: アカウントBを無効化
    App->>DB: 統合履歴の記録
    App->>DB: コミット
    
    App->>User: 統合完了通知
    Note over User: 統合後の特典期間付与
```

### 2. 監視項目

- 決済成功率/失敗率
- Webhook処理遅延
- サブスクリプションのチャーン率
- MRR（月次経常収益）の推移
- 支払い方法別の失敗率
- 地域別の決済パフォーマンス
- 通貨別の売上と為替影響

#### 通貨別レポーティング

```rust
// src/service/reporting_service.rs
pub async fn get_revenue_by_currency(
    db: &DatabaseConnection,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
) -> Result<HashMap<Currency, RevenueReport>, AppError> {
    let payments = PaymentRepository::find_by_date_range(
        db,
        start_date,
        end_date,
    ).await?;
    
    let mut revenue_by_currency = HashMap::new();
    
    for payment in payments {
        let currency = Currency::from_str(&payment.currency)?;
        let report = revenue_by_currency.entry(currency).or_insert(RevenueReport {
            total_amount: 0,
            transaction_count: 0,
            average_amount: 0,
        });
        
        report.total_amount += payment.amount_cents;
        report.transaction_count += 1;
    }
    
    // 平均額の計算
    for report in revenue_by_currency.values_mut() {
        if report.transaction_count > 0 {
            report.average_amount = report.total_amount / report.transaction_count as i64;
        }
    }
    
    Ok(revenue_by_currency)
}

// 為替レート変動の影響を分析
pub async fn analyze_exchange_rate_impact(
    stripe: &StripeService,
    base_currency: Currency,
) -> Result<ExchangeRateImpact, AppError> {
    // Stripeから実際の為替レート情報を取得
    let balance_transactions = stripe.list_balance_transactions().await?;
    
    let mut impact = ExchangeRateImpact::default();
    
    for transaction in balance_transactions {
        if let Some(exchange_rate) = transaction.exchange_rate {
            impact.add_transaction(
                transaction.currency,
                transaction.amount,
                exchange_rate,
            );
        }
    }
    
    Ok(impact)
}
```

### 3. コンプライアンス

- PCI DSS準拠（カード情報非保持）
- GDPR対応（決済データの削除・エクスポート）
- 特定商取引法に基づく表記
- SCA（強力な顧客認証）対応
- 税務コンプライアンス（インボイス制度対応）

## まとめ

本ドキュメントは、既存のRustバックエンドシステムにStripe決済機能を統合するための包括的なガイドラインです。主要な実装ポイント：

1. **段階的な実装アプローチ**: 基盤構築から始め、徐々に機能を拡張
2. **堅牢なエラーハンドリング**: リトライ戦略とダニング管理の実装
3. **スケーラビリティの考慮**: 水平スケーリングと分散処理対応
4. **セキュリティの徹底**: PCI DSS準拠と暗号化の実装
5. **ビジネスシナリオの網羅**: あらゆるケースに対応した設計

実装時は、このガイドラインを参考にしながら、ビジネス要件に応じて適切にカスタマイズしてください。特に、以下の点に注意：

- **SeaORMのトランザクション管理**を活用した一貫性のあるデータ更新
- **非同期処理とリトライ戦略**による信頼性の高い決済処理
- **イベント駆動アーキテクチャ**による疎結合な設計
- **包括的な監視とアラート**による運用の安定性確保

継続的な改善を通じて、ユーザーにとって使いやすく、ビジネスにとって収益性の高いサブスクリプションシステムを構築してください。