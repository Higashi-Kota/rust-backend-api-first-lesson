# Rust Backend API - Feature-based Architecture

## 📌 現在の状態 (2025-07-13)

**Phase 23 完了** - dead_code削減目標達成 ✅
- **達成内容**: 
  - `#[allow(dead_code)]` を5個に削減（src: 4個、tests: 1個）
  - 必要最小限の設定・インフラ系のみ維持
  - 全ユニットテスト: 218 passed ✅
- **現在の課題**: `cargo clippy --all-targets --all-features -- -D warnings` の警告・エラー解消

## 🎯 Clippy警告・エラー完全解消方針

### 基本原則
- **すべての警告・エラーをゼロにする**
- **未使用コードは削除またはAPIとして活用**
- **価値のある機能は新規エンドポイントとして公開**
- **すべてのテストがパスする状態を維持**

### 対応方針
1. **未使用変数・フィールド**
   - 使用予定がある → 実装で活用
   - 使用予定がない → 削除（YAGNI原則）
   - テスト用 → `_` プレフィックスを付与
   - **コメントアウトでの対処は禁止** → 削除か活用の二択

2. **未使用メソッド・関数**
   - APIとして価値がある → エンドポイント化して統合テスト追加
   - 内部実装のみ → 削除
   - テスト用ヘルパー → 維持（例外）
   - **コメントアウトでの対処は禁止** → 削除か活用の二択

3. **型の不一致・借用エラー**
   - 適切な型変換を実装
   - 不要な借用・クローンを削除

4. **横着な対処の禁止**
   - コードのコメントアウトによる警告回避は禁止
   - `#[allow(...)]` アノテーションの追加は禁止
   - 問題の根本的な解決を行うこと

### 統合テスト修正方針
- 正しいインポートパスに修正
- 実データによる検証（モックデータを避ける）
- AAAパターンの適用
- エラーケースの網羅（最低5パターン）

## 🧩 実装ガイドライン

### 1. **ドメイン統合の原則**
- **既存ドメインとの重複・競合は禁止**
- **APIのスラグなど機能別の統一感を意識**
- **パスパラメータは `{param}` 形式を使用（Axum 0.8の仕様）**

### 2. **機能追加の原則：実用的で価値の高い機能に集中**
- **実用性**: 実際のユーザーニーズに基づいているか
- **価値**: 実装コストに見合う価値を提供するか
- **保守性**: 長期的な保守が可能か
- **既存機能との整合性**: 既存のアーキテクチャと調和するか

### 3. **データベース設計の原則**
- **テーブル名は必ず複数形**
- **カラム名は snake_case**
- **標準カラム**: `id` (UUID型), `created_at`, `updated_at`
- **マイグレーションファイル命名規則**: `m{YYYYMMDD}_{連番6桁}_{説明}.rs`

### 4. **dead_code ポリシー**
- `#![allow(dead_code)]` や `#[allow(dead_code)]` の**新規追加は禁止**
- **既存アノテーションからAPIとして価値提供できる場合は積極的に外す**
- **未使用コード・シグネチャ・構造体は削除**
- **例外**: テスト用ヘルパー関数のみ許可

### 5. **プロダクションコードの品質基準**
- **すべての公開APIは実装で使用される**こと
- **テストは実装の動作を検証**するものであること
- **未使用の警告が出ないこと**（dead_code警告を含む）

### 6. **APIセキュリティとルーティング規則**
- **管理者専用APIの原則**: センシティブ情報は必ず `/admin/*` パスで保護
- **認証・認可の設定**: 適切な権限レベルを必ず設定
- **CORS設定**: 本番環境ではワイルドカード禁止

### 7. **CI・Lint 要件**
```bash
cargo clippy --all-targets --all-features -- -D warnings
```
→ **エラー・警告が完全にゼロであること**

```bash
make ci-check-fast
```
→ **すべてのテストにパスすること**

## 🏗️ アーキテクチャ概要

### 現在の構造
```
src/
├── features/          # 機能別モジュール（13個）
│   ├── auth/         # 認証・認可
│   ├── task/         # タスク管理
│   ├── team/         # チーム管理
│   ├── organization/ # 組織管理
│   ├── storage/      # ストレージ
│   ├── gdpr/         # GDPR対応
│   ├── security/     # セキュリティ
│   ├── admin/        # 管理者機能
│   ├── analytics/    # 分析機能
│   ├── subscription/ # サブスクリプション
│   ├── payment/      # 支払い処理
│   ├── user/         # ユーザー管理
│   └── system/       # システム機能
├── api/              # APIの共通定義（AppState等）
├── shared/           # 共通型・ユーティリティ
├── infrastructure/   # 技術基盤
└── main.rs
```

