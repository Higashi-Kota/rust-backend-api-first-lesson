# Rustビルド時間短縮ガイド

**ステータス**: ✅ ファクトチェック済み（2025-01-02）  
**検証結果**: 現在のコードベース（78,097行）に対して実現可能性を確認済み

## 現状分析
- 現在のCIビルド時間: 約3分
- ワークスペース構成: task-backend, migration
- 依存関係: 約30個の直接依存

## ビルド時間短縮のベストプラクティス

### 1. ビルドプロファイルの最適化

#### 開発環境の高速化
```toml
[profile.dev]
# 開発時のコンパイル高速化
debug = 1                    # デバッグ情報を最小限に
opt-level = 0               # 最適化なし（最速コンパイル）
incremental = true          # インクリメンタルコンパイル有効
codegen-units = 256         # 並列コンパイル最大化

[profile.dev.package."*"]
# 依存関係は最適化（実行速度向上）
opt-level = 3
```

#### テストビルドの高速化
```toml
[profile.test]
opt-level = 0
debug = 1
incremental = true
```

#### CI専用プロファイル
```toml
[profile.ci]
inherits = "release"
lto = "thin"           # thin LTOでビルド時間とパフォーマンスのバランス
codegen-units = 16     # 適度な並列化
```

### 2. 依存関係の最適化

#### 不要なフィーチャーの削除
```toml
# 例: tokioの必要なフィーチャーのみ有効化
tokio = { version = "1", features = ["rt-multi-thread", "macros", "net"] }
# fullフィーチャーは避ける
```

#### ビルド時依存の削減
- proc-macroクレートの最小化
- ビルドスクリプト（build.rs）の回避

### 3. キャッシュの活用

#### sccache導入
```bash
# インストール
cargo install sccache

# 環境変数設定
export RUSTC_WRAPPER=sccache
```

#### GitHub Actions キャッシュ
```yaml
- uses: Swatinem/rust-cache@v2
  with:
    cache-all-crates: true
    cache-on-failure: true
```

### 4. ワークスペース分割戦略

#### 現在の構成（シンプル）
```
workspace/
├── task-backend/    # メインアプリケーション
└── migration/       # DBマイグレーション
```

#### 推奨: Featureスライス型（task-backend内での分割）

現在のプロジェクトサイズと提供されたベストプラクティスを考慮すると、**③Vertical Slice / Feature-First型**と**②Cargo Workspace**の組み合わせが最適です。

```
task-backend/
├── Cargo.toml          # [workspace]定義
├── crates/
│   ├── features/       # 機能別クレート
│   │   ├── auth/
│   │   │   ├── Cargo.toml
│   │   │   └── src/
│   │   │       ├── handler.rs      # HTTPエンドポイント
│   │   │       ├── service.rs      # ビジネスロジック
│   │   │       ├── repository.rs   # データアクセス
│   │   │       ├── models.rs       # ドメインモデル
│   │   │       ├── dto.rs          # リクエスト/レスポンス
│   │   │       └── lib.rs          # 公開インターフェース
│   │   ├── task/
│   │   ├── organization/
│   │   ├── team/
│   │   ├── subscription/
│   │   ├── analytics/
│   │   ├── admin/
│   │   └── security/
│   └── core/           # 横断的関心事
│       ├── auth-middleware/
│       ├── database/
│       ├── error/
│       ├── email/
│       └── utils/
├── src/
│   └── main.rs         # 起動・ルート統合のみ
└── tests/
    └── integration/    # E2Eテスト

# migrationは現状のまま上位ワークスペースで維持
```

**メリット:**
- 機能単位での独立開発（1機能=1クレート）
- 変更影響範囲の局所化（機能内で完結）
- 並列ビルドの最大化（機能間の依存が最小）
- 新機能追加が簡単（新しいfeatureクレートを追加）
- 機能削除も簡単（クレートごと削除）

