## 実現トピック

- [マルチテナント機能要件定義](./マルチテナント機能要件定義.md) - チーム・組織単位でのデータ共有・操作機能の実装

### マルチテナント機能実装タスクリスト（実装ガイドライン準拠）

#### フェーズ1: 統一権限チェックミドルウェアの実装
- [x] **リソース名定数とアクション定義の統一化**
   - [x] `src/middleware/authorization.rs` に `RequirePermission` ミドルウェア実装
   - [x] リソース名定数（`resources::TASK`, `resources::TEAM` 等）の定義
   - [x] 権限チェックマクロ `require_permission!` の実装
   - [x] エラーコンテキスト命名規則（`"モジュール名::関数名"`）の徹底

- [x] **既存APIへの統一権限チェック適用と統廃合（リファレンス実装）**
   - [x] 各ハンドラーでの直接ロールチェックを `RequirePermission` ミドルウェアに置換（サンプル）
   - [x] error_helper関数の一貫した使用（`internal_server_error`, `not_found_error`, `conflict_error`）
   - [x] 構造化ログによるエラー追跡の実装
   - [x] リファレンス実装（task_handler_v2.rs）は参考実装として保持
   - 📝 **注**: 既存APIへの実際の統合は、フェーズ2-3の実装と合わせて段階的に実施予定
   - 📝 既存のPermissionServiceから統一権限チェックミドルウェアへの移行は慎重に実施

#### フェーズ2: データベーススキーマ拡張
- [x] **tasksテーブルのマルチテナント対応**
   - [x] マイグレーション作成: `m20250719_000001_add_multitenant_fields_to_tasks.rs`
   - [x] 追加カラム: `team_id` (UUID, nullable), `organization_id` (UUID, nullable), `visibility` (ENUM: personal/team/organization), `assigned_to` (UUID, nullable)
   - [x] 外部キー制約とインデックス設計（`team_id`, `organization_id`, 複合インデックス）
   - [x] 既存データのデフォルト値設定（`visibility = 'personal'`）
   - [x] PostgreSQL enum型 `task_visibility` の作成

- [x] **タスクモデルの拡張**
   - [x] `TaskVisibility` enum実装（Personal, Team, Organization）をdomain/task_visibility.rsに作成
   - [x] task_model.rsにマルチテナントフィールドを追加（team_id, organization_id, visibility, assigned_to）
   - [x] ヘルパーメソッド実装（is_owned_by, belongs_to_team, is_accessible_by等）
   - [x] ActiveModel用ヘルパーメソッド実装（set_as_team_task, assign_to等）
   - [x] TaskDtoをマルチテナント対応に拡張
   - [x] team_task_dto.rsにチーム/組織タスク用DTOを作成
   - [x] TaskSearchQueryにマルチテナントフィルタを追加
   - [x] Timestamp型による日時フィールドのUnix timestamp対応（既存実装を活用）
   - [x] 既存の個人タスクとの後方互換性保証（デフォルトvisibility=Personal）

#### フェーズ3: API実装（ドメイン統合原則準拠）
- [x] **タスクサービス層の拡張**
   - [x] `get_tasks_with_scope` メソッド実装（スコープ: personal/team/organization）
   - [x] 権限に基づくフィルタリングロジック（TeamService活用）
   - [x] error_helperによる一貫したエラーハンドリング
   - [x] Repository層でOption<Uuid>対応（organization_idのnullable対応）

- [x] **チームタスクCRUD API実装**
   - [x] `/teams/{team_id}/tasks` エンドポイント（パスパラメータは `{param}` 形式）
   - [x] 作成: `POST /teams/{team_id}/tasks`
   - [x] 更新: `PATCH /tasks/{id}/multi-tenant`
   - [x] 削除: `DELETE /tasks/{id}/multi-tenant`
   - [x] 権限チェック: チームメンバーシップベースの制御

- [x] **タスク一覧APIのスコープ対応**
   - [x] `GET /tasks/scoped?visibility={personal|team|organization}&team_id={id}`
   - [x] 複数チーム所属時の適切なフィルタリング
   - [x] ページネーション対応（既存実装との統合）
   - [x] レスポンスのUnix timestamp形式統一