### Feature モジュール標準構造
```
features/{feature_name}/
├── mod.rs           # 公開API定義
├── handlers/        # HTTPハンドラー
├── services/        # ビジネスロジック
├── repositories/    # データアクセス
├── models/          # ドメインモデル
├── dto/             # データ転送オブジェクト
│   ├── mod.rs
│   ├── requests/    # リクエストDTO
│   └── responses/   # レスポンスDTO
└── usecases/        # 複雑なビジネスロジック（オプション）
```

## 🧪 テスト要件

### 統合テスト（Integration Test）

#### **AAA（Arrange-Act-Assert）パターンによる実装**

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

#### **エラーパスの網羅**
```rust
// 各APIエンドポイントに対して最低限以下のケースをテスト
test_endpoint_success()           // 正常系
test_endpoint_validation_error()  // バリデーションエラー
test_endpoint_unauthorized()      // 認証エラー
test_endpoint_forbidden()         // 認可エラー
test_endpoint_not_found()         // リソース不在
```

## 📋 現在進行中のタスク

### Phase 23後のClippy対応

#### 完了済み
- [x] 未使用変数 `feature_usage_data` に `_` プレフィックス追加
- [x] 不要なクローン `organization.subscription_tier.clone()` を削除
- [x] `cloned()` を `copied()` に変更
- [x] 不要な借用 `ref role` を削除

#### 残タスク
1. **統合テストのコンパイルエラー修正**
   - [ ] インポートパスの修正（5ファイル）
     - `use crate::common::{setup_test_app, TestDatabase}` → 正しいパスに修正
   - [ ] 型の不一致解消
     - `&0` → `0`、`&chrono::Utc::now()` → `chrono::Utc::now()`
   - [ ] 存在しないインポートの修正
     - `PermissionCheckRequest` → 正しい型名に修正

2. **未使用警告の解消（多数）**
   - [ ] 未使用フィールドの削除または活用
   - [ ] 未使用メソッドのAPI化または削除
   - [ ] 特に以下のモジュールに注意：
     - `admin/repositories/bulk_operation_history.rs`
     - `analytics/models/daily_activity_summary.rs`
     - `analytics/repositories/`
     - `organization/repositories/`
     - `task/repositories/task_repository.rs`
     - `user/services/user_service.rs`

3. **テストの修正**
   - [ ] `has_feature` メソッドの使用箇所を修正（既に2箇所修正済み）

### 成功指標
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` がエラーなし
- [ ] `make ci-check-fast` で全テストがパス
- [ ] 統合テストが実データで動作確認
- [ ] dead_code警告が5個以下を維持

## 🚀 実装完了後の期待される状態

1. **`cargo clippy`で警告ゼロ**（dead_code警告5個以下）
2. **`make ci-check-fast`ですべてのテストがグリーン**
3. **APIドキュメントと実装が一致**
4. **テストが実装の実際の動作を検証**
5. **プロダクションコードがクリーンで保守しやすい**

## 🔄 次セッションへの引き継ぎ事項

### 現在の状況
1. **Phase 23完了** - dead_code注釈を5個まで削減成功
2. **Clippy対応開始** - 基本的な警告は修正済み、統合テストと未使用警告が残存

### 次セッションで実施すべきこと
1. **統合テストの修正を最優先**
   - 正しいインポートパスを調査・修正
   - 型エラーの解消
   - テストが実際に動作することを確認

2. **未使用コードの整理**
   - YAGNI原則に基づき、使用予定のないコードは削除
   - 価値のある機能はAPIエンドポイント化
   - 各エンドポイントに統合テスト（最低3パターン）を追加

3. **最終確認**
   - `cargo clippy --all-targets --all-features -- -D warnings` でエラーゼロ
   - `make ci-check-fast` で全テストがパス
   - dead_code警告が5個以下を維持

### 注意事項
- **dead_code注釈の新規追加は禁止**
- **未使用コードは価値があればAPI化、なければ削除**
- **すべての新規APIには統合テストを実装**
- **コメントアウトによる警告回避は禁止** → 根本的な解決を行うこと

---
最終更新: 2025-07-13
Phase 23: dead_code削減完了（5個達成）✅
次フェーズ: Clippy警告・エラーの完全解消（進行中）