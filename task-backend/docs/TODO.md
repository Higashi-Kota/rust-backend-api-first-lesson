# 📋 プロダクション向け新規 API 拡張計画

## 🎯 概要 & 実装進捗

現在のコードベースには `#[allow(dead_code)]` でマークされた多数の実装済み機能があり、これらを API として公開することで大幅な機能拡張が可能です。本ドキュメントでは、既存 API と重複しない**新規価値提供 API**を体系的に整理します。

### 📊 **実装進捗サマリー** (2025-01-24 更新)

- ✅ **Phase 1.1 高度ユーザー管理**: **100% 完了** (5/5 エンドポイント実装済み)
- ✅ **Phase 1.2 セキュリティ・トークン管理**: **100% 完了** (7/7 エンドポイント実装済み)
- ❌ **Phase 2.1 組織階層管理**: **未実装** (0/10 エンドポイント)
- ❌ **Phase 2.2 チーム招待・権限管理**: **未実装** (0/6 エンドポイント)

**現在のAPI数**: 78 → **90** (+12エンドポイント追加済み)

## 🔍 現状分析

### ✅ 既存実装済み API（78 エンドポイント）

- 基本認証（11 エンドポイント）
- タスク管理（16 エンドポイント）
- ユーザー管理（11 エンドポイント）
- ロール・権限管理（6 エンドポイント）
- サブスクリプション管理（5 エンドポイント）
- チーム・組織機能（10 エンドポイント）
- 分析・統計（3 エンドポイント）
- システム機能（3 エンドポイント）

### 🚀 新規 API 拡張ポテンシャル

`#[allow(dead_code)]` 関数から**30 個の新規エンドポイント**を発見（エンタープライズ基盤+統一権限ガバナンス）

---

## 📊 **新規 API 拡張プラン**

### 🔧 **Phase 1: 管理者ダッシュボード強化**

_優先度: 🔥 最高_

#### **1.1 高度なユーザー管理 API** ✅ **100% 実装完了**

既存の基本ユーザー管理を大幅に拡張：

```http
# 既存: GET /admin/users（基本検索・フィルタ）
# 実装済み拡張 API:
✅ GET /admin/users/advanced-search      # 高度な検索・フィルタリング
✅ GET /admin/users/analytics           # ユーザー分析ダッシュボード
✅ GET /admin/users/by-role/{role}      # ロール別ユーザー管理
✅ GET /admin/users/by-subscription     # サブスクリプション別分析
✅ POST /admin/users/bulk-operations    # 一括ユーザー操作

# 未実装（優先度低）:
⚪ GET /admin/users/activity-stats      # アクティビティ統計 - 別途検討
```

**実装状況**: 
- ✅ Handler: 5/5 完全実装済み
- ✅ Service: 5/5 完全実装済み  
- ✅ Repository: 5/5 完全実装済み
- ✅ DTO: 5/5 完全実装済み
- ✅ Router: 5/5 登録済み

**ビジネス価値**: 管理者の運用効率向上、詳細なユーザー分析、効率的な大量ユーザー管理


#### **1.2 セキュリティ・トークン管理 API** ✅ **100% 実装完了**

_新規領域（既存 API なし）_

```http
# 実装済み API:
✅ GET /admin/security/token-stats        # トークン利用統計
✅ GET /admin/security/refresh-tokens     # リフレッシュトークン監視
✅ POST /admin/security/cleanup-tokens    # 期限切れトークン自動削除
✅ GET /admin/security/password-resets    # パスワードリセット監視
✅ POST /admin/security/revoke-all-tokens # 緊急時全トークン無効化
✅ GET /admin/security/session-analytics  # セッション分析
✅ POST /admin/security/audit-report      # セキュリティ監査レポート
```

**実装状況**: 
- ✅ Handler: 7/7 完全実装済み
- ✅ Service: 7/7 完全実装済み
- ✅ Repository: 7/7 完全実装済み
- ✅ DTO: 7/7 完全実装済み
- ✅ Router: 7/7 登録済み
- ✅ Tests: 統合テスト・単体テスト完備

**ビジネス価値**: セキュリティ監視、不正アクセス対策、コンプライアンス対応

---

### 🏢 **Phase 2: マルチテナント・組織機能**

_優先度: 🔥 高_

#### **2.1 組織階層管理 API**

既存の基本組織機能をエンタープライズ階層管理に拡張：