- [x] **タスク割り当てAPI**
   - [x] `POST /tasks/{id}/assign` - メンバーへの割り当て
   - [x] 権限チェック: タスクへのアクセス権限確認
   - [ ] `POST /tasks/{id}/transfer` - タスクの引き継ぎ（未実装）
   - [ ] 監査ログの記録（未実装）

#### フェーズ4: 統合テスト実装（AAA パターン準拠）
- [x] **チームタスク基本機能テスト**
   - [x] `test_create_team_task_success` - 正常系
   - [x] `test_create_team_task_non_member_forbidden` - 権限エラー（非メンバー）
   - [x] `test_update_team_task_success` - 更新正常系
   - [x] `test_update_team_task_non_member_forbidden` - 更新権限エラー
   - [x] `test_delete_team_task_success` - 削除正常系
   - [x] `test_delete_team_task_non_member_forbidden` - 削除権限エラー

- [x] **スコープベースアクセステスト**
   - [x] `test_list_tasks_with_scope_filter` - スコープベースのフィルタリング
   - [x] 個人スコープ: 自分のタスクのみ取得
   - [x] チームスコープ: 所属チームのタスク取得
   - [x] 権限エラー: 非メンバーはチームタスクにアクセス不可

- [x] **権限ベースアクセステスト**
   - [x] `test_assign_task_within_team` - チーム内タスク割り当て
   - [x] メンバー: 作成・閲覧・更新・削除可能
   - [x] 非メンバー: アクセス拒否（403 Forbidden）

- [x] **データ分離テスト**
   - [x] `test_team_data_isolation` - チーム間のデータ分離確認
   - [x] チームAのメンバーはチームBのタスクにアクセス不可

#### パフォーマンス・セキュリティ対応
- [x] **パフォーマンステスト実装**
   - [x] `test_large_scale_team_task_creation_performance` - 100タスク作成
   - [x] `test_large_scale_team_task_query_performance` - クエリ性能測定
   - [x] `test_concurrent_team_member_access_performance` - 同時アクセステスト
   - [ ] インデックス効果の検証（未実装）

- [ ] **監査ログ実装**
   - [ ] アクセスログの構造化記録
   - [ ] 権限変更の追跡
   - [ ] セキュリティイベントの記録

#### 品質保証
- [x] **dead_code対応**
   - [x] 新規実装での `#[allow(dead_code)]` 使用禁止
   - [x] 未使用コードの削除
   - [x] テスト用ヘルパーのみ例外許可

- [x] **CI/CD対応**
   - [x] `cargo clippy --all-targets --all-features -- -D warnings` 警告ゼロ
   - [x] すべてのマルチテナント統合テストがパス
   - [x] 実データによる検証実装

#### 実装完了基準
- [x] チームメンバー間でタスクを共有・協業できる
- [x] 権限に基づいた適切なアクセス制御が機能する
- [x] 既存の個人タスク機能に影響を与えない（後方互換性確保）
- [x] パフォーマンステスト実装（100タスク作成10秒以内）
- [ ] セキュリティ上の問題がない（ペネトレーションテスト未実施）
- [x] すべての新規APIに統合テスト実装（9個のテスト）
- [ ] APIドキュメントと実装が完全に一致（ドキュメント更新未実施）

### 統一権限チェックミドルウェア適用タスクリスト

#### フェーズ0: 基礎実装（完了）
- [x] **統一権限チェックミドルウェアの基礎実装**
   - [x] `RequirePermission`構造体の実装
   - [x] `Action`列挙型の定義（View, Create, Update, Delete, Admin）
   - [x] `require_permission!`マクロの実装
   - [x] `permission_middleware`ヘルパー関数の実装
   - [x] `admin_permission_middleware`ヘルパー関数の実装
   - [x] `PermissionContext`構造体とヘルパーメソッドの実装

