## 実現トピック

### 📊 統合テストの拡充と機能重複の解消ならびに実装がモック実装になっている箇所の正規実装

#### 🎯 目的
1. **モック実装の正規実装化**
   - ハードコードされた値の動的計算への置き換え
   - プレースホルダー実装の完成
   - 未実装機能の実装

2. **統合テストカバレッジの向上**
   - 既存APIの網羅的なテスト
   - エッジケースとエラーパスの検証
   - 権限チェックの完全性確認

3. **機能重複の解消**
   - 類似機能の統合
   - セマンティックな整理
   - APIエンドポイントの最適化

#### 📋 実装計画

##### Phase 1: モック実装の正規実装化【優先度: 高】

1. **Analytics Handler の実装**
   - `src/api/handlers/analytics_handler.rs`
     - Line 98: `user_growth_rate` を実際のデータから計算
     - Lines 222-303: システム統計を実データベースから取得
     - Lines 1039-1136: モックデータ生成を実際のデータ集計に置き換え
   
   **必要なDB設計**:
   ```sql
   -- 機能使用状況追跡テーブル
   CREATE TABLE feature_usage_metrics (
     id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
     user_id UUID REFERENCES users(id),
     feature_name VARCHAR(100) NOT NULL,
     action_type VARCHAR(50) NOT NULL,
     created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
     metadata JSONB
   );
   
   -- 日次活動サマリーテーブル
   CREATE TABLE daily_activity_summaries (
     date DATE NOT NULL,
     total_users INTEGER NOT NULL DEFAULT 0,
     active_users INTEGER NOT NULL DEFAULT 0,
     new_users INTEGER NOT NULL DEFAULT 0,
     tasks_created INTEGER NOT NULL DEFAULT 0,
     tasks_completed INTEGER NOT NULL DEFAULT 0,
     PRIMARY KEY (date)
   );
   ```

2. **User Service の実装**
   - `src/service/user_service.rs`
     - Lines 222-250: `get_user_stats_for_analytics()` の実装
     - Lines 563-573: UpdateRole 一括操作の実装
     - Lines 371-389: メールトークン検証の完全実装
     - Lines 418-443: ユーザー設定の永続化
   
   **必要なDB設計**:
   ```sql
   -- ユーザー設定テーブル
   CREATE TABLE user_settings (
     user_id UUID PRIMARY KEY REFERENCES users(id),
     language VARCHAR(10) NOT NULL DEFAULT 'ja',
     timezone VARCHAR(50) NOT NULL DEFAULT 'Asia/Tokyo',
     notifications_enabled BOOLEAN NOT NULL DEFAULT true,
     email_notifications JSONB NOT NULL DEFAULT '{}',
     ui_preferences JSONB NOT NULL DEFAULT '{}',
     created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
     updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
   );
   ```

3. **Security Service の実装**
   - `src/service/security_service.rs`
     - Lines 55-56: トークン年齢を実際のデータから計算
     - Lines 68-70: リクエスト数を実際にカウント

##### Phase 2: 統合テストの追加【優先度: 高】

1. **欠落している統合テストの実装**
   ```
   tests/integration/
   ├── organization/
   │   ├── organization_settings_tests.rs    # 新規作成
   │   └── organization_subscription_tests.rs # 新規作成
   ├── gdpr/                                  # 新規フォルダ
   │   ├── mod.rs
   │   ├── data_export_tests.rs
   │   ├── data_deletion_tests.rs
   │   ├── consent_management_tests.rs
   │   └── admin_gdpr_tests.rs
   ├── analytics/
   │   └── behavior_analytics_tests.rs       # 新規作成
   ├── team/
   │   └── team_invitation_tests.rs          # 新規作成
   └── user/
       └── bulk_operations_tests.rs           # 新規作成
   ```

   **注意**: 新規フォルダ（gdpr/）を作成する場合は、`.github/workflows/ci.yml`のテストマトリックスに`integration::gdpr`を追加する必要があります。

2. **各APIエンドポイントの必須テストパターン**
   ```rust
   // 組織設定APIのテスト例
   #[tokio::test]
   async fn test_update_organization_settings_success() { }
   
   #[tokio::test]
   async fn test_update_organization_settings_validation_error() { }
   
   #[tokio::test]
   async fn test_update_organization_settings_unauthorized() { }
   
   #[tokio::test]
   async fn test_update_organization_settings_forbidden() { }
   
   #[tokio::test]
   async fn test_update_organization_settings_not_found() { }
   ```

3. **既存テストの整理**
   - `security/gdpr_compliance_test.rs`を新規`gdpr/`ディレクトリへ移動
   - 重複する`auth/password_reset_test.rs`と`auth/password_reset_tests.rs`を統合
   - テストファイル名を`*_tests.rs`（複数形）に統一

##### Phase 3: 機能重複の解消【優先度: 中】

1. **UserService の統合**
   - `list_users_with_roles_paginated()` と `get_all_users_with_roles_paginated()` を統合
   - 共通のページネーションロジックを抽出

2. **OrganizationService の統合**
   - `get_organizations()` と `get_organizations_paginated()` を統合
   - 統一されたクエリビルダーの実装

3. **権限チェックの集約**
   - 分散している権限チェックロジックを `PermissionService` に集約
   - 共通の権限検証ミドルウェアの実装

##### Phase 4: DB最適化とインデックス追加【優先度: 中】

1. **検索パフォーマンス改善**
   ```sql
   -- 組織名検索用インデックス
   CREATE INDEX idx_organizations_name_search ON organizations USING gin(name gin_trgm_ops);
   
   -- ユーザー検索用インデックス
   CREATE INDEX idx_users_email_search ON users(email);
   CREATE INDEX idx_users_username_search ON users(username);
   
   -- 機能使用状況の集計用インデックス
   CREATE INDEX idx_feature_usage_metrics_date ON feature_usage_metrics(created_at);
   CREATE INDEX idx_feature_usage_metrics_user_feature ON feature_usage_metrics(user_id, feature_name);
   ```

2. **一括操作履歴テーブル**
   ```sql
   CREATE TABLE bulk_operation_histories (
     id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
     operation_type VARCHAR(50) NOT NULL,
     performed_by UUID REFERENCES users(id),
     affected_count INTEGER NOT NULL,
     status VARCHAR(20) NOT NULL,
     error_details JSONB,
     created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
     completed_at TIMESTAMPTZ
   );
   ```

#### 🏁 完了基準
1. **dead_codeアノテーションがテスト用途以外でゼロ**
2. **すべてのモック実装が実データに基づく実装に置き換わっている**
3. **すべての公開APIに統合テストが存在**
4. **各APIに最低5パターンのテスト**（正常系、バリデーション、認証、権限、リソース不在）
5. **機能重複が解消され、シンプルなAPI構造**
6. **必要なDBテーブルとインデックスが作成されている**
7. **CI/CDパイプラインですべてのテストが通過**

#### 📈 メトリクス
- モック実装の残存数: 0
- テストカバレッジ: 80%以上
- 統合テスト数: 各API × 5パターン以上
- API応答時間: 200ms以下（95パーセンタイル）

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

* APIレベルでの**E2Eフロー確認**

  * リクエスト／レスポンス構造の妥当性
  * DB書き込み・読み出しの整合性
  * エラーハンドリングの検証

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

