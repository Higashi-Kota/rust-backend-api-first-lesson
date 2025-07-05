# Stripe決済統合ドキュメント

## 📖 概要

本ドキュメントは、Task Backend APIにおけるStripe決済統合の実装・設定・運用に関する技術文書です。

## 🗂️ ドキュメント構成

| ドキュメント | 内容 | 対象読者 |
|------------|------|---------|
| [01_SETUP.md](./01_SETUP.md) | Stripeアカウント作成、商品登録、カスタマーポータル設定、環境変数設定 | 初回セットアップ担当者 |
| [02_DEVELOPMENT.md](./02_DEVELOPMENT.md) | 開発モード・テストモードでの開発方法、テスト戦略 | 開発者 |
| [03_WEBHOOK.md](./03_WEBHOOK.md) | Webhook設定、ローカルテスト、イベント処理フロー | 開発者・運用担当者 |
| [04_IMPLEMENTATION.md](./04_IMPLEMENTATION.md) | アーキテクチャ詳細、決済フロー、実装シナリオ | アーキテクト・開発者 |
| [05_PRODUCTION.md](./05_PRODUCTION.md) | 本番環境への移行手順、チェックリスト | 運用担当者・DevOps |

## 🚀 クイックスタート

### 最短で動作確認したい場合

1. **開発モード（モック決済）で動作確認** → [02_DEVELOPMENT.md#開発モード](./02_DEVELOPMENT.md#開発モード)
2. **Stripeテストモードで動作確認** → [01_SETUP.md](./01_SETUP.md) → [02_DEVELOPMENT.md#テストモード](./02_DEVELOPMENT.md#テストモード)

### 本番環境への移行

1. **事前準備** → [05_PRODUCTION.md#事前準備](./05_PRODUCTION.md#事前準備)
2. **移行手順** → [05_PRODUCTION.md#移行手順](./05_PRODUCTION.md#移行手順)

## 📊 現在の実装状況

### ✅ 実装済み機能

- **決済機能**
  - チェックアウトセッション作成
  - カスタマーポータルセッション作成
  - Webhook処理（署名検証含む）

- **サブスクリプション管理**
  - 現在のプラン確認
  - 利用可能プラン一覧
  - アップグレードオプション

- **セキュリティ**
  - Webhook署名検証
  - 環境別設定分離
  - エラーハンドリング

### 🔧 使用技術

- **Rustクレート**: async-stripe v0.39
- **APIバージョン**: Stripe API 2024-04-10
- **対応プラン**: Free, Pro (¥5,000/月), Enterprise (¥20,000/月)

## 📝 重要な注意事項

1. **環境変数の扱い**
   - 本番環境では必ず `STRIPE_WEBHOOK_SECRET` を設定すること
   - 価格IDは `price_` で始まるID（商品IDの `prod_` ではない）

2. **テスト環境**
   - CIでは開発モード（モック）を使用
   - ローカルではStripe CLIでWebhookフォワーディング可能

3. **必須の設定**
   - Stripeダッシュボードで商品・価格の作成
   - カスタマーポータルの設定（本番・テスト両方）

## 🔗 関連リンク

- [Stripe Dashboard](https://dashboard.stripe.com/)
- [Stripe API Docs](https://stripe.com/docs/api)
- [async-stripe Docs](https://docs.rs/async-stripe/)