# 設計原則とガイドライン

このドキュメントでは、Feature別統一構造実装における設計原則、命名規則、アーキテクチャガイドラインを定めています。

## 📝 命名規則の統一

### 1. Request/Response DTOの命名規則

```rust
// ✅ 推奨される命名パターン
// requests/
CreateTeamRequest       // 作成
UpdateTeamRequest       // 更新
DeleteTeamRequest       // 削除（bodyがある場合）
ListTeamsRequest        // 一覧取得
GetTeamRequest          // 単一取得（query params）
SearchTeamsRequest      // 検索

// responses/
TeamResponse            // 単一エンティティ
TeamsResponse           // 複数エンティティ
TeamCreatedResponse     // 作成結果
TeamUpdatedResponse     // 更新結果
TeamDeletedResponse     // 削除結果
TeamStatsResponse       // 統計情報

// ❌ 避けるべき命名
TeamDto                 // 曖昧
TeamData               // 曖昧
CreateTeamDto          // DTOサフィックスは使わない
TeamResponseDto        // 二重サフィックス
```

### 2. サービスメソッドの命名規則

```rust
// ✅ 推奨される命名パターン
impl TeamService {
    // 基本CRUD
    async fn create_team(&self, request: CreateTeamRequest) -> Result<TeamResponse>
    async fn get_team(&self, team_id: Uuid) -> Result<TeamResponse>
    async fn update_team(&self, team_id: Uuid, request: UpdateTeamRequest) -> Result<TeamResponse>
    async fn delete_team(&self, team_id: Uuid) -> Result<TeamDeletedResponse>
    async fn list_teams(&self, request: ListTeamsRequest) -> Result<TeamsResponse>
    
    // ビジネスロジック
    async fn add_member(&self, team_id: Uuid, user_id: Uuid) -> Result<TeamMemberResponse>
    async fn remove_member(&self, team_id: Uuid, user_id: Uuid) -> Result<TeamMemberRemovedResponse>
    async fn transfer_ownership(&self, team_id: Uuid, new_owner_id: Uuid) -> Result<TeamResponse>
}
```

### 3. リポジトリメソッドの命名規則

```rust
// ✅ 推奨される命名パターン
impl TeamRepository {
    // 基本CRUD（DBアクセス層なのでfind/save/deleteを使用）
    async fn find_by_id(&self, team_id: Uuid) -> Result<Option<Team>>
    async fn find_all(&self, pagination: PaginationQuery) -> Result<Vec<Team>>
    async fn save(&self, team: &Team) -> Result<Team>
    async fn delete(&self, team_id: Uuid) -> Result<bool>
    
    // 特定条件での検索
    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Vec<Team>>
    async fn find_by_organization_id(&self, org_id: Uuid) -> Result<Vec<Team>>
    async fn exists_by_name(&self, name: &str, org_id: Uuid) -> Result<bool>
    async fn count_by_organization_id(&self, org_id: Uuid) -> Result<i64>
}
```

### 4. モデルの命名規則

```rust
// models/
Team                    // エンティティ
TeamMember             // 関連エンティティ
TeamRole               // 値オブジェクト
TeamStatus             // Enum
TeamPermission         // Enum

// ❌ 避けるべき命名
TeamModel              // Modelサフィックスは不要
TeamEntity             // Entityサフィックスは不要
TeamTable              // DB層の詳細を露出
```

### 5. ハンドラー関数の命名規則

```rust
// ✅ 推奨される命名パターン
pub async fn create_team_handler(/* params */) -> Result<impl IntoResponse>
pub async fn get_team_handler(/* params */) -> Result<impl IntoResponse>
pub async fn update_team_handler(/* params */) -> Result<impl IntoResponse>
pub async fn delete_team_handler(/* params */) -> Result<impl IntoResponse>
pub async fn list_teams_handler(/* params */) -> Result<impl IntoResponse>

// ルーター関数
pub fn team_routes() -> Router<AppState>
```

### 6. 共通接頭辞・接尾辞のルール

| 種別 | 接頭辞 | 接尾辞 | 例 |
|------|--------|--------|-----|
| Request DTO | {Action}{Entity} | Request | CreateTeamRequest |
| Response DTO | {Entity}{Variant}? | Response | TeamResponse, TeamCreatedResponse |
| Service | {Entity} | Service | TeamService |
| Repository | {Entity} | Repository | TeamRepository |
| Handler関数 | {action}_{entity} | _handler | create_team_handler |
| Model | - | - | Team（接尾辞なし） |
| UseCase | {BusinessAction} | UseCase | TransferOwnershipUseCase |