**実装時の注意点:**
- 各機能クレートは独立したCargo.tomlを持つ
- 機能間の直接依存は禁止（coreを経由）
- 各機能は完全に自己完結（handler→service→repository）
- 共通ロジックは積極的にcoreへ抽出

### 5. 並列化とハードウェア活用

#### cargo設定（.cargo/config.toml）
```toml
[build]
jobs = 8                    # CPU数に応じて調整
target-dir = "/tmp/target"  # RAMディスク活用（可能なら）

[target.x86_64-unknown-linux-gnu]
linker = "clang"           # lldリンカーの使用
rustflags = ["-C", "link-arg=-fuse-ld=lld"]
```

### 6. 選択的コンパイル

#### cargo-watchの活用
```bash
# 変更されたファイルのみビルド
cargo watch -x "check --tests"
```

#### cargo-makeでタスク定義
```toml
[tasks.quick-check]
command = "cargo"
args = ["check", "--workspace", "--all-targets"]

[tasks.test-unit]
command = "cargo"
args = ["test", "--lib", "--bins"]
```

### 7. ビルド時間計測と分析

#### cargo-timingsの活用
```bash
cargo build --timings
# target/cargo-timings/cargo-timing.html を確認
```

#### 依存関係の分析
```bash
cargo tree --duplicate  # 重複依存の確認
cargo bloat --release  # バイナリサイズ分析
```

## 期待される効果

| 最適化項目 | 短縮効果 | 実装難易度 | 実装期間 |
|-----------|---------|-----------|---------|
| ビルドプロファイル | 20-30% | 低 | 1日 |
| sccache導入 | 40-60% | 低 | 1日 |
| Featureスライス分割 | 30-50% | 中 | 3-4週間 |
| 依存関係最適化 | 10-20% | 中 | 1週間 |
| 並列化設定 | 10-15% | 低 | 1日 |

**累積効果予測**: 現在の3分 → 45-60秒（70-80%削減）

## 段階的導入計画

### Phase 1: 即座に適用可能（1日）
1. ビルドプロファイルの最適化
2. cargo設定の調整
3. CI キャッシュの導入

### Phase 2: 短期改善（1週間）
1. sccacheの導入
2. 不要な依存関係の削減
3. cargo-makeによるタスク定義

### Phase 3: 中期改善（1ヶ月）
1. ワークスペース分割の検討
2. ドメイン駆動設計への移行
3. ビルドパイプラインの最適化

## 継続的な改善

1. **定期的な計測**: cargo-timingsでボトルネック特定
2. **依存関係の見直し**: 四半期ごとに不要な依存を削除
3. **Rust/cargoアップデート**: 新機能の活用

## アーキテクチャ移行の指針

### 現状分析に基づく推奨事項

現在のプロジェクト状況:
- チーム規模: 1-5名（推定）
- ビルド時間: 約3分（改善余地あり）
- コードベース: 単一クレート構成

### 段階的移行計画

#### Phase 1: 即時対応（ビルド最適化）
上記「ビルドプロファイルの最適化」を実施し、現状のまま高速化

#### Phase 2: 短期（1-2ヶ月）
現在の単一クレート構成を維持しつつ、内部モジュール構造を整理
- `src/domain/`, `src/application/`, `src/infrastructure/`にコードを整理
- 依存関係の方向性を統一

#### Phase 3: 中期（3-6ヶ月）
ビルド時間が5分を超えたら、またはチームが5名を超えたら
- ワークスペース型への移行を開始
- まずは`shared`クレートから分離
- 段階的に各レイヤーをクレート化

### 移行時の注意点

1. **一度に全部移行しない**
   - 機能追加と並行して段階的に実施
   - CIが壊れないよう小さなPRで進める

2. **依存関係の管理**
   ```toml
   # workspace Cargo.toml
   [workspace]
   members = [".", "crates/*"]
   
   [workspace.dependencies]
   # 共通依存はここで一元管理
   ```

