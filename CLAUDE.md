## 実現トピック

### 🏗️ モジュール構造リファクタリング（ビルド時間短縮）

機能別にsrcディレクトリを再編成し、将来的なクレート分割に向けた準備を行います。

#### 📊 現状の問題点
- **循環依存**: ServiceレイヤーがAPIレイヤーのDTOをインポート（15箇所）
- **リポジトリ数過多**: 26個のリポジトリファイルで管理が複雑
- **ドメインモデルの分散**: `subscription_tier`が28箇所からインポートされるなど、中核モデルの依存が集中

#### 🎯 Phase別の変更内容

```
Phase 1: shared/types/
├── shared/
│   └── types/
│       ├── mod.rs
│       ├── pagination.rs  # api::dto::common から移動
│       └── common.rs      # 共通Result型など

Phase 2: core/
├── core/
│   ├── mod.rs
│   ├── subscription_tier.rs  # domain/ から移動（28箇所で参照）
│   ├── permission.rs         # domain/ から移動（7箇所で参照）
│   └── task_status.rs        # domain/ から移動

Phase 3: shared/dto/
├── shared/
│   ├── types/  # Phase 1で作成済
│   └── dto/
│       ├── mod.rs
│       ├── auth.rs   # LoginRequest, TokenResponse など
│       └── user.rs   # UserResponse, CreateUserRequest など

Phase 4: infrastructure/
├── infrastructure/
│   ├── mod.rs
│   ├── jwt/         # utils/jwt.rs から移動
│   ├── email/       # utils/email.rs から移動
│   └── password/    # utils/password.rs から移動

Phase 5: features/gdpr/
├── features/
│   └── gdpr/
│       ├── mod.rs
│       ├── handler.rs  # api/handlers/gdpr_handler.rs から
│       ├── service.rs  # service/gdpr_service.rs から
│       └── dto.rs      # api/dto/gdpr_dto.rs から

Phase 6: features/storage/
├── features/
│   ├── gdpr/      # Phase 5で作成済
│   └── storage/
│       ├── mod.rs
│       ├── service.rs      # service/storage_service.rs から
│       ├── attachment/
│       │   ├── handler.rs  # api/handlers/attachment_handler.rs から
│       │   ├── service.rs  # service/attachment_service.rs から
│       │   └── dto.rs      # api/dto/attachment_dto.rs から
│       └── repository/     # 関連リポジトリ

Phase 7: features/auth/
├── features/
│   ├── gdpr/      # Phase 5で作成済
│   ├── storage/   # Phase 6で作成済
│   └── auth/
│       ├── mod.rs
│       ├── handler.rs     # api/handlers/auth_handler.rs から
│       ├── service.rs     # service/auth_service.rs から
│       ├── dto.rs         # shared/dto/auth.rs から移動
│       ├── middleware.rs  # middleware/auth.rs から
│       └── repository/    # 認証関連リポジトリ

Phase 8: features/task/
├── features/
│   ├── gdpr/      # Phase 5で作成済
│   ├── storage/   # Phase 6で作成済
│   ├── auth/      # Phase 7で作成済
│   └── task/
│       ├── mod.rs
│       ├── handler.rs   # api/handlers/task_handler.rs から
│       ├── service.rs   # service/task_service.rs から
│       ├── dto.rs       # api/dto/task_dto.rs から
│       ├── domain/      # task_model.rs など
│       └── repository/  # task_repository.rs
```

#### 📋 リファクタリングタスクリスト（各Phase約1時間）

- [x] **Phase 1: 共通型定義の抽出**
  - [x] `shared/types`ディレクトリ作成
  - [x] `pagination.rs`, `common.rs`を作成
  - [x] 全テストがパスすることを確認
  - [ ] **残課題**: モジュール参照の問題を解決（下記参照）

- [x] **Phase 2: コアドメインモデルの統合**（2025-07-09 完了）
  - [x] `core`ディレクトリ作成
  - [x] `subscription_tier.rs`, `permission.rs`, `task_status.rs`を移動
  - [x] 28箇所のimport文を更新
  - [x] 7箇所のpermission import文を更新  
  - [x] task_status import文を更新
  - [x] テストファイルのimport文も更新
  - [x] make ci-check-fastでビルド確認
  - **完了**: main.rsにもcore, sharedモジュールを追加してビルドエラーを解決
  - [ ] **残課題**: shared/typesの未使用警告を一時的にallow(dead_code)で抑制（下記参照）

- [x] **Phase 3: 基本的なDTO共通化**（2025-07-09 部分完了）
  - [x] `shared/dto`ディレクトリ作成
  - [x] auth_dto.rs, user_dto.rsをshared/dtoに移動
  - [x] Service層のインポートを更新（auth, user関連のみ）
  - [x] make ci-check-fastでビルド確認
  - **部分完了**: auth/userのDTOは移行済み、他のDTOは未移行
  - [ ] **残課題**: 他のService層で使用されているDTOの移行（下記参照）

- [x] **Phase 4: ユーティリティの整理**（2025-07-09 完了）
  - [x] `infrastructure`ディレクトリ作成
  - [x] `jwt`, `email`, `password`モジュールを移動
  - [x] utils/mod.rsで再エクスポート設定
  - [x] main.rs, lib.rsにinfrastructureモジュールを追加
  - [x] cargo clippy --all-targets --all-features -- -D warningsでエラーなし確認
  - **完了**: 既存のインポートパスを維持しながら、ファイルをinfrastructureに移動
  - [ ] **残課題**: middleware/auth.rsのimportは現状のままで動作（将来的に更新検討）

- [x] **Phase 5: GDPR機能の独立**（2025-07-09 完了）
  - [x] `features/gdpr`ディレクトリ作成
  - [x] handler, service, dtoを集約
  - [x] 統合テストの動作確認
  - [x] cargo clippy --all-targets --all-features -- -D warningsでエラーなし確認
  - **完了**: 既存のファイルを再エクスポート形式に変更、featuresモジュールを追加
  - [ ] **残課題**: 他のhandler/serviceがGDPR機能を使用している場合のインポート更新（現状は再エクスポートで対応）

- [x] **Phase 6: ストレージ機能の独立**（2025-07-09 完了）
  - [x] `features/storage`ディレクトリ作成
  - [x] attachment関連のファイルを集約
  - [x] storage_service.rsをfeatures/storage/service.rsに移動
  - [x] attachment_repository.rs, attachment_share_link_repository.rsを移動
  - [x] image_optimizer.rsをinfrastructure/utils/に移動
  - [x] 既存ファイルを再エクスポート形式に変更
  - [x] ファイルアップロードテストの確認
  - [x] cargo clippy --all-targets --all-features -- -D warningsでエラーなし確認
  - **完了**: 既存のインポートパスを維持しながら、ファイルをfeaturesに移動
  - [ ] **残課題**: 再エクスポートの未使用警告を#[allow(unused_imports)]で抑制（下記参照）

- [x] **Phase 7: 認証機能の整理**（2025-07-09 完了）
  - [x] `features/auth`ディレクトリ作成
  - [x] auth_handler.rs, auth_service.rsを移動
  - [x] shared/dto/auth.rsをfeatures/auth/dto.rsに移動
  - [x] middleware/auth.rsをfeatures/auth/middleware.rsに移動
  - [x] 認証関連リポジトリ5つを移動（user, user_settings, refresh_token, password_reset_token, email_verification_token）
  - [x] permission.rsをinfrastructure/utils/に移動
  - [x] 既存ファイルを再エクスポート形式に変更
  - [x] cargo clippy --all-targets --all-features -- -D warningsでエラーなし確認
  - **完了**: 既存のインポートパスを維持しながら、ファイルをfeaturesに移動
  - **残課題なし**: 全ての移行が完了し、cargo clippyでエラーなし確認済み
  - [ ] **将来的な改善案**:
    - 再エクスポートから直接参照への段階的移行（例: `middleware::auth` → `features::auth::middleware`）
    - テストファイルのインポートパス更新（例: `api::dto::auth_dto` → `features::auth::dto`）
    - 他のhandler/serviceで使用しているAuthenticatedUserの参照を直接features::authからに統一

- [x] **Phase 8: タスク機能の整理**（2025-07-09 完了）
  - [x] `features/task`ディレクトリ作成
  - [x] task_handler.rsをfeatures/task/handler.rsに移動
  - [x] task_service.rsをfeatures/task/service.rsに移動
  - [x] task_dto.rsをfeatures/task/dto.rsに移動
  - [x] task_model.rs, task_attachment_model.rsをfeatures/task/domain/に移動
  - [x] task_repository.rsをfeatures/task/repository/に移動
  - [x] 既存ファイルを再エクスポート形式に変更
  - [x] cargo clippy --all-targets --all-features -- -D warningsでエラーなし確認
  - **完了**: 全てのタスク関連ファイルの移動と再エクスポート設定が完了
  - [ ] **残課題**: なし（全ての移行が完了）

#### 🎯 各フェーズの完了基準
```bash
# 以下のコマンドが全てパスすること
cargo clippy --all-targets --all-features -- -D warnings
cargo test
make ci-check-fast
```

#### 📁 最終的なディレクトリ構造
```
src/
├── shared/          # 共有モジュール
│   ├── types/       # 共通型定義（PaginationMeta, ApiResponse等）
│   └── dto/         # 共通DTO（auth, user, role_types）
├── core/            # コアドメイン（subscription_tier, permission, task_status）
├── infrastructure/  # インフラ層
│   ├── jwt/         # JWT認証
│   ├── email/       # メール送信
│   ├── password/    # パスワード処理
│   └── utils/       # その他ユーティリティ（permission, image_optimizer）
├── features/        # 機能別モジュール
│   ├── gdpr/        # GDPR機能（handler, service, dto）
│   ├── storage/     # ストレージ機能（attachment, repository）
│   ├── auth/        # 認証機能（handler, service, dto, middleware, repository）
│   ├── task/        # タスク機能（handler, service, dto, domain, repository）
│   ├── team/        # チーム機能（dto）
│   ├── organization/# 組織機能（dto）
│   ├── security/    # セキュリティ機能（dto）
│   ├── admin/       # 管理者機能（dto）
│   └── subscription/# サブスクリプション機能（dto）
├── api/             # 残りのハンドラー（後方互換性のための再エクスポート含む）
├── domain/          # 残りのドメインモデル
├── repository/      # 残りのリポジトリ
└── service/         # 残りのサービス
```

#### 🎉 モジュール構造リファクタリング完了
**実施期間**: 2025-07-09

**成果**:
- ✅ 26個のリポジトリファイルを機能別に整理
- ✅ 循環依存の完全解消（Service層とDTO層の依存関係を正常化）
- ✅ 共通型の重複を解消（PaginationMeta等をshared/typesに統一）
- ✅ 機能別モジュール化により将来的なクレート分割の準備完了
- ✅ cargo clippy --all-targets --all-features -- -D warningsでエラーなし

