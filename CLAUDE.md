# Rust Backend API - Feature-based Architecture

## 📌 現在の状態 (2025-07-11)

**Phase 22 完了** - 残課題の解消完了 ✅
- Phase 19完了: 全テストパス達成 (523/523 パス、6 ignored)
- Phase 20完了: ✅
  - ✅ 空の再エクスポートハンドラー削除完了
  - ✅ api/dtoディレクトリ完全削除
  - ✅ インポートパス更新完了
  - ✅ payment関連ファイルの移行完了
  - ✅ domain/ディレクトリ内の全ファイル（19個）を適切なfeatureへ移行
  - ✅ repository/ディレクトリ内の全ファイル（14個）を適切なfeatureへ移行
  - ✅ service/ディレクトリ内の全ファイル（9個）を適切なfeatureへ移行
  - ✅ api/handlers/内の残りファイル（3個）を適切なfeatureへ移行
  - ✅ 空になったレガシーディレクトリ（domain/, repository/, service/, api/handlers/）削除
- Phase 21完了:
  - ✅ features/payment/ 作成（DTOを含む完全な構造）
  - ✅ features/user/ 作成（サービスとハンドラーのみ）
  - ✅ features/system/ 作成（ハンドラーのみ）
  - ✅ 全テストのインポートパス更新
  - ✅ ビルド成功（エラー0、警告のみ）
- Phase 22完了:
  - ✅ features/user/の完全化（UserRepository、UserSettingsRepository、DTOの移行）
  - ✅ features/system/の完全化（SystemService実装、ヘルスチェック、メトリクス機能追加）
  - ✅ 既存featureの構造補完（auth、taskの標準化）
  - ✅ 全インポートパスの更新とビルド成功維持

## 🏗️ アーキテクチャ概要

