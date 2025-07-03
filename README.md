# rust-backend-api-first-lesson

## 開発コマンド

### クイックスタート

```bash
# 開発環境の起動
make dev

# ステップごとの手順
docker-compose up postgres -d
make migrate
make run
```

### よく使うコマンド

```bash
# ビルドとテスト
make build                    # ワークスペース全体をビルド
make test                     # すべてのテストを実行
make fmt && make clippy       # フォーマット＆リント

# データベース操作
make migrate                  # マイグレーション実行
make migrate-status           # マイグレーションの状態確認
make migrate-down             # 最後のマイグレーションをロールバック

# 開発ワークフロー
make ci-check                 # CIチェックをローカルで実行（fmt + clippy + test）
make ci-check-fast            # 高速CIチェック（CIプロファイル使用）
cargo watch -x "run --package task-backend"  # 変更時に自動再起動
cargo test --package task-backend --lib      # 単体テストのみ（高速）
cargo test integration::tasks::crud_tests    # 特定の統合テストを実行
make test-integration GROUP=integration::auth # 特定グループの統合テスト実行

# ビルド最適化
make build-ci                 # CI用最適化ビルド（thin LTO）
make build-dev                # 開発用高速ビルド
export RUSTC_WRAPPER=sccache  # sccacheを有効化（要: make install-sccache）
```

### Docker 操作

```bash
make docker-build            # Dockerイメージをビルド
make docker-run              # Docker Composeで実行
docker-compose logs -f app   # アプリのログを表示
```

---

## アーキテクチャ概要

このプロジェクトは **Rust 製タスク管理 API** で、**Axum** と **PostgreSQL** を用いて構築され、**ユーザーの役割とサブスクリプション階層に基づく動的パーミッションシステム** を特徴としています。

### コアアーキテクチャパターン

**レイヤードアーキテクチャ**:

- **API レイヤー**: Axum ハンドラ（`task-backend/src/api/handlers/`）
- **サービスレイヤー**: ビジネスロジック（`task-backend/src/service/`）
- **リポジトリレイヤー**: データアクセス（`task-backend/src/repository/`）
- **ドメインレイヤー**: コアモデル（`task-backend/src/domain/`）

**主要な設計コンセプト**:

1. **動的パーミッションシステム**: 同一エンドポイントが、ユーザーの役割とサブスクリプション階層によって異なる応答を返す
2. **ワークスペース構成**: `task-backend`（本体アプリ）と `migration`（DB マイグレーション）の Rust ワークスペース
3. **JWT 認証**: 役割ベースの認可付き多層ミドルウェア
4. **サブスクリプション機能**: Free / Pro / Enterprise 各階層で異なる機能提供

---

## 重要コンポーネント

### 動的パーミッションシステム（コアの革新）

ユーザーの状態によって **同一エンドポイントが異なる動作をする** パターンを採用：

```rust
// 同じエンドポイントが、ユーザーにより異なる動作
GET /tasks/dynamic
// Freeユーザー: 最大100件、基本機能
// Proユーザー: 最大1万件、高度なフィルタ・エクスポート
// Enterprise: 無制限、すべての機能利用可
```

**パーミッション階層**:

- `PermissionScope`: 自分 → チーム → 組織 → グローバル
- `SubscriptionTier`: Free → Pro → Enterprise
- `Privilege`: 階層ごとのクォータ・機能を定義

### 認証フロー

**複数ミドルウェア**:

- `jwt_auth_middleware`: JWT の基本検証
- `role_aware_auth_middleware`: DB から詳細な役割情報を読み込む
- `admin_only_middleware`: 管理者専用エンドポイント
- `optional_auth_middleware`: 認証任意のパブリックエンドポイント

**トークン管理**:

- アクセストークン: 15 分（短命）
- リフレッシュトークン: 7 日間、自動更新あり
- パスワードリセットトークン: 1 時間・使い切り

### サービス層のパターン

サービスは **動的な動作切替** を実装：

```rust
impl TaskService {
    pub async fn list_tasks_dynamic(&self, user: &AuthenticatedUser, filter: Option<TaskFilterDto>) -> AppResult<TaskResponse> {
        let permission_result = user.can_perform_action("tasks", "read", None);
        match permission_result {
            PermissionResult::Allowed { privilege, scope } => {
                self.execute_task_query(user, filter, privilege, scope).await
            }
            // パーミッション結果に基づき異なる処理を実行
        }
    }
}
```