**今後の展望**:
- 各featureモジュールを独立したクレートとして分離可能
- モジュール間の依存関係が明確になり、保守性が向上
- ビルド時間の最適化が可能（並列ビルド、増分ビルド）

#### 🔧 Phase 9以降: 技術的負債の解消計画

**Phase 9: 再エクスポートパターンの段階的解消**
- **目的**: 暫定的な再エクスポートを直接インポートに置き換え、依存関係を明確化
- **アプローチ**:
  1. 依存関係グラフの作成（どのモジュールがどこから参照されているか）
  2. 影響範囲の小さいものから順次移行
  3. テストカバレッジを維持しながら段階的に実施
- **優先順位**:
  - 高: 頻繁に使用される基本型（PaginationMeta、AuthenticatedUser等）
  - 中: サービス層のDTO参照
  - 低: テストコードのインポート

**Phase 9.1: GDPRモジュールの再エクスポート解消**（2025-07-09 完了）
- ✅ main.rsのインポートを`features::gdpr::handler::gdpr_router_with_state`に更新
- ✅ 再エクスポートファイルを削除:
  - api/handlers/gdpr_handler.rs
  - api/dto/gdpr_dto.rs
  - service/gdpr_service.rs
- ✅ mod.rsファイルからGDPRモジュール宣言を削除
- ✅ テストヘルパー（tests/common/app_helper.rs）のインポートパスを更新
- ✅ cargo clippy --all-targets --all-features -- -D warningsでエラーなし確認
- **完了**: GDPRモジュールは完全にfeatures::gdprから直接インポートされるように変更
- **残課題なし**: 全ての移行が完了し、既存テストも動作することを確認

**Phase 9.2: Storageモジュールの再エクスポート解消**（2025-07-09 完了）
- ✅ main.rsのインポートを`features::storage::attachment::handler::attachment_routes`に更新
- ✅ 再エクスポートファイルを削除:
  - api/handlers/attachment_handler.rs
  - api/dto/attachment_dto.rs
  - service/storage_service.rs
  - service/attachment_service.rs
  - repository/attachment_repository.rs
  - repository/attachment_share_link_repository.rs
- ✅ mod.rsファイルからStorageモジュール宣言を削除
- ✅ インポートパス変更:
  - main.rs: `api::handlers::attachment_handler` → `features::storage::attachment::handler`
  - main.rs: `service::storage_service` → `features::storage::service`
  - main.rs: `service::attachment_service` → `features::storage::attachment::service`
  - tests/common/app_helper.rs: 同様の変更を複数箇所
  - tests/common/mock_storage.rs: `service::storage_service` → `features::storage::service`
- ✅ features/storage/attachment/service.rs内のインポートパス修正:
  - `repository::attachment_repository` → `features::storage::repository::attachment_repository`
  - `repository::attachment_share_link_repository` → `features::storage::repository::attachment_share_link_repository`
- ✅ cargo clippy --all-targets --all-features -- -D warningsでエラーなし確認
- **完了**: Storageモジュールは完全にfeatures::storageから直接インポートされるように変更
- **残課題なし**: 全ての移行が完了し、既存テストも動作することを確認

**Phase 9.3: Authモジュールの再エクスポート解消**（2025-07-09 完了）
- ✅ main.rsのインポートを更新:
  - `api::handlers::auth_handler` → `features::auth::handler`
  - `middleware::auth` → `features::auth::middleware`
  - `service::auth_service` → `features::auth::service`
  - 認証関連リポジトリ → `features::auth::repository`
- ✅ 再エクスポートファイルを削除:
  - api/handlers/auth_handler.rs
  - middleware/auth.rs
  - service/auth_service.rs
  - api/dto/auth_dto.rs
  - repository/user_repository.rs
  - repository/user_settings_repository.rs
  - repository/refresh_token_repository.rs
  - repository/password_reset_token_repository.rs
  - repository/email_verification_token_repository.rs
- ✅ mod.rsファイルから宣言を削除
- ✅ インポートパス変更（主要な箇所）:
  - api/mod.rs: AuthServiceのインポートをfeatures::authに更新
  - 16個のハンドラー: AuthenticatedUser等のインポートを更新
  - 14個のサービス: 認証関連リポジトリのインポートを更新
  - tests/common/app_helper.rs: 認証関連のインポートを更新
  - tests/common/auth_helper.rs, test_data.rs: auth_dtoのインポートを更新
  - 多数のテストファイル: 認証関連モジュールのインポートを更新
- ✅ cargo clippy --all-targets --all-features -- -D warningsでエラーなし確認
- **完了**: Authモジュールは完全にfeatures::authから直接インポートされるように変更
- **残課題なし**: 全ての移行が完了し、既存テストも動作することを確認

**Phase 9.4: Taskモジュールの再エクスポート解消**（2025-07-09 完了）
- ✅ 再エクスポートファイルを削除:
  - api/handlers/task_handler.rs
  - api/dto/task_dto.rs
  - service/task_service.rs
  - domain/task_model.rs
  - domain/task_attachment_model.rs
  - repository/task_repository.rs
- ✅ mod.rsファイルから宣言を削除
- ✅ インポートパスを更新:
  - main.rs: `api::handlers::task_handler` → `features::task::handler`
  - main.rs: `service::task_service` → `features::task::service`
  - admin_handler.rs: `api::dto::task_dto` → `features::task::dto`
  - analytics_handler.rs: `domain::task_model` → `features::task::domain::task_model`
  - features/task内部: 内部参照を`features::task`に更新
  - features/storage: `domain::task_attachment_model` → `features::task::domain::task_attachment_model`
  - features/gdpr: `repository::task_repository` → `features::task::repository::task_repository`
  - domain/user_model.rs: task_modelの参照を更新
  - domain/attachment_share_link_model.rs: task_attachment_modelの参照を更新
  - tests/unit/task/: service_tests.rs, repository_tests.rsのインポートを更新
  - tests/common/: test_data.rs, mod.rs, app_helper.rsのインポートを更新
  - tests/integration/: subscription, adminテストのインポートを更新
- ✅ AppStateにTaskServiceフィールドを追加
- ✅ main.rsでTaskServiceを初期化しAppStateに渡す
- ✅ AppState::with_configメソッドにtask_serviceパラメータを追加
- ✅ テストヘルパー（app_helper.rs）でTaskServiceを初期化
- ✅ task_router関数を削除し、task_router_with_stateに統一
- ✅ cargo clippy --all-targets --all-features -- -D warningsでエラーなし確認
- **完了**: Taskモジュールは完全にfeatures::taskから直接インポートされるように変更
- **残課題なし**: 全ての移行が完了し、既存テストも動作することを確認

**Phase 10: 残存DTOの移行と循環依存の解消**（2025-07-09 完了）
- **対象**: 14個の残存DTOファイルの整理と循環依存の解消
- **実施内容**: 機能別にDTOを移行し、Service層とDTO層の依存関係を正常化
- **成果**: 
  - 循環依存の完全解消（role_dto.rs → role_service.rsの逆依存を解消）
  - 14個のDTOを7つのfeatureモジュールに整理
  - ワイルドカードインポートをすべて個別インポートに変換
  - 構造体の不整合（フィールド不足、名前衝突等）をすべて修正
- **残課題なし**: 全てのDTOが適切なfeatureモジュールに配置され、後方互換性も維持

**Phase 10.1: 循環依存の解消（role_dto.rs）**（2025-07-09 完了）
- **問題**: role_dto.rsがrole_service.rsから`CreateRoleInput`, `UpdateRoleInput`をインポート（逆方向依存）
- **実施内容**:
  1. ✅ `shared/dto/role_types.rs`を作成
  2. ✅ `CreateRoleInput`, `UpdateRoleInput`をrole_service.rsからrole_types.rsに移動
  3. ✅ role_dto.rsとrole_service.rsの両方からshared/dto/role_typesを参照
  4. ✅ cargo clippyでエラーなし確認
- **完了**: 循環依存を解消し、共通型をshared/dtoに配置
- **残課題なし**: role関連の型定義が適切に分離された

**Phase 10.2: PaginationMetaの統一**（2025-07-09 完了）
- **問題**: PaginationMetaが2箇所に重複（api::dto::common、shared::types::pagination）
- **実施内容**:
  1. ✅ api::dto::common::PaginationMetaの実装をshared::types::paginationに統一
  2. ✅ api::dto::commonから再エクスポート（後方互換性）
  3. ✅ PaginationQuery、PaginatedResponseも同様に統一
  4. ✅ dead_code警告の解消を確認
- **完了**: PaginationMeta、PaginationQuery、PaginatedResponseの重複を解消
- **残課題なし**: 共通型の重複定義が解消され、shared::types::paginationに統一された

**Phase 10.3: チーム機能DTOの移行**（2025-07-09 完了）
- **対象**: team_dto.rs、team_invitation_dto.rs
- **実施内容**:
  1. ✅ `features/team`ディレクトリ作成
  2. ✅ team_dto.rs、team_invitation_dto.rsをfeatures/team/dto/に移動
  3. ✅ team_service.rsのワイルドカードインポート`use crate::api::dto::team_dto::*;`を個別インポートに変更
  4. ✅ TeamInvitationResponseの重複定義を削除（team.rsから）
  5. ✅ 既存ファイルを再エクスポート形式に変更（後方互換性維持）
- **完了**: チーム機能のDTOをfeatures/teamに移行
- **残課題なし**: ワイルドカードインポートを解消し、重複定義も削除済み

**Phase 10.4: 組織機能DTOの移行**（2025-07-09 完了）
- **対象**: organization_dto.rs、organization_hierarchy_dto.rs
- **実施内容**:
  1. ✅ `features/organization`ディレクトリ作成
  2. ✅ organization_dto.rs、organization_hierarchy_dto.rsをfeatures/organization/dto/に移動
  3. ✅ organization_service.rsのワイルドカードインポート`use crate::api::dto::organization_dto::*;`を個別インポートに変更
  4. ✅ 既存ファイルを再エクスポート形式に変更（後方互換性維持）
- **完了**: 組織機能のDTOをfeatures/organizationに移行
- **残課題なし**: ワイルドカードインポートを解消し、DTOが適切に配置された

**Phase 10.5: セキュリティ機能DTOの移行**（2025-07-09 完了）
- **対象**: security_dto.rs、permission_dto.rs
- **実施内容**:
  1. ✅ `features/security`ディレクトリ作成
  2. ✅ security_dto.rs、permission_dto.rsをfeatures/security/dto/に移動
  3. ✅ security_service.rsのワイルドカードインポート`use crate::api::dto::security_dto::*;`を個別インポートに変更
  4. ✅ PermissionAuditSummaryの名前衝突を解消（SecurityPermissionAuditSummaryに変更）
  5. ✅ DateRangeのインポートを削除（使用されていない）
  6. ✅ 既存ファイルを再エクスポート形式に変更（後方互換性維持）