### 7. 複数形の使用ルール

- ディレクトリ名：複数形（handlers/, services/, models/）
- コレクションを返すメソッド：複数形（list_teams, find_teams）
- 単一エンティティを扱うメソッド：単数形（get_team, create_team）
- レスポンスDTO：単数形（TeamResponse）、複数形（TeamsResponse）

## 🎯 Services vs UseCases: ビジネスロジックの配置指針

### 1. 基本的な役割分担

```rust
// Services: 単一エンティティに関する基本的なビジネスロジック
// - CRUD操作 + 簡単なビジネスルール
// - 1つのリポジトリを主に使用
// - エンティティ中心の操作

impl TeamService {
    // ✅ Serviceに適したロジック
    async fn create_team(&self, request: CreateTeamRequest) -> Result<TeamResponse> {
        // バリデーション
        self.validate_team_name(&request.name)?;
        
        // エンティティ作成
        let team = Team::new(request.name, request.owner_id);
        
        // 永続化
        let saved_team = self.repository.save(&team).await?;
        
        // レスポンス変換
        Ok(TeamResponse::from(saved_team))
    }
    
    async fn add_member(&self, team_id: Uuid, user_id: Uuid) -> Result<TeamMemberResponse> {
        // 単一チームへのメンバー追加
        let team = self.repository.find_by_id(team_id).await?
            .ok_or(Error::NotFound)?;
            
        team.add_member(user_id)?; // ドメインロジック
        
        self.repository.save(&team).await?;
        Ok(TeamMemberResponse::new(team_id, user_id))
    }
}

// UseCases: 複数のエンティティやサービスをまたぐ複雑なビジネスロジック
// - 複数のサービスを協調させる
// - トランザクション境界を管理
// - 複雑なビジネスフロー

pub struct TransferTeamOwnershipUseCase {
    team_service: Arc<TeamService>,
    user_service: Arc<UserService>,
    notification_service: Arc<NotificationService>,
    audit_service: Arc<AuditService>,
}

impl TransferTeamOwnershipUseCase {
    // ✅ UseCaseに適したロジック
    pub async fn execute(&self, team_id: Uuid, new_owner_id: Uuid, actor_id: Uuid) -> Result<TeamOwnershipTransferredResponse> {
        // 1. 権限チェック
        let team = self.team_service.get_team(team_id).await?;
        if team.owner_id != actor_id {
            return Err(Error::Forbidden);
        }
        
        // 2. 新オーナーの検証
        let new_owner = self.user_service.get_user(new_owner_id).await?;
        if !new_owner.is_active() {
            return Err(Error::InvalidUser);
        }
        
        // 3. 所有権の移転（トランザクション内）
        let transferred_team = self.team_service
            .transfer_ownership(team_id, new_owner_id)
            .await?;
        
        // 4. 通知送信
        self.notification_service
            .send_ownership_transfer_notification(&team, &new_owner)
            .await?;
        
        // 5. 監査ログ記録
        self.audit_service
            .log_ownership_transfer(team_id, actor_id, new_owner_id)
            .await?;
        
        Ok(TeamOwnershipTransferredResponse {
            team: transferred_team,
            previous_owner_id: team.owner_id,
            new_owner_id,
            transferred_at: Utc::now(),
        })
    }
}
```

### 2. 判断基準

| 観点 | Service | UseCase |
|------|---------|---------|
| **責務** | 単一エンティティの操作 | 複数エンティティの協調 |
| **複雑度** | シンプル〜中程度 | 複雑なビジネスフロー |
| **依存** | 1-2個のリポジトリ | 複数のサービス |
| **トランザクション** | 単一 | 複数の可能性 |
| **例** | ユーザー作成、チーム更新 | 注文処理、承認フロー |

### 3. 実装パターンの選択肢

```rust
// パターン1: Service Only（シンプルなアプリケーション）
features/team/
├── handlers/
├── services/     # すべてのビジネスロジック
├── repositories/
├── models/
└── dto/

// パターン2: Service + UseCase（中〜大規模アプリケーション）
features/team/
├── handlers/
├── services/     # 基本的なCRUD + 簡単なビジネスロジック
├── usecases/     # 複雑なビジネスフロー
├── repositories/
├── models/
└── dto/

// パターン3: Service + Domain Service（DDD寄り）
features/team/
├── handlers/
├── services/           # アプリケーションサービス
├── domain/
│   ├── models/        # エンティティ、値オブジェクト
│   └── services/      # ドメインサービス（ドメインロジック）
├── repositories/
└── dto/
```

