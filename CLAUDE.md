# Rust Backend API - Feature-based Architecture

## 📌 現在の状態 (2025-07-13)

**Phase 25 開始** - クレート分割によるビルド最適化
- **前フェーズ達成内容**: 
  - Phase 23: `#[allow(dead_code)]` を5個に削減（src: 4個、tests: 1個）
  - Phase 24: リファクタリング完了
  - すべての単体テストと統合テストがパス ✅
- **現在の目標**: クレート分割によるビルド時間の最適化

## 🚀 Phase 25: クレート分割によるビルド最適化

> **重要**: クレート分割作業を実施する際は必ず[CLAUDE-GUIDELINES-CRATE-SPLITTING.md](./CLAUDE-GUIDELINES-CRATE-SPLITTING.md)を参照してください。このガイドラインには詳細な設計原則、実装手順、トラブルシューティングが記載されています。

### 作業の基本原則
1. **構造変更のみに集中** - 新規機能の追加は一切行わない
2. **コード品質の維持** - 警告を放置せず根本的に解決
3. **テストの継続的パス** - すべての単体テストと統合テストが常にパス

### 現状分析（2025-07-13）
- **コードベース規模**: 51,147行（273ファイル）
- **現在のビルド時間**: 2分以上（クリーン・インクリメンタル共に）
- **単一モノリシッククレート**: task-backend
- **問題点**: 
  - 小さな変更でも全体の再コンパイルが発生
  - 並列ビルドが不可能
  - 依存関係が複雑に絡み合っている

### クレート分割の効果（確実に効果あり）
- **並列ビルドによる高速化**: 各クレートを並行してビルド可能
- **インクリメンタルビルドの改善**: 変更箇所のみ再ビルド
- **キャッシュ効率の向上**: クレート単位でキャッシュ
- **依存関係の明確化**: 循環依存の解消

### 目標
- **ビルド時間を50%以上短縮**（2分→1分以下）
- **インクリメンタルビルドを10秒以内に**
- **並列ビルドの最大化**
- **依存関係の最適化**

### クレート分割戦略（依存関係分析に基づく）

#### 依存関係の現状
- **相互依存**: auth ↔ user, auth ↔ security
- **中心モジュール**: user（最多参照）, auth（認証基盤）
- **独立モジュール**: payment, storage, gdpr, system

#### 循環依存の解決方針
1. **共通トレイトの抽出**: `shared-core`クレートに共通インターフェースを定義
2. **依存の逆転**: 具象型への依存をトレイトへの依存に変更
3. **イベント駆動**: 直接的な相互参照を避ける

#### 1. **基盤層（Foundation Layer）**
- [ ] `common`: 基本型・エラー定義（依存なし）
  - types/: UUID, DateTime等の基本型
  - errors/: AppError, Result型定義
  - traits/: Repository等の基本トレイト
- [ ] `infrastructure`: 技術基盤（依存: common）
  - database/: DB接続プール、トランザクション管理
  - redis/: Redis接続、キャッシュ
  - config/: 環境設定
  - external/: メール送信、S3等の外部サービス

#### 2. **コア層（Core Layer）**
- [ ] `shared-core`: 共有コア（依存: common）
  - domain/: 共有ドメインモデル
  - services/: 共有サービストレイト
  - traits/: Authenticatable, RoleProvider等の共通インターフェース
- [ ] `user-core`: ユーザー基盤（4,837行）
  - models/: User, UserProfile
  - traits/: UserRepository, UserService trait
  - dto/: 基本的なUser DTO
- [ ] `auth-core`: 認証基盤（4,535行）
  - models/: JWT, Session, Token
  - traits/: AuthService, TokenProvider trait
  - security/: 認証・認可の基本型
- [ ] `security-core`: セキュリティ基盤（7,722行）
  - models/: Role, Permission
  - traits/: SecurityService trait
  - policies/: セキュリティポリシー定義

#### 3. **機能層（Feature Layer）**
##### 第1段階（独立性が高い）
- [ ] `payment`: 支払い処理（1,170行）
- [ ] `storage`: ストレージ（1,995行）
- [ ] `gdpr`: GDPR対応（1,229行）
- [ ] `system`: システム機能（468行）

##### 第2段階（依存関係を整理後）
- [ ] `task`: タスク管理（2,969行）
- [ ] `team`: チーム管理（5,491行）
- [ ] `organization`: 組織管理（6,029行）
- [ ] `analytics`: 分析機能（2,740行）

##### 第3段階（複雑な依存）
- [ ] `admin`: 管理者機能（3,010行）
- [ ] `subscription`: サブスクリプション（2,743行）

#### 4. **統合層（Integration Layer）**
- [ ] `api`: HTTPハンドラー・ルーティング
  - 全機能の統合
  - ミドルウェア
  - main.rs

### 実装タスク

#### Phase 25-1: 基盤準備と循環依存の解消
1. [x] **現在の状況分析**（完了）
   - コードベース: 51,147行
   - ビルド時間: 2分以上
   - 相互依存: auth ↔ user, auth ↔ security