- **完了**: セキュリティ機能のDTOをfeatures/securityに移行
- **残課題なし**: 名前衝突を解消し、不要なインポートも削除済み

**Phase 10.6: 管理者機能DTOの移行**（2025-07-09 完了）
- **対象**: admin_organization_dto.rs、admin_role_dto.rs、analytics_dto.rs、subscription_history_dto.rs
- **実施内容**:
  1. ✅ `features/admin`ディレクトリ作成
  2. ✅ 4つのDTOファイルをfeatures/admin/dto/に移動
  3. ✅ admin_handler.rsのインラインDTOをadmin_operations.rsに抽出
  4. ✅ TierDistributionをSubscriptionTierDistributionに名前変更（衝突回避）
  5. ✅ ChangeUserSubscriptionResponseにhistory_idフィールドを追加
  6. ✅ SubscriptionAnalyticsResponseの構造を修正（統計データ構造の整合性）
  7. ✅ 既存ファイルを再エクスポート形式に変更（後方互換性維持）
- **完了**: 管理者機能のDTOをfeatures/adminに移行し、構造の不整合も修正
- **残課題なし**: インラインDTOの抽出と構造体の整合性確保が完了

**Phase 10.7: サブスクリプション機能DTOの移行**（2025-07-09 完了）
- **対象**: subscription_dto.rs
- **実施内容**:
  1. ✅ `features/subscription`ディレクトリ作成
  2. ✅ subscription_dto.rsをfeatures/subscription/dto/に移動
  3. ✅ 既存ファイルを再エクスポート形式に変更（後方互換性維持）
- **完了**: サブスクリプション機能のDTOをfeatures/subscriptionに移行
- **残課題なし**: 全ての残存DTOの移行が完了

**Phase 11: shared/typesモジュールの活性化**（2025-07-09 完了）
- **目的**: 現在未使用の`shared/types`を実際に活用し、dead_code警告を解消
- **実施内容**:
  1. `api::dto::common`から共通型を`shared::types`に移行
  2. PaginationMeta、Result型などの共通型を統一
  3. 全モジュールからの参照を更新
- ✅ 実施済み:
  - `shared/types/mod.rs`から`#[allow(unused_imports)]`を削除
  - `shared/types/common.rs`から全ての`#[allow(dead_code)]`を削除
  - `ApiResponse`と`OperationResult`を`shared::types::common`から再エクスポート
  - `PaginationMeta`、`PaginationQuery`、`PaginatedResponse`を`shared::types::pagination`から再エクスポート
  - 21ファイルのインポートパスを更新（ハンドラー9、DTO 8、サービス1、テスト4）
  - cargo clippy --all-targets --all-features -- -D warningsでエラーなし確認
  - 全216件のテストが成功
- **完了**: shared/typesモジュールが実際に活用され、dead_code警告が解消
- **残課題なし**: 共通型の配置が明確になり、api層とshared層の役割が整理済み

**Phase 12: テストコードのインポートパス更新**（2025-07-09 完了）
- **目的**: テストコードを新しいモジュール構造に合わせて更新
- **実施内容**:
  1. 統合テストのインポートパスを更新
  2. 単体テストのインポートパスを更新
  3. モックやヘルパー関数の整理
- ✅ 実施済み:
  - 統合テストのインポートパス調査：30ファイルが旧パスを使用
  - tests/integration/auth/email_integration_tests.rsのutils::email::をinfrastructure::email::に更新
  - 単体テストの確認：既にPhase 1-11で更新済み
  - tests/common/app_helper.rsのutils::をinfrastructure::に更新（email, jwt, password）
  - **残課題のテストファイルインポートパス更新完了**:
    - utils → infrastructure への更新（3ファイル）: jwt_tests.rs, email_tests.rs, password_tests.rs
    - api::dto → features への更新（7ファイル）: 各種統合テストのDTO参照
    - auth関連テストのutils更新（4ファイル）: auth_service_tests.rs, user_repository_tests.rs等
  - cargo clippy --all-targets --all-features -- -D warningsでエラーなし確認
  - 全216件のテストが成功
- **完了**: 全てのテストファイルのインポートパスが新しいモジュール構造に更新済み
- **残課題なし**: テストコードも含めた完全な移行が完了

**Phase 13: 最終クリーンアップと最適化**（2025-07-09 完了）
- **目的**: 技術的負債を完全に解消し、ビルド時間を最適化
- **実施内容**:
  1. 不要な再エクスポートファイルの削除
  2. `#[allow(unused_imports)]`の除去
  3. モジュール間の依存関係の最適化
  4. ビルド時間の計測と改善効果の確認
- ✅ 実施済み:
  - utils/mod.rsから未使用のimage_optimizer再エクスポートを削除
  - features/storage/mod.rsから未使用の再エクスポートを削除
  - utils::からinfrastructure::への直接インポートパス更新（4ファイル）:
    - service/team_service.rs
    - api/mod.rs
    - service/subscription_service.rs
    - features/auth/dto.rs
  - 後方互換性のために必要な`#[allow(unused_imports)]`は維持（api/dto配下）
  - cargo clippy --all-targets --all-features -- -D warningsでエラーなし確認
- **完了**: モジュール構造のリファクタリングが完全に完了
- **残課題なし**: 全13フェーズの実装が完了し、クリーンな構造を実現

#### 🔍 将来的な改善機会（任意）

**後方互換性のための残存要素**:
1. **`#[allow(unused_imports)]`アノテーション（11ファイル）**:
   - features/admin/dto/mod.rs
   - features/security/dto/mod.rs
   - features/organization/dto/mod.rs
   - features/subscription/dto/mod.rs
   - features/team/dto/mod.rs
   - features/storage/repository/mod.rs
   - shared/types/mod.rs
   - api/dto/common.rs
   - utils/permission.rs
   - shared/dto/auth.rs
   - service/team_service.rs
   - **理由**: api/dtoからの後方互換性のための再エクスポートで使用中

2. **utils/mod.rsの再エクスポート**:
   ```rust
   pub use crate::infrastructure::email;
   pub use crate::infrastructure::jwt;
   pub use crate::infrastructure::password;
   ```
   - **理由**: 既存コードの後方互換性維持のため
   - **将来**: 全てのインポートをinfrastructure::に更新後に削除可能

3. **ビルド時間の最適化機会**:
   - 各featureモジュールを独立クレートとして分離可能
   - 並列ビルドによる高速化が期待できる
   - 増分ビルドの効率化

これらは現在のコードベースの動作に影響を与えないため、必要に応じて段階的に対応可能です。

### 🎨 Feature別統一構造実装（Phase 14以降）

**目的**: 各featureモジュールに統一的な構造を持たせ、循環依存を完全に排除し、マルチバックエンドシステム向けのクレート分割を可能にする

#### 📐 統一構造の定義（ベストプラクティス版）

**依存関係の原則**:
```
handler → service → repository → domain
   ↓         ↓          ↓          ↓
  dto    usecase      dto       (core)
```

各featureモジュールは以下の構造を持つ：
```
features/{feature_name}/
├── mod.rs           # モジュール定義と公開API
├── handlers/        # HTTPハンドラー層（複数可）
│   ├── mod.rs
│   └── *.rs         # 各ハンドラー実装
├── services/        # ビジネスロジック層（複数可）
│   ├── mod.rs
│   └── *.rs         # 各サービス実装
├── repositories/    # データアクセス層
│   ├── mod.rs
│   └── *.rs         # 各リポジトリ実装
├── dto/             # データ転送オブジェクト
│   ├── mod.rs
│   ├── requests/    # リクエストDTO
│   │   ├── mod.rs
│   │   └── *.rs
│   └── responses/   # レスポンスDTO
│       ├── mod.rs
│       └── *.rs
├── models/          # ドメインモデル（domainから変更）
│   ├── mod.rs
│   └── *.rs         # 各モデル定義
└── usecases/        # 複雑なビジネスロジック（オプション）
    ├── mod.rs
    └── *.rs         # ユースケース実装
```

**重要な変更点**:
1. 単数形から複数形へ（例: `handler` → `handlers`）- Rustの慣例に従う
2. `domain` → `models` - より明確で一般的な名称
3. `request.rs`/`response.rs` → `requests/`/`responses/` - 拡張性を考慮
4. 各層は下位層のみに依存（循環依存を防ぐ）

#### 📝 命名規則の統一

**1. Request/Response DTOの命名規則**

```rust
// ✅ 推奨される命名パターン
// requests/
CreateTeamRequest       // 作成
UpdateTeamRequest       // 更新
DeleteTeamRequest       // 削除（bodyがある場合）
ListTeamsRequest        // 一覧取得
GetTeamRequest          // 単一取得（query params）
SearchTeamsRequest      // 検索

// responses/
TeamResponse            // 単一エンティティ
TeamsResponse           // 複数エンティティ
TeamCreatedResponse     // 作成結果
TeamUpdatedResponse     // 更新結果
TeamDeletedResponse     // 削除結果
TeamStatsResponse       // 統計情報

// ❌ 避けるべき命名
TeamDto                 // 曖昧
TeamData               // 曖昧
CreateTeamDto          // DTOサフィックスは使わない
TeamResponseDto        // 二重サフィックス
```

**2. サービスメソッドの命名規則**

```rust
// ✅ 推奨される命名パターン
impl TeamService {
    // 基本CRUD
    async fn create_team(&self, request: CreateTeamRequest) -> Result<TeamResponse>
    async fn get_team(&self, team_id: Uuid) -> Result<TeamResponse>
    async fn update_team(&self, team_id: Uuid, request: UpdateTeamRequest) -> Result<TeamResponse>
    async fn delete_team(&self, team_id: Uuid) -> Result<TeamDeletedResponse>
    async fn list_teams(&self, request: ListTeamsRequest) -> Result<TeamsResponse>
    
    // ビジネスロジック
    async fn add_member(&self, team_id: Uuid, user_id: Uuid) -> Result<TeamMemberResponse>
    async fn remove_member(&self, team_id: Uuid, user_id: Uuid) -> Result<TeamMemberRemovedResponse>
    async fn transfer_ownership(&self, team_id: Uuid, new_owner_id: Uuid) -> Result<TeamResponse>
}
```

**3. リポジトリメソッドの命名規則**

```rust
// ✅ 推奨される命名パターン
impl TeamRepository {
    // 基本CRUD（DBアクセス層なのでfind/save/deleteを使用）
    async fn find_by_id(&self, team_id: Uuid) -> Result<Option<Team>>
    async fn find_all(&self, pagination: PaginationQuery) -> Result<Vec<Team>>
    async fn save(&self, team: &Team) -> Result<Team>
    async fn delete(&self, team_id: Uuid) -> Result<bool>
    
    // 特定条件での検索
    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Vec<Team>>
    async fn find_by_organization_id(&self, org_id: Uuid) -> Result<Vec<Team>>
    async fn exists_by_name(&self, name: &str, org_id: Uuid) -> Result<bool>
    async fn count_by_organization_id(&self, org_id: Uuid) -> Result<i64>
}
```

