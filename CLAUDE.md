## 実現トピック

ページネーションの統一処理
task-backend/src/shared/types/pagination.rs

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

### 9. **統一権限チェックシステムの使用**

#### 新規APIエンドポイントの権限設定

* **すべての新規APIエンドポイントには統一権限チェックミドルウェアを適用すること**
  ```rust
  use crate::require_permission;
  use crate::middleware::authorization::{resources, Action};
  
  .route(
      "/resources/{id}",
      get(handler).route_layer(require_permission!(resources::RESOURCE, Action::View))
  )
  ```

* **管理者専用エンドポイントは `admin_permission_middleware()` を使用**
  ```rust
  .route_layer(admin_permission_middleware())
  ```

* **直接的な権限チェック（`is_admin()`など）の使用は禁止**
  - すべての権限チェックは統一ミドルウェアを通じて実施
  - サービス層での追加チェックが必要な場合は、明確な理由をコメントで記載

#### 権限チェックの階層

1. **ミドルウェアレベル**: 基本的なロールベース権限チェック
2. **サービスレベル**: リソース固有の詳細な権限チェック（所有者確認など）

#### テスト時の権限考慮

* 権限テストは必ず以下の3パターンを含めること：
  - 正常系（権限あり）
  - 権限なしエラー（403 Forbidden）
  - 認証なしエラー（401 Unauthorized）

### 10. **統一ロギングパターンの使用**

#### ロギング実装の必須要件

* **すべてのサービス層でlog_with_context!マクロを使用すること**
  ```rust
  use crate::log_with_context;
  
  log_with_context!(
      tracing::Level::DEBUG,
      "Creating resource",
      "user_id" => user_id,
      "resource_type" => "task"
  );
  ```

* **ログレベルの使い分け**
  - `DEBUG`: 操作開始時、詳細な処理内容
  - `INFO`: 操作成功時、重要なイベント
  - `WARN`: 警告すべき状況（権限不足、リトライなど）
  - `ERROR`: エラー発生時、処理失敗

* **3段階ログパターンの実装**
  ```rust
  // 1. 操作開始（DEBUG）
  log_with_context!(
      tracing::Level::DEBUG,
      "Starting operation",
      "user_id" => user_id,
      "operation" => "create_task"
  );
  
  // 2. 操作成功（INFO）
  log_with_context!(
      tracing::Level::INFO,
      "Operation completed successfully",
      "user_id" => user_id,
      "resource_id" => created_id
  );
  
  // 3. エラー時（ERROR）
  .map_err(|e| {
      log_with_context!(
          tracing::Level::ERROR,
          "Operation failed",
          "user_id" => user_id,
          "error" => &e.to_string()
      );
      e
  })?;
  ```

#### ミドルウェア設定

* **main.rsでのミドルウェア追加（順序重要）**
  ```rust
  .layer(axum_middleware::from_fn(crate::logging::logging_middleware))
  .layer(axum_middleware::from_fn(crate::logging::inject_request_context))
  ```

* **RequestContextの自動付与**
  - request_id: 各リクエストに一意のID
  - user_id: 認証済みユーザーのID（オプション）
  - path: リクエストパス
  - method: HTTPメソッド

#### ログ出力の構造化

* **すべてのログは構造化され、以下の情報を含む**
  - message: ログメッセージ
  - request_id: リクエスト追跡用ID
  - 任意のkey-valueペア（user_id, resource_idなど）

* **HTTPレスポンスログの自動レベル調整**
  - 5xx: ERROR
  - 4xx: WARN
  - その他: INFO

#### 既存のtracing直接使用の禁止

* **以下の直接使用は禁止（log_with_context!を使用）**
  - `tracing::debug!`
  - `tracing::info!`
  - `tracing::warn!`
  - `tracing::error!`

### 11. **UUID/ID検証パターンの統一実装**

#### 統一UUID Extractorの使用

* **すべてのパスパラメータのUUID検証でValidatedUuidを使用すること**
  ```rust
  use crate::extractors::ValidatedUuid;
  
  // ✅ 推奨
  pub async fn get_resource_handler(
      ValidatedUuid(resource_id): ValidatedUuid,
      State(state): State<AppState>,
  ) -> Result<ApiResponse<ResourceDto>> {
      // resource_idは検証済みのUuid
  }
  
  // ❌ 避けるべき
  pub async fn get_resource_handler(
      Path(resource_id): Path<Uuid>,
      State(state): State<AppState>,
  ) -> Result<ApiResponse<ResourceDto>> {
      // 個別のUUID検証が必要
  }
  ```

#### 複数パラメータの処理