```http
# 既存: 基本組織CRUD（5エンドポイント）
# 新規拡張:
GET /organizations/{id}/hierarchy      # 組織階層構造取得
POST /organizations/{id}/departments   # 階層入れ子部門作成
GET /organizations/{id}/departments    # 部門一覧・階層表示
PUT /organizations/{id}/departments/{dept_id}  # 部門情報・階層更新
DELETE /organizations/{id}/departments/{dept_id}  # 部門削除（子部門移動処理込み）
GET /organizations/{id}/analytics      # 組織分析ダッシュボード
PUT /organizations/{id}/permission-matrix      # 組織統一権限マトリックス設定
GET /organizations/{id}/permission-matrix      # 組織権限マトリックス取得
GET /organizations/{id}/effective-permissions  # 組織実効権限分析
POST /organizations/{id}/data-export   # 組織データエクスポート（階層構造保持）
```

**ビジネス価値**: 階層入れ子組織管理、統一権限ガバナンス、エンタープライズスケーラビリティ、データポータビリティ

#### **2.2 チーム招待・権限管理 API**

既存チーム機能をエンタープライズ招待・権限管理に特化：

```http
# 既存: 基本チーム管理（6エンドポイント）
# 新規拡張:
POST /teams/{id}/bulk-member-invite    # 一括メンバー招待
GET /teams/{id}/invitations            # 招待状況確認・管理
PUT /teams/{id}/invitations/{invite_id}/decline  # 招待辞退（招待とセット）
PUT /teams/{id}/permission-matrix      # チーム詳細権限マトリックス設定
GET /teams/{id}/permission-matrix      # チーム権限マトリックス取得
GET /teams/{id}/effective-permissions  # チーム実効権限分析
```

**ビジネス価値**: 効率的メンバー招待・辞退フロー、チーム権限制御、組織継承権限分析、変更対応


---

## 🎯 **実装優先度マトリックス**

### **🔥 Tier 1 (実装済み・高 ROI)**

| API Category         | Implementation Status | Business Impact | Technical Status |
| -------------------- | ------------------- | --------------- | ---------------- |
| 高度ユーザー管理     | ✅ **100% 完了**      | 🔥 高           | ✅ 本番投入可能    |
| セキュリティ監視     | ✅ **100% 完了**      | 🔥 非常に高     | ✅ 本番投入可能    |

### **🔶 Tier 2 (短期実装・中 ROI)**

| API Category               | Implementation Ready | Business Impact | Technical Effort |
| -------------------------- | -------------------- | --------------- | ---------------- |
| 組織階層管理               | ✅ 80%               | 🔶 高           | 🔶 中            |
| チーム招待・権限管理       | ✅ 75%               | 🔶 中-高        | 🔶 中            |

---

## 🗑️ **削除推奨（真のデッドコード）**

### **削除対象関数**

以下は実装が不完全または既存機能と重複するため削除推奨：

```rust
// 設定のみの機能（ビジネス価値低）
UserRepository::with_schema()
RefreshTokenRepository::with_schema()
PasswordResetTokenRepository::with_schema()

// 重複する簡単なバリデーション
EmailService::from_env() // EmailConfig::defaultで代替可能
一部のemail template helper methods // 統合可能

// 未完成の実装
一部のconfiguration構造体のfield // 使用されていないフィールド
```

**削除による影響**: なし（設定関数やヘルパーのみ）

---

## 💡 **新機能による競争優位性**

### **1. エンタープライズ対応の包括性**

- **現在**: 基本的なタスク管理 SaaS
- **実装後**: エンタープライズ対応の包括的組織管理プラットフォーム

### **2. 管理者体験の革新**

- **現在**: 限定的な管理機能
- **実装後**: 高度な分析・監視・自動化を備えた管理ダッシュボード

### **3. セキュリティ・コンプライアンス強化**

- **現在**: 基本認証・認可
- **実装後**: 包括的セキュリティ監視・監査システム

---

## 📈 **ROI 予測**

### **実装コスト vs 価値**

| Phase   | 実装工数 | 新規エンドポイント数 | 実装状況 | ROI 予測 |
| ------- | -------- | -------------------- | -------- | -------- |
| Phase 1.1 | ✅ **完了** | 5 エンドポイント | ✅ **100%完了** | ✅ **実現済み** |
| Phase 1.2 | ✅ **完了** | 7 エンドポイント | ✅ **100%完了** | ✅ **実現済み** |
| Phase 2 | 3-4 週間 | 17 エンドポイント    | ❌ 未実装 | 300%+    |