#### フェーズ1: 既存APIへの段階的適用準備
- [x] **既存権限チェックの調査と分類**
   - [x] 直接的な`is_admin()`チェックを使用しているハンドラーの一覧化
   - [x] `permission_service.check_resource_access()`を使用しているハンドラーの一覧化
   - [ ] 権限チェックパターンの分類（管理者専用、リソースベース、チームベース等）
   - [ ] 各ハンドラーの権限要件のドキュメント化

- [ ] **テスト環境の準備**
   - [ ] 統一権限チェックミドルウェアを使用する場合のテストヘルパー関数作成
   - [ ] モックPermissionContextの作成ヘルパー実装
   - [ ] 既存テストの権限チェック部分の抽出と共通化

#### フェーズ2: 段階的な適用実装
- [ ] **管理者専用APIへの適用**
   - [ ] `security_handler`の各エンドポイントへの適用
   - [ ] `admin_handler`内の個別`is_admin()`チェックの除去
   - [x] 管理者専用ルーターへの一括適用の検証（`admin_router`に`admin_permission_middleware`適用済み）
   - [ ] 統合テストの修正と動作確認

- [ ] **リソースベース権限APIへの適用**
   - [x] タスクCRUD操作への段階的適用（実験的実装：`task_router_with_unified_permission`）
   - [x] チームCRUD操作への段階的適用（実験的実装：`team_router_with_unified_permission`）
   - [ ] 組織関連操作への適用
   - [ ] 各リソースタイプ毎のアクション権限マッピング

- [ ] **複雑な権限チェックの移行**
   - [ ] チームメンバーシップベースの権限チェック
   - [ ] 階層的権限（組織→チーム→個人）の実装
   - [ ] 動的権限（コンテキストに基づく権限）の対応

#### フェーズ3: テストの完全対応
- [ ] **既存テストの修正**
   - [ ] 権限チェックを期待するテストケースの洗い出し
   - [ ] テスト用の権限設定ヘルパーの実装
   - [ ] モックミドルウェアによるテスト簡略化
   - [ ] E2Eテストでの権限チェック動作確認

- [ ] **新規テストの追加**
   - [ ] 統一権限チェックミドルウェアの単体テスト
   - [ ] 各リソース・アクションの組み合わせテスト
   - [ ] エッジケースの権限チェックテスト
   - [ ] パフォーマンステスト（権限チェックのオーバーヘッド測定）

#### フェーズ4: 完全移行と最適化
- [ ] **既存権限チェックコードの削除**
   - [ ] 直接的な`is_admin()`チェックの完全削除
   - [ ] PermissionServiceの旧メソッドの非推奨化
   - [ ] 不要になったヘルパー関数の削除
   - [ ] `#[allow(dead_code)]`アノテーションの削除

- [ ] **ドキュメントとガイドライン更新**
   - [ ] 統一権限チェックミドルウェアの使用ガイド作成
   - [ ] 新規エンドポイント追加時の権限設定手順
   - [ ] トラブルシューティングガイド
   - [ ] パフォーマンスチューニングガイド

#### 実装時の注意事項
- 各フェーズ完了時点で`make ci-check-fast`が通ること
- 既存のAPIの動作に影響を与えないよう、段階的に適用すること
- 権限チェックの抜け漏れがないよう、網羅的なテストを実施すること
- パフォーマンスへの影響を測定し、必要に応じて最適化すること

#### 現在の実装状況
- 統一権限チェックミドルウェアの基礎実装は完了
- `admin_router`では既に`admin_permission_middleware`が適用済み
- 実験的実装として`/tasks/unified/*`と`/teams/unified/*`エンドポイントで動作確認済み
- 既存APIへの本格適用は`#[allow(dead_code)]`でペンディング（テストへの影響を考慮）

## 🧩 実装ガイドライン

### 1. **ドメイン統合の原則**

* **既存ドメインとの重複・競合は禁止**
  * 同じ意味の別表現、似たが異なるロジック、バリエーション増加は避ける
  * APIのスラグなど機能別の統一感を意識
  * パスパラメータは `{param}` 形式を使用（Axum 0.8の仕様）
  * APIのスラグなどルーティングの重複排除＆集約統合を意識
