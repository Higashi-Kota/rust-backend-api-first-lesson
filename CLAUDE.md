## 実現トピック

### 🔍 統合テストの品質改善

#### 目的
既存の統合テストを横断的に精査し、AAA（Arrange-Act-Assert）パターンに従った実効性のあるテストに改善する。特に実データによる検証、副作用の確認、時間依存テストの適切な実装に重点を置く。

#### Phase 1: 未実装テストの作成

**1. GDPRコンプライアンステストの実装**
- `tests/integration/gdpr/` フォルダを新規作成
- 実装対象：
  - `mod.rs` - モジュール定義
  - `data_export_tests.rs` - ユーザーデータエクスポート機能
  - `data_deletion_tests.rs` - ユーザーデータ削除（忘れられる権利）
  - `consent_management_tests.rs` - 同意管理機能
  - `admin_gdpr_tests.rs` - 管理者向けGDPR機能
- CI設定の更新（`.github/workflows/ci.yml`に`integration::gdpr`を追加）

**2. 組織設定テストの実装**
- `tests/integration/organization/organization_settings_tests.rs`
  - 組織設定の更新（名前、説明、設定JSON）
  - 権限チェック（組織管理者のみ更新可能）
  - バリデーション（必須項目、文字数制限）
  - 監査ログの記録確認

**3. 組織サブスクリプションテストの実装**
- `tests/integration/organization/organization_subscription_tests.rs`
  - サブスクリプションプランの変更
  - アップグレード/ダウングレードの制約
  - 課金関連の権限チェック
  - プラン変更履歴の記録

**4. チーム招待テストの実装**
- `tests/integration/team/team_invitation_tests.rs`
  - 招待の作成・送信
  - 招待の承認・拒否
  - 招待の有効期限
  - 重複招待の防止
  - 招待権限の確認

#### Phase 2: 実データ検証の改善

**1. Cleanup操作テストの改善**
- 対象: `tests/integration/admin/cleanup_operations_tests.rs`
- 改善内容：
  - 実際の古いデータを作成してから削除をテスト
  - タイムスタンプを操作して時間経過をシミュレート
  - 削除対象と保持対象のデータを明確に分離
  - 削除後のデータベース状態を検証

**2. Analytics関連テストの値検証強化**
- 対象ファイル：
  - `analytics/user_analytics_tests.rs`
  - `analytics/behavior_analytics_tests.rs`
  - `analytics/admin_analytics_test.rs`
- 改善内容：
  - 事前に特定の行動データを作成
  - 集計結果の数値が正確であることを検証
  - 比較データの妥当性を確認
  - 時系列データの整合性チェック

**3. Feature Usage Metricsテストの充実**
- 対象: `admin/feature_usage_analytics_test.rs`
- 改善内容：
  - 実際の機能使用履歴を作成
  - 集計期間による結果の違いを検証
  - ゼロ件の場合と複数件の場合の両方をテスト

#### Phase 3: 副作用検証の追加

**1. 権限変更の伝播確認**
- 対象ファイル：
  - `roles/role_management_tests.rs`
  - `permission/permission_check_test.rs`
- 追加検証：
  - ロール変更後の権限反映タイミング
  - キャッシュの無効化確認
  - 関連するセッションの更新

**2. 組織階層変更の影響確認**
- 対象: `organization_hierarchy/organization_hierarchy_tests.rs`
- 追加検証：
  - 部門移動時の権限継承
  - 階層変更時の通知
  - 循環参照の防止

**3. 一括操作の履歴記録**
- 対象: `analytics/user_analytics_tests.rs` の bulk operation
- 追加検証：
  - `bulk_operation_histories` テーブルへの記録
  - 操作ログの詳細度
  - エラー時のロールバック

#### Phase 4: エラーハンドリングの網羅性向上

**1. 境界値テストの追加**
- 全APIエンドポイントに対して：
  - 最大値を超えるデータ
  - 空配列・null値の扱い
  - 特殊文字を含むデータ
  - 巨大なJSONペイロード

**2. 同時実行・競合状態のテスト**
- 追加対象：
  - タスクの同時更新
  - チームメンバーの同時追加/削除
  - 組織設定の同時変更
- 楽観的ロックの動作確認

**3. 依存リソース不在時の動作**
- 削除されたユーザーのタスクアクセス
- 解散したチームのメンバー一覧
- 無効化された組織のデータ

#### Phase 5: テストヘルパーの改善

**1. 時間操作ユーティリティの追加**
- `tests/common/time_helper.rs` を作成
- 任意の過去・未来のタイムスタンプでデータ作成
- テスト用の時間進行シミュレーション

**2. データビルダーパターンの導入**
- 各エンティティ用のテストデータビルダー
- 関連データの自動生成
- 大量データの効率的な作成

**3. アサーションヘルパーの充実**
- データベース状態の検証ヘルパー
- 複雑なJSONレスポンスの検証
- 副作用の包括的チェック

#### 完了基準
- すべての統合テストがAAAパターンに準拠
- 実データによる動的な検証の実装
- 副作用（DB更新、ログ、通知）の検証
- エラーケースの網羅（各API最低5パターン）
- テスト実行時間の計測と最適化（並列実行可能）
- カバレッジレポートでの改善確認

## 🧩 実装ガイドライン

### 1. **ドメイン統合の原則**

* **既存ドメインとの重複・競合は禁止**
  * 同じ意味の別表現、似たが異なるロジック、バリエーション増加は避ける
  * APIのスラグなど機能別の統一感を意識
  * APIのスラグなどルーティングの重複排除＆集約統合を意識
* **「亜種」API・ドメイン定義の増加は避ける**
  * 新規定義が必要な場合は、**既存の責務・境界に統合**できるか再検討

### 2. **データベース設計の原則**

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

### 3. **dead\_code ポリシー**

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

### 4. **プロダクションコードの品質基準**

* **すべての公開APIは実装で使用される**こと
* **テストは実装の動作を検証**するものであること
* **未使用の警告が出ないこと**（dead_code警告を含む）

### 5. **CI・Lint 要件**

* 以下のコマンドで **エラー・警告が完全にゼロ** であること：

  ```bash
  cargo clippy --all-targets --all-features -- -D warnings
  ```

* 既存CI（テスト）コマンド：

  ```bash
  make ci-check
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

---

## 🔥 クリーンアップ方針

* **使用されていないコード**の取り扱い：
  1. **テストでのみ使用** → 実装で活用するよう統合
  2. **どこでも未使用** → 削除
  3. **将来の拡張用** → 現時点で価値提供できるよう実装

* dead\_code で検知される要素への対応：
  * **公開API（pub）** → 実装での活用を検討
  * **内部実装（非pub）** → 使用されていなければ削除
  * **テスト用ユーティリティ** → そのまま維持

---

## 🚀 実装完了後の期待される状態

1. **`cargo clippy`で警告ゼロ**（dead_code警告を含む）
2. **`make ci-check`ですべてのテストがグリーン**
3. **APIドキュメントと実装が一致**
4. **テストが実装の実際の動作を検証**
5. **プロダクションコードがクリーンで保守しやすい**

---

