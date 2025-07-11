# Feature-based Architecture - Phase実装詳細

## 📌 概要

Phase 19までの実装により、基本的なfeature分割が完了し、全テストがパスする状態を達成しました。
Phase 20以降は、レガシーコードの完全削除と残存機能のfeature化を通じて、真のマルチクレート対応アーキテクチャを実現します。

## 🏁 完了済みPhase (14-19)

| Phase | 機能 | 状態 | 完了日 |
|-------|------|------|--------|
| 14 | Team機能 | ✅ 完了 | - |
| 15 | Organization機能 | ✅ 完了 | - |
| 16 | Security機能 | ✅ 完了 | - |
| 17 | Admin機能 | ✅ 完了 | - |
| 18 | Subscription機能 | ✅ 完了 | - |
| 19 | エラー修正・テスト全パス | ✅ 完了 | 2025-07-11 |

## 📋 Phase 20: レガシーコードの完全削除

### 目的
再エクスポートによる暫定対応を解消し、全コードを適切なfeatureモジュールに配置する

### 作業内容

#### 1. 空の再エクスポートファイルの削除
```bash
# 削除対象
rm src/api/handlers/analytics_handler.rs
rm src/api/handlers/organization_handler.rs
rm src/api/handlers/security_handler.rs
rm src/api/handlers/subscription_handler.rs
```

#### 2. DTOの移行
| 旧ファイル | 移行先 |
|-----------|--------|
| api/dto/admin_organization_dto.rs | features/admin/dto/responses/organization.rs |
| api/dto/admin_role_dto.rs | features/admin/dto/responses/role.rs |
| api/dto/analytics_dto.rs | features/analytics/dto/responses/analytics.rs |
| api/dto/organization_hierarchy_dto.rs | features/organization/dto/hierarchy.rs |
| api/dto/subscription_history_dto.rs | features/subscription/dto/responses/history.rs |
| api/dto/team_dto.rs | features/team/dto/responses/team.rs |

#### 3. モデルの移行
```
domain/stripe_subscription_model.rs → features/subscription/models/stripe.rs
domain/subscription_history_model.rs → features/subscription/models/history.rs
```

#### 4. リポジトリ・サービスの移行
```
repository/stripe_subscription_repository.rs → features/subscription/repositories/stripe.rs
repository/subscription_history_repository.rs → features/subscription/repositories/history.rs
service/subscription_service.rs → features/subscription/services/subscription.rs
```

### 実装手順

1. **依存関係の調査**
   ```bash
   # 各ファイルを参照している箇所を特定
   rg "admin_organization_dto" --type rust
   rg "stripe_subscription_model" --type rust
   # ... 各ファイルについて実施
   ```

2. **移行とインポート更新**
   ```rust
   // 例: 旧インポート
   use crate::api::dto::admin_organization_dto::*;
   
   // 新インポート
   use crate::features::admin::dto::responses::organization::*;
   ```

3. **テストの実行**
   ```bash
   # 各移行後にテストを実行
   cargo test --package task-backend
   cargo clippy --all-targets --all-features -- -D warnings
   ```

### 完了条件
- [ ] `api/handlers/`, `api/dto/` ディレクトリが空
- [ ] `domain/`, `repository/`, `service/` から移行対象ファイルが削除
- [ ] 全テストがパス
- [ ] Clippy警告ゼロ

### リスクと対策
- **リスク**: インポートパスの見落とし
- **対策**: `rg`コマンドで網羅的に検索、段階的な移行

---

## 📋 Phase 21: 未移行機能のFeature化

### 目的
Payment、User、System機能を独立したfeatureモジュールとして実装

### 作業内容

#### 1. Payment Feature
```
features/payment/
├── mod.rs
├── handlers/
│   ├── mod.rs
│   ├── payment.rs         # 決済処理
│   ├── webhook.rs         # Stripe webhook
│   └── subscription.rs    # サブスク決済
├── services/
│   ├── mod.rs
│   ├── payment.rs
│   └── stripe.rs
├── repositories/
│   ├── mod.rs
│   └── payment_history.rs
├── models/
│   ├── mod.rs
│   ├── payment.rs
│   └── stripe_event.rs
└── dto/
    ├── mod.rs
    ├── requests/
    │   ├── mod.rs
    │   └── payment.rs
    └── responses/
        ├── mod.rs
        └── payment.rs
```

#### 2. User Feature
```
features/user/
├── mod.rs
├── handlers/
│   ├── mod.rs
│   ├── profile.rs         # プロフィール管理
│   ├── settings.rs        # ユーザー設定
│   └── admin.rs          # 管理者用ユーザー管理
├── services/
│   ├── mod.rs
│   ├── user.rs
│   └── settings.rs
├── repositories/
│   ├── mod.rs
│   └── user_settings.rs
├── models/
│   ├── mod.rs
│   └── user_settings.rs
└── dto/
    ├── mod.rs
    ├── requests/
    └── responses/
```

