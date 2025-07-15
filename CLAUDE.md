## 実現トピック

## 2. エラーハンドリングパターンの統一 ✅✅✅

### 実装完了内容

#### 1. 統一エラー型の実装 ✅
- `AppError`型から旧バリデーションエラー型を削除
  - `ValidationError(String)` → 削除完了
  - `ValidationErrors(Vec<String>)` → 削除完了
- 新しいバリデーションエラー型を統一使用
  - `Validation(#[from] ValidationError)` - 単一フィールドエラー
  - `ValidationFailure(#[from] ValidationErrors)` - 複数フィールドエラー
  - `BadRequest(String)` - 汎用バリデーションエラー

#### 2. エラーヘルパー関数の全面活用 ✅
- **全ハンドラーでerror_helperを使用**
  - `convert_validation_errors()` - 全バリデーションエラーで使用
  - コンテキスト名を含むログ出力を実現
- **統一されたエラー処理パターン**
  ```rust
  payload.validate()
      .map_err(|e| convert_validation_errors(e, "context_name"))?;
  ```

#### 3. エラーレスポンス構造の簡素化 ✅
- 未使用の`ErrorResponse`構造体を削除
- `IntoResponse`実装を整理
- `to_error_detail()`メソッドを適切に実装

#### 4. 全ハンドラー・サービスの移行完了 ✅
- **103箇所以上のエラー処理を統一**
  - api/handlers/: 47箇所
  - repository/: 2箇所  
  - service/: 49箇所
  - utils/: 6箇所

### 品質確認結果
- **cargo clippy**: 警告ゼロ ✅
- **cargo test**: 全テストパス ✅
- **cargo fmt**: フォーマット済み ✅

### 今後のメンテナンス方針
1. 新規エラー処理はerror_helperを必ず使用
2. バリデーションエラーはconvert_validation_errors経由で変換
3. コンテキスト情報を含むログ出力を維持

---

## 📢 次回セッションへの引継ぎ事項

### 作業依頼内容：エラーハンドリングの更なる改善

#### 1. error_helper関数の全サービスでの活用

**現状の問題点**:
- 多くのサービス層で`AppError::InternalServerError(format!())`を直接使用
- error_helperの`internal_server_error()`, `not_found_error()`, `conflict_error()`関数が活用されていない

**対応箇所の例**:
```rust
// 以下のファイルに多数存在
task-backend/src/service/attachment_service.rs: 6箇所
task-backend/src/service/task_service.rs: 12箇所  
task-backend/src/service/auth_service.rs: 9箇所
task-backend/src/service/payment_service.rs: 3箇所
task-backend/src/service/user_service.rs: 2箇所
task-backend/src/service/organization_hierarchy_service.rs: 3箇所
task-backend/src/service/storage_service.rs: 11箇所
```

**修正方法**:
```rust
// 現状
.map_err(|e| AppError::InternalServerError(format!("Failed to count tasks: {}", e)))?

// 改善後
use crate::utils::error_helper::internal_server_error;

.map_err(|e| internal_server_error(
    e,
    "task_service::get_task_stats",  // コンテキスト
    "Failed to count tasks"          // ユーザー向けメッセージ
))?
```

#### 2. エラーコンテキストの命名規則統一

**現状の問題点**:
- コンテキスト名が一貫性なく、エラー発生箇所の特定が困難

**現在のコンテキスト名の例**:
```rust
convert_validation_errors(e, "team")  // モジュール情報なし
convert_validation_errors(e, "signup")  // 関数名のみ
convert_validation_errors(e, "system_stats_query")  // クエリ名
```

**統一後の命名規則**:
```rust
// 形式: "モジュール名::関数名[:詳細]"
convert_validation_errors(e, "team_handler::create_team")
convert_validation_errors(e, "auth_handler::signup") 
convert_validation_errors(e, "analytics_handler::get_system_stats")
```

**対応が必要な箇所**:
- api/handlers/配下の全ハンドラー: 約40箇所以上
- サービス層でerror_helperを使用する際のコンテキスト名

### 作業手順

1. **Phase 1**: error_helper関数のサービス層での全面活用
   - 各サービスファイルで`AppError::InternalServerError(format!())`を検索
   - error_helperの適切な関数に置き換え
   - コンテキスト名を統一規則に従って設定

2. **Phase 2**: エラーコンテキストの命名規則統一
   - 全ハンドラーで`convert_validation_errors`のコンテキスト名を更新
   - "モジュール名::関数名"形式に統一

3. **テスト確認**
   - `cargo test`で全テストがパスすることを確認
   - `cargo clippy`で警告がないことを確認

### 期待される成果

1. **エラーログの品質向上**
   - すべてのエラーが構造化ログとして記録
   - コンテキスト情報によりデバッグが容易

2. **セキュリティの向上**
   - 内部エラー詳細がユーザーに漏洩しない

3. **保守性の向上**
   - エラー処理パターンが完全に統一
   - 新規開発者も一貫したパターンを学習可能

### 推奨される次のステップ

#### 1. 旧バリデーションエラー型の削除
```rust
// AppErrorから以下を削除
- ValidationError(String),
- ValidationErrors(Vec<String>),
```

#### 2. 全ハンドラーでerror_helperを活用
```rust
// 例: user_handler.rs
use crate::utils::error_helper::convert_validation_errors;

// 旧コード
payload.validate().map_err(|e| {
    let errors = e.field_errors()
        .into_iter()
        .map(|(field, errors)| format!("{}: {}", field, errors.join(", ")))
        .collect();
    AppError::ValidationErrors(errors)
})?;

// 新コード
payload.validate()
    .map_err(convert_validation_errors)?;
```

#### 3. エラーレスポンス構造の簡素化
```rust
// IntoResponseの実装をシンプルに
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Validation(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            AppError::ValidationFailure(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::Unauthorized(_) => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.clone()),
            AppError::DbErr(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()),
            AppError::InternalServerError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()),
            // 他のエラータイプ
        };
        
        let error_response = json!({
            "error": message,
            "status": status.as_u16()
        });
        
        (status, Json(error_response)).into_response()
    }
}

// DTOへの自動バリデーション
#[async_trait]
impl<T> FromRequest for ValidatedJson<T>
where
    T: DeserializeOwned + Validate + Send,
{
    type Rejection = AppError;
    
    async fn from_request(req: Request, state: &State) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|_| AppError::Validation(ValidationError::new("Invalid JSON")))?;
        
        value.validate()?;
        Ok(ValidatedJson(value))
    }
}
```

#### 4. データベースエラーの統一処理
```rust
// 旧コード
.map_err(|e| AppError::InternalServerError(format!("Failed to create: {}", e)))?;

// 新コード  
.map_err(|e| {
    error!("Failed to create organization: {:?}", e);
    AppError::DbErr(e)
})?;
```

#### 5. 移行スケジュール
1. **Phase 1（即座）**: error_helperの全面活用
2. **Phase 2（1週間以内）**: 旧バリデーションエラー型の削除
3. **Phase 3（2週間以内）**: エラーレスポンス構造の統一
4. **Phase 4（1ヶ月以内）**: 全ハンドラーでの適用完了


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