* **複数のUUIDパラメータを含むパスには専用の構造体を定義**
  ```rust
  use crate::extractors::deserialize_uuid;
  use serde::Deserialize;
  
  #[derive(Deserialize)]
  pub struct TeamMemberPath {
      #[serde(deserialize_with = "deserialize_uuid")]
      pub team_id: Uuid,
      #[serde(deserialize_with = "deserialize_uuid")]
      pub member_id: Uuid,
  }
  
  pub async fn update_member_handler(
      Path(params): Path<TeamMemberPath>,
      State(state): State<AppState>,
  ) -> Result<ApiResponse<MemberDto>> {
      // params.team_id, params.member_idで検証済みのUUIDにアクセス
  }
  ```

#### ルーティングパスパラメータの命名規則

* **Axum 0.8のパスパラメータ形式 `{param}` を使用**
* **パラメータ名は対応する構造体のフィールド名と完全一致させること**
  ```rust
  // ✅ 正しい例
  .route("/teams/{team_id}/members/{member_id}", patch(update_member))
  // TeamMemberPath構造体のフィールド名と一致
  
  // ❌ 間違った例
  .route("/teams/{id}/members/{member}", patch(update_member))
  // フィールド名と不一致
  ```

#### エラーメッセージの統一

* **すべてのUUID検証エラーは統一フォーマットを使用**
  - フォーマット: `"Invalid UUID format: 'xxx'"`
  - エラータイプ: `AppError::BadRequest`
  - 一貫性のあるユーザー体験を提供

#### 実装の利点

1. **検証ロジックの一元化**: UUID検証が単一の場所に集約
2. **エラーメッセージの一貫性**: すべてのエンドポイントで同じエラー形式
3. **型安全性の向上**: コンパイル時にUUID検証を保証
4. **保守性の向上**: 検証ロジックの変更が容易
5. **テストの簡素化**: Extractor自体のテストで網羅的な検証が可能

### 12. **ページネーションの統一実装**

#### 統一ページネーション型の使用

* **すべてのページネーションでshared/types/pagination.rsの型を使用すること**
  ```rust
  use crate::shared::types::{PaginatedResponse, PaginationMeta};
  use crate::types::query::PaginationQuery;
  ```

* **定数の定義と使用**
  ```rust
  // shared/types/pagination.rs
  pub const DEFAULT_PAGE_SIZE: u32 = 20;
  pub const MAX_PAGE_SIZE: u32 = 100;
  ```

#### APIハンドラーでの実装

* **Query DTOでPaginationQueryを使用**
  ```rust
  // ✅ 推奨: フラット化して使用
  #[derive(Debug, Deserialize)]
  pub struct TaskSearchQuery {
      #[serde(flatten)]
      pub pagination: PaginationQuery,
      // その他のフィルタ条件
  }
  
  // ✅ 推奨: 直接使用
  pub async fn get_resources_handler(
      Query(query): Query<PaginationQuery>,
      State(state): State<AppState>,
  ) -> Result<ApiResponse<PaginatedResponse<ResourceDto>>>
  
  // ❌ 避けるべき: 独自実装
  let page = params.get("page").and_then(|p| p.parse().ok()).unwrap_or(1);
  ```

#### サービス層での実装

* **get_pagination()メソッドで統一的に値を取得**
  ```rust
  let (page, per_page) = query.pagination.get_pagination();
  // pageは1ベース、per_pageは1〜100の範囲で自動的に制限される
  ```

* **PaginationMeta::new()でメタ情報を生成**
  ```rust
  let pagination = PaginationMeta::new(page, per_page, total_count);
  let response = PaginatedResponse::new(items, page, per_page, total_count);
  ```

#### リポジトリ層での実装

* **ページサイズの重複制限は不要**
  ```rust
  // ❌ 避けるべき: get_pagination()で既に制限済み
  let page_size = std::cmp::min(page_size, 100);
  
  // ✅ 推奨: そのまま使用
  let offset = (page - 1) * page_size;
  ```

#### レスポンス型の統一

* **型エイリアスを使用してレスポンスを定義**
  ```rust
  pub type TaskPaginationResponse = PaginatedResponse<TaskDto>;
  pub type UserListResponse = PaginatedResponse<UserSummary>;
  ```

#### 実装の利点

1. **一貫性**: すべてのAPIで同じページネーション動作
2. **保守性**: ページネーションロジックが一箇所に集約
3. **型安全性**: コンパイル時にページネーション構造を保証
4. **DRY原則**: 重複コードの排除
5. **拡張性**: 将来的な変更が容易

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

---