### 4. 推奨アプローチ

1. **最初はServiceのみで開始**
   - すべてのビジネスロジックをServiceに実装
   - シンプルで理解しやすい

2. **複雑になったらUseCaseを導入**
   - Serviceが肥大化したとき
   - 複数のServiceを協調させる必要が出たとき
   - 明確なビジネスフローが識別できたとき

3. **UseCaseの命名例**
   ```rust
   // ビジネスフローを表す名前
   ProcessOrderUseCase
   ApproveDocumentUseCase
   TransferOwnershipUseCase
   GenerateMonthlyReportUseCase
   OnboardNewUserUseCase
   ```

### 5. アンチパターンの回避

```rust
// ❌ 避けるべき: ServiceがUseCaseを呼ぶ
impl TeamService {
    async fn do_something(&self) {
        self.some_usecase.execute().await // 逆依存！
    }
}

// ✅ 正しい: UseCaseがServiceを呼ぶ
impl SomeUseCase {
    async fn execute(&self) {
        self.team_service.do_something().await
    }
}

// ❌ 避けるべき: 不明確な責務
impl TeamService {
    // これはUseCase？Service？
    async fn process_team_with_notification_and_audit(&self) { }
}

// ✅ 正しい: 明確な分離
impl TeamService {
    async fn update_team(&self) { } // 基本操作
}

impl UpdateTeamWithNotificationUseCase {
    async fn execute(&self) { } // 複雑なフロー
}
```

## 🎯 移行戦略の原則

### 1. 後方互換性の維持
- 既存のインポートが動作し続けるよう、段階的に移行
- 一度に全てを変更せず、小さなステップで実施

### 2. テスト駆動での移行
- 各変更前後でテストスイートが通ることを確認
- 新しいインポートパスでのテストを先に作成

### 3. 影響範囲の最小化
- 一度に1つのモジュールのみを変更
- 依存関係の少ないものから着手

### 4. ドキュメント化
- 各Phaseの実施内容と結果を記録
- 新しいモジュール構造の使用方法を文書化

## 🚫 循環依存を防ぐための設計原則

### 1. レイヤー間の依存方向
```
handlers → services → repositories → models
   ↓          ↓           ↓            ↓
  dto     usecases      dto         core
```
- 上位層は下位層に依存（逆は禁止）
- 同一層内での相互依存も避ける

### 2. Feature間の依存関係
- 直接的な相互依存は禁止
- 共通機能は`shared/`または`core/`に抽出
- インターフェース（trait）による疎結合化

### 3. DTO設計の原則
- DTOはその機能内で完結（他featureのDTOを参照しない）
- 共通型は`shared/types/`に配置
- Service層からDTO層への逆依存は絶対禁止

### 4. 依存関係のチェック方法
```bash
# 各サブフェーズ完了時に実行
cargo test --lib
cargo clippy --all-targets

# 循環依存の確認
cargo deps --all-features | grep -E "cycle|circular"
```

### 5. 問題が発生した場合の対処
- 共通型の抽出：`shared/types/`へ
- インターフェースの導入：trait定義
- イベント駆動：直接呼び出しを避ける
- 依存性注入：コンストラクタでの注入

## 🛡️ リファクタリング時のリスク軽減方針

### 1. Feature間の相互依存への対処

```rust
// ❌ 避けるべき: 直接的な相互依存
// features/team/services/team.rs
use crate::features::organization::services::OrganizationService;

// ✅ 推奨: インターフェース経由
// shared/traits/organization.rs
pub trait OrganizationProvider {
    async fn get_organization(&self, id: Uuid) -> Result<Organization>;
    async fn validate_membership(&self, org_id: Uuid, user_id: Uuid) -> Result<bool>;
}

// features/team/services/team.rs
pub struct TeamService<O: OrganizationProvider> {
    organization_provider: Arc<O>,
}
```

**依存関係の優先順位**:
1. **Phase 14-18の実装順序**:
   ```
   Organization → Team → Security → Admin → Subscription
   （依存される側から実装）
   ```

