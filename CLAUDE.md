# Rust Backend API - Feature-based Architecture

## 📌 現在の状態 (2025-07-11)

**Phase 19 完了** - 全テストパス達成 ✅
- ビルド成功: エラー・警告なし
- Clippy: 全警告解消
- テスト: 523/523 パス (6 ignored)
- CI: `make ci-check-fast` 成功

## 🏗️ アーキテクチャ概要

### 現在の構造
```
src/
├── features/          # 機能別モジュール (Phase 14-18で実装)
│   ├── auth/         # 認証・認可
│   ├── task/         # タスク管理
│   ├── team/         # チーム管理
│   ├── organization/ # 組織管理
│   ├── storage/      # ストレージ
│   ├── gdpr/         # GDPR対応
│   ├── security/     # セキュリティ
│   ├── admin/        # 管理者機能
│   ├── analytics/    # 分析機能
│   └── subscription/ # サブスクリプション
├── api/              # レガシーAPI層（段階的に削除予定）
├── domain/           # レガシードメイン層（段階的に削除予定）
├── repository/       # レガシーリポジトリ層（段階的に削除予定）
├── service/          # レガシーサービス層（段階的に削除予定）
├── shared/           # 共通型・ユーティリティ
├── infrastructure/   # 技術基盤
└── main.rs
```

### Feature モジュール標準構造
```
features/{feature_name}/
├── mod.rs           # 公開API定義
├── handlers/        # HTTPハンドラー
├── services/        # ビジネスロジック
├── repositories/    # データアクセス
├── models/          # ドメインモデル
├── dto/             # データ転送オブジェクト
│   ├── mod.rs
│   ├── requests/    # リクエストDTO
│   └── responses/   # レスポンスDTO
└── usecases/        # 複雑なビジネスロジック（オプション）
```

## 📋 残タスク一覧

### Phase 20: レガシーコードの完全削除
**目的**: 再エクスポートによる暫定対応を解消し、全コードを適切な場所に配置

#### 削除対象ファイル
```
# 空の再エクスポートファイル
api/handlers/analytics_handler.rs
api/handlers/organization_handler.rs
api/handlers/security_handler.rs
api/handlers/subscription_handler.rs

# 移行が必要なDTO
api/dto/admin_organization_dto.rs
api/dto/admin_role_dto.rs
api/dto/analytics_dto.rs
api/dto/organization_hierarchy_dto.rs
api/dto/subscription_history_dto.rs
api/dto/team_dto.rs

# 移行が必要なモデル
domain/stripe_subscription_model.rs
domain/subscription_history_model.rs

# 移行が必要なリポジトリ
repository/stripe_subscription_repository.rs
repository/subscription_history_repository.rs

# 移行が必要なサービス
service/subscription_service.rs
```

### Phase 21: 未移行機能のFeature化
**目的**: 残りの機能を独立したfeatureモジュールとして実装

#### 新規Feature作成
1. **features/payment/**
   - `api/handlers/payment_handler.rs` → `features/payment/handlers/`
   - `service/payment_service.rs` → `features/payment/services/`
   - Stripe統合ロジックの整理

2. **features/user/**
   - `api/handlers/user_handler.rs` → `features/user/handlers/`
   - `service/user_service.rs` → `features/user/services/`
   - authから独立したユーザー管理機能

3. **features/system/**
   - `api/handlers/system_handler.rs` → `features/system/handlers/`
   - ヘルスチェック、メトリクス等のシステム機能

### Phase 22: 品質改善
**目的**: コード品質とテストカバレッジの向上

#### タスク
1. **Ignoredテストの修正** (6件)
   - Admin settings削除エンドポイントの実装
   - Feature usage tracking機能の完全実装

2. **Dead Code削除** (約70箇所)
   - `#[allow(dead_code)]` アノテーションの除去
   - 未使用関数・構造体の削除

3. **DTOの一貫性確保**
   - 全featureでrequests/responses構造を統一
   - グロブインポートを明示的インポートに変更

### Phase 23: ワークスペース準備
**目的**: マルチクレート構成への移行準備

#### 最終構造案
```
rust-backend-api/
├── Cargo.toml          # ワークスペース定義
├── crates/
│   ├── shared/         # 共通型・ユーティリティ
│   ├── core/           # コアドメイン
│   ├── infrastructure/ # 技術基盤
│   ├── feature-auth/   # 認証機能
│   ├── feature-task/   # タスク管理
│   ├── feature-team/   # チーム管理
│   ├── feature-org/    # 組織管理
│   └── ...            # 他のfeatureクレート
└── apps/
    ├── api-server/     # メインAPIサーバー
    └── worker/         # バックグラウンドワーカー
```

## 📚 詳細ドキュメント

設計原則、実装ガイドライン、各Phaseの詳細な実装手順については以下のドキュメントを参照してください：

- **[設計原則とガイドライン](./CLAUDE-GUIDELINES.md)**
  - 命名規則の統一
  - Services vs UseCases: ビジネスロジックの配置指針
  - 循環依存を防ぐための設計原則
  - リファクタリング時のリスク軽減方針
  - 警告抑制の運用ルール

- **[Phase実装詳細](./CLAUDE-PHASES.md)**
  - Phase 20-23の詳細な実装手順
  - 各Phaseの完了条件と成功指標
  - タイムラインとリスク管理
  - ワークスペース構成への移行準備

## 🎯 設計原則サマリー

### 依存関係の原則
```
handler → service → repository → domain
   ↓         ↓          ↓          ↓
  dto    usecase      dto       (core)
```
- 上位層は下位層のみに依存
- 循環依存は絶対に避ける
- 横断的関心事はsharedモジュールに配置

詳細は[CLAUDE-GUIDELINES.md](./CLAUDE-GUIDELINES.md)を参照してください。

## 🚀 次のステップ

### 即座に実行可能なタスク
1. レガシーDTOファイルの移行（機械的作業）
2. 空の再エクスポートファイルの削除
3. インポートパスの一括更新

### 検討が必要なタスク
1. Payment機能の設計とFeature化
2. User機能のauth機能からの分離方法
3. ワークスペース構成への移行戦略

### 成功指標
- [ ] `api/`, `domain/`, `repository/`, `service/` ディレクトリが空
- [ ] 全機能がfeaturesモジュール配下に存在
- [ ] `#[allow(dead_code)]` が0件
- [ ] ignoredテストが0件
- [ ] 各featureが独立してビルド可能（将来）

## 📝 備考

このドキュメントは生きたドキュメントとして、プロジェクトの進行に合わせて更新してください。
各Phaseの完了時には、完了日と主な成果を記録することを推奨します。

---
最終更新: 2025-07-11
Phase 19完了、全テストパス達成