---

## データベーススキーマパターン

**マルチテナンシー対応**:

- スキーマベースの分離（`DATABASE_SCHEMA`で設定可能）
- ユーザー単位のデータアクセス
- サブスクリプション履歴の追跡

**主要テーブル**:

- `users`: 基本ユーザーデータ + サブスクリプション階層
- `roles`: パーミッション定義
- `subscription_history`: プラン変更の監査記録
- `tasks`: ユーザー所有のタスク
- トークン関連: `refresh_tokens`, `password_reset_tokens`

---

## 設定システム

**統合設定ファイル**（`src/config.rs`）:

- 環境変数に基づく設定読み込み
- サーバー・DB・JWT・メール・セキュリティの個別設定
- 開発／本番のモード検出

**主要環境変数**:

```bash
DATABASE_URL=postgres://postgres:password@localhost:5432/taskdb
SERVER_ADDR=0.0.0.0:3000
DATABASE_SCHEMA=custom_schema  # スキーマ分離（任意）
RUST_LOG=info
```

---

## 動的パーミッションシステムの設計

本システムの核は、**ユーザー文脈に応じて API 動作を切り替えること**です。

### パーミッションモデル

```rust
pub struct Permission {
    pub resource: String,      // "tasks", "users", "reports" など
    pub action: String,        // "read", "write", "delete", "admin"
    pub scope: PermissionScope,
}

pub enum PermissionScope {
    Own,           // 自分のデータ
    Team,          // チーム単位
    Organization,  // 組織全体
    Global,        // 全体アクセス
}

pub struct Privilege {
    pub name: String,
    pub subscription_tier: SubscriptionTier,
    pub quota: Option<PermissionQuota>,
}
```

### サブスクリプション階層

- **Free**: 自分の範囲、最大 100 件、基本機能
- **Pro**: チーム範囲、最大 1 万件、高度な検索やエクスポート可
- **Enterprise**: 全体範囲、無制限、すべての機能利用可

### 実装パターン（サービス層の動作切替）

```rust
match (scope, privilege.subscription_tier) {
    (PermissionScope::Own, SubscriptionTier::Free) => {
        self.list_tasks_for_user_limited(user_id, privilege.quota).await
    }
    (PermissionScope::Team, SubscriptionTier::Pro) => {
        self.list_tasks_for_team_with_features(user_id, &privilege.features, filter).await
    }
    (PermissionScope::Global, SubscriptionTier::Enterprise) => {
        self.list_all_tasks_unlimited(filter).await
    }
    _ if user.is_admin() => {
        self.list_all_tasks_unlimited(filter).await
    }
    _ => {
        self.list_tasks_for_user(user_id).await.map(TaskResponse::Limited)
    }
}
```

---

## テスト戦略

**テスト構成**:

- 単体テスト: `src/*/mod.rs`（高速・独立）
- 統合テスト: `tests/integration/`（`testcontainers`使用）
- テスト用共通ユーティリティ: `tests/common/`

**DB テスト**:

- PostgreSQL + `testcontainers`
- 並列実行のためのスキーマ分離
- 各テストで自動マイグレーション実行

**テスト実行コマンド**:

```bash
cargo test --lib                           # 単体テストのみ（高速）
cargo test integration::tasks::crud_tests  # 特定統合テスト
cargo test --test integration -- --test-threads 1  # 直列実行
```

---

## 実装上の重要事項

- **エラーハンドリング**: `AppError` による HTTP ステータスマッピング

- **バリデーション**: `validator` クレート + カスタムロジック

- **セキュリティ機能**:

  - Argon2 パスワードハッシュ＋自動リハッシュ
  - CORS 設定、セキュリティヘッダ
  - レートリミット対応準備済み

- **バッチ操作**: 全 CRUD が最大 100 件のバッチ対応

- **フィルタリングとページネーション**: 動的パーミッション考慮の上で柔軟に対応

---

このコードベースを扱う際は、**API 変更が動的パーミッションに与える影響**を常に意識し、**異なるユーザーコンテキストごとに十分なテストカバレッジ**を確保してください。
