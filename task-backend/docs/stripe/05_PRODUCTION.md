# 本番環境移行ガイド

## ⚠️ 重要な注意事項

```
本番環境への移行前に、必ずテスト環境で全機能を検証してください。
本番環境では実際の決済が発生します。
```

## 📋 目次

1. [事前準備](#事前準備)
2. [Stripe本番設定](#stripe本番設定)
3. [環境変数の更新](#環境変数の更新)
4. [デプロイチェックリスト](#デプロイチェックリスト)
5. [移行後の確認](#移行後の確認)

## 📝 事前準備

### 1. Stripeアカウントの本番化

1. [Stripeダッシュボード](https://dashboard.stripe.com/)にログイン
2. アカウント設定を完了：
   - ✅ ビジネス情報の登録
   - ✅ 銀行口座の登録
   - ✅ 本人確認書類の提出
   - ✅ 利用規約への同意

### 2. 本番APIキーの確認

```mermaid
flowchart LR
    A[ダッシュボード] --> B[本番モード切替]
    B --> C[開発者]
    C --> D[APIキー]
    D --> E[本番キー確認]
    E --> F["sk_live_"で始まるキー]
```

### 3. セキュリティ設定

- ✅ APIキーのアクセス権限を最小限に設定
- ✅ Webhookエンドポイントの署名シークレットを必須化
- ✅ レート制限の設定

## 💳 Stripe本番設定

### 1. 商品・価格の作成

本番モードで商品を再作成：

```
⚠️ テストモードの商品は本番モードでは使用できません
```

#### 手順

1. ダッシュボードを「本番モード」に切り替え
2. 商品 → 商品を追加
3. テスト環境と同じ商品・価格を作成
4. 価格IDを記録（`price_live_XXXXX`）

### 2. カスタマーポータル設定

本番環境用の設定：

1. 設定 → Billing → カスタマーポータル
2. 本番URLを設定：
   ```
   ホームページURL: https://your-domain.com
   利用規約URL: https://your-domain.com/terms
   プライバシーポリシーURL: https://your-domain.com/privacy
   ```

### 3. Webhookエンドポイント設定

1. 開発者 → Webhooks → エンドポイントを追加
2. 設定内容：
   ```
   エンドポイントURL: https://api.your-domain.com/webhooks/stripe
   
   リッスンするイベント:
   - checkout.session.completed
   - customer.subscription.deleted
   - invoice.payment_failed
   ```
3. 署名シークレットを記録

## 🔧 環境変数の更新

### 本番環境の.env

```bash
# 環境設定
ENVIRONMENT=production

# データベース（本番DB）
DATABASE_URL=postgresql://user:password@production-db:5432/prod_db

# 決済設定
PAYMENT_DEVELOPMENT_MODE=false  # 必ずfalse

# Stripe本番キー
STRIPE_SECRET_KEY=sk_live_XXXXX        # 本番シークレットキー
STRIPE_PUBLISHABLE_KEY=pk_live_XXXXX   # 本番公開可能キー

# 本番価格ID
STRIPE_PRO_PRICE_ID=price_live_XXXXX
STRIPE_ENTERPRISE_PRICE_ID=price_live_XXXXX

# Webhook署名（必須）
STRIPE_WEBHOOK_SECRET=whsec_XXXXX      # 本番Webhookシークレット

# セキュリティ
JWT_SECRET_KEY=your-production-secret-key-minimum-32-chars
FRONTEND_URL=https://your-frontend-domain.com
CORS_ALLOWED_ORIGINS=https://your-frontend-domain.com
```

### 環境変数の管理

```mermaid
flowchart TD
    A[環境変数] --> B{管理方法}
    B --> C[AWS Secrets Manager]
    B --> D[Kubernetes Secrets]
    B --> E[環境変数サービス]
    
    C --> F[自動ローテーション]
    D --> G[暗号化保存]
    E --> H[アクセス制御]
```

## ✅ デプロイチェックリスト

### デプロイ前

- [ ] 本番Stripeアカウントの設定完了
- [ ] 本番商品・価格の作成完了
- [ ] カスタマーポータル設定完了
- [ ] Webhookエンドポイント設定完了
- [ ] 環境変数の設定完了
- [ ] SSL証明書の設定完了
- [ ] データベースバックアップ完了
- [ ] ロールバック計画の準備

### セキュリティ確認

- [ ] `PAYMENT_DEVELOPMENT_MODE=false`
- [ ] `STRIPE_WEBHOOK_SECRET`が設定されている
- [ ] 本番APIキー（`sk_live_`）を使用
- [ ] HTTPS通信のみ許可
- [ ] CORSが適切に設定されている

### インフラ確認

- [ ] 十分なサーバーリソース
- [ ] データベース接続プール設定
- [ ] ロードバランサー設定
- [ ] 監視・アラート設定

## 🔍 移行後の確認

### 1. ヘルスチェック

```bash
# APIの疎通確認
curl https://api.your-domain.com/

# 認証確認
curl -X POST https://api.your-domain.com/auth/signin \
  -H "Content-Type: application/json" \
  -d '{"identifier": "test@example.com", "password": "password"}'
```

### 2. 決済フロー確認

1. **テストユーザーで決済フロー確認**
   - チェックアウトセッション作成
   - 実際のカードで少額決済
   - Webhook受信確認
   - DB更新確認

2. **カスタマーポータル確認**
   - ポータルアクセス
   - サブスクリプション管理

### 3. 監視項目

```mermaid
graph LR
    A[監視対象] --> B[API応答時間]
    A --> C[Stripe API成功率]
    A --> D[Webhook処理時間]
    A --> E[エラー率]
    A --> F[決済成功率]
```

### 4. ログ確認

```bash
# Stripe関連ログ
tail -f /var/log/app.log | grep -i stripe

# エラーログ
tail -f /var/log/app.log | grep -i error

# Webhookログ
tail -f /var/log/app.log | grep -i webhook
```

## 🚨 トラブルシューティング

### よくある問題

1. **Webhook署名エラー**
   - 本番用のWebhook署名シークレットを使用しているか確認
   - エンドポイントURLが正しいか確認

2. **商品・価格が見つからない**
   - 本番モードで商品を作成したか確認
   - 価格IDが`price_live_`で始まっているか確認

3. **決済が拒否される**
   - Stripeアカウントが有効化されているか確認
   - 利用可能な国・通貨か確認

### ロールバック手順

問題が発生した場合：

1. 環境変数を以前の設定に戻す
2. デプロイを以前のバージョンに戻す
3. Stripeダッシュボードで問題を確認
4. 必要に応じてStripeサポートに連絡

## 📞 サポート

- **Stripeサポート**: [https://support.stripe.com/](https://support.stripe.com/)
- **ドキュメント**: [https://stripe.com/docs](https://stripe.com/docs)
- **ステータス**: [https://status.stripe.com/](https://status.stripe.com/)

## 🎉 本番運用開始

全ての確認が完了したら、本番運用を開始できます。定期的に以下を確認してください：

- 決済の成功率
- エラーログ
- ユーザーからのフィードバック
- Stripeダッシュボードのアラート