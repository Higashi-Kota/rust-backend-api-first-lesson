## 🧩 実装ガイドライン

### 1. **ドメイン統合の原則**

* **既存ドメインとの重複・競合は禁止**
  * 同じ意味の別表現、似たが異なるロジック、バリエーション増加は避ける
  * APIのスラグなど機能別の統一感を意識
  * APIのスラグなどルーティングの重複排除＆集約統合を意識
* **「亜種」API・ドメイン定義の増加は避ける**
  * 新規定義が必要な場合は、**既存の責務・境界に統合**できるか再検討

### 2. **dead\_code ポリシー**

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

### 3. **プロダクションコードの品質基準**

* **すべての公開APIは実装で使用される**こと
* **テストは実装の動作を検証**するものであること
* **未使用の警告が出ないこと**（dead_code警告を含む）

### 4. **CI・Lint 要件**

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

## 📊 最終dead_code解消設計（2025-01-02）

### 現在の状況分析

#### 残存dead_code（7個）の内訳

1. **テスト専用（4個）** - そのまま維持
   - `AppConfig::for_testing()` (config.rs:199)
   - `RefreshTokenRepository::with_schema()` (refresh_token_repository.rs:24)
   - `UserRepository::with_schema()` (user_repository.rs:24)
   - `PasswordResetTokenRepository::with_schema()` (password_reset_token_repository.rs:24)

2. **未使用メソッド（2個）** - 詳細分析結果
   - `UserClaims::is_member()` (user_model.rs:227)
   - `UserRepository::find_all_with_roles()` (user_repository.rs:287)

3. **組織モデル（1個）** - 詳細分析結果
   - `Organization` impl全体 (organization_model.rs:67)

---

### 🔍 詳細分析と推奨アプローチ

#### 1. `UserClaims::is_member()` の分析

**実装内容**:
```rust
pub fn is_member(&self) -> bool {
    if let Some(ref role) = self.role {
        PermissionChecker::is_member(role)
    } else {
        PermissionChecker::check_permission_by_role_name(
            &self.role_name,
            PermissionType::IsMember,
            None,
        )
    }
}
```

**分析結果**:
- `is_admin()`メソッドの対となる実装
- 権限チェックの一貫性のために必要
- `PermissionChecker`への単なる委譲ではなく、Claims内のロール情報を活用

**推奨**: **維持して活用**
- 権限チェックAPIで`is_member`フィールドとして公開
- 例: `GET /permissions/check`のレスポンスに含める

#### 2. `UserRepository::find_all_with_roles()` の分析

**実装内容**:
```rust
pub async fn find_all_with_roles(&self) -> Result<Vec<SafeUserWithRole>, DbErr> {
    // ページネーションなしで全ユーザーを取得
    let results = UserEntity::find()
        .join(JoinType::InnerJoin, user_model::Relation::Role.def())
        .select_also(RoleEntity)
        .order_by(user_model::Column::CreatedAt, Order::Desc)
        .all(&self.db)
        .await?;
    // ...
}
```

**分析結果**:
- `find_all_with_roles_paginated()`が存在し、同じ機能を提供
- ページネーションなしの全件取得はパフォーマンスリスク
- 特別な用途（データエクスポート等）以外では不要

**推奨**: **削除**
- ページネーション版で十分
- 全件必要な場合は高いpage_sizeを指定

#### 3. `Organization` implブロックの分析

**実装メソッド一覧**:
```rust
impl Organization {
    - get_subscription_tier()     // 階層情報の取得
    - get_settings()             // 設定JSONのパース
    - update_settings()          // 設定の更新
    - update_subscription_tier() // 階層と制限の更新
    - can_add_teams()           // チーム追加可否チェック
    - can_add_members()         // メンバー追加可否チェック
    - update_name()             // 名前の更新
    - update_description()      // 説明の更新
    - to_organization()         // SeaORM→ドメインモデル変換
    - from_organization()       // ドメインモデル→SeaORM変換
}
```

**分析結果**:
- ビジネスロジックを含む重要なドメインモデル
- サブスクリプション階層による制限管理
- 既にDBスキーマとサービス層が存在

**推奨**: **維持して組織管理APIで活用**
- 組織の容量管理API（`can_add_teams`, `can_add_members`）
- 組織設定管理API（`update_settings`）
- サブスクリプション管理との統合

---

### 🎯 実装計画

#### 短期対応（1個削減）
1. **`UserRepository::find_all_with_roles()`を削除**
   - 使用箇所なし、ページネーション版で代替可能
   - **結果**: dead_code 7個 → 6個

#### 中期対応（2個活用）
2. **`UserClaims::is_member()`を権限チェックAPIで活用**
   ```yaml
   # 既存の GET /permissions/check を拡張
   response:
     allowed: boolean
     is_admin: boolean
     is_member: boolean  # 追加
     permissions: array
   ```

3. **Organization implを組織管理APIで活用**
   ```yaml
   # 新規エンドポイント
   GET /organizations/{id}/capacity
     response:
       can_add_teams: boolean
       can_add_members: boolean
       current_teams: number
       max_teams: number
       current_members: number
       max_members: number
   
   PATCH /organizations/{id}/settings
     request:
       settings: object
   ```

---

### ✅ 最終状態の設計

#### プロダクションコード（dead_code: 4個）
1. テスト専用メソッド（4個） - `#[allow(dead_code)]`のまま維持
   - 理由: テストでのみ使用、`#[cfg(test)]`への変更は任意

#### 活用されるコード
1. `UserClaims::is_member()` - 権限チェックAPIで使用
2. `Organization` impl - 組織管理APIで使用

#### 削除されるコード
1. `UserRepository::find_all_with_roles()` - 不要

### 📈 効果
- dead_codeの明確な理由付け
- ドメインロジックの適切な活用
- APIの機能拡張