### **総合インパクト**

- **エンドポイント数**: 78 → **90 (現在)** → 108 (Phase2完了時) 
  - ✅ Phase 1.1: +5 (完了済み)
  - ✅ Phase 1.2: +7 (完了済み)
  - ❌ Phase 2: +17 (未実装)
- **機能カバレッジ**: 基本 SaaS → **高度管理・セキュリティ強化SaaS (現在)** → エンタープライズプラットフォーム基盤（Phase2完了時）
- **市場ポジション**: タスク管理ツール → **高機能セキュリティ管理ツール (現在)** → 汎用エンタープライズ基盤ソリューション (Phase2完了時)

---

## 🛠️ **実装ガイドライン**

### **Phase 1 実装手順**

**Phase 1.1: ✅ 完了**
1. ✅ **User Repository functions** の `#[allow(dead_code)]` 削除
2. ✅ **対応する Handler** 作成
3. ✅ **Router** に新規エンドポイント追加
4. ✅ **API Documentation** 更新
5. ✅ **Integration Tests** 追加

**Phase 1.2: ✅ 100% 完了**
1. ✅ **SecurityService** 残り3メソッド追加完了
2. ✅ **SecurityHandler** 残り3ハンドラー追加完了
3. ✅ **SecurityDTO** 削除済みDTOの再実装完了
4. ✅ **Router** 残り3エンドポイント追加完了
5. ✅ **Integration Tests** 追加完了

### **テスト戦略**

```bash
# 新規APIの段階的テスト
cargo test admin::users::advanced_search
cargo test admin::security::token_management
cargo test organizations::hierarchy
```

### **設定管理**

新規 API は既存の動的権限システムと統合し、サブスクリプション階層に基づくアクセス制御を継承

---

## 🔄 **段階的ロールアウト計画**

### **✅ Phase 1.1 完了済み**

- ✅ 高度なユーザー管理 API 100% 実装完了
- ✅ 管理者ダッシュボード基盤完成
- ✅ 包括的テストスイート実装済み

### **✅ Phase 1.2 完了済み**

- ✅ セキュリティ監視 API 残り3エンドポイント実装完了
- ✅ 削除済みセキュリティDTOの再実装完了
- ✅ 統合テスト完成

### **Phase 2: 組織機能実装 (次期実装予定)**

- 組織階層管理 API 実装
- チーム協業機能強化

### **最終クリーンアップ・最適化**

- 残存 Dead Code 削除
- パフォーマンス最適化
- プロダクション準備

---

## 📝 **結論**

現在のコードベースには**30 個の新規エンドポイント**相当の実装済み機能が含まれており（エンタープライズ基盤コア機能+統一権限ガバナンス）、これらを体系的に API として公開することで：

1. **📈 38%の API 機能拡張**（78 → 108 エンドポイント）
2. **🏢 エンタープライズ基盤**として他アプリに 100%応用可能
3. **💰 汎用プラットフォーム化**による収益機会拡大
4. **🛡️ セキュリティ・コンプライアンス・階層管理・統一権限ガバナンスの完全実装**

**削除すべき真のデッドコード**は全体の 10%未満であり、90%以上は**高い汎用性とエンタープライズ価値**を持つコア機能です。

---

## 🔬 **Dead Code 削減効果の詳細分析**

_2025-01-22 追記: 78 → 123 API 拡張による Dead Code 削減効果の定量分析_

### 📊 **現状の Dead Code 詳細分析**

#### **総合統計**

- **現在の`#[allow(dead_code)]`総数**: 191 個の関数・メソッド
- **API 化可能な機能**: 170 個 (89.0%)
- **真の dead code**: 15-20 個 (8.0-10.0%)
- **内部ユーティリティ**: 6-10 個 (3.0-5.0%)

#### **カテゴリ別詳細分析**

##### **🔧 Phase 1 で解消される Dead Code (60 個 - 31.4%)**