* **「亜種」API・ドメイン定義の増加は避ける**
  * 新規定義が必要な場合は、**既存の責務・境界に統合**できるか再検討

### 2. **エラーハンドリングのベストプラクティス**

#### エラーコンテキストの命名規則

**形式**: `"モジュール名::関数名[:詳細]"`

```rust
// ✅ 推奨される命名規則
convert_validation_errors(e, "user_handler::update_profile")
convert_validation_errors(e, "auth_handler::signup")
convert_validation_errors(e, "analytics_handler::get_system_stats")

// ❌ 避けるべき例
convert_validation_errors(e, "team")  // モジュール情報なし
convert_validation_errors(e, "validation")  // 汎用的すぎる
```

#### error_helper関数の活用

**すべてのサービス層でerror_helper関数を使用すること**

```rust
use crate::utils::error_helper::{internal_server_error, not_found_error, conflict_error};

// ✅ 推奨: error_helper使用
self.repo.count_tasks()
    .await
    .map_err(|e| internal_server_error(
        e,
        "task_service::get_stats",  // コンテキスト
        "Failed to count tasks"      // ユーザー向けメッセージ
    ))?;

// ❌ 避けるべき: 直接エラー生成
.map_err(|e| AppError::InternalServerError(format!("Failed to count: {}", e)))?;
```

#### エラーログの一貫性

- error_helper関数は自動的に構造化ログを出力
- コンテキスト情報により、エラー発生箇所の特定が容易
- 本番環境でのデバッグ・監視に有効

### 3. **機能追加の原則：実用的で価値の高い機能に集中**

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

### 4. **API日時フォーマットの統一（Unix Timestamp）**

* **APIレスポンスの日時フィールドはUnix Timestamp（秒単位）で統一**
  * すべての日時フィールドは数値型（i64）として返す
  * タイムゾーンに依存しない UTC ベースの絶対時刻
  * 例: `"created_at": 1736922123` （2025-01-15 05:55:23 UTC）

* **実装詳細**
  * カスタム `Timestamp` 型により型安全性を確保
  * DBレイヤーは `DateTime<Utc>` のまま変更なし（`TIMESTAMPTZ`型）
  * シリアライゼーション時のみUnix timestampに変換

* **フロントエンド実装ガイドライン**
  ```javascript
  // APIレスポンスの変換（秒→ミリ秒）
  const createdAt = new Date(response.created_at * 1000);
  
  // APIリクエストの変換（ミリ秒→秒）
  const payload = {
    due_date: Math.floor(selectedDate.getTime() / 1000)
  };
  
  // オプショナルフィールドの処理
  const completedAt = response.completed_at 
    ? new Date(response.completed_at * 1000) 
    : null;
  ```

* **メリット**
  * **パフォーマンス**: 数値は文字列より転送サイズが小さく、パース処理も高速
  * **多言語対応**: Unix timestampは言語・プラットフォーム非依存
  * **一貫性**: すべてのAPIで統一されたフォーマット
  * **業界標準**: Twitter、Stripe等の主要APIで採用されている形式

#### プレースホルダー実装の正規化におけるDB設計変更

* **簡易実装・プレースホルダー実装を正規実装に置き換える際は、必要に応じてマイグレーションによるDB設計の変更も行う**
  * ハードコードされた値を実データから計算するために必要なカラムを追加
  * 分析や集計を高速化するためのインデックスやサマリーテーブルを追加
  * 既存テーブルに不足しているカラム情報があれば追加

### 5. **dead\_code ポリシー**

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
    * `TestDatabase::_container`

### 6. **プロダクションコードの品質基準**

* **すべての公開APIは実装で使用される**こと
* **テストは実装の動作を検証**するものであること
* **未使用の警告が出ないこと**（dead_code警告を含む）

### 7. **APIセキュリティとルーティング規則**

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

### 8. **CI・Lint 要件**

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