**4. モデルの命名規則**

```rust
// models/
Team                    // エンティティ
TeamMember             // 関連エンティティ
TeamRole               // 値オブジェクト
TeamStatus             // Enum
TeamPermission         // Enum

// ❌ 避けるべき命名
TeamModel              // Modelサフィックスは不要
TeamEntity             // Entityサフィックスは不要
TeamTable              // DB層の詳細を露出
```

**5. ハンドラー関数の命名規則**

```rust
// ✅ 推奨される命名パターン
pub async fn create_team_handler(/* params */) -> Result<impl IntoResponse>
pub async fn get_team_handler(/* params */) -> Result<impl IntoResponse>
pub async fn update_team_handler(/* params */) -> Result<impl IntoResponse>
pub async fn delete_team_handler(/* params */) -> Result<impl IntoResponse>
pub async fn list_teams_handler(/* params */) -> Result<impl IntoResponse>

// ルーター関数
pub fn team_routes() -> Router<AppState>
```

**6. 共通接頭辞・接尾辞のルール**

| 種別 | 接頭辞 | 接尾辞 | 例 |
|------|--------|--------|-----|
| Request DTO | {Action}{Entity} | Request | CreateTeamRequest |
| Response DTO | {Entity}{Variant}? | Response | TeamResponse, TeamCreatedResponse |
| Service | {Entity} | Service | TeamService |
| Repository | {Entity} | Repository | TeamRepository |
| Handler関数 | {action}_{entity} | _handler | create_team_handler |
| Model | - | - | Team（接尾辞なし） |
| UseCase | {BusinessAction} | UseCase | TransferOwnershipUseCase |

**7. 複数形の使用ルール**

- ディレクトリ名：複数形（handlers/, services/, models/）
- コレクションを返すメソッド：複数形（list_teams, find_teams）
- 単一エンティティを扱うメソッド：単数形（get_team, create_team）
- レスポンスDTO：単数形（TeamResponse）、複数形（TeamsResponse）

#### 🎯 Services vs UseCases: ビジネスロジックの配置指針

**1. 基本的な役割分担**

```rust
// Services: 単一エンティティに関する基本的なビジネスロジック
// - CRUD操作 + 簡単なビジネスルール
// - 1つのリポジトリを主に使用
// - エンティティ中心の操作

impl TeamService {
    // ✅ Serviceに適したロジック
    async fn create_team(&self, request: CreateTeamRequest) -> Result<TeamResponse> {
        // バリデーション
        self.validate_team_name(&request.name)?;
        
        // エンティティ作成
        let team = Team::new(request.name, request.owner_id);
        
        // 永続化
        let saved_team = self.repository.save(&team).await?;
        
        // レスポンス変換
        Ok(TeamResponse::from(saved_team))
    }
    
    async fn add_member(&self, team_id: Uuid, user_id: Uuid) -> Result<TeamMemberResponse> {
        // 単一チームへのメンバー追加
        let team = self.repository.find_by_id(team_id).await?
            .ok_or(Error::NotFound)?;
            
        team.add_member(user_id)?; // ドメインロジック
        
        self.repository.save(&team).await?;
        Ok(TeamMemberResponse::new(team_id, user_id))
    }
}

// UseCases: 複数のエンティティやサービスをまたぐ複雑なビジネスロジック
// - 複数のサービスを協調させる
// - トランザクション境界を管理
// - 複雑なビジネスフロー

pub struct TransferTeamOwnershipUseCase {
    team_service: Arc<TeamService>,
    user_service: Arc<UserService>,
    notification_service: Arc<NotificationService>,
    audit_service: Arc<AuditService>,
}

impl TransferTeamOwnershipUseCase {
    // ✅ UseCaseに適したロジック
    pub async fn execute(&self, team_id: Uuid, new_owner_id: Uuid, actor_id: Uuid) -> Result<TeamOwnershipTransferredResponse> {
        // 1. 権限チェック
        let team = self.team_service.get_team(team_id).await?;
        if team.owner_id != actor_id {
            return Err(Error::Forbidden);
        }
        
        // 2. 新オーナーの検証
        let new_owner = self.user_service.get_user(new_owner_id).await?;
        if !new_owner.is_active() {
            return Err(Error::InvalidUser);
        }
        
        // 3. 所有権の移転（トランザクション内）
        let transferred_team = self.team_service
            .transfer_ownership(team_id, new_owner_id)
            .await?;
        
        // 4. 通知送信
        self.notification_service
            .send_ownership_transfer_notification(&team, &new_owner)
            .await?;
        
        // 5. 監査ログ記録
        self.audit_service
            .log_ownership_transfer(team_id, actor_id, new_owner_id)
            .await?;
        
        Ok(TeamOwnershipTransferredResponse {
            team: transferred_team,
            previous_owner_id: team.owner_id,
            new_owner_id,
            transferred_at: Utc::now(),
        })
    }
}
```

**2. 判断基準**

| 観点 | Service | UseCase |
|------|---------|---------|
| **責務** | 単一エンティティの操作 | 複数エンティティの協調 |
| **複雑度** | シンプル〜中程度 | 複雑なビジネスフロー |
| **依存** | 1-2個のリポジトリ | 複数のサービス |
| **トランザクション** | 単一 | 複数の可能性 |
| **例** | ユーザー作成、チーム更新 | 注文処理、承認フロー |

**3. 実装パターンの選択肢**

```rust
// パターン1: Service Only（シンプルなアプリケーション）
features/team/
├── handlers/
├── services/     # すべてのビジネスロジック
├── repositories/
├── models/
└── dto/

// パターン2: Service + UseCase（中〜大規模アプリケーション）
features/team/
├── handlers/
├── services/     # 基本的なCRUD + 簡単なビジネスロジック
├── usecases/     # 複雑なビジネスフロー
├── repositories/
├── models/
└── dto/

// パターン3: Service + Domain Service（DDD寄り）
features/team/
├── handlers/
├── services/           # アプリケーションサービス
├── domain/
│   ├── models/        # エンティティ、値オブジェクト
│   └── services/      # ドメインサービス（ドメインロジック）
├── repositories/
└── dto/
```

**4. 推奨アプローチ**

1. **最初はServiceのみで開始**
   - すべてのビジネスロジックをServiceに実装
   - シンプルで理解しやすい

2. **複雑になったらUseCaseを導入**
   - Serviceが肥大化したとき
   - 複数のServiceを協調させる必要が出たとき
   - 明確なビジネスフローが識別できたとき

3. **UseCaseの命名例**
   ```rust
   // ビジネスフローを表す名前
   ProcessOrderUseCase
   ApproveDocumentUseCase
   TransferOwnershipUseCase
   GenerateMonthlyReportUseCase
   OnboardNewUserUseCase
   ```

**5. アンチパターンの回避**

```rust
// ❌ 避けるべき: ServiceがUseCaseを呼ぶ
impl TeamService {
    async fn do_something(&self) {
        self.some_usecase.execute().await // 逆依存！
    }
}

// ✅ 正しい: UseCaseがServiceを呼ぶ
impl SomeUseCase {
    async fn execute(&self) {
        self.team_service.do_something().await
    }
}

// ❌ 避けるべき: 不明確な責務
impl TeamService {
    // これはUseCase？Service？
    async fn process_team_with_notification_and_audit(&self) { }
}

// ✅ 正しい: 明確な分離
impl TeamService {
    async fn update_team(&self) { } // 基本操作
}

impl UpdateTeamWithNotificationUseCase {
    async fn execute(&self) { } // 複雑なフロー
}
```

#### 🚀 Phase 14: Team機能の完全実装

**現状**: DTOのみ存在
**目標**: 完全な機能モジュールとして再構築

##### Phase 14.1: Models層の移行（30分）
- [ ] `features/team/models/`ディレクトリを作成
- [ ] `domain/team_model.rs` → `features/team/models/team.rs`
- [ ] `domain/team_member_model.rs` → `features/team/models/team_member.rs`
- [ ] `domain/team_invitation_model.rs` → `features/team/models/team_invitation.rs`
- [ ] models/mod.rsで公開APIを定義
- [ ] 既存のdomain/からの参照を更新
- [ ] `cargo test --lib` でビルド確認

##### Phase 14.2: Repositories層の移行（30分）
- [ ] `features/team/repositories/`ディレクトリを作成
- [ ] `repository/team_repository.rs` → `features/team/repositories/team.rs`
- [ ] `repository/team_member_repository.rs` → `features/team/repositories/team_member.rs`
- [ ] `repository/team_invitation_repository.rs` → `features/team/repositories/team_invitation.rs`
- [ ] repositories/mod.rsで公開APIを定義
- [ ] modelsへのインポートパスを`super::models`に更新
- [ ] `cargo test --lib` でビルド確認

##### Phase 14.3: Services層の移行（30分）
- [ ] `features/team/services/`ディレクトリを作成
- [ ] `service/team_service.rs` → `features/team/services/team.rs`
- [ ] services/mod.rsで公開APIを定義
- [ ] repositoriesへのインポートを`super::repositories`に更新
- [ ] modelsへのインポートを`super::models`に更新
- [ ] DTOへのインポートを`super::dto`に更新（一時的に既存パス維持）
- [ ] `cargo test service::team_service` で既存テストの動作確認

##### Phase 14.4: DTOの再構成（30分）
- [ ] `features/team/dto/requests/`ディレクトリを作成
- [ ] `features/team/dto/responses/`ディレクトリを作成
- [ ] 既存のdto/team.rs, dto/team_invitation.rsを分析
- [ ] リクエストDTOをrequests/に分割配置
- [ ] レスポンスDTOをresponses/に分割配置
- [ ] dto/mod.rsで後方互換性のための再エクスポート
- [ ] `cargo clippy --all-targets` で警告なし確認

##### Phase 14.5: Handlers層の移行（30分）
- [ ] `features/team/handlers/`ディレクトリを作成
- [ ] `api/handlers/team_handler.rs` → `features/team/handlers/team.rs`
- [ ] handlers/mod.rsで公開APIを定義
- [ ] servicesへのインポートを`super::services`に更新
- [ ] DTOへのインポートを`super::dto`に更新
- [ ] `team_router_with_state`関数の動作確認
- [ ] `cargo test` で全テストがパスすることを確認

##### Phase 14.6: 最終統合とクリーンアップ（30分）
- [ ] features/team/mod.rsで全モジュールを適切に公開
- [ ] main.rsのインポートを`features::team::handlers`に更新
- [ ] 元ファイルを削除（後方互換性が不要な場合）
- [ ] または再エクスポートファイルとして維持
- [ ] `make ci-check-fast` で全テストがパス
- [ ] `cargo clippy --all-targets --all-features -- -D warnings`

#### 🏢 Phase 15: Organization機能の完全実装

**現状**: DTOのみ存在
**目標**: 階層構造を持つ組織管理機能として再構築