2. [ ] **循環依存の解消**
   - [ ] shared-coreクレートの作成
     - Authenticatable trait（auth ↔ user解消用）
     - RoleProvider trait（auth ↔ security解消用）
     - UserProvider, AuthenticationProvider等の共通インターフェース
   - [ ] 具象型への依存をトレイトへの依存に変更
   - [ ] 依存方向を一方向に整理（user → auth → securityの順）
   - [ ] **必要に応じて追加のトレイトやモジュール構成を柔軟に調整**

3. [ ] **ワークスペース構造の準備**
   - ルートCargo.tomlをワークスペース化
   - crates/ディレクトリの作成
   - 共通依存関係の設定

#### Phase 25-2: 基盤クレートの作成
1. [ ] **commonクレートの作成**
   - src/shared/types → common/src/types
   - src/shared/errors → common/src/errors
   - Repository等の基本トレイト定義
   - 依存: なし

2. [ ] **infrastructureクレートの作成**
   - src/infrastructure → infrastructure/src
   - DB/Redis接続の分離
   - 環境設定の移行
   - 依存: common

3. [ ] **shared-coreクレートの作成**
   - 共通インターフェース定義
   - Authenticatable, RoleProvider等のトレイト
   - 循環依存解消のための共通型
   - 依存: common

#### Phase 25-3: コア層と機能層の分離
##### コア層の作成
1. [ ] **user-core, auth-core, security-coreクレート作成**
   - Userモデルとトレイト（user-core）
   - JWT, Session管理（auth-core）
   - Role, Permission管理（security-core）
   - 依存: common, shared-core
   - 合計: 17,094行

##### 第1段階（独立モジュール）
2. [ ] **payment, storage, gdpr, systemクレート作成**
   - 各4クレートを並行して作成
   - 依存: common, infrastructure
   - 合計: 4,862行（高速ビルド）

##### 第2段階（ビジネス機能）
3. [ ] **task, team, organization, analyticsクレート作成**
   - 依存関係を整理しながら分離
   - 依存: 各種coreクレート
   - 合計: 17,229行

##### 第3段階（統合機能）
4. [ ] **admin, subscriptionクレート作成**
   - 複数機能に依存する統合的な機能
   - 依存: 複数の機能クレート
   - 合計: 5,753行

#### Phase 25-4: 統合とパフォーマンス検証
1. [ ] **apiクレートの作成**
   - 全ハンドラーの統合
   - ルーティング設定
   - main.rsを最小化

2. [ ] **ビルドパフォーマンス検証**
   - クリーンビルド: 目標1分以内
   - インクリメンタル: 目標10秒以内
   - 並列度の確認

3. [ ] **テストとCI/CDの更新**
   - 各クレートの独立テスト
   - 統合テストの配置
   - GitHub Actionsの並列化

### 成功指標
- [ ] フルビルド時間が50%以上短縮（2分→1分以内）
- [ ] インクリメンタルビルドが10秒以内（小規模変更時）
- [ ] 各クレートの独立したテストが可能
- [ ] 並列ビルドでCPUコア数に応じた高速化
- [ ] cargo clippyが全クレートでパス
- [ ] **すべての単体テストと統合テストがパス**

### 予想されるクレート構成
```
crates/
├── common/          # 基本型・エラー（依存なし）
├── infrastructure/  # 技術基盤（依存: common）
├── shared-core/     # 共有インターフェース（依存: common）
├── user-core/       # ユーザー基盤（依存: common, shared-core）
├── auth-core/       # 認証基盤（依存: common, shared-core）
├── security-core/   # セキュリティ（依存: common, shared-core）
├── payment/         # 支払い（依存: common, infrastructure, user-core）
├── storage/         # ストレージ（依存: common, infrastructure）
├── gdpr/           # GDPR（依存: common, user-core）
├── system/         # システム（依存: common）
├── task/           # タスク（依存: common, user-core, auth-core）
├── team/           # チーム（依存: common, user-core, auth-core）
├── organization/   # 組織（依存: common, user-core, team）
├── analytics/      # 分析（依存: common, organization）
├── admin/          # 管理者（依存: 複数）
├── subscription/   # サブスク（依存: common, user-core, payment）
└── api/            # 統合API（依存: 全て）
```

### 制約とテスト要件

1. **必須制約**
   - **すべての単体テストと統合テストが常にパスすること**
   - 既存の機能を一切壊さないこと
   - APIの互換性を維持すること
   - **新規機能の追加は一切行わない**（構造変更のみに集中）

2. **コード品質の厳守**
   - **コメントアウトによる横着は絶対禁止**
   - **`#[allow(dead_code)]`の新規追加は禁止**
   - **`#[allow(unused_imports)]`の新規追加は禁止**
   - 警告が出たら根本的に解決すること
   - 使用しないコードは削除、使用するコードは適切に配置

3. **テスト戦略**
   - 各クレートで独立したユニットテスト
   - apiクレートで統合テスト実施
   - 移行の各段階でフルテスト実行

4. **段階的移行**
   - 1つのクレートずつ移行
   - 各段階でビルド・テスト確認
   - 問題があれば即座にロールバック可能な構造