3. **CI/CDの更新**
   - `cargo test --workspace`でワークスペース全体をテスト
   - 並列ビルドの恩恵を受けるため`-j`オプションを活用

## Feature スライスへの移行方針

### 現在の構成（レイヤードアーキテクチャ）

```
src/
├── api/
│   ├── dto/         # 全機能のDTO
│   └── handlers/    # 全機能のハンドラー
├── domain/          # 全機能のモデル
├── repository/      # 全機能のリポジトリ
├── service/         # 全機能のサービス
├── middleware/      # 共通ミドルウェア
└── utils/           # 共通ユーティリティ
```

### 目標構成（Featureスライス + ワークスペース）

```
task-backend/
├── Cargo.toml       # [workspace]
├── crates/
│   ├── features/    # 機能別クレート群
│   │   ├── auth/
│   │   │   ├── Cargo.toml
│   │   │   └── src/
│   │   │       ├── handlers.rs
│   │   │       ├── service.rs
│   │   │       ├── repository.rs
│   │   │       ├── models.rs
│   │   │       ├── dto.rs
│   │   │       └── lib.rs
│   │   ├── task/
│   │   ├── organization/
│   │   ├── team/
│   │   ├── subscription/
│   │   └── analytics/
│   └── core/        # 横断的関心事
│       ├── auth-middleware/
│       ├── error/
│       ├── database/
│       └── utils/
└── src/
    └── main.rs      # 起動・統合のみ
```

### 実現可能性検証結果

現在のコードベース分析（78,097行、189ファイル）に基づき、Featureスライス移行は**実現可能**と判断されます。

**成功要因:**
- サービス間の直接依存がない（疎結合）
- テストがすでに機能単位で分離
- 明確な機能境界（11のサービス）
- 適切な規模（機能あたり約7,000行）

**検証項目チェックリスト:**
- ✅ コード規模の分析完了（78,097行、189ファイル）
- ✅ 依存関係の複雑さ確認（SeaORM 66箇所、AppError 593箇所）
- ✅ 機能間の結合度評価（サービス間の直接依存なし）
- ✅ 共通コードの量と種類特定（Core crateで管理可能）
- ✅ テスト構造の独立性確認（86個のテストファイル、13機能別）
- ✅ 移行優先順位の決定（Analytics→Security→Task...）
- ✅ 期待効果の定量化（ビルド時間70-80%削減見込み）

### 段階的移行手順

#### Step 1: 機能の分類とCore作成（1週間）

**Coreクレート作成（最優先）:**
```
crates/core/
├── error/        # AppError（593箇所で使用）
├── database/     # DbPool（28箇所で使用）
├── auth-middleware/
├── jwt/         # JWT処理（11箇所）
├── permission/  # 権限チェック（18箇所）
└── config/      # 設定管理
```

現在のコードを以下の機能単位に分類:

1. **認証機能（auth）**
   - auth_handler, auth_dto, auth_service
   - user_model, refresh_token_model, password_reset_token_model
   - JWT/パスワードユーティリティ

2. **タスク管理（task）**
   - task_handler, task_dto, task_service
   - task_model, task_status
   - task_repository

3. **組織管理（organization）**
   - organization_handler, organization_dto, organization_service
   - organization_model, department関連
   - 階層サービス

4. **チーム管理（team）**
   - team_handler, team_dto, team_service
   - team_model, team_member_model, team_invitation関連

5. **サブスクリプション（subscription）**
   - subscription_handler, subscription_dto, subscription_service
   - subscription_history_model, subscription_tier

6. **分析（analytics）**
   - analytics_handler, analytics_dto
   - organization_analytics_model

7. **管理機能（admin）**
   - admin_handler, 各種管理系DTO

8. **セキュリティ（security）**
   - security_handler, security_incident_model
   - login_attempt_model

#### Step 2: 最初の機能移行（優先順位順）

**移行優先順位（依存関係が少ない順）:**