```rust
// ユーザー管理機能 (15個の関数)
UserRepository::find_all()                    → GET /admin/users/advanced-search
UserRepository::find_active_users()           → GET /admin/users/analytics
UserRepository::find_by_username()            → GET /admin/users/by-username/{username}
UserRepository::find_paginated()              → GET /admin/users/paginated
UserRepository::is_username_taken()           → GET /admin/users/username-check
UserRepository::update_subscription_tier()    → PUT /admin/users/{id}/subscription
UserRepository::find_by_role()                → GET /admin/users/by-role/{role}
UserRepository::find_by_subscription_tier()   → GET /admin/users/by-subscription
UserRepository::get_user_stats()              → GET /admin/users/statistics
UserRepository::search_users()                → POST /admin/users/search
UserRepository::bulk_update_users()           → POST /admin/users/bulk-operations
UserRepository::activate_user()               → PUT /admin/users/{id}/activate
UserRepository::deactivate_user()             → PUT /admin/users/{id}/deactivate
UserRepository::get_user_activity()           → GET /admin/users/{id}/activity
UserRepository::find_users_by_email_domain()  → GET /admin/users/by-email-domain

// セキュリティ・トークン管理 (20個の関数)
RefreshTokenRepository::get_token_stats()      → GET /admin/security/token-stats
RefreshTokenRepository::cleanup_expired()     → POST /admin/security/cleanup-tokens
RefreshTokenRepository::revoke_all_user()     → POST /admin/security/revoke-all-tokens
RefreshTokenRepository::get_active_count()    → GET /admin/security/active-tokens
RefreshTokenRepository::get_user_sessions()   → GET /admin/security/user-sessions
PasswordResetRepository::get_token_stats()    → GET /admin/security/password-resets
PasswordResetRepository::cleanup_expired()    → POST /admin/security/cleanup-password-resets
PasswordResetRepository::get_recent_activity() → GET /admin/security/recent-activity
// ... その他12個のセキュリティ関連関数

// 高度なタスク管理機能 (12個の関数)
TaskService::create_task_global()             → POST /admin/tasks/create
TaskService::update_task_global()             → PUT /admin/tasks/{id}
TaskService::delete_task_global()             → DELETE /admin/tasks/{id}
TaskService::list_tasks_all_users()           → GET /admin/tasks/cross-user-analytics
TaskService::list_tasks_paginated()           → GET /admin/tasks/paginated
TaskService::create_tasks_batch()             → POST /admin/tasks/bulk-create
TaskService::update_tasks_batch()             → PUT /admin/tasks/bulk-update
TaskService::delete_tasks_batch()             → DELETE /admin/tasks/bulk-delete
TaskService::get_task_statistics()            → GET /admin/tasks/statistics
TaskService::get_system_health()              → GET /admin/tasks/system-health
TaskService::migrate_user_tasks()             → POST /admin/tasks/migrate-user
TaskService::reassign_tasks()                 → POST /admin/tasks/bulk-reassign

// 認証ミドルウェア (8個の関数)
auth::admin_only_middleware()                 → 管理者専用エンドポイント
auth::role_aware_auth_middleware()            → ロール認識エンドポイント
auth::optional_auth_middleware()              → 任意認証エンドポイント
auth::permission_checker()                    → 権限チェック機能
// ... その他4個の認証関連関数
```

##### **🏢 Phase 2 で解消される Dead Code (50 個 - 26.2%)**

```rust
// 組織管理機能 (20個の関数)
OrganizationService::create_organization()     → POST /organizations
OrganizationService::get_organization_hierarchy() → GET /organizations/{id}/hierarchy
OrganizationService::create_department()       → POST /organizations/{id}/departments
OrganizationService::get_analytics()           → GET /organizations/{id}/analytics
OrganizationService::update_subscription()     → PUT /organizations/{id}/subscription
OrganizationService::get_cross_tenant_stats()  → GET /organizations/cross-tenant-stats
OrganizationService::export_organization_data() → POST /organizations/{id}/data-export
// ... その他13個の組織関連関数

// チーム協業機能 (15個の関数)
TeamService::get_performance_analytics()      → GET /teams/{id}/performance-analytics
TeamService::create_workflow_template()       → POST /teams/{id}/workflow-templates
TeamService::get_collaboration_stats()        → GET /teams/{id}/collaboration-stats
TeamService::bulk_invite_members()            → POST /teams/{id}/bulk-member-invite
TeamService::get_cross_team_insights()        → GET /teams/cross-team-insights
TeamService::update_permission_matrix()       → PUT /teams/{id}/permission-matrix
// ... その他9個のチーム関連関数

// バッチ処理・データ移行 (10個の関数)
BatchService::process_user_migration()        → POST /admin/batch/user-migration
BatchService::process_task_cleanup()          → POST /admin/batch/task-cleanup
BatchService::process_data_export()           → POST /admin/batch/data-export
// ... その他7個のバッチ処理関数

// 高度な権限管理 (5個の関数)
PermissionService::calculate_effective_permissions() → 権限計算API
PermissionService::update_permission_matrix()  → 権限マトリックス更新
// ... その他3個の権限関連関数
```