### 現在の構造
```
src/
├── features/          # 機能別モジュール
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

## 📋 残タスク一覧

### Phase 20: レガシーコードの完全削除
**目的**: 再エクスポートによる暫定対応を解消し、全コードを適切な場所に配置

#### 完了タスク ✅
1. **空の再エクスポートハンドラー削除**
   - `task-backend/src/api/handlers/analytics_handler.rs` 削除済み
   - `task-backend/src/api/handlers/subscription_handler.rs` 削除済み
   - `task-backend/src/api/handlers/mod.rs` 更新済み

2. **api/dtoディレクトリの完全削除**
   - `task-backend/src/api/dto/` ディレクトリ全体削除済み
   - `role_dto.rs` → `features/security/dto/legacy/role_dto.rs` 移行済み
   - `api/mod.rs` から dto モジュール参照削除済み

3. **インポートパス更新**
   - `api::dto::common::*` → `shared::types::common::*`
   - `api::dto::user_dto::*` → `shared::dto::user::*`
   - `api::dto::role_dto::*` → `features/security/dto/legacy/role_dto::*`
   - その他全てのDTO参照を適切なfeatureモジュールに更新済み

4. **payment関連ファイルの移行**
   - `domain/stripe_payment_history_model.rs` → `features/payment/models/stripe_payment_history.rs` 移行済み
   - `repository/stripe_payment_history_repository.rs` → `features/payment/repositories/` 移行済み
   - `service/payment_service.rs` → `features/payment/services/` 移行済み
   - `api/handlers/payment_handler.rs` → `features/payment/handlers/` 移行済み

#### 残タスク ⏳
1. **レガシーモデルファイルの移行**（現在19ファイル）
   - organization, team, user, role, security関連モデルを適切なfeatureへ

2. **レガシーリポジトリファイルの移行**（現在14ファイル）
   - organization, team, role, security関連リポジトリを適切なfeatureへ

3. **レガシーサービスファイルの移行**（現在9ファイル）
   - organization, team, role, security関連サービスを適切なfeatureへ

### Phase 21: 未移行機能のFeature化 ✅ 完了
**目的**: 残りの機能を独立したfeatureモジュールとして実装

#### 完了タスク ✅
1. **features/payment/** 
   - ✅ `api/handlers/payment_handler.rs` → `features/payment/handlers/` 移行済み
   - ✅ `service/payment_service.rs` → `features/payment/services/` 移行済み
   - ✅ `domain/stripe_payment_history_model.rs` → `features/payment/models/` 移行済み
   - ✅ `repository/stripe_payment_history_repository.rs` → `features/payment/repositories/` 移行済み
   - ✅ DTOの整理: `dto/requests/`, `dto/responses/` 構造で実装済み
   - ✅ Stripe統合ロジックの整理完了

2. **features/user/**
   - ✅ `api/handlers/user_handler.rs` → `features/user/handlers/` 移行済み
   - ✅ `service/user_service.rs` → `features/user/services/` 移行済み
   - ⏳ DTOの整理は未実装（現在はshared/dto/userを使用）
   - ⏳ リポジトリ・モデルの移行は未実装

3. **features/system/**
   - ✅ `api/handlers/system_handler.rs` → `features/system/handlers/` 移行済み
   - ⏳ サービス層の実装は未実装（ハンドラーから直接実装）

#### Phase 21 残課題
1. **features/user/の完全化**
   - `domain/user_model.rs` → `features/user/models/` への移行
   - UserRepositoryの`features/user/repositories/`への移行（現在auth配下）
   - DTOの整理: `shared/dto/user/` → `features/user/dto/` への移行
   - UserSettingsサービスの統合

2. **features/system/の完全化**
   - システムヘルスチェックサービスの作成
   - メトリクス収集サービスの実装
   - DTOの整理（必要に応じて）

### Phase 22: 残課題の解消 ✅ 完了
**目的**: Phase 20-21で積み残した移行作業と機能の完全実装

#### 完了タスク ✅
1. **Phase 20残作業の完了**
   - ✅ レガシーモデルファイル（19ファイル）の適切なfeatureへの移行
   - ✅ レガシーリポジトリファイル（14ファイル）の適切なfeatureへの移行
   - ✅ レガシーサービスファイル（9ファイル）の適切なfeatureへの移行
   - ✅ `api/handlers/`内の残りハンドラー（3ファイル）の移行
   - ✅ 空になったレガシーディレクトリの削除

2. **Phase 21残課題の解消**
   - ✅ **features/user/の完全化**
     - ✅ `domain/user_model.rs` → `features/user/models/`
     - ✅ UserRepositoryを`features/auth/`から`features/user/repositories/`へ
     - ✅ `shared/dto/user/` → `features/user/dto/`への構造化
     - ✅ UserSettingsサービスの統合
   - ✅ **features/system/の完全化**
     - ✅ システムヘルスチェックサービスの作成
     - ✅ メトリクス収集サービスの実装
     - ✅ DTOの整理

3. **既存featureの構造補完**
   - ✅ authの構造標準化（repository → repositories）
   - ✅ taskの構造標準化（domain → models、repository → repositories、ファイル → ディレクトリ構造）

#### Phase 22 残課題（Phase 23への積み残し）
1. **Feature構造の改善が必要なモジュール**
   - **gdpr feature**: `repositories/`、`services/`、`handlers/`、`dto/`の標準ディレクトリ構造化が必要
   - **storage feature**: `repositories/`、`services/`、`handlers/`、`dto/`の標準ディレクトリ構造化が必要
   - **auth feature**: ネストされた`src/features/auth/`ディレクトリの解消、フラットファイルのディレクトリ構造化

2. **技術的負債**
   - **Dead Code警告** (51ファイル): security(14)、organization(10)、team(8)、analytics(5)、subscription(5)、admin(4)、auth(2)、task(2)、user(1)
   - **Ignoredテスト** (6件): analytics feature関連のテスト（ハンドラー移行待ち）
   - **APIディレクトリ**: `src/api/`に`AppState`のみ残存（より適切な場所への移動が必要）

3. **オプション改善項目**
   - `usecases/`ディレクトリの追加検討（現在はorganizationとsecurityのみ実装）
   - インポートパスの一貫性確保とTODOコメントのクリーンアップ

### Phase 23: 品質改善
**目的**: コード品質とテストカバレッジの向上

#### タスク
1. **Feature構造の標準化完了** (優先度: 高)
   - gdpr featureの標準ディレクトリ構造化
   - storage featureの標準ディレクトリ構造化
   - auth featureのネストディレクトリ解消とフラットファイルの構造化
   - analytics featureのハンドラー移行完了

2. **Ignoredテストの修正** (6件)
   - analytics featureのハンドラー移行後にテストを有効化
   - Feature usage tracking機能の完全実装

3. **Dead Code削除** (51ファイル)
   - security feature (14ファイル)の`#[allow(dead_code)]`除去
   - organization feature (10ファイル)の未使用コード削除
   - team feature (8ファイル)の未使用コード削除
   - その他のfeature (19ファイル)のクリーンアップ

4. **技術的負債の解消**
   - `src/api/`ディレクトリの`AppState`を適切な場所へ移動
   - インポートパスの一貫性確保
   - Phase 19互換性のためのTODOコメントの削除

5. **オプション改善**
   - 必要に応じて`usecases/`ディレクトリの追加
   - グロブインポートを明示的インポートに変更

### Phase 24: ワークスペース準備
**目的**: マルチクレート構成への移行準備