##### Phase 15.1: Models層の移行（30分）
- [ ] `features/organization/models/`ディレクトリを作成
- [ ] `domain/organization_model.rs` → `features/organization/models/organization.rs`
- [ ] `domain/department_model.rs` → `features/organization/models/department.rs`
- [ ] `domain/department_member_model.rs` → `features/organization/models/department_member.rs`
- [ ] models/mod.rsで公開APIを定義
- [ ] 循環依存チェック：他モデルへの参照を確認
- [ ] `cargo test --lib` でビルド確認

##### Phase 15.2: Repositories層の移行（30分）
- [ ] `features/organization/repositories/`ディレクトリを作成
- [ ] `repository/organization_repository.rs` → `features/organization/repositories/organization.rs`
- [ ] `repository/department_repository.rs` → `features/organization/repositories/department.rs`
- [ ] `repository/department_member_repository.rs` → `features/organization/repositories/department_member.rs`
- [ ] repositories/mod.rsで公開APIを定義
- [ ] modelsへのインポートパスを`super::models`に更新
- [ ] `cargo test --lib` でビルド確認

##### Phase 15.3: Services層の移行（45分）
- [ ] `features/organization/services/`ディレクトリを作成
- [ ] `service/organization_service.rs` → `features/organization/services/organization.rs`
- [ ] `service/organization_hierarchy_service.rs` → `features/organization/services/hierarchy.rs`
- [ ] services/mod.rsで公開APIを定義
- [ ] repositoriesへのインポートを`super::repositories`に更新
- [ ] modelsへのインポートを`super::models`に更新
- [ ] 外部依存（team等）の確認と整理
- [ ] `cargo test service::organization` で既存テストの動作確認

##### Phase 15.4: Usecases層の作成（30分）
- [ ] `features/organization/usecases/`ディレクトリを作成
- [ ] 階層構造操作の複雑なロジックを`hierarchy_operations.rs`に抽出
- [ ] usecases/mod.rsで公開APIを定義
- [ ] servicesから複雑なロジックを移動
- [ ] `cargo test` でテスト確認

##### Phase 15.5: DTOの再構成（30分）
- [ ] `features/organization/dto/requests/`ディレクトリを作成
- [ ] `features/organization/dto/responses/`ディレクトリを作成
- [ ] 既存のdto/organization.rs, dto/organization_hierarchy.rsを分析
- [ ] リクエストDTOをrequests/に分割配置
- [ ] レスポンスDTOをresponses/に分割配置
- [ ] 階層構造用のDTOを`responses/hierarchy.rs`に整理
- [ ] `cargo clippy --all-targets` で警告なし確認

##### Phase 15.6: Handlers層の移行と統合（45分）
- [ ] `features/organization/handlers/`ディレクトリを作成
- [ ] `api/handlers/organization_handler.rs` → `features/organization/handlers/organization.rs`
- [ ] handlers/mod.rsで公開APIを定義
- [ ] servicesへのインポートを`super::services`に更新
- [ ] usecasesへのインポートを`super::usecases`に更新
- [ ] main.rsのインポートを更新
- [ ] `make ci-check-fast` で全テストがパス

#### 🔐 Phase 16: Security機能の完全実装

**現状**: DTOのみ存在、role関連はshared/dto/role_types.rsに分離済み
**目標**: ロール・権限管理機能として再構築

##### Phase 16.1: Models層の移行（30分）
- [ ] `features/security/models/`ディレクトリを作成
- [ ] `domain/role_model.rs` → `features/security/models/role.rs`
- [ ] `domain/role_permission_model.rs` → `features/security/models/role_permission.rs`
- [ ] `domain/user_permission_model.rs` → `features/security/models/user_permission.rs`
- [ ] models/mod.rsで公開APIを定義
- [ ] shared/dto/role_types.rsへの依存を確認
- [ ] `cargo test --lib` でビルド確認

##### Phase 16.2: Repositories層の移行（30分）
- [ ] `features/security/repositories/`ディレクトリを作成
- [ ] `repository/role_repository.rs` → `features/security/repositories/role.rs`
- [ ] `repository/role_permission_repository.rs` → `features/security/repositories/role_permission.rs`
- [ ] `repository/user_permission_repository.rs` → `features/security/repositories/user_permission.rs`
- [ ] repositories/mod.rsで公開APIを定義
- [ ] modelsへのインポートパスを`super::models`に更新
- [ ] `cargo test --lib` でビルド確認

##### Phase 16.3: Services層の移行（45分）
- [ ] `features/security/services/`ディレクトリを作成
- [ ] `service/security_service.rs` → `features/security/services/security.rs`
- [ ] `service/role_service.rs` → `features/security/services/role.rs`
- [ ] `service/permission_service.rs` → `features/security/services/permission.rs`
- [ ] services/mod.rsで公開APIを定義
- [ ] shared/dto/role_typesのインポートを維持
- [ ] repositoriesへのインポートを`super::repositories`に更新
- [ ] `cargo test service::security` で既存テストの動作確認

##### Phase 16.4: Usecases層の作成（30分）
- [ ] `features/security/usecases/`ディレクトリを作成
- [ ] 権限チェックロジックを`permission_checker.rs`として抽出
- [ ] ロール階層処理を`role_hierarchy.rs`として抽出
- [ ] usecases/mod.rsで公開APIを定義
- [ ] infrastructure/utils/permissionとの連携を確認
- [ ] `cargo test` でテスト確認

##### Phase 16.5: DTOの再構成（30分）
- [ ] `features/security/dto/requests/`ディレクトリを作成
- [ ] `features/security/dto/responses/`ディレクトリを作成
- [ ] 既存のdto/security.rs, dto/permission.rsを分析
- [ ] shared/dto/role_types.rsは共通型として維持
- [ ] リクエストDTOをrequests/に配置
- [ ] レスポンスDTOをresponses/に配置
- [ ] `cargo clippy --all-targets` で警告なし確認

##### Phase 16.6: Handlers層の移行と統合（45分）
- [ ] `features/security/handlers/`ディレクトリを作成
- [ ] `api/handlers/security_handler.rs` → `features/security/handlers/security.rs`
- [ ] `api/handlers/role_handler.rs` → `features/security/handlers/role.rs`
- [ ] `api/handlers/permission_handler.rs` → `features/security/handlers/permission.rs`
- [ ] handlers/mod.rsで統合ルーターを提供
- [ ] main.rsのインポートを更新
- [ ] `make ci-check-fast` で全テストがパス

#### 👨‍💼 Phase 17: Admin機能の完全実装

**現状**: DTOのみ存在（最も複雑）、複数のサービスが分散
**目標**: 管理者向け統合機能として再構築

##### Phase 17.1: Services層の分析と移行（45分）
- [ ] `features/admin/services/`ディレクトリを作成
- [ ] `service/admin_organization_service.rs` → `features/admin/services/organization.rs`
- [ ] `service/analytics_service.rs` → `features/admin/services/analytics.rs`
- [ ] services/mod.rsで公開APIを定義
- [ ] 依存関係の分析（他featureのサービスへの依存を確認）
- [ ] 循環依存がないことを確認
- [ ] `cargo test --lib` でビルド確認

##### Phase 17.2: Repositories層の確認（15分）
- [ ] Admin専用のリポジトリが必要か確認
- [ ] 既存の他featureのリポジトリを再利用するパターンを確認
- [ ] 必要に応じて`features/admin/repositories/`を作成
- [ ] 統計情報用の専用リポジトリが必要な場合は作成

##### Phase 17.3: Usecases層の作成（45分）
- [ ] `features/admin/usecases/`ディレクトリを作成
- [ ] 組織管理操作を`organization_management.rs`に整理
- [ ] 統計・分析処理を`analytics_operations.rs`に整理
- [ ] ユーザー管理操作を`user_management.rs`に整理
- [ ] サブスクリプション管理を`subscription_management.rs`に整理
- [ ] usecases/mod.rsで公開APIを定義
- [ ] `cargo test` でテスト確認

##### Phase 17.4: DTOの整理とサブモジュール化（45分）
- [ ] `features/admin/dto/requests/`ディレクトリを作成
- [ ] `features/admin/dto/responses/`ディレクトリを作成
- [ ] 機能別サブディレクトリを作成:
  - [ ] `dto/requests/organization/`
  - [ ] `dto/requests/analytics/`
  - [ ] `dto/requests/subscription/`
  - [ ] `dto/responses/organization/`
  - [ ] `dto/responses/analytics/`
  - [ ] `dto/responses/subscription/`
- [ ] 既存のDTOを適切なサブディレクトリに配置
- [ ] admin_operations.rsのインラインDTOも整理
- [ ] `cargo clippy --all-targets` で警告なし確認

##### Phase 17.5: Handlers層の移行と統合（45分）
- [ ] `features/admin/handlers/`ディレクトリを作成
- [ ] `api/handlers/admin_handler.rs` → `features/admin/handlers/admin.rs`
- [ ] `api/handlers/analytics_handler.rs` → `features/admin/handlers/analytics.rs`
- [ ] handlers/mod.rsで統合ルーターを提供
- [ ] servicesへのインポートを`super::services`に更新
- [ ] usecasesへのインポートを`super::usecases`に更新
- [ ] 他featureへの依存を整理（features::team::services等）
- [ ] `cargo test` で既存テストの動作確認

##### Phase 17.6: 最終統合とテスト（45分）
- [ ] features/admin/mod.rsで全モジュールを適切に公開
- [ ] main.rsのインポートを`features::admin::handlers`に更新
- [ ] 管理者権限のミドルウェアとの連携確認
- [ ] 統合テストの実行と確認
- [ ] `make ci-check-fast` で全テストがパス
- [ ] `cargo clippy --all-targets --all-features -- -D warnings`

#### 💳 Phase 18: Subscription機能の完全実装

**現状**: DTOのみ存在、core/subscription_tier.rsとの連携必要
**目標**: サブスクリプション管理機能として再構築

##### Phase 18.1: Models層の移行（30分）
- [ ] `features/subscription/models/`ディレクトリを作成
- [ ] `domain/subscription_history_model.rs` → `features/subscription/models/history.rs`
- [ ] models/mod.rsで公開APIを定義
- [ ] core::subscription_tierへの依存を確認
- [ ] `cargo test --lib` でビルド確認

##### Phase 18.2: Repositories層の移行（30分）
- [ ] `features/subscription/repositories/`ディレクトリを作成
- [ ] `repository/subscription_history_repository.rs` → `features/subscription/repositories/history.rs`
- [ ] repositories/mod.rsで公開APIを定義
- [ ] modelsへのインポートパスを`super::models`に更新
- [ ] `cargo test --lib` でビルド確認

##### Phase 18.3: Services層の移行（30分）
- [ ] `features/subscription/services/`ディレクトリを作成
- [ ] `service/subscription_service.rs` → `features/subscription/services/subscription.rs`
- [ ] services/mod.rsで公開APIを定義
- [ ] repositoriesへのインポートを`super::repositories`に更新
- [ ] core::subscription_tierの使用を確認
- [ ] `cargo test service::subscription` で既存テストの動作確認