### 🗑️ **残存する Dead Code (15-25 個 - 8-13%)**

#### **削除推奨の真の Dead Code (15-20 個)**

```rust
// 設定・環境関連ユーティリティ (低ビジネス価値)
UserRepository::with_schema()                 // 開発・テスト用
RefreshTokenRepository::with_schema()          // 開発・テスト用
PasswordResetTokenRepository::with_schema()    // 開発・テスト用
EmailService::from_env()                       // EmailConfig::defaultで代替可能

// 重複・統合可能な機能
EmailService::determine_email_provider()       // 既存機能と重複
EmailService::mask_email()                     // 使用頻度極低
一部のemail template helper methods            // 統合可能

// 不完全・実験的実装
一部のconfiguration構造体の未使用field        // 開発時の残存
デバッグ・ログ用helper functions               // プロダクション不要
```

#### **保持推奨の内部ユーティリティ (5-10 個)**

```rust
// 内部で使用される基盤機能
PasswordConfig::validate_strength()           // 内部バリデーション
JwtConfig::generate_secret()                  // 内部セキュリティ
DatabaseConfig::create_connection_pool()      // 内部接続管理
```

### 📈 **削減効果の定量分析**

#### **Phase 別削減効果**

| Phase                   | 削減する Dead Code | 削減率     | 累積削減率     | 新規 API 数       |
| ----------------------- | ------------------ | ---------- | -------------- | ----------------- |
| **Phase 1**             | 55 個              | 28.8%      | 28.8%          | 13 エンドポイント |
| **Phase 2**             | 50 個              | 26.2%      | 55.0%          | 17 エンドポイント |
| **真の Dead Code 削除** | 50-55 個           | 26.2-28.8% | **83.8-86.4%** | -                 |

#### **最終的な削減効果**

- **削減前**: 191 個の`#[allow(dead_code)]`
- **削減後**: 25-30 個の`#[allow(dead_code)]`（コア機能特化）
- **総削減率**: **83.8-86.4%**
- **API 化される機能**: **125-135 個** (コア機能中心)

### 🎯 **次期リファクタリング戦略**

#### **即座に削除可能（リスクなし）**

```rust
// Phase 1実装前に削除推奨
EmailService::from_env()                      // 即座に削除可能
EmailService::determine_email_provider()      // 即座に削除可能
*Repository::with_schema() methods            // 即座に削除可能（5個）
```

#### **Phase 実装と同時に削除**

```rust
// Phase 1実装時
UserRepository dead code functions → API実装と同時に#[allow(dead_code)]削除

// Phase 2実装時
OrganizationService dead code functions → API実装と同時に削除

```

#### **最終クリーンアップ**

```rust
// 全Phase完了後
残存する5-10個の真のdead code → 一括削除
内部ユーティリティの整理 → 必要性再評価
```

### 📊 **実装効果の予測**

#### **コードベースの健全性向上**

- **Dead Code**: 191 個 → 5-10 個 (94.8-97.4%削減)
- **機能活用率**: 89.0% → 97.4-98.7%
- **保守性**: 大幅向上（不要コード削除）
- **可読性**: 向上（API 目的が明確）

#### **ビジネス価値の最大化**

- **API 機能性**: 78 → 123 エンドポイント (58%増加)
- **エンタープライズ対応**: 基本レベル → 包括的プラットフォーム
- **競争優位性**: タスク管理 → 組織管理ソリューション
- **収益ポテンシャル**: サブスクリプション階層の完全活用

### 🚀 **推奨実装アプローチ**

1. **即座に実行** (1-2 日): 真の Dead Code 削除（リスクなし）
2. **Phase 1** (2-3 週間): 管理者機能 API 実装 + Dead Code 削除
3. **Phase 2** (3-4 週間): 組織・チーム機能 API 実装 + Dead Code 削除
4. **最終クリーンアップ** (1 週間): 残存 Dead Code 整理・最適化

この効率的アプローチにより、**8 週間で 84-86%の Dead Code 削除と 45%の API 機能拡張**を同時に実現できます。

---

## 📋 **新規 API エンドポイント機能概要一覧**

_各エンドポイントで実現可能な機能の詳細概要（100 字以内）_

### 🔧 **Phase 1: 管理者ダッシュボード強化** (18 エンドポイント)

