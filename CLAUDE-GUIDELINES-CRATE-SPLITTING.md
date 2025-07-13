# クレート分割ガイドライン

このドキュメントは、モノリシックなRustプロジェクトをマルチクレート構造に移行するための設計原則と実装ガイドラインを定めています。

## 🎯 クレート分割の目的と効果

### 目的
- **ビルド時間の短縮**: 並列ビルドによる高速化（50%以上の短縮）
- **開発効率の向上**: 変更箇所のみの再ビルド
- **依存関係の明確化**: 循環依存の解消と可視化
- **保守性の向上**: モジュール境界の明確化

### 期待される効果
- フルビルド: 2分 → 1分以内
- インクリメンタルビルド: 10秒以内
- 並列度の向上: CPUコア数に応じたスケーラビリティ

## 📋 クレート構成と依存関係

### 1. 基盤層（Foundation Layer）
```
crates/
├── common/          # 依存: なし
│   ├── types/       # 基本型定義（UUID, DateTime等）
│   ├── errors/      # エラー型定義（AppError, Result<T>）
│   └── traits/      # 共通トレイト（Repository, Service等）
│
└── infrastructure/  # 依存: common
    ├── database/    # DB接続プール、トランザクション管理
    ├── redis/       # Redis接続、キャッシュ
    ├── config/      # 環境設定、設定管理
    └── external/    # 外部サービス統合（メール、S3等）
```

### 2. コア層（Core Layer）
```
crates/
├── shared-core/     # 依存: common
│   ├── domain/      # 共有ドメインモデル
│   └── services/    # 共有サービストレイト
│
├── user-core/       # 依存: common, shared-core
│   ├── models/      # User, UserProfile等
│   ├── traits/      # UserRepository, UserService trait
│   └── dto/         # 基本的なUser DTO
│
├── auth-core/       # 依存: common, shared-core
│   ├── models/      # JWT, Session, Token等
│   ├── traits/      # AuthService, TokenProvider trait
│   └── security/    # 認証・認可の基本型
│
└── security-core/   # 依存: common, shared-core
    ├── models/      # Role, Permission等
    ├── traits/      # SecurityService trait
    └── policies/    # セキュリティポリシー定義
```

### 3. 機能層（Feature Layer）
```
crates/
├── payment/         # 依存: common, infrastructure, user-core
├── storage/         # 依存: common, infrastructure
├── gdpr/           # 依存: common, user-core
├── system/         # 依存: common
├── task/           # 依存: common, user-core, auth-core
├── team/           # 依存: common, user-core, auth-core
├── organization/   # 依存: common, user-core, team
├── analytics/      # 依存: common, organization
├── admin/          # 依存: 複数の機能クレート
├── subscription/   # 依存: common, user-core, payment
└── api/            # 依存: 全て（統合層）
```

## 🔄 循環依存の解消戦略

### 1. auth ↔ user の解消

#### 問題
```rust
// auth needs user
use crate::features::user::models::User;
use crate::features::user::services::UserService;

// user needs auth  
use crate::features::auth::services::AuthService;
use crate::features::auth::models::JWT;
```

#### 解決策
```rust
// shared-core/src/traits/auth.rs
pub trait Authenticatable {
    fn get_id(&self) -> Uuid;
    fn get_email(&self) -> &str;
    fn is_active(&self) -> bool;
}

pub trait AuthenticationProvider {
    async fn authenticate(&self, credentials: &Credentials) -> Result<AuthToken>;
    async fn validate_token(&self, token: &str) -> Result<TokenClaims>;
}

// user-core/src/models/user.rs
impl Authenticatable for User {
    fn get_id(&self) -> Uuid { self.id }
    fn get_email(&self) -> &str { &self.email }
    fn is_active(&self) -> bool { self.is_active }
}

// auth-core/src/services/auth.rs
pub struct AuthService<U: Authenticatable> {
    user_provider: Arc<dyn UserProvider<User = U>>,
}
```

### 2. auth ↔ security の解消

#### 問題
```rust
// auth needs security roles
use crate::features::security::models::Role;

// security needs auth for permission checks
use crate::features::auth::services::AuthService;
```