#### 3. System Feature
```
features/system/
├── mod.rs
├── handlers/
│   ├── mod.rs
│   ├── health.rs          # ヘルスチェック
│   ├── metrics.rs         # メトリクス
│   └── info.rs           # システム情報
├── services/
│   ├── mod.rs
│   └── monitoring.rs
└── dto/
    ├── mod.rs
    └── responses/
        ├── mod.rs
        └── health.rs
```

### 実装手順

1. **ディレクトリ構造の作成**
2. **既存コードの分析と分割**
3. **インターフェースの定義**
4. **段階的な移行**
5. **テストの作成・更新**

### 完了条件
- [ ] 3つの新featureモジュールが作成
- [ ] `api/handlers/` から該当ハンドラーが削除
- [ ] 全テストがパス
- [ ] 各featureが独立して機能

---

## 📋 Phase 22: 品質改善

### 目的
コード品質の向上とテストカバレッジの改善

### 作業内容

#### 1. Ignoredテストの修正 (6件)
```rust
// 修正対象
- test_delete_user_settings
- test_get_feature_usage_requires_auth
- test_get_feature_usage_stats_as_admin
- test_get_user_feature_usage_as_admin
- test_non_admin_cannot_access_analytics
- test_track_feature_usage
```

#### 2. Dead Code削除
```bash
# dead_codeアノテーションの検索
rg "#\[allow\(dead_code\)\]" --type rust

# 各箇所について：
# 1. 本当に不要か確認
# 2. 不要なら削除
# 3. 必要なら適切に使用または公開
```

#### 3. DTOの一貫性確保
```rust
// 悪い例（グロブインポート）
pub use requests::*;
pub use responses::*;

// 良い例（明示的インポート）
pub use requests::{
    CreateUserRequest,
    UpdateUserRequest,
};
pub use responses::{
    UserResponse,
    UserListResponse,
};
```

### 実装手順

1. **Ignoredテストの調査**
   - なぜignoredか確認
   - 必要な機能の実装
   - テストの有効化

2. **Dead Codeの精査**
   - 使用箇所の確認
   - 削除または活用の判断
   - リファクタリング

3. **DTOの整理**
   - グロブインポートの特定
   - 明示的インポートへの変更
   - 名前衝突の解消

### 完了条件
- [ ] Ignoredテスト: 0件
- [ ] `#[allow(dead_code)]`: 0件
- [ ] グロブインポート: 最小限
- [ ] テストカバレッジ: 80%以上

---

## 📋 Phase 23: ワークスペース準備

### 目的
マルチクレート構成への移行準備と依存関係の整理

### 作業内容

#### 1. 依存関係グラフの作成
```bash
# cargo-dephを使用
cargo install cargo-depgraph
cargo depgraph --all-deps | dot -Tpng > deps.png
```

#### 2. Feature間の依存関係整理
```toml
# 各feature/Cargo.toml（将来）
[dependencies]
shared = { path = "../shared" }
infrastructure = { path = "../infrastructure" }
# feature間の依存は最小限に
```

#### 3. 共通コードの抽出
```
shared/
├── types/          # 共通型定義
├── errors/         # エラー型
├── utils/          # ユーティリティ
└── traits/         # 共通トレイト
```

### 実装手順

1. **依存関係の可視化と分析**
2. **循環依存の特定と解消**
3. **共通コードの識別**
4. **インターフェース境界の明確化**
5. **ビルド時間の測定**

### 完了条件
- [ ] 依存関係グラフが生成
- [ ] 循環依存: 0件
- [ ] Feature間の直接依存: 最小限
- [ ] ビルド時間のベンチマーク完了

### 次のステップ（Phase 24以降）
- ワークスペース化の実施
- 個別クレートの作成
- CI/CDパイプラインの更新
- マイクロサービス化の検討

---

## 📊 進捗管理

### タイムライン（推定）
| Phase | 期間 | 開始予定 | 完了予定 |
|-------|------|----------|----------|
| 20 | 2-3日 | - | - |
| 21 | 3-4日 | - | - |
| 22 | 2-3日 | - | - |
| 23 | 1-2日 | - | - |

### リスク管理
1. **技術的リスク**
   - 予期しない依存関係
   - テストの破損
   - パフォーマンス劣化

2. **対策**
   - 段階的な移行
   - 継続的なテスト実行
   - パフォーマンス測定

### 成功の測定基準
- ビルド時間: 50%以上短縮
- テストカバレッジ: 80%以上
- 開発者体験: 機能単位での独立開発が可能
- 保守性: 新機能追加が既存コードに影響しない

---
最終更新: 2025-07-11
Phase 20-23の実装計画策定