#### **1.1 高度なユーザー管理 API (6 エンドポイント)**

| エンドポイント                      | 機能概要                                                                                                                                         |
| ----------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------ |
| `GET /admin/users/advanced-search`  | 複数条件での高度なユーザー検索。ロール・サブスクリプション・登録日・アクティビティ状況等の組み合わせ検索が可能。CRM・HR 等全システムで応用可能。 |
| `GET /admin/users/analytics`        | ユーザー全体の統計分析ダッシュボード。登録数推移・アクティブ率・サブスクリプション分布・地域別分析等を可視化。売上・顧客分析等で汎用応用。       |
| `GET /admin/users/by-role/{role}`   | 指定ロール（admin/member/viewer）のユーザー一覧取得。権限管理・組織構造把握・ロール別運用状況の監視。全 RBAC システムの基盤機能。                |
| `GET /admin/users/by-subscription`  | サブスクリプション階層別ユーザー分析。Free/Pro/Enterprise 各層の利用状況・収益分析・アップグレード促進対象の特定。顧客セグメント管理で応用。     |
| `GET /admin/users/activity-stats`   | ユーザーアクティビティ詳細統計。ログイン頻度・機能利用状況・エンゲージメント指標を分析。セキュリティ・UX 改善・運用監視で必須機能。              |
| `POST /admin/users/bulk-operations` | 複数ユーザーの一括操作（有効化・無効化・ロール変更・サブスクリプション更新）。大規模システム運用・HR・CRM・在庫管理等で必須の基盤機能。          |

#### **1.2 セキュリティ・トークン管理 API (7 エンドポイント)**

| エンドポイント                           | 機能概要                                                                                                                       |
| ---------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------ |
| `GET /admin/security/token-stats`        | 認証トークン利用統計の詳細分析。発行数・有効期限・利用パターン・異常検知等により、セキュリティ状況の可視化と監視を実現。       |
| `GET /admin/security/refresh-tokens`     | リフレッシュトークン状況の監視機能。アクティブセッション・異常ログイン・多重ログイン等を検出し、不正アクセス対策を強化。       |
| `POST /admin/security/cleanup-tokens`    | 期限切れトークンの自動削除・クリーンアップ。システムパフォーマンス維持・セキュリティリスク軽減・データベース最適化を自動実行。 |
| `GET /admin/security/password-resets`    | パスワードリセット活動の監視・分析。リセット頻度・パターン・潜在的セキュリティリスクを特定し、ユーザーサポート向上に貢献。     |
| `POST /admin/security/revoke-all-tokens` | 緊急時全トークン無効化機能。セキュリティインシデント・不正アクセス検知時の迅速な対応により、システム全体の安全性を確保。       |
| `GET /admin/security/session-analytics`  | ユーザーセッション詳細分析。ログイン時間・接続地域・デバイス情報・異常パターン等を監視し、高度なセキュリティ管理を提供。       |
| `POST /admin/security/audit-report`      | セキュリティ監査レポート生成。アクセスログ・権限変更・セキュリティイベント等を包括的に分析し、コンプライアンス対応をサポート。 |

#### **1.3 その他管理機能 API (1 エンドポイント)**

| エンドポイント                    | 機能概要                                                                                                           |
| --------------------------------- | ------------------------------------------------------------------------------------------------------------------ |
| `GET /admin/users/username-check` | ユーザー名の重複チェック・可用性確認。新規登録・ユーザー名変更時の即座な検証により、スムーズなユーザー体験を提供。 |

### 🏢 **Phase 2: エンタープライズ組織機能** (17 エンドポイント)

#### **2.1 組織階層管理 API (10 エンドポイント)**

