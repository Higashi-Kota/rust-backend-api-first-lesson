# Rust Backend API - Feature-based Architecture

## 📌 現在の状態 (2025-07-12)

**Phase 23 進行中** - dead_code完全削除への取り組み
- **前回セッション成果**: 
  - `#[allow(dead_code)]` 16個 → 31個（clippy対応で一時的に増加）
  - `cargo clippy --all-targets --all-features -- -D warnings` ✅ エラーゼロ達成
  - 全テスト: 528 passed, 0 failed, 1 ignored ✅
- **現在の課題**: dead_codeアノテーションを実質5個程度まで削減（~/higashi-wrksp/aaaと同等レベル）

## 🎯 dead_code削減ポリシー

### 基本原則
- `#![allow(dead_code)]` や `#[allow(dead_code)]` の**新規追加は禁止**
- **既存アノテーションからAPIとして価値提供できる場合は積極的に外す**
- **未使用コード・シグネチャ・構造体は削除**
- **例外**: テスト用ヘルパー関数のみ許可
  - `AppConfig::for_testing`
  - `setup_test_app`
  - `TestDatabase::_container`

### 対応優先順位
1. **テストでのみ使用** → 実装で活用するよう統合
2. **どこでも未使用** → 即座に削除
3. **将来の拡張用** → 削除（YAGNI原則）
4. **公開API（pub）** → 実装での活用を検討
5. **内部実装（非pub）** → 使用されていなければ削除

### 削除時の統合テスト要件
新規APIとして活用する場合、以下3パターンのテストを必須とする：
```rust
#[tokio::test]
async fn test_feature_success() { /* 正常系 */ }

#[tokio::test]
async fn test_feature_invalid_data() { /* 異常系 */ }

#[tokio::test]
async fn test_feature_forbidden() { /* 権限エラー */ }
```

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

## 📋 Phase 23: dead_code完全削除

### 現状分析（2025-07-12）
- **総dead_codeアノテーション数**: 31個
- **目標**: 5個以下（~/higashi-wrksp/aaaと同等）
- **削減必要数**: 26個

### dead_code箇所の分類と対応方針

#### 1. 設定・インフラ系（4個） - 維持推奨
```
- task-backend/src/config/app.rs:14,16,58
- task-backend/src/infrastructure/email/mod.rs:39
```
→ 設定フィールドは将来的に使用する可能性が高いため維持

#### 2. Public API系（12個） - 実装での活用を検討
```
- infrastructure/utils/permission.rs:177
- features/user/services/user_service.rs:162
- features/organization/services/organization.rs:702
- features/analytics/repositories/*.rs（4箇所）
- features/analytics/services/feature_tracking.rs:47
- features/auth/repositories/login_attempt_repository.rs:79
- features/auth/handlers/middleware.rs:83
- features/subscription/models/history.rs:115
- features/subscription/services/subscription.rs:185
```
→ これらは公開APIとして価値があるため、実装での活用を検討

#### 3. モジュールレベル（15個） - 個別精査が必要
```
- features/task/repositories/task_repository.rs
- features/task/services/task.rs
- features/user/services/user_service.rs
- 他のサービス・リポジトリ層
```
→ モジュール内の未使用メソッドを個別に確認し、必要に応じて削除

### 実装計画

#### ステップ1: テスト駆動での活用（優先度: 高）
1. **analytics系メソッドの活用**
   - `get_daily_summary`、`get_feature_metrics`など
   - 管理者向けダッシュボードAPIとして実装
   - `/admin/analytics/*`エンドポイントの追加

2. **user系メソッドの活用**
   - `get_user_activity_stats`
   - ユーザープロフィール拡張APIとして実装
   - `/users/{id}/stats`エンドポイントの追加

#### ステップ2: 不要コードの削除（優先度: 中）
1. **未使用の内部メソッド削除**
   - privateメソッドで参照されていないもの
   - テストでも使用されていないヘルパー関数

2. **レガシーコードの削除**
   - コメントアウトされたコード
   - 古いAPIの残骸

#### ステップ3: リファクタリング（優先度: 低）
1. **モジュールレベルのallow削除**
   - 個別メソッドへの移動
   - より細かい粒度での制御

### 成功指標
- [ ] `#[allow(dead_code)]`が5個以下
- [ ] `cargo clippy --all-targets --all-features -- -D warnings`がエラーなし
- [ ] 全テストがパス（ignoredなし）
- [ ] 新規追加したAPIに統合テストが存在

## 📋 Phase 24: プロダクション品質達成

### 目的
CI/CD要件を完全に満たし、プロダクションレディな状態を実現

### 達成基準
1. **Lintクリーン**
   ```bash
   cargo clippy --all-targets --all-features -- -D warnings
   # エラー・警告が完全にゼロ
   ```

2. **テスト完全パス**
   ```bash
   make ci-check-fast
   # すべてのテストがグリーン（ignored: 0）
   ```

3. **コード品質**
   - dead_code警告: 5個以下（必要最小限）
   - 未使用imports: 0個
   - テストカバレッジ: 80%以上

### 実装内容

#### 1. 統合テストの品質向上
- **AAA（Arrange-Act-Assert）パターンの徹底**
- **実データによる検証**（ハードコード値の排除）
- **エラーパスの網羅**（最低5パターン/エンドポイント）

#### 2. セキュリティ強化
- **管理者専用APIの徹底**
  - `/admin/*`パスの権限チェック
  - センシティブ情報の保護
- **CORS設定の本番対応**
  - 環境変数による制御
  - ワイルドカード禁止

#### 3. パフォーマンス最適化
- **N+1クエリの解消**
- **適切なインデックス設計**
- **バッチ処理の最適化**

### 移行準備（将来のワークスペース化）
```
rust-backend-api/
├── Cargo.toml          # ワークスペース定義
├── crates/
│   ├── shared/         # 共通型・ユーティリティ
│   ├── core/           # コアドメイン
│   ├── infrastructure/ # 技術基盤
│   └── features/       # 各featureクレート
└── apps/
    └── api-server/     # メインAPIサーバー
```

## 🚀 即座に実行可能なタスク

### Phase 23タスク（dead_code削減）
1. **analytics APIの実装**
   - [ ] `/admin/analytics/daily-summary`
   - [ ] `/admin/analytics/feature-usage`
   - [ ] 統合テスト追加（3パターン/エンドポイント）

2. **user統計APIの実装**
   - [ ] `/users/{id}/activity-stats`
   - [ ] 統合テスト追加

3. **未使用コードの削除**
   - [ ] privateメソッドの精査
   - [ ] テスト未使用コードの削除

### Phase 24タスク（品質達成）
1. **ignoredテスト修正**（1件）
2. **統合テストの品質向上**
3. **セキュリティ監査**
4. **パフォーマンス測定**

## 📝 重要な注意事項

1. **破壊的変更の禁止**
   - 既存APIの後方互換性を維持
   - データベーススキーマの互換性維持

2. **段階的な改善**
   - 一度に大量の変更を避ける
   - 各ステップでテストを実行

3. **ドキュメント化**
   - 新規APIは必ずドキュメント化
   - 変更履歴の記録

---
最終更新: 2025-07-12
Phase 23: dead_code削減作業開始、目標5個以下