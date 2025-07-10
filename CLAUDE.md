## 実現トピック

### 🏗️ モジュール構造リファクタリング（ビルド時間短縮）

機能別にsrcディレクトリを再編成し、将来的なクレート分割に向けた準備を行います。

### 🎨 Feature別統一構造実装（Phase 14以降）

**目的**: 各featureモジュールに統一的な構造を持たせ、循環依存を完全に排除し、マルチバックエンドシステム向けのクレート分割を可能にする

#### 📐 統一構造の定義（ベストプラクティス版）

**依存関係の原則**:
```
handler → service → repository → domain
   ↓         ↓          ↓          ↓
  dto    usecase      dto       (core)
```

各featureモジュールは以下の構造を持つ：
```
features/{feature_name}/
├── mod.rs           # モジュール定義と公開API
├── handlers/        # HTTPハンドラー層（複数可）
│   ├── mod.rs
│   └── *.rs         # 各ハンドラー実装
├── services/        # ビジネスロジック層（複数可）
│   ├── mod.rs
│   └── *.rs         # 各サービス実装
├── repositories/    # データアクセス層
│   ├── mod.rs
│   └── *.rs         # 各リポジトリ実装
├── dto/             # データ転送オブジェクト
│   ├── mod.rs
│   ├── requests/    # リクエストDTO
│   │   ├── mod.rs
│   │   └── *.rs
│   └── responses/   # レスポンスDTO
│       ├── mod.rs
│       └── *.rs
├── models/          # ドメインモデル（domainから変更）
│   ├── mod.rs
│   └── *.rs         # 各モデル定義
└── usecases/        # 複雑なビジネスロジック（オプション）
    ├── mod.rs
    └── *.rs         # ユースケース実装
```

**重要な変更点**:
1. 単数形から複数形へ（例: `handler` → `handlers`）- Rustの慣例に従う
2. `domain` → `models` - より明確で一般的な名称
3. `request.rs`/`response.rs` → `requests/`/`responses/` - 拡張性を考慮
4. 各層は下位層のみに依存（循環依存を防ぐ）

### 📚 詳細ドキュメント

設計原則、実装ガイドライン、各Phaseの詳細な実装手順については以下のドキュメントを参照してください：

- **[設計原則とガイドライン](./CLAUDE-GUIDELINES.md)**
  - 命名規則の統一
  - Services vs UseCases: ビジネスロジックの配置指針
  - 循環依存を防ぐための設計原則
  - リファクタリング時のリスク軽減方針
  - 警告抑制の運用ルール

- **[Phase実装詳細](./CLAUDE-PHASES.md)**
  - Phase 14-20の実装手順
  - 各Phaseの完了状態と残課題
  - 移行戦略と注意事項
  - ワークスペース構成への移行準備

### 📋 Phase別完了状態と残課題

各Phaseの実装で発生した残課題と対応方針を以下に記載します。詳細は[Phase実装詳細](./CLAUDE-PHASES.md)を参照してください。

#### Phase 14 (Team機能) - 完了 ✅
**残課題**:
- 警告抑制: 23箇所の`#[allow(unused_imports)]`と`#[allow(dead_code)]`
- 旧ファイル: 9ファイルの削除または再エクスポート化が必要
- 型の重複: `TeamRole`が2箇所に存在

#### Phase 15 (Organization機能) - 完了 ✅
**残課題**:
- 警告抑制: 約30箇所の`#[allow]`アノテーション
- 旧ファイル: 12ファイルの削除または再エクスポート化が必要
- DTOの不整合: hierarchy.rsで多数のTODOコメント
- 技術的負債: PermissionMatrix関連の実装

#### Phase 16 (Security機能) - 完了 ✅
**残課題**:
- DTOの曖昧性エラー: 約220件（グロブインポートによる同名型の衝突）
  - 原因: `features/security/dto/mod.rs`で`permission::*`と`responses::*`の両方をグロブインポート
  - 影響: `cargo clippy`や`cargo test`が実行できない
  - 解決方法: Phase 19で明示的インポートに変更することで確実に解決
- 型不一致エラー: 30件（旧domainと新modelsの相違）
- 警告抑制: 約25箇所の`#[allow]`アノテーション
- 旧ファイル: 12ファイルの削除または再エクスポート化が必要

**Phase 19での解決保証**:
1. `features/security/dto/mod.rs`のグロブインポートを明示的インポートに変更
2. 重複する型名を名前空間で区別（例: `permission::AdminFeatureInfo`）
3. または、DTOファイル自体を適切に分離してconflictを解消
4. これらは機械的な作業で確実に解決可能