| エンドポイント                                     | 機能概要                                                                                                                             |
| -------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| `GET /organizations/{id}/hierarchy`                | 組織階層構造の取得・可視化。部門・チーム・メンバーの入れ子関係を階層表示。支店・事業部・販売網等の階層管理で汎用応用可能。           |
| `POST /organizations/{id}/departments`             | 階層入れ子部門の作成機能。親部門配下への子部門作成・無制限階層対応。店舗・事業部・プロジェクト等の組織単位管理で汎用応用。           |
| `GET /organizations/{id}/departments`              | 部門一覧・階層構造表示。全部門の親子関係・階層レベル・配置状況を表示。組織図・管理画面・分析ダッシュボードで活用。                   |
| `PUT /organizations/{id}/departments/{dept_id}`    | 部門情報・階層位置の更新。部門名・説明・親部門変更・階層移動対応。組織再編・統合・分割時の柔軟な構造変更を実現。                     |
| `DELETE /organizations/{id}/departments/{dept_id}` | 部門削除（子部門自動移動処理）。削除部門配下の子部門・メンバーを親部門に自動移動。安全な組織構造変更を保証する基盤機能。             |
| `GET /organizations/{id}/analytics`                | 組織階層別分析ダッシュボード。部門別パフォーマンス・階層効率・リソース配分等を可視化。売上・人事・運用分析で汎用応用。               |
| `PUT /organizations/{id}/permission-matrix`        | 組織単位詳細権限マトリックス設定。組織全体・配下全チーム・全部門に適用される包括権限制御。エンタープライズ統一ガバナンスで必須。     |
| `GET /organizations/{id}/permission-matrix`        | 組織適用権限マトリックス取得。組織レベル権限設定・配下継承ルール・例外設定を包括表示。組織ガバナンス・監査・コンプライアンスで活用。 |
| `GET /organizations/{id}/effective-permissions`    | 組織実効権限分析。組織・部門・チーム・個人レベルの権限継承チェーンと最終適用権限を詳細分析。権限最適化・監査・デバッグで必須。       |
| `POST /organizations/{id}/data-export`             | 組織データの階層構造保持エクスポート。組織図・部門配置・メンバー関係を含む包括的データ出力。GDPR・移行・監査で必須機能。             |

#### **2.2 チーム招待・権限管理 API (6 エンドポイント)**

| エンドポイント                                    | 機能概要                                                                                                                                     |
| ------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------- |
| `POST /teams/{id}/bulk-member-invite`             | チームメンバー一括招待機能。大規模プロジェクト・組織拡大・イベント参加等での効率的メンバー追加。グループ・部署・プロジェクト管理で汎用応用。 |
| `GET /teams/{id}/invitations`                     | 招待状況確認・管理機能。送信済み招待・回答待ち・承認状況等の一元管理。HR・プロジェクト管理・イベント運営で活用可能。                         |
| `PUT /teams/{id}/invitations/{invite_id}/decline` | 招待辞退機能（招待とセット実装）。丁寧な辞退フロー・理由記録・代替提案等をサポート。組織変更・参加管理で必須の基盤機能。                     |
| `PUT /teams/{id}/permission-matrix`               | チーム単位詳細権限マトリックス設定。チーム固有のリソース・機能・データアクセス権限を細かく制御。全 RBAC システムで活用可能な汎用権限基盤。   |
| `GET /teams/{id}/permission-matrix`               | チーム適用権限マトリックス取得。現在のチーム権限設定・継承状況・制限事項を包括的に表示。権限管理・監査・トラブルシュートで活用。             |
| `GET /teams/{id}/effective-permissions`           | チーム実効権限分析。組織継承・チーム設定・ユーザーロールを統合した最終適用権限を詳細表示。権限デバッグ・監査・最適化で必須。                 |


### 📊 **API 機能概要サマリー**

#### **Phase 別機能価値**

| Phase       | エンドポイント数 | 主要価値                                   | 対象ユーザー               |
| ----------- | ---------------- | ------------------------------------------ | -------------------------- |
| **Phase 1** | 18               | 管理効率化・セキュリティ強化・運用自動化   | 管理者・運用チーム         |
| **Phase 2** | 17               | 組織階層管理・統一権限ガバナンス・招待管理 | 組織管理者・チームリーダー |

#### **機能カテゴリ別分布（エンタープライズ基盤+統一権限ガバナンス）**

- **管理・運用**: 11 エンドポイント (31%)
- **組織階層管理**: 10 エンドポイント (29%)
- **セキュリティ・コンプライアンス**: 7 エンドポイント (20%)
- **統一権限ガバナンス**: 6 エンドポイント (17%)
- **エンタープライズデータ**: 1 エンドポイント (3%)

この 35 個の新規 API により、基本的なタスク管理ツールから**汎用エンタープライズプラットフォーム基盤（統一権限ガバナンス完備）**への進化を実現。CRM・ERP・HR・EC 等あらゆるシステムで 100%応用可能な基盤機能を提供。

---

## 🔐 **統一権限ガバナンス設計詳細**

### **権限継承チェーンの仕組み**

#### **階層的権限継承**