### ビルド最適化のベストプラクティス
1. **最小限の公開API**
   - pub使用の最小化
   - 内部実装の隠蔽
   - 安定したインターフェース

2. **スマートな依存関係**
   - 軽量な依存の選択
   - features flagの活用
   - dev-dependenciesの分離

3. **並列化の最大化**
   - 独立したクレート構造
   - 依存グラフの平坦化
   - ビルドパイプラインの最適化

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

### Phase 25: クレート分割によるビルド最適化

#### 現在のフェーズ: Phase 25-1 基盤準備
- [x] コードベース分析完了（51,147行、273ファイル）
- [x] ビルド時間計測完了（2分以上）
- [x] 依存関係分析完了（相互依存の特定）
- [ ] 循環依存の解消作業
- [ ] ワークスペース構造の実装

#### 確認された問題
- **ビルド時間**: クリーン・インクリメンタル共に2分以上
- **循環依存**: auth ↔ user, auth ↔ security
- **モノリシック**: 単一クレートで全機能を含む

### ビルド最適化の進捗
- [x] ベースライン計測（2分以上）
- [x] クレート分割設計（16クレート構成）
- [ ] 循環依存の解消
- [ ] 基盤クレートの作成
- [ ] 段階的な機能分離
- [ ] パフォーマンス検証

### 次の作業
1. **auth ↔ user の循環依存解消**
   - UserAuthTrait等のインターフェース抽出
   - 依存方向の整理
2. **commonクレートの作成**
   - shared/types, shared/errorsの移行
   - 基本トレイトの定義

## 🚀 実装完了後の期待される状態

1. **`cargo clippy`で警告ゼロ**（dead_code警告5個以下）
2. **`make ci-check-fast`ですべてのテストがグリーン**
3. **APIドキュメントと実装が一致**
4. **テストが実装の実際の動作を検証**
5. **プロダクションコードがクリーンで保守しやすい**

## 🔄 次セッションへの引き継ぎ事項

### 現在の状況
1. **Phase 24完了** - リファクタリング完了
2. **Phase 25開始** - クレート分割によるビルド最適化タスクを定義

### 次セッションで実施すべきこと
1. **Phase 25-1: 循環依存の解消（最優先）**
   - [ ] shared-coreクレートの作成
     - Authenticatable trait定義
     - RoleProvider, PermissionChecker trait定義
     - UserProvider, AuthenticationProvider trait定義
   - [ ] 既存コードのトレイト抽出
   - [ ] 依存関係の整理とテスト

2. **Phase 25-2: 基盤クレート作成**
   - [ ] commonクレートの実装
     - src/shared/types → crates/common/src/types
     - src/shared/errors → crates/common/src/errors
     - 基本トレイト定義
   - [ ] infrastructureクレートの実装
     - src/infrastructure → crates/infrastructure/src
     - DB/Redis接続の分離

3. **Phase 25-3: コア層の実装**
   - [ ] user-coreクレート（Userモデル基盤）
   - [ ] auth-coreクレート（認証基盤）
   - [ ] security-coreクレート（セキュリティ基盤）

4. **Phase 25-4: 段階的な機能分離**
   - 第1段階: payment, storage, gdpr, system（独立性高）
   - 第2段階: task, team（user/auth依存）
   - 第3段階: organization, analytics（複雑な依存）
   - 第4段階: admin, subscription（統合機能）

### 注意事項
- **作業開始前に必ず[CLAUDE-GUIDELINES-CRATE-SPLITTING.md](./CLAUDE-GUIDELINES-CRATE-SPLITTING.md)を参照**
- **ビルド時間は必ず計測して比較すること**
- **既存の機能を壊さないよう段階的に移行**
- **各クレートは独立してテスト可能にする**
- **公開APIは最小限に留める**
- **すべての単体テストと統合テストが常にパスすることを確認**
- **新規機能追加は禁止（構造変更のみ）**
- **コメントアウトでの警告回避は禁止**
- **`#[allow(dead_code)]`と`#[allow(unused_imports)]`の新規追加は禁止**

### フォルダ構成の柔軟性
- **開発のしやすさを最優先**に、フォルダ構成は柔軟に調整可能
- **ベストプラクティスに基づいた最適化**を積極的に実施
- 予想外の依存関係や構造的な問題が発見された場合、より良い構成に変更可能
- **最終的な成果（ビルド時間短縮）が達成できれば、構成の詳細は柔軟に対応**

### 期待される成果
- **ビルド時間50%以上の短縮**（最重要目標）
- より良いモジュール分離
- 並列ビルドの効率化
- 保守性の向上
- 開発効率の向上（インクリメンタルビルドの高速化）

---
最終更新: 2025-07-13
Phase 24: リファクタリング完了 ✅
Phase 25: クレート分割によるビルド最適化（詳細分析・タスク定義完了）
- コードベース: 51,147行（273ファイル）
- 現在のビルド時間: 2分以上
- 目標: 1分以内（50%短縮）
- クレート構成: 16クレート予定