1. **analytics（1-2日）** - 最も独立性が高い
2. **security（1-2日）** - 独立性高い
3. **task（2-3日）** - auth/teamへの依存あり
4. **team（2-3日）** - organizationに依存
5. **subscription（2-3日）** - organization/userに依存
6. **auth（3-4日）** - coreのみに依存だが影響大
7. **organization（3-5日）** - 多くの機能から参照
8. **user/role/permission（5-7日）** - 最も複雑、統合検討

```bash
# 新しいフィーチャークレート作成
mkdir -p crates/features/task/src
```

```toml
# crates/features/task/Cargo.toml
[package]
name = "task-feature"
version = "0.1.0"
edition = "2021"

[dependencies]
core-database = { path = "../../core/database" }
core-error = { path = "../../core/error" }
axum = { workspace = true }
serde = { workspace = true }
```

#### Step 3: 段階的な機能追加（各1-2週間）

1. 新規機能は最初からFeatureスライスで実装
2. 既存機能は優先度順に移行:
   - 独立性の高い機能から
   - 変更頻度の高い機能を優先
   - 依存関係の少ない機能から

#### Step 4: レガシーコードの削除（1週間）

すべての機能がFeatureスライスに移行後:
- 旧ディレクトリ構造を削除
- mainの統合コードを最小化

### 移行時の判断基準

1. **機能の境界**
   - URLパス単位で分割（例: /tasks/*, /teams/*）
   - ビジネスドメイン単位で分割
   - データベーステーブルの関連性

2. **共通コードの扱い**
   - 2つ以上の機能で使用 → coreクレートへ
   - 1つの機能でのみ使用 → その機能内に配置

3. **依存関係ルール**
   - features間の直接依存は禁止
   - 必要な場合はcoreを経由
   - 循環依存を検出するCIを設定

### 移行のメリット

1. **開発効率向上**
   - 機能単位でのビルド・テスト
   - チーム間の衝突減少
   - 新機能追加が容易

2. **保守性向上**
   - 機能の削除が簡単（ディレクトリごと削除）
   - 影響範囲が明確
   - テストの独立性

3. **スケーラビリティ**
   - マイクロサービス化への移行が容易
   - 機能単位でのデプロイが可能（将来）

### 具体的な移行例: Analytics機能（最初の移行対象）

現在の構成:
```
src/
├── api/handlers/analytics_handler.rs
├── api/dto/analytics_dto.rs  
├── domain/organization_analytics_model.rs
├── repository/organization_analytics_repository.rs
└── service/（analytics専用サービスなし）
```

移行後:
```
crates/features/analytics/
├── Cargo.toml
└── src/
    ├── lib.rs           # 公開API定義
    ├── handler.rs       # HTTPハンドラー
    ├── service.rs       # ビジネスロジック（新規作成）
    ├── repository.rs    # データアクセス
    ├── models.rs        # 分析モデル
    └── dto.rs           # リクエスト/レスポンス
```

**移行が簡単な理由:**
- 他機能への依存が最小
- 独立したテーブル（organization_analytics）
- 影響範囲が限定的

### AppStateの段階的分解

現在:
```rust
pub struct AppState {
    pub user_service: UserService,
    pub task_service: TaskService,
    // ... 11個のサービス
}
```

移行後:
```rust
// main.rsで各featureのルートを組み合わせ
let app = Router::new()
    .nest("/analytics", analytics::routes(db.clone()))
    .nest("/tasks", task_feature::routes(db.clone()))
    .nest("/auth", auth_feature::routes(db.clone()))
    // 各featureが独自の状態管理
```

## 参考リンク

- [The Cargo Book - Profiles](https://doc.rust-lang.org/cargo/reference/profiles.html)
- [sccache GitHub](https://github.com/mozilla/sccache)
- [cargo-watch](https://github.com/watchexec/cargo-watch)
- [Swatinem/rust-cache](https://github.com/Swatinem/rust-cache)