##### Phase 18.4: DTOの再構成（30分）
- [ ] `features/subscription/dto/requests/`ディレクトリを作成
- [ ] `features/subscription/dto/responses/`ディレクトリを作成
- [ ] 既存のdto/subscription.rsを分析
- [ ] リクエストDTOをrequests/に配置
- [ ] レスポンスDTOをresponses/に配置
- [ ] `cargo clippy --all-targets` で警告なし確認

##### Phase 18.5: Handlers層の移行（30分）
- [ ] `features/subscription/handlers/`ディレクトリを作成
- [ ] `api/handlers/subscription_handler.rs` → `features/subscription/handlers/subscription.rs`
- [ ] handlers/mod.rsで公開APIを定義
- [ ] servicesへのインポートを`super::services`に更新
- [ ] main.rsのインポートを更新
- [ ] `make ci-check-fast` で全テストがパス

#### 🔄 Phase 19: 残存ファイルの整理と移行

**目標**: api/, service/, repository/, domain/ディレクトリに残存するファイルを適切に移行

##### Phase 19.1: 残存ファイルの調査（30分）
- [ ] `find src/api -name "*.rs" | grep -v mod.rs` で残存ハンドラーをリスト
- [ ] `find src/service -name "*.rs" | grep -v mod.rs` で残存サービスをリスト
- [ ] `find src/repository -name "*.rs" | grep -v mod.rs` で残存リポジトリをリスト
- [ ] `find src/domain -name "*.rs" | grep -v mod.rs` で残存モデルをリスト
- [ ] 各ファイルの機能と依存関係を分析
- [ ] 移行先の決定（既存feature or 新規feature or infrastructure）

##### Phase 19.2: Health機能の移行（30分）
- [ ] `infrastructure/health/`ディレクトリを作成
- [ ] `api/handlers/health_handler.rs` → `infrastructure/health/handler.rs`
- [ ] ヘルスチェック関連のロジックを整理
- [ ] main.rsのインポートを更新
- [ ] `cargo test` でテスト確認

##### Phase 19.3: User関連の統合（30分）
- [ ] user_model.rsの移行先を決定（features/auth/models/へ）
- [ ] user関連の残存ファイルをfeatures/authに統合
- [ ] 依存関係の更新
- [ ] `cargo test` でテスト確認

##### Phase 19.4: その他の残存ファイル処理（30分）
- [ ] 汎用的なユーティリティは`shared/`へ
- [ ] インフラ系は`infrastructure/`へ
- [ ] ビジネスロジックは適切なfeatureへ
- [ ] 不要なファイルは削除
- [ ] `cargo clippy --all-targets` で警告なし確認

##### Phase 19.5: ディレクトリクリーンアップ（30分）
- [ ] 空になったディレクトリの削除
- [ ] mod.rsファイルの整理
- [ ] 不要な再エクスポートの削除
- [ ] `make ci-check-fast` で全テストがパス

#### 🏗️ Phase 20: Workspace構成への移行準備

**目標**: 各featureモジュールを独立したクレートとして分離可能にする

##### Phase 20.1: 依存関係の分析（45分）
- [ ] 各featureモジュールの外部依存をリストアップ
- [ ] feature間の依存関係をグラフ化
- [ ] 循環依存がないことを確認
- [ ] 共通依存の最小化案を作成

##### Phase 20.2: インターフェース定義（45分）
- [ ] 各featureの公開APIを明確化
- [ ] trait定義による抽象化の検討
- [ ] featureプラグインシステムの設計
- [ ] 依存注入パターンの適用箇所を特定

##### Phase 20.3: Cargo.toml案の作成（45分）
- [ ] ワークスペースレベルのCargo.toml案
- [ ] 各featureクレートのCargo.toml案
- [ ] 共通依存の管理方法を決定
- [ ] ビルド最適化設定の検討

##### Phase 20.4: 移行計画の策定（45分）
- [ ] 段階的移行のロードマップ作成
- [ ] 各段階でのテスト計画
- [ ] ロールバック計画
- [ ] ドキュメント更新計画

**ワークスペース構造案**:
```
rust-backend-api/
├── Cargo.toml           # ワークスペース定義
├── crates/
│   ├── shared/          # 共通型・ユーティリティ
│   ├── core/            # コアドメイン
│   ├── infrastructure/  # インフラ層
│   ├── feature-auth/    # 認証機能
│   ├── feature-task/    # タスク管理
│   ├── feature-team/    # チーム管理
│   ├── feature-org/     # 組織管理
│   ├── feature-storage/ # ストレージ
│   ├── feature-gdpr/    # GDPR
│   ├── feature-security/# セキュリティ
│   ├── feature-admin/   # 管理者
│   └── feature-subscription/ # サブスク
└── apps/
    ├── api-server/      # メインAPIサーバー
    └── worker/          # バックグラウンドワーカー（将来）
```

#### 📊 実装効果の測定

**Phase 14-20完了後の期待効果**:

1. **ビルド時間の短縮**
   - 現在: 全体ビルド（推定5-10分）
   - 目標: 変更されたクレートのみビルド（30秒-2分）

2. **開発効率の向上**
   - 機能別の独立開発が可能
   - チーム間の作業競合を最小化
   - テストの並列実行

3. **保守性の向上**
   - 明確な責任境界
   - 依存関係の可視化
   - 機能の追加・削除が容易

4. **マルチバックエンドへの対応**
   - 機能の組み合わせで異なるAPIサーバーを構築
   - マイクロサービス化への移行パスを確保
   - 特定機能のみのデプロイが可能

#### 🎯 移行戦略の原則

1. **後方互換性の維持**
   - 既存のインポートが動作し続けるよう、段階的に移行
   - 一度に全てを変更せず、小さなステップで実施

2. **テスト駆動での移行**
   - 各変更前後でテストスイートが通ることを確認
   - 新しいインポートパスでのテストを先に作成

3. **影響範囲の最小化**
   - 一度に1つのモジュールのみを変更
   - 依存関係の少ないものから着手

4. **ドキュメント化**
   - 各Phaseの実施内容と結果を記録
   - 新しいモジュール構造の使用方法を文書化

#### 🚫 循環依存を防ぐための設計原則

1. **レイヤー間の依存方向**
   ```
   handlers → services → repositories → models
      ↓          ↓           ↓            ↓
     dto     usecases      dto         core
   ```
   - 上位層は下位層に依存（逆は禁止）
   - 同一層内での相互依存も避ける

2. **Feature間の依存関係**
   - 直接的な相互依存は禁止
   - 共通機能は`shared/`または`core/`に抽出
   - インターフェース（trait）による疎結合化

3. **DTO設計の原則**
   - DTOはその機能内で完結（他featureのDTOを参照しない）
   - 共通型は`shared/types/`に配置
   - Service層からDTO層への逆依存は絶対禁止

4. **依存関係のチェック方法**
   ```bash
   # 各サブフェーズ完了時に実行
   cargo test --lib
   cargo clippy --all-targets
   
   # 循環依存の確認
   cargo deps --all-features | grep -E "cycle|circular"
   ```

5. **問題が発生した場合の対処**
   - 共通型の抽出：`shared/types/`へ
   - インターフェースの導入：trait定義
   - イベント駆動：直接呼び出しを避ける
   - 依存性注入：コンストラクタでの注入

#### 🛡️ リファクタリング時のリスク軽減方針

**1. Feature間の相互依存への対処**

```rust
// ❌ 避けるべき: 直接的な相互依存
// features/team/services/team.rs
use crate::features::organization::services::OrganizationService;

// ✅ 推奨: インターフェース経由
// shared/traits/organization.rs
pub trait OrganizationProvider {
    async fn get_organization(&self, id: Uuid) -> Result<Organization>;
    async fn validate_membership(&self, org_id: Uuid, user_id: Uuid) -> Result<bool>;
}

// features/team/services/team.rs
pub struct TeamService<O: OrganizationProvider> {
    organization_provider: Arc<O>,
}
```

**依存関係の優先順位**:
1. **Phase 14-18の実装順序**:
   ```
   Organization → Team → Security → Admin → Subscription
   （依存される側から実装）
   ```

2. **共通インターフェースの事前定義**:
   - Phase 14開始前に`shared/traits/`を作成
   - 各featureが必要とする最小限のインターフェースを定義
   - 実装時はインターフェース経由で依存

**2. 共通機能の抽出タイミング**

```rust
// shared/へ移動する判断基準
// 1. 2つ以上のfeatureから参照される
// 2. ビジネスロジックを含まない
// 3. 純粋な型定義またはユーティリティ

// ✅ shared/に配置すべき例
pub struct Pagination { ... }           // 汎用的な型
pub trait Auditable { ... }            // 共通trait
pub fn validate_email(email: &str) { } // 汎用的なバリデーション

// ❌ shared/に配置すべきでない例
pub struct TeamMemberRole { ... }      // Team固有の型
pub fn calculate_subscription_fee() {} // ビジネスロジック
```

**抽出のタイミング**:
- **即座に抽出**: 明らかに汎用的な型（Pagination, Result型など）
- **2つ目の使用時**: 最初は各feature内、2つ目のfeatureが必要としたら抽出
- **Phase 19で一括整理**: 残存ファイル整理時に最終判断

**3. ビルド時間増加への対処**

**並列ビルド戦略**:
```bash
# 1. 変更したfeatureのみをテスト（開発中）
cargo test -p feature_team

# 2. 関連featureも含めてテスト（サブフェーズ完了時）
cargo test -p feature_team -p feature_organization

# 3. 全体テスト（Phase完了時のみ）
make ci-check-fast
```

**増分ビルドの最適化**:
```toml
# .cargo/config.toml
[build]
incremental = true

[profile.dev]
split-debuginfo = "unpacked"
opt-level = 0

[profile.test]
incremental = true
```

**ビルドキャッシュの活用**:
- sccache導入の検討
- CI/CDでのキャッシュ戦略
- 開発環境でのtarget/ディレクトリ管理

**4. 段階的移行のチェックポイント**

各サブフェーズで必ず確認:
- [ ] `cargo check` - コンパイルエラーなし
- [ ] `cargo test --lib` - ユニットテストパス
- [ ] `cargo clippy` - 警告なし
- [ ] 関連featureのテスト - 影響範囲の確認

Phase完了時に確認:
- [ ] `make ci-check-fast` - 全テストパス
- [ ] ビルド時間の計測と記録
- [ ] 依存関係グラフの更新
- [ ] ドキュメントの更新

**5. 緊急時の対処**

**ビルドが通らない場合**:
1. 直前のサブフェーズにrevert
2. 問題の原因を特定
3. 小さい単位で再実施

**テストが大量に失敗する場合**:
1. 失敗の共通原因を特定
2. 最も影響の大きい箇所から修正
3. 必要なら一時的にテストをskip（後で必ず修正）