#### 解決策
```rust
// shared-core/src/traits/security.rs
pub trait RoleProvider {
    async fn get_user_roles(&self, user_id: Uuid) -> Result<Vec<String>>;
}

pub trait PermissionChecker {
    async fn has_permission(&self, user_id: Uuid, permission: &str) -> Result<bool>;
}

// 各クレートで実装
impl RoleProvider for SecurityService { ... }
impl PermissionChecker for SecurityService { ... }
```

## 📁 推奨フォルダ構造

### クレート内部構造
```
crates/{crate-name}/
├── Cargo.toml
├── src/
│   ├── lib.rs          # クレートのエントリーポイント
│   ├── models/         # ドメインモデル
│   │   ├── mod.rs
│   │   └── {model}.rs
│   ├── services/       # ビジネスロジック
│   │   ├── mod.rs
│   │   └── {service}.rs
│   ├── repositories/   # データアクセス（feature cratesのみ）
│   │   ├── mod.rs
│   │   └── {repository}.rs
│   ├── handlers/       # HTTPハンドラー（apiクレートのみ）
│   │   ├── mod.rs
│   │   └── {handler}.rs
│   ├── dto/           # データ転送オブジェクト
│   │   ├── mod.rs
│   │   ├── requests/
│   │   └── responses/
│   └── traits/        # 公開トレイト定義
│       ├── mod.rs
│       └── {trait}.rs
└── tests/
    └── integration/
```

### 公開APIの設計原則
```rust
// crates/user-core/src/lib.rs
// 必要最小限の公開API
pub mod models {
    pub use self::user::{User, UserStatus};
}

pub mod traits {
    pub use self::repository::UserRepository;
    pub use self::service::UserService;
}

pub mod dto {
    pub use self::responses::UserResponse;
}

// 内部実装は隠蔽
mod internal;
```

## 🚀 実装手順

### Phase 1: 基盤準備
1. ワークスペース構造の作成
   ```toml
   # Cargo.toml
   [workspace]
   members = ["crates/*", "migration"]
   resolver = "2"
   
   [workspace.dependencies]
   # 共通依存関係をここで定義
   ```

2. 循環依存の解消
   - 共通トレイトの抽出
   - インターフェース定義
   - 依存方向の整理

### Phase 2: 基盤クレート作成
1. **common**クレート
   ```bash
   cargo new crates/common --lib
   # src/shared/types → crates/common/src/types
   # src/shared/errors → crates/common/src/errors
   ```

2. **infrastructure**クレート
   ```bash
   cargo new crates/infrastructure --lib
   # src/infrastructure → crates/infrastructure/src
   ```

### Phase 3: 段階的移行
1. 独立性の高いモジュールから開始
2. テストを維持しながら移行
3. 各段階でビルド時間を計測

## ✅ チェックリスト

### クレート作成時
- [ ] Cargo.tomlに適切なメタデータを設定
- [ ] 必要最小限の公開APIのみexport
- [ ] 内部実装は`pub(crate)`または`private`
- [ ] ドキュメントコメントを追加
- [ ] 単体テストを含める

### 依存関係
- [ ] 循環依存がないことを確認
- [ ] 依存の方向が一方向であること
- [ ] 不要な依存を含まない
- [ ] features flagを適切に使用

### パフォーマンス
- [ ] ビルド時間を計測・記録
- [ ] 並列ビルドが効いているか確認
- [ ] インクリメンタルビルドの効果を確認

## 🔍 トラブルシューティング

### 循環依存エラー
```bash
error: cyclic package dependency: package `auth-core v0.1.0`
```
→ 共通トレイトをshared-coreに抽出

### ビルドが遅い
- 依存関係グラフを確認: `cargo tree --depth=2`
- 不要な依存を削除
- features flagで機能を分割

### テストが失敗
- インポートパスを新しいクレート構造に更新
- `use crate::` → `use {crate_name}::`

## 📊 成功指標

1. **ビルド時間**
   - フルビルド: 50%以上短縮
   - インクリメンタル: 10秒以内

2. **コード品質**
   - cargo clippy: 警告ゼロ
   - cargo test: 全テストパス（218個）

3. **保守性**
   - 明確なモジュール境界
   - 依存関係の可視化
   - 公開APIの最小化