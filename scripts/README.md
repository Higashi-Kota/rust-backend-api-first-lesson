# Stripe Subscription Test Scripts

このディレクトリには、Stripeサブスクリプション機能をテストするためのシナリオ別スクリプトが含まれています。

## スクリプト一覧

### 1. `00_basic_stripe_flow.sh`
- 新規ユーザー登録からStripeチェックアウトまでの基本フロー
- Proプランへのアップグレードテスト

### 2. `00_existing_user_stripe_test.sh`
- 既存ユーザーでのStripeテスト
- ログインエラーハンドリング付き

### 3. `01_free_plan_limits.sh`
- 無料プランの機能制限確認
- チーム作成制限（1チームまで）
- メンバー制限（3人まで）
- タスク制限（100個まで）
- API呼び出し制限（1日1,000回まで）

### 4. `02_pro_plan_upgrade.sh`
- Proプランへのアップグレードプロセス
- アップグレード後の機能拡張確認
- チーム作成（5チームまで）
- メンバー（10人まで）
- タスク（1,000個まで）
- API呼び出し（1日10,000回まで）

### 5. `03_enterprise_plan_upgrade.sh`
- Enterpriseプランへのアップグレード
- 無制限機能のテスト
- すべての制限が解除されることを確認

### 6. `04_subscription_cancellation.sh`
- サブスクリプションのキャンセルプロセス
- キャンセル後の機能制限（Freeプランに戻る）
- 既存データの保持確認

### 7. `05_plan_comparison.sh`
- 3つのプラン（Free, Pro, Enterprise）の機能比較
- 複数ユーザーでの同時テスト
- アップグレード推奨理由の表示

## 使用方法

### 前提条件

1. **環境変数の設定**
   ```bash
   # .envファイルにStripe関連の環境変数を設定
   STRIPE_SECRET_KEY=sk_test_xxxxx
   STRIPE_WEBHOOK_SECRET=whsec_xxxxx
   STRIPE_PRO_PRICE_ID=price_xxxxx
   STRIPE_ENTERPRISE_PRICE_ID=price_xxxxx
   ```

2. **Stripe CLIでWebhookリスナーを起動**
   ```bash
   make stripe-listen
   # または
   stripe listen --forward-to localhost:5000/webhooks/stripe
   ```

3. **バックエンドサーバーを起動**
   ```bash
   make dev
   # サーバーが http://localhost:5000 で起動することを確認
   ```

4. **必要なツール**
   - `jq`コマンドがインストールされていること
   - Stripe CLIがインストールされていること

### 基本的な実行順序

1. **無料プランの制限確認**
   ```bash
   ./01_free_plan_limits.sh
   ```

2. **Proプランへのアップグレード**
   ```bash
   ./02_pro_plan_upgrade.sh
   # チェックアウトURLが表示されるので、ブラウザで開いて決済を完了
   # テストカード: 4242 4242 4242 4242
   ```

3. **Enterpriseプランへのアップグレード**
   ```bash
   ./03_enterprise_plan_upgrade.sh
   # 同様にチェックアウトを完了
   ```

4. **サブスクリプションのキャンセル**
   ```bash
   ./04_subscription_cancellation.sh
   # カスタマーポータルURLからキャンセル
   ```

5. **プラン比較**
   ```bash
   ./05_plan_comparison.sh
   ```

## Stripeテストカード

- **成功する支払い**: `4242 4242 4242 4242`
- **認証が必要**: `4000 0025 0000 3155`
- **拒否される**: `4000 0000 0000 9995`

その他のフィールド：
- **有効期限**: 任意の将来の日付（例: 12/34）
- **CVC**: 任意の3桁（例: 123）
- **郵便番号**: 任意の5桁（例: 12345）

## トラブルシューティング

### ログイン失敗
- ユーザーが存在しない場合は自動的に作成されます
- パスワードは全スクリプトで統一: `SecureP@ssw0rd!`

### Webhook処理
- Webhookの処理には数秒かかることがあります
- `tail -f server.log | grep -i stripe`でログを確認

### 機能制限エラー
- プランの制限に達した場合、適切なエラーメッセージが表示されます
- アップグレードが必要な場合はチェックアウトURLが提供されます

## 注意事項

- これらのスクリプトはテスト環境用です
- 本番環境では実際の決済が発生するため注意してください
- Stripeのテストモードを使用していることを確認してください