**循環依存が発生した場合**:
1. `cargo deps --all-features`で依存関係を可視化
2. 共通部分を`shared/`に抽出
3. trait経由の依存に変更

#### 🚧 モジュール移行時の共通課題と対処方針

**モジュール参照問題について**:
- 各Phaseで新しいモジュール構造を作成する際、`crate::新モジュール`のパス解決で問題が発生する可能性がある
- **対処方針**: 以下の優先順位で進める
  1. **構造優先**: まず新しいディレクトリ構造とファイルを作成
  2. **動作維持**: 既存の場所に実装を残し、ビルド・テストが通る状態を維持
  3. **後日統合**: 全Phase完了後、モジュール参照問題をまとめて解決

**Phase 1での具体例**:
- ✅ `shared/types`ディレクトリとファイルは作成済み
- ✅ `api/dto/common.rs`に型定義を残してビルドを通している
- 🔄 将来的に`crate::shared::types`から再エクスポートする形に移行予定

**Phase 2での具体例**:
- ✅ `core`ディレクトリとファイルは作成済み
- ✅ すべてのインポートをdomain::からcore::に更新済み
- 🔄 `shared/types`モジュールの未使用警告を`#[allow(dead_code)]`で一時的に抑制
- 🔄 Phase 3でDTOを移行する際に、shared/typesの活用と警告解除を予定

**Phase 3での具体例**:
- ✅ `shared/dto`ディレクトリとファイルは作成済み
- ✅ auth_dto.rs, user_dto.rsをコピーし、元ファイルは再エクスポート形式に変更
- ✅ auth_service.rs, user_service.rsのインポートをshared::dtoに更新済み
- 🔄 以下のDTOは未移行（Service層で使用されているが、api::dtoに残っている）:
  - `task_dto.rs` - task_serviceで使用
  - `team_dto.rs` - team_serviceで使用（ワイルドカードインポート）
  - `organization_dto.rs` - organization_serviceで使用（ワイルドカードインポート）
  - `gdpr_dto.rs` - gdpr_serviceで使用（ワイルドカードインポート）
  - `security_dto.rs` - security_serviceで使用（ワイルドカードインポート）
  - `attachment_dto.rs` - attachment_serviceで使用（AttachmentSortBy, SortOrder）
- 🔄 `PaginationMeta`の重複問題:
  - `api::dto::common::PaginationMeta`がtask_serviceで使用中
  - `shared::types::pagination::PaginationMeta`が未使用（dead_code警告）
- 🔄 循環依存の問題:
  - `role_dto.rs`が`role_service.rs`から型をインポート（逆方向の依存）

**Phase 4での具体例**:
- ✅ `infrastructure`ディレクトリとファイルは作成済み
- ✅ jwt.rs, email.rs, password.rsをinfrastructure/配下に移動
- ✅ utils/mod.rsで再エクスポート設定（後方互換性維持）
- ✅ main.rs, lib.rsにinfrastructureモジュールを追加
- 🔄 `middleware/auth.rs`のインポートパス:
  - 現在: `use crate::utils::jwt::JwtManager;`（再エクスポート経由で動作）
  - 将来: `use crate::infrastructure::jwt::JwtManager;`への更新を検討
  - 現状のままでも動作に問題なし（Phase 7で認証機能整理時に一括更新予定）

**Phase 5での具体例**:
- ✅ `features/gdpr`ディレクトリとファイルは作成済み
- ✅ handler, service, dtoを集約
- ✅ 既存のファイルを再エクスポート形式に変更
- ✅ featuresモジュールをmain.rs, lib.rsに追加
- 🔄 他のhandler/serviceがGDPR機能を使用している場合:
  - 現状は既存パス（api::handlers::gdpr_handler等）の再エクスポートで対応
  - 将来的にfeatures::gdpr::handlerへの直接参照への更新を検討

**Phase 6での具体例**:
- ✅ `features/storage`ディレクトリとファイルは作成済み
- ✅ attachment関連のhandler, service, dtoを集約
- ✅ storage_service.rsをfeatures/storage/service.rsに移動
- ✅ attachment_repository.rs, attachment_share_link_repository.rsを移動
- ✅ image_optimizer.rsをinfrastructure/utils/に移動し、utilsから再エクスポート
- 🔄 再エクスポートの未使用警告:
  - `#[allow(unused_imports)]`で一時的に抑制
  - api::dto::attachment_dto, api::handlers::attachment_handler等から再エクスポート
  - service::storage_service, service::attachment_service等から再エクスポート
  - repository::attachment_repository, repository::attachment_share_link_repository等から再エクスポート
- 🔄 mock_storage.rsは`tests/common/`に残留（テスト用のため現状維持で問題なし）

**Phase 7での具体例**:
- ✅ `features/auth`ディレクトリとファイルは作成済み
- ✅ auth関連の全コンポーネントを集約:
  - handler.rs（api/handlers/auth_handler.rsから）
  - service.rs（service/auth_service.rsから）
  - dto.rs（shared/dto/auth.rsから）
  - middleware.rs（middleware/auth.rsから）
  - 5つのリポジトリ（user, user_settings, refresh_token, password_reset_token, email_verification_token）
- ✅ permission.rsをinfrastructure/utils/に移動（middlewareで使用）
- ✅ 既存のファイルを再エクスポート形式に変更（後方互換性維持）
- ✅ 移動したファイル内のインポートパス更新:
  - `crate::utils::jwt` → `crate::infrastructure::jwt`
  - `crate::utils::email` → `crate::infrastructure::email`
  - `crate::utils::password` → `crate::infrastructure::password`
  - `crate::utils::permission` → `crate::infrastructure::utils::permission`
- ✅ api/dto/auth_dto.rsの再エクスポートをfeatures/auth/dtoへの直接参照に変更
- 🔄 現状は全て再エクスポート経由で動作しており、エラーや警告なし

**Phase 8での具体例**:
- ✅ `features/task`ディレクトリとファイルは作成済み
- ✅ task関連の全コンポーネントを集約:
  - handler.rs（api/handlers/task_handler.rsから）
  - service.rs（service/task_service.rsから）
  - dto.rs（api/dto/task_dto.rsから）
  - domain/task_model.rs（domain/task_model.rsから）
  - domain/task_attachment_model.rs（domain/task_attachment_model.rsから）
  - repository/task_repository.rs（repository/task_repository.rsから）
- ✅ 既存のファイルを再エクスポート形式に変更（後方互換性維持）
- ✅ 時間計算メソッドの修正:
  - `num_hours()` → `num_seconds() / 3600.0`に変更
  - `num_days()` → `num_seconds() / 86400.0`に変更
- ✅ cargo clippy --all-targets --all-features -- -D warningsでエラーなし確認
- **完了**: 既存のインポートパスを維持しながら、ファイルをfeaturesに移動
- **残課題なし**: 全ての移行が完了し、cargo clippyでエラーなし確認済み

**Phase 9.1での具体例**:
- ✅ GDPRモジュールの再エクスポートパターンを完全に解消
- ✅ 影響範囲が最小（main.rsとテストヘルパーのみ）で安全に移行完了
- ✅ 削除したファイル:
  - api/handlers/gdpr_handler.rs（再エクスポートのみ）
  - api/dto/gdpr_dto.rs（再エクスポートのみ）
  - service/gdpr_service.rs（再エクスポートのみ）
- ✅ インポートパス変更:
  - main.rs: `api::handlers::gdpr_handler` → `features::gdpr::handler`
  - tests/common/app_helper.rs: 同様の変更を2箇所
- **完了**: 最初の再エクスポート解消として成功、他のモジュールの参考となる実装

**Phase 9.2: Storageモジュールの再エクスポート解消**（2025-07-09 完了）
- ✅ main.rsのインポートを`features::storage::attachment::handler::attachment_routes`に更新
- ✅ 再エクスポートファイルを削除:
  - api/handlers/attachment_handler.rs
  - api/dto/attachment_dto.rs
  - service/storage_service.rs
  - service/attachment_service.rs
  - repository/attachment_repository.rs
  - repository/attachment_share_link_repository.rs
- ✅ mod.rsファイルからStorageモジュール宣言を削除
- ✅ インポートパス変更:
  - main.rs: `api::handlers::attachment_handler` → `features::storage::attachment::handler`
  - main.rs: `service::storage_service` → `features::storage::service`
  - main.rs: `service::attachment_service` → `features::storage::attachment::service`
  - tests/common/app_helper.rs: 同様の変更を複数箇所
  - tests/common/mock_storage.rs: `service::storage_service` → `features::storage::service`
- ✅ features/storage/attachment/service.rs内のインポートパス修正:
  - `repository::attachment_repository` → `features::storage::repository::attachment_repository`
  - `repository::attachment_share_link_repository` → `features::storage::repository::attachment_share_link_repository`
- ✅ cargo clippy --all-targets --all-features -- -D warningsでエラーなし確認
- **完了**: Storageモジュールは完全にfeatures::storageから直接インポートされるように変更
- **残課題なし**: 全ての移行が完了し、既存テストも動作することを確認

**各Phase実施時の注意**:
```
1. 新モジュール構造を作成
2. 既存コードはそのまま維持（ビルドが通る状態を保つ）
3. 「TODO: Phase X完了後にモジュール参照を修正」とコメント追加
4. CLAUDE.mdの各Phaseに残課題として記録
```

## 🧩 実装ガイドライン

### 1. **ドメイン統合の原則**

* **既存ドメインとの重複・競合は禁止**
  * 同じ意味の別表現、似たが異なるロジック、バリエーション増加は避ける
  * APIのスラグなど機能別の統一感を意識
  * パスパラメータは `{param}` 形式を使用（Axum 0.8の仕様）
  * APIのスラグなどルーティングの重複排除＆集約統合を意識
* **「亜種」API・ドメイン定義の増加は避ける**
  * 新規定義が必要な場合は、**既存の責務・境界に統合**できるか再検討

### 2. **機能追加の原則：実用的で価値の高い機能に集中**

* **新機能の採用基準**
  * **実用性**: 実際のユーザーニーズに基づいているか
  * **価値**: 実装コストに見合う価値を提供するか
  * **保守性**: 長期的な保守が可能か
  * **既存機能との整合性**: 既存のアーキテクチャと調和するか

* **機能の優先順位付け**
  * **高優先度**: 直接的なビジネス価値、ユーザー体験の大幅改善
  * **中優先度**: 運用効率化、パフォーマンス改善
  * **低優先度**: Nice to have、将来的な拡張性のみ

* **実装を見送る判断基準**
  * クライアント側で効率的に実装可能な機能
  * 既存の外部サービス/ツールで代替可能な機能
  * 複雑性に対して得られる価値が低い機能
  * 別システムとして独立実装した方が柔軟性が高い機能