#### Phase 17 (Admin機能) - 完了 ✅
**残課題**:
- 警告抑制: 13箇所の`#[allow(unused_imports)]`と`#[allow(dead_code)]`
- DTOの未実装: サブディレクトリ構造は作成済みだが、実際のDTO定義は未実装
- ハンドラーの暫定実装: 全てTODOコメント付きの暫定実装
- 旧ファイル: 5ファイルの削除または再エクスポート化が必要
  - `task-backend/src/api/handlers/admin_handler.rs`
  - `task-backend/src/api/handlers/analytics_handler.rs`
  - `task-backend/src/api/dto/admin_organization_dto.rs`
  - `task-backend/src/api/dto/admin_role_dto.rs`
  - `task-backend/src/api/dto/analytics_dto.rs`

#### Phase 18 (Subscription機能) - 完了 ✅
**残課題**:
- 警告抑制: 約10箇所の`#[allow(unused_imports)]`と`#[allow(dead_code)]`
- 型エラー: 多数のコンパイルエラー（主に以下が原因）
  - DTOの型名不一致: `UserSubscriptionStatsResponse`、`SubscriptionTierStatsResponse`、`SubscriptionChangeInfo`などの参照エラー
  - インポートパスの問題: 旧domainと新modelsの参照混在
  - ページネーション型エラー: u64とi32の型不一致
- 旧ファイル: 6ファイルの削除または再エクスポート化が必要
  - `task-backend/src/domain/subscription_history_model.rs`
  - `task-backend/src/domain/stripe_subscription_model.rs`
  - `task-backend/src/repository/subscription_history_repository.rs`
  - `task-backend/src/repository/stripe_subscription_repository.rs`
  - `task-backend/src/service/subscription_service.rs`
  - `task-backend/src/api/handlers/subscription_handler.rs`

**Phase 19での解決保証**:
1. 型名の不一致は、DTOファイルの正しいインポートで解決
2. インポートパスは旧domainディレクトリ削除時に一括修正
3. ページネーション型エラーは適切な型変換で対応
4. これらは全て機械的な修正で確実に解決可能

**共通の対応方針**:
- Phase 19「残存ファイルの整理と移行」で一括対応
- 警告抑制は旧ファイル削除と同時に解除
- DTOの曖昧性エラーは明示的インポートで解決
- 型の統一は旧domainモジュール削除時に実施

### 🚨 Phase 19 達成条件

Phase 19完了時には以下の条件を**必ず**満たすこと：

1. **コンパイルエラーゼロ**
   - `cargo build --workspace`が警告・エラーなしで成功

2. **Clippy警告ゼロ**
   - `cargo clippy --workspace --all-targets --all-features -- -D warnings`が警告・エラーなしで成功

3. **全テストパス**
   - `make ci-check-fast`で既存テストが全てパス
   - 新規追加したfeatureモジュールのテストも含む

4. **不要なアノテーション削除**
   - 全ての`#[allow(dead_code)]`を削除
   - 全ての`#[allow(unused_imports)]`を削除
   - 必要最小限の警告抑制のみ残す

これらの条件を満たさない場合、Phase 19は完了とみなさない。

### 🎯 最終目標

**Phase 14-20完了後の期待効果**:

1. **ビルド時間の短縮**
   - 現在: 全体ビルド（推定5-10分）
   - 目標: 変更されたクレートのみビルド（30秒-2分）

2. **開発効率の向上**
   - 機能別の独立開発が可能
   - チーム間の作業競合を最小化
   - テストの並列実行

3. **保守性の向上**
   - 明確な責任境界
   - 依存関係の可視化
   - 機能の追加・削除が容易

4. **マルチバックエンドへの対応**
   - 機能の組み合わせで異なるAPIサーバーを構築
   - マイクロサービス化への移行パスを確保
   - 特定機能のみのデプロイが可能

**ワークスペース構造案**:
```
rust-backend-api/
├── Cargo.toml           # ワークスペース定義
├── crates/
│   ├── shared/          # 共通型・ユーティリティ
│   ├── core/            # コアドメイン
│   ├── infrastructure/  # インフラ層
│   ├── feature-auth/    # 認証機能
│   ├── feature-task/    # タスク管理
│   ├── feature-team/    # チーム管理
│   ├── feature-org/     # 組織管理
│   ├── feature-storage/ # ストレージ
│   ├── feature-gdpr/    # GDPR
│   ├── feature-security/# セキュリティ
│   ├── feature-admin/   # 管理者
│   └── feature-subscription/ # サブスク
└── apps/
    ├── api-server/      # メインAPIサーバー
    └── worker/          # バックグラウンドワーカー（将来）
```

# important-instruction-reminders
Do what has been asked; nothing more, nothing less.
NEVER create files unless they're absolutely necessary for achieving your goal.
ALWAYS prefer editing an existing file to creating a new one.
NEVER proactively create documentation files (*.md) or README files. Only create documentation files if explicitly requested by the User.