2. **共通インターフェースの事前定義**:
   - Phase 14開始前に`shared/traits/`を作成
   - 各featureが必要とする最小限のインターフェースを定義
   - 実装時はインターフェース経由で依存

### 2. 共通機能の抽出タイミング

```rust
// shared/へ移動する判断基準
// 1. 2つ以上のfeatureから参照される
// 2. ビジネスロジックを含まない
// 3. 純粋な型定義またはユーティリティ

// ✅ shared/に配置すべき例
pub struct Pagination { ... }           // 汎用的な型
pub trait Auditable { ... }            // 共通trait
pub fn validate_email(email: &str) { } // 汎用的なバリデーション

// ❌ shared/に配置すべきでない例
pub struct TeamMemberRole { ... }      // Team固有の型
pub fn calculate_subscription_fee() {} // ビジネスロジック
```

**抽出のタイミング**:
- **即座に抽出**: 明らかに汎用的な型（Pagination, Result型など）
- **2つ目の使用時**: 最初は各feature内、2つ目のfeatureが必要としたら抽出
- **Phase 19で一括整理**: 残存ファイル整理時に最終判断

### 3. ビルド時間増加への対処

**並列ビルド戦略**:
```bash
# 1. 変更したfeatureのみをテスト（開発中）
cargo test -p feature_team

# 2. 関連featureも含めてテスト（サブフェーズ完了時）
cargo test -p feature_team -p feature_organization

# 3. 全体テスト（Phase完了時のみ）
make ci-check-fast
```

### 4. 段階的移行のチェックポイント

各サブフェーズで必ず確認:
- [ ] `cargo check` - コンパイルエラーなし
- [ ] `cargo test --lib` - ユニットテストパス
- [ ] `cargo clippy` - 警告なし
- [ ] 関連featureのテスト - 影響範囲の確認

Phase完了時に確認:
- [ ] `make ci-check-fast` - 全テストパス
- [ ] ビルド時間の計測と記録
- [ ] 依存関係グラフの更新
- [ ] ドキュメントの更新

**各Phase実施時の注意**:
```
1. 新モジュール構造を作成
2. 既存コードはそのまま維持（ビルドが通る状態を保つ）
3. 「TODO: Phase X完了後にモジュール参照を修正」とコメント追加
4. CLAUDE.mdの各Phaseに残課題として記録
5. `cargo clippy --workspace --all-targets --all-features -- -D warnings`で警告が出る場合は、
   一時的に#[allow(...)]アノテーションで抑制し、TODOコメントで削除予定を明記
```

## 📋 警告抑制の運用ルール

各Phase実装時に`cargo clippy --workspace --all-targets --all-features -- -D warnings`を実行し、
エラーや警告が発生した場合は以下の方針で対処：

### 1. 一時的な警告抑制の使用
- 移行期間中の後方互換性維持による警告は`#[allow(...)]`で抑制
- 必ずTODOコメントで「Phase 19で削除予定」を明記

### 2. よく使用する警告抑制アノテーション
```rust
#[allow(unused_imports)]          // 未使用インポート
#[allow(dead_code)]               // 未使用コード
#[allow(ambiguous_glob_reexports)] // 曖昧なグロブ再エクスポート
#[allow(unused_variables)]        // 未使用変数（_プレフィックスも可）
```

### 3. 警告抑制の配置例
```rust
// TODO: Phase 19で古い参照を削除後、#[allow(unused_imports)]を削除
#[allow(unused_imports)]
use some::old::path::Module;
```

### 4. 残課題セクションへの記載
- 各Phaseの「完了時の残課題」セクションに警告抑制の詳細を記録
- ファイルパス、行番号、警告の種類を明記

## 🔧 実装時のベストプラクティス

### 1. コードレビュー時の確認事項
- [ ] 命名規則に従っているか
- [ ] 依存関係の方向が正しいか
- [ ] DTOの責務が明確か
- [ ] ビジネスロジックの配置が適切か
- [ ] テストが十分か

### 2. テスト戦略
- ユニットテスト：各層で独立してテスト
- 統合テスト：feature単位でテスト
- E2Eテスト：主要なユースケースをテスト

### 3. パフォーマンス考慮事項
- DTOの変換コストを最小化
- 不要な抽象化を避ける
- 適切なキャッシュ戦略

### 4. セキュリティ考慮事項
- 入力値の検証はhandler層で実施
- 認証・認可の確認
- SQLインジェクション対策
- 機密情報のログ出力禁止