#### タスク
1. **ワークスペース構成への段階的移行**
   - Cargo.tomlのワークスペース設定追加
   - 各featureの独立性確認とクレート分離準備
   - 依存関係の整理と最適化

2. **最終構造への移行準備**
   ```
   rust-backend-api/
   ├── Cargo.toml          # ワークスペース定義
   ├── crates/
   │   ├── shared/         # 共通型・ユーティリティ
   │   ├── core/           # コアドメイン
   │   ├── infrastructure/ # 技術基盤
   │   ├── feature-auth/   # 認証機能
   │   ├── feature-task/   # タスク管理
   │   ├── feature-team/   # チーム管理
   │   ├── feature-org/    # 組織管理
   │   └── ...            # 他のfeatureクレート
   └── apps/
       ├── api-server/     # メインAPIサーバー
       └── worker/         # バックグラウンドワーカー
   ```

3. **移行計画の策定**
   - 各featureの依存関係マップ作成
   - 移行順序の決定
   - リスク評価とロールバック計画

## 📚 詳細ドキュメント

設計原則、実装ガイドライン、各Phaseの詳細な実装手順については以下のドキュメントを参照してください：

- **[設計原則とガイドライン](./CLAUDE-GUIDELINES.md)**
  - 命名規則の統一
  - Services vs UseCases: ビジネスロジックの配置指針
  - 循環依存を防ぐための設計原則
  - リファクタリング時のリスク軽減方針
  - 警告抑制の運用ルール

- **[Phase実装詳細](./CLAUDE-PHASES.md)**
  - Phase 20-23の詳細な実装手順
  - 各Phaseの完了条件と成功指標
  - タイムラインとリスク管理
  - ワークスペース構成への移行準備

## 🎯 設計原則サマリー

### 依存関係の原則
```
handler → service → repository → domain
   ↓         ↓          ↓          ↓
  dto    usecase      dto       (core)
```
- 上位層は下位層のみに依存
- 循環依存は絶対に避ける
- 横断的関心事はsharedモジュールに配置

詳細は[CLAUDE-GUIDELINES.md](./CLAUDE-GUIDELINES.md)を参照してください。

## 🚀 次のステップ

### 即座に実行可能なタスク
1. ~~レガシーDTOファイルの移行（機械的作業）~~ ✅ 完了
2. ~~空の再エクスポートファイルの削除~~ ✅ 完了
3. ~~インポートパスの一括更新~~ ✅ 完了
4. レガシーモデルファイルの移行（Phase 20残作業）
5. レガシーリポジトリファイルの移行確認（Phase 20残作業）

### 検討が必要なタスク
1. Payment機能の設計とFeature化
2. User機能のauth機能からの分離方法
3. ワークスペース構成への移行戦略

### 成功指標
- [ ] `api/`, `domain/`, `repository/`, `service/` ディレクトリが空
- [ ] 全機能がfeaturesモジュール配下に存在
- [ ] `#[allow(dead_code)]` が0件
- [ ] ignoredテストが0件
- [ ] 各featureが独立してビルド可能（将来）

## 📝 備考

このドキュメントは生きたドキュメントとして、プロジェクトの進行に合わせて更新してください。
各Phaseの完了時には、完了日と主な成果を記録することを推奨します。

---
最終更新: 2025-07-11
Phase 22完了: 残課題の解消完了、レガシーコード完全削除、feature構造の大幅改善

## 🔄 次回セッション引き継ぎ情報

### 開始時の指示
「続きからお願いします。」でOKです。

### 現在の状況
- **作業ディレクトリ**: `task-backend/`内で作業中
- **ビルド状態**: 正常（エラー0、警告43件 - すべてdead_code警告）
- **テスト状態**: 523 passed, 0 failed, 6 ignored
- **Phase 22進捗**: 完了 ✅
  - レガシーディレクトリ（domain/, repository/, service/, api/handlers/）完全削除
  - features/user/とfeatures/system/の完全化
  - authとtaskの構造標準化

### 次の作業内容
Phase 23: 品質改善
1. **Feature構造の標準化完了**（優先度：高）
   - gdpr featureの標準ディレクトリ構造化
   - storage featureの標準ディレクトリ構造化
   - auth featureのネストディレクトリ解消
   - analytics featureのハンドラー移行完了

2. **Ignoredテストの修正**（優先度：高）
   - analytics feature関連の6テストを有効化

3. **Dead Code削除**（優先度：中）
   - 51ファイルのdead_code警告を解消
   - security、organization、teamから優先的に実施

### 達成見込み
Phase 23は品質改善に特化したフェーズとして、2-3セッションで完了可能です。
Feature構造の標準化は機械的な作業が中心のため、次回セッションで大部分を完了できる見込みです。