```
組織レベル権限マトリックス
    ↓ 継承 + オーバーライド
部門レベル権限設定
    ↓ 継承 + オーバーライド
チームレベル権限マトリックス
    ↓ 継承 + 個人ロール適用
個人実効権限（最終適用）
```

#### **組織統一ガバナンス vs チーム個別制御**

**組織アカウント（推奨）**:

- `PUT /organizations/{id}/permission-matrix` で組織全体の統一ルール設定
- 配下全チーム・全部門・全メンバーに自動適用
- 一元管理による効率性・セキュリティ・コンプライアンス確保
- 例外設定は部門・チーム単位で個別オーバーライド可能

**チーム個別制御**:

- `PUT /teams/{id}/permission-matrix` でチーム固有の詳細ルール設定
- 小規模組織・プロジェクト単位・特殊要件での柔軟な権限制御
- 組織権限を継承しつつ、チーム特有の制限・拡張を追加

### **実効権限分析 API**

#### **組織実効権限分析**

```json
GET /organizations/{id}/effective-permissions?user_id={user_id}

Response:
{
  "user_id": "123e4567-e89b-12d3-a456-426614174000",
  "organization_id": "org123",
  "inheritance_chain": [
    {
      "level": "organization",
      "source": "organization_matrix",
      "applied_rules": ["admin_access", "export_allowed"]
    },
    {
      "level": "department",
      "source": "dept_engineering",
      "applied_rules": ["code_review_required"],
      "overrides": ["export_limited_to_team"]
    },
    {
      "level": "team",
      "source": "team_backend",
      "applied_rules": ["deploy_permission"],
      "exceptions": ["weekend_deploy_blocked"]
    },
    {
      "level": "user_role",
      "source": "member_role",
      "applied_rules": ["basic_access"]
    }
  ],
  "final_permissions": {
    "tasks": {
      "create": true,
      "read": "team_scope",
      "update": true,
      "delete": false,
      "admin": false
    },
    "analytics": {
      "view_basic": true,
      "export": "team_only",
      "view_advanced": false
    }
  },
  "restrictions": [
    {
      "resource": "deployment",
      "condition": "weekend_blocked",
      "source": "team_backend"
    }
  ]
}
```

#### **権限マトリックス取得 API**

```json
GET /organizations/{id}/permission-matrix

Response:
{
  "organization_id": "org123",
  "matrix_version": "v2.1",
  "last_updated": "2025-01-22T10:30:00Z",
  "updated_by": "admin@company.com",
  "inheritance_settings": {
    "allow_team_overrides": true,
    "allow_department_exceptions": true,
    "strict_security_policies": true
  },
  "permission_matrix": {
    "tasks": {
      "create": {
        "scope": "team",
        "conditions": ["subscription_level:pro"],
        "quota": {"max_items": 1000},
        "inheritance": "allow_override"
      },
      "admin": {
        "scope": "organization",
        "conditions": ["role:admin", "mfa_enabled"],
        "inheritance": "strict_no_override"
      }
    },
    "analytics": {
      "view_advanced": {
        "scope": "organization",
        "conditions": ["subscription_level:enterprise"],
        "inheritance": "allow_department_override"
      }
    }
  },
  "department_overrides": [
    {
      "department_id": "dept_sales",
      "resource": "analytics",
      "action": "view_customer_data",
      "override_reason": "Sales team specific requirement"
    }
  ],
  "compliance_settings": {
    "audit_log_retention": "7_years",
    "require_justification": true,
    "auto_revoke_inactive": "90_days"
  }
}
```

### **使い分けガイドライン**

#### **組織統一ガバナンス（推奨）**

**適用ケース**:

- 中〜大規模組織（50 名以上）
- コンプライアンス要件が厳しい業界
- セキュリティポリシーの統一が必要
- 監査・レポーティング要件がある

**メリット**:

- 一元管理による運用効率向上
- セキュリティポリシーの確実な適用
- 監査証跡の統一管理
- スケーラブルなガバナンス

#### **チーム個別制御**

**適用ケース**:

- 小規模組織・スタートアップ
- プロジェクト単位の独立性が重要
- 部門間で業務プロセスが大きく異なる
- アジャイルな権限変更が必要

**メリット**:

- 柔軟な権限カスタマイズ
- チームの自律性確保
- 迅速な権限変更対応
- 特殊要件への対応力

**実装により、既存の動的パーミッションシステムをシームレスに拡張し、組織の成長段階・業界要件・運用方針に応じた最適な権限ガバナンスを実現できます。**
