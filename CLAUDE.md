# ✅ Phase 2 実装計画

Phase 1で基本的な機能統合を完了しました。Phase 2では、残り120箇所以上の`#[allow(dead_code)]`アノテーションを削除し、高度な権限管理システムとトランザクション管理機能を実装します。

---

## 📌 Phase 2 重点実装対象

### 1. **Permissionシステムの完全統合**（最優先 - 24箇所）
**ファイル**: `task-backend/src/domain/permission.rs`
- [ ] `Permission::new` - 権限の動的生成
- [ ] `Permission::read_own` - 自己リソース読み取り権限
- [ ] `Permission::write_own` - 自己リソース書き込み権限
- [ ] `Permission::admin_global` - グローバル管理者権限
- [ ] `Permission::matches` - 権限マッチング
- [ ] `PermissionResult` の判定メソッド群（is_allowed, is_denied等）
- [ ] `can_perform_action` の本格実装と活用

### 2. **認証システムの高度化**（19箇所）
**ファイル**: `task-backend/src/middleware/auth.rs`
- [ ] カスタム認証エラーレスポンスの活用
- [ ] 詳細な認証ログの実装
- [ ] トークンリフレッシュ機能の改善
- [ ] セッション管理機能の統合

### 3. **権限ユーティリティの統合**（17箇所）
**ファイル**: `task-backend/src/utils/permission.rs`
- [ ] `Privilege` 階層別権限設定
- [ ] リソース別アクセス制御
- [ ] 権限の継承メカニズム
- [ ] 権限キャッシュ機能

### 4. **APIエラーハンドリングの統一**（9箇所）
**ファイル**: `task-backend/src/api/dto/common.rs`
- [ ] `ApiError::new` - 基本エラー生成
- [ ] `ApiError::with_details` - 詳細付きエラー
- [ ] `ApiError::validation_error` - バリデーションエラー
- [ ] 各種エラータイプ別メソッド（unauthorized, forbidden, not_found等）

---

## 🧩 実装ガイドライン

### 1. **ドメイン統合の原則**

* **既存ドメインとの重複・競合は禁止**
  * 同じ意味の別表現、似たが異なるロジック、バリエーション増加は避ける
* **「亜種」API・ドメイン定義の増加は避ける**
  * 新規定義が必要な場合は、**既存の責務・境界に統合**できるか再検討

### 2. **dead\_code ポリシー**

* `#![allow(dead_code)]` や `#[allow(dead_code)]` の**新規追加は禁止**
* **既存アノテーションからAPIとして価値提供できる場合は積極的に外す**
* **未使用コード・シグネチャ・構造体は削除**
  * ただし、テストで使用されているコードは、実装で適切に活用する

### 3. **プロダクションコードの品質基準**

* **すべての公開APIは実装で使用される**こと
* **テストは実装の動作を検証**するものであること
* **未使用の警告が出ないこと**（dead_code警告を含む）

### 4. **実装統合の優先順位**

1. **セキュリティ関連機能**（権限チェック、認証）
2. **データ整合性機能**（トランザクション、リトライ）
3. **ユーザー体験向上機能**（詳細なエラーメッセージ、メタデータ）
4. **運用支援機能**（ログ、監査証跡）

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
  3. **将来の拡張用API** → 現時点で価値提供できるよう実装

* dead\_code で検知される要素への対応：
  * **公開API（pub）** → 実装での活用を検討
  * **内部実装（非pub）** → 使用されていなければ削除
  * **テスト用ユーティリティ** → `#[cfg(test)]` で適切に分離

---

## 📋 実装例

### 1. **トランザクション管理の活用例**

```rust
// ❌ 現在の実装（基本的なトランザクションのみ）
pub async fn create_team_with_members(&self, ...) -> AppResult<Team> {
    self.db.execute_service_transaction(move |txn| {
        // チーム作成とメンバー追加
    }).await
}

// ✅ 改善後（TransactionOperationsを活用）
pub async fn create_team_with_members(&self, ...) -> AppResult<Team> {
    self.db.execute_service_transaction(move |txn| {
        let ops = TransactionOperations::new(txn);
        
        // 操作1: チーム作成
        let team = ops.execute("create_team", 
            team_repo.create_team(&team_data)
        ).await?;
        
        // 操作2: メンバー追加
        ops.execute("add_members", 
            team_repo.add_members(&member_data)
        ).await?;
        
        Ok(team)
    }).await
}
```