* **例：ファイルアップロード機能の判断**
  * ✅ 採用: 署名付きURL（サーバー負荷軽減、セキュリティ向上）
  * ✅ 採用: 自動圧縮（ストレージコスト削減、実用的価値）
  * ❌ 見送り: サムネイル生成（クライアント側実装が効率的）
  * ❌ 見送り: ウイルススキャン（専用システムとして実装すべき）

### 3. **データベース設計の原則**

* **テーブル名は必ず複数形**
  * `users`, `tasks`, `teams`, `organizations` など
  * ジャンクションテーブルも複数形: `team_members`, `department_members`
* **カラム名は snake_case**
  * 外部キーは `{参照テーブル単数形}_id` 形式: `user_id`, `team_id`
* **標準カラム**
  * すべてのテーブルに `id` (UUID型), `created_at`, `updated_at` を含める
  * タイムスタンプは必ず `TIMESTAMPTZ` 型を使用
* **インデックス設計**
  * 外部キー、頻繁に検索される項目には必ずインデックスを作成
  * 複合インデックスは順序を考慮して設計
* **マイグレーションファイル命名規則**
  * 形式: `m{YYYYMMDD}_{連番6桁}_{説明}.rs`
  * 連番は既存の最後のマイグレーションファイルの次の番号を使用
  * 例: 最後が `m20250704_180001_` なら次は `m20250704_180002_`
  * 日付をまたぐ場合は `000001` から開始

#### プレースホルダー実装の正規化におけるDB設計変更

* **簡易実装・プレースホルダー実装を正規実装に置き換える際は、必要に応じてマイグレーションによるDB設計の変更も行う**
  * ハードコードされた値を実データから計算するために必要なカラムを追加
  * 分析や集計を高速化するためのインデックスやサマリーテーブルを追加
  * 既存テーブルに不足しているカラム情報があれば追加

### 4. **dead\_code ポリシー**

* `#![allow(dead_code)]` や `#[allow(dead_code)]` の**新規追加は禁止**
* **既存アノテーションからAPIとして価値提供できる場合は積極的に外す**
  * 新規APIには統合テストを実施
    ```rust
    // 必須: 3パターンのテスト
    #[tokio::test]
    async fn test_feature_success() { /* 正常系 */ }

    #[tokio::test]
    async fn test_feature_invalid_data() { /* 異常系 */ }

    #[tokio::test]
    async fn test_feature_forbidden() { /* 権限エラー */ }
    ```
  * 必要に応じてマイグレーションによるDB設計も考慮
* **未使用コード・シグネチャ・構造体は削除**
  * ただし、テストで使用されているコードは、実装で適切に活用する
* **例外: テスト用ヘルパー関数**
  * テスト用については `#[allow(dead_code)]` を許可
    * `AppConfig::for_testing`
    * `setup_test_app`
    * `TestDatabase::_container`

### 5. **プロダクションコードの品質基準**

* **すべての公開APIは実装で使用される**こと
* **テストは実装の動作を検証**するものであること
* **未使用の警告が出ないこと**（dead_code警告を含む）

### 6. **APIセキュリティとルーティング規則**

* **管理者専用APIの原則**
  * システム情報、設定情報、統計情報などのセンシティブな情報を提供するAPIは **必ず管理者専用** にする
  * 例: システム情報 (`/admin/system/info`)、決済設定 (`/admin/payments/config`)
  * **任意のユーザーからアクセス可能にするとセキュリティリスクとなる**

* **APIルーティングの統一規則**
  * **`/api/` プレフィックスは使用しない**
  * 各APIは機能に応じた適切なプレフィックスを使用:
    * `/admin/*` - 管理者専用機能
    * `/auth/*` - 認証関連
    * `/tasks/*` - タスク管理
    * `/teams/*` - チーム管理
    * `/payments/*` - 決済関連（ユーザー向け）
    * `/organizations/*` - 組織管理
  * パスパラメータは `{param}` 形式を使用（Axum 0.8の仕様）

* **認証・認可の設定**
  * `skip_auth_paths` - 認証不要のパス（公開エンドポイント）
  * `admin_only_paths` - 管理者権限が必要なパス（`/admin` で統一）
  * 新規APIを追加する際は、適切な認証・認可レベルを必ず設定する

* **CORS設定**
  * 環境変数 `CORS_ALLOWED_ORIGINS` で許可するオリジンを設定可能
  * デフォルトは `FRONTEND_URL` の値を使用、それもなければ `http://localhost:3000`
  * 本番環境では必ず具体的なオリジンを指定し、ワイルドカード（`*`）は使用しない

### 7. **CI・Lint 要件**

* 以下のコマンドで **エラー・警告が完全にゼロ** であること：

  ```bash
  cargo clippy --all-targets --all-features -- -D warnings
  ```

* 既存CI（テスト）コマンド：

  ```bash
  make ci-check-fast
  ```

  → **すべてのテストにパスすること（新旧含む）**

---

## 🧪 テスト要件

### 単体テスト（Unit Test）

* **新規ロジックに対する細粒度のテストを実装**

  * 条件分岐、バリデーション、エラーケースなどを網羅
  * 概念テスト・型だけのテストは不可

### 統合テスト（Integration Test）

#### **基本要件**

* APIレベルでの**E2Eフロー確認**
  * リクエスト／レスポンス構造の妥当性
  * DB書き込み・読み出しの整合性
  * エラーハンドリングの検証

#### **AAA（Arrange-Act-Assert）パターンによる実装**

バックエンド統合テストでは、AAAパターンを採用し、各テストを以下の3つのフェーズで構成する：

```rust
#[tokio::test]
async fn test_example_feature() {
    // Arrange（準備）: テストの前提条件を設定
    let (app, _schema, _db) = setup_full_app().await;
    let user = create_and_authenticate_user(&app).await;
    let initial_data = create_test_data();
    
    // Act（実行）: テスト対象の操作を実行
    let response = app.oneshot(
        create_request("POST", "/api/endpoint", &user.token, &initial_data)
    ).await.unwrap();
    
    // Assert（検証）: 期待される結果を確認
    assert_eq!(response.status(), StatusCode::OK);
    verify_database_state(&db, &expected_state).await;
    verify_side_effects(&app).await;
}
```

#### **テスト設計の必須要素**

1. **Arrange（準備）フェーズ**
   - 実際のデータを作成（モックやハードコードされた値を避ける）
   - 必要な前提条件をすべて満たす
   - テスト環境の初期状態を明確に定義

2. **Act（実行）フェーズ**
   - 実際のユーザー操作を再現
   - 1つのテストにつき1つの主要なアクションに焦点を当てる
   - APIエンドポイントへの実際のHTTPリクエストを実行

3. **Assert（検証）フェーズ**
   - レスポンスのステータスコードと本文を検証
   - データベースの状態変更を確認
   - 副作用（ログ、通知、関連データの更新）を検証
   - エラーケースではエラーメッセージの内容も確認

#### **統合テストのベストプラクティス**

1. **独立性の確保**
   ```rust
   // 各テストは独立したスキーマで実行され、他のテストに影響しない
   let (app, schema_name, db) = setup_full_app().await;
   // テスト終了時に自動的にクリーンアップ
   ```

2. **実データによる検証**
   ```rust
   // ❌ 避けるべき例
   assert_eq!(response["deleted_count"], 0); // 常に0を期待
   
   // ✅ 推奨される例
   // 実際にデータを作成
   create_test_records(&db, 5).await;
   // 削除操作を実行
   let response = delete_old_records(&app).await;
   // 実際の削除数を検証
   assert_eq!(response["deleted_count"], 5);
   ```

3. **時間依存テストの扱い**
   ```rust
   // 時間を操作可能にする
   let old_data = create_data_with_timestamp(
       Utc::now() - Duration::days(91)
   ).await;
   let recent_data = create_data_with_timestamp(
       Utc::now() - Duration::days(30)
   ).await;
   
   // 90日以上古いデータの削除をテスト
   let result = cleanup_old_data(&app, 90).await;
   assert_eq!(result.deleted_count, 1);
   ```

4. **エラーパスの網羅**
   ```rust
   // 各APIエンドポイントに対して最低限以下のケースをテスト
   test_endpoint_success()           // 正常系
   test_endpoint_validation_error()  // バリデーションエラー
   test_endpoint_unauthorized()      // 認証エラー
   test_endpoint_forbidden()         // 認可エラー
   test_endpoint_not_found()         // リソース不在
   ```

#### **アンチパターンと回避策**

| アンチパターン | 問題点 | 改善策 |
|--------------|--------|--------|
| 構造のみの検証 | `assert!(response["data"].is_object())` | 実際の値も検証: `assert_eq!(response["data"]["count"], 10)` |
| 固定値への依存 | モックが常に同じ値を返す | 実データを作成して動的に検証 |
| 副作用の未検証 | APIレスポンスのみ確認 | DB状態、ログ、関連データも確認 |
| テスト間の依存 | 実行順序により結果が変わる | 各テストで必要なデータを準備 |

#### **テスト完全性チェックリスト**

統合テスト実装時の確認事項：
- [ ] 実際のユーザーシナリオを再現しているか
- [ ] データは動的に作成されているか（ハードコード値を避ける）
- [ ] レスポンスの値まで検証しているか（構造だけでなく）
- [ ] データベースの変更を確認しているか
- [ ] エラーケースを網羅しているか（最低5パターン）
- [ ] テストが独立して実行可能か
- [ ] クリーンアップが適切に行われるか

#### **構造だけの空テストの削除**

以下のような構造のみを検証し、実際の値を確認しないテストは削除すること：

```rust
// ❌ 削除対象の例
assert!(response["data"].is_object());
assert!(response["items"].is_array());
assert!(response["count"].is_number());

// ✅ 代わりに実際の値を検証
assert_eq!(response["data"]["user_id"], user.id);
assert_eq!(response["items"].as_array().unwrap().len(), 5);
assert_eq!(response["count"], 10);
```

* **構造だけのテストは実装の正しさを保証しない**
* **必ず実際の値まで検証すること**
* **動的に作成したデータと結果を比較すること**

---

## 🔥 クリーンアップ方針

* **使用されていないコード**の取り扱い：
  1. **テストでのみ使用** → 実装で活用するよう統合
  2. **どこでも未使用** → 削除
  3. **将来の拡張用** → 削除（YAGNI原則：You Aren't Gonna Need It）

* dead\_code で検知される要素への対応：
  * **公開API（pub）** → 実装での活用を検討
  * **内部実装（非pub）** → 使用されていなければ削除
  * **テスト用ユーティリティ** → そのまま維持

* **重要**: 「将来のために」という理由でコードを残さない
  * 必要になったときに実装する
  * 未使用のエラータイプ、構造体、関数は削除
  * コメントアウトされたコードは削除

---

## 🚀 実装完了後の期待される状態

1. **`cargo clippy`で警告ゼロ**（dead_code警告を含む）
2. **`make ci-check-fast`ですべてのテストがグリーン**
3. **APIドキュメントと実装が一致**
4. **テストが実装の実際の動作を検証**
5. **プロダクションコードがクリーンで保守しやすい**
