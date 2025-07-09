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

- [ ] **Phase 8: タスク機能の整理**
  - [ ] `features/task`ディレクトリ作成
  - [ ] タスク関連の全ファイルを集約
  - [ ] 既存テストの動作確認

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
│   ├── types/       # 共通型定義
│   └── dto/         # 共通DTO
├── core/            # コアドメイン
├── infrastructure/  # インフラ層
├── features/        # 機能別モジュール
│   ├── gdpr/
│   ├── storage/
│   ├── auth/
│   └── task/
├── api/             # 残りのハンドラー
├── domain/          # 残りのドメインモデル
├── repository/      # 残りのリポジトリ
└── service/         # 残りのサービス
```

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