### 2. **権限チェックの活用例**

```rust
// ❌ 現在の実装（基本的な権限チェックのみ）
if user.is_admin() {
    // 処理
}

// ✅ 改善後（詳細な権限チェック）
if user.can_update_resource("team", Some(team.owner_id)) {
    // より細かい権限制御
}

// 動的権限チェック
match user.can_perform_action("team", "invite_member", Some(team_id)) {
    PermissionResult::Allowed { scope, .. } => {
        // スコープに応じた処理
    }
    PermissionResult::Denied { reason } => {
        return Err(AppError::forbidden(reason));
    }
}
```

### 3. **APIレスポンスの活用例**

```rust
// ❌ 現在の実装（基本的なJSON返却）
Ok(Json(TeamResponse { team }))

// ✅ 改善後（詳細なレスポンス）
Ok(Json(ApiResponse::success_with_metadata(
    "Team created successfully",
    TeamResponse { team },
    json!({
        "members_added": member_count,
        "subscription_tier": team.subscription_tier,
        "features_enabled": team_features
    })
)))
```

---

## 🚀 実装完了後の期待される状態

1. **`cargo clippy`で警告ゼロ**（dead_code警告を含む）
2. **`make ci-check`ですべてのテストがグリーン**
3. **APIドキュメントと実装が一致**
4. **テストが実装の実際の動作を検証**
5. **プロダクションコードがクリーンで保守しやすい**

---

## 📈 進捗状況

### ✅ Phase 1（完了）- 2024/12/29
初回の実装で以下を達成：

#### 実装済み機能
1. **トランザクション管理**
   - `execute_with_retry` - RoleServiceのassign_role_to_userで活用

2. **APIレスポンスの詳細メソッド**
   - `ApiResponse::success_message` - user_handler.rsで活用
   - `ApiResponse::success_with_metadata` - analytics_handler.rsで活用
   - `OperationResult::created` - analytics_handler.rsのexport機能で活用
   - `OperationResult::deleted` - organization_hierarchy_handler.rsで活用

3. **権限管理システム**
   - `has_subscription_tier` - チーム作成時のサブスクリプション階層チェック（3チームまで）
   - `can_access_user` - user_handler.rsで活用
   - `can_update_resource` - user_handler.rs, role_service.rsで活用
   - `can_create_resource` - role_service.rsで活用
   - `can_delete_resource` - role_service.rsで活用

#### 成果
- **320個のテストすべて成功** ✅
- **CI完全通過** ✅
- **コンパイル時のdead_code警告ゼロ** ✅

---

## 🚧 Phase 2 実装手順

### Step 1: 現状分析（実施中）
```bash
# 1. 詳細なdead_codeリストの生成
rg "#\[allow\(dead_code\)\]" task-backend/src -B 2 -A 2 > dead_code_analysis.txt

# 2. ファイル別統計
# permission.rs: 24箇所
# auth.rs: 19箇所  
# utils/permission.rs: 17箇所
# common.rs: 9箇所
# その他: 51箇所
```

### Step 2: 実装順序
1. **Permission システム** → 権限管理の基盤
2. **APIエラーハンドリング** → 統一的なエラー処理
3. **認証システム** → セキュリティ強化
4. **その他のドメインモデル** → 機能拡張

### Step 3: 実装パターン
```rust
// 1. テストでの使用を確認
// 2. 適切なハンドラー/サービスで活用
// 3. #[allow(dead_code)]を削除
// 4. テスト実行で動作確認
```

---

## 🎯 Phase 2 目標

1. **すべての`#[allow(dead_code)]`削除** (120箇所)
2. **動的権限管理システムの実装**
3. **統一的エラーハンドリング**
4. **CI完全通過（警告ゼロ）**

---
