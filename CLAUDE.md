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

#### 📝 命名規則の統一

**1. Request/Response DTOの命名規則**

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

**2. サービスメソッドの命名規則**

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

**3. リポジトリメソッドの命名規則**

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

**4. モデルの命名規則**

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

**5. ハンドラー関数の命名規則**

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

**6. 共通接頭辞・接尾辞のルール**

| 種別 | 接頭辞 | 接尾辞 | 例 |
|------|--------|--------|-----|
| Request DTO | {Action}{Entity} | Request | CreateTeamRequest |
| Response DTO | {Entity}{Variant}? | Response | TeamResponse, TeamCreatedResponse |
| Service | {Entity} | Service | TeamService |
| Repository | {Entity} | Repository | TeamRepository |
| Handler関数 | {action}_{entity} | _handler | create_team_handler |
| Model | - | - | Team（接尾辞なし） |
| UseCase | {BusinessAction} | UseCase | TransferOwnershipUseCase |

**7. 複数形の使用ルール**

- ディレクトリ名：複数形（handlers/, services/, models/）
- コレクションを返すメソッド：複数形（list_teams, find_teams）
- 単一エンティティを扱うメソッド：単数形（get_team, create_team）
- レスポンスDTO：単数形（TeamResponse）、複数形（TeamsResponse）

#### 🎯 Services vs UseCases: ビジネスロジックの配置指針

**1. 基本的な役割分担**

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

**2. 判断基準**

| 観点 | Service | UseCase |
|------|---------|---------|
| **責務** | 単一エンティティの操作 | 複数エンティティの協調 |
| **複雑度** | シンプル〜中程度 | 複雑なビジネスフロー |
| **依存** | 1-2個のリポジトリ | 複数のサービス |
| **トランザクション** | 単一 | 複数の可能性 |
| **例** | ユーザー作成、チーム更新 | 注文処理、承認フロー |

**3. 実装パターンの選択肢**

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

**4. 推奨アプローチ**

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

**5. アンチパターンの回避**

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

#### 🚀 Phase 14: Team機能の完全実装

**現状**: DTOのみ存在
**目標**: 完全な機能モジュールとして再構築

##### Phase 14.1: Models層の移行（30分）✅
- [x] `features/team/models/`ディレクトリを作成
- [x] `domain/team_model.rs` → `features/team/models/team.rs`
- [x] `domain/team_member_model.rs` → `features/team/models/team_member.rs`
- [x] `domain/team_invitation_model.rs` → `features/team/models/team_invitation.rs`
- [x] models/mod.rsで公開APIを定義
- [x] 既存のdomain/からの参照を更新
- [x] `cargo test --lib` でビルド確認

##### Phase 14.2: Repositories層の移行（30分）✅
- [x] `features/team/repositories/`ディレクトリを作成
- [x] `repository/team_repository.rs` → `features/team/repositories/team.rs`
- [x] `repository/team_member_repository.rs` → `features/team/repositories/team_member.rs`
- [x] `repository/team_invitation_repository.rs` → `features/team/repositories/team_invitation.rs`
- [x] repositories/mod.rsで公開APIを定義
- [x] modelsへのインポートパスを`super::models`に更新
- [x] `cargo test --lib` でビルド確認

##### Phase 14.3: Services層の移行（30分）✅
- [x] `features/team/services/`ディレクトリを作成
- [x] `service/team_service.rs` → `features/team/services/team.rs`
- [x] services/mod.rsで公開APIを定義
- [x] repositoriesへのインポートを`super::repositories`に更新
- [x] modelsへのインポートを`super::models`に更新
- [x] DTOへのインポートを`super::dto`に更新（一時的に既存パス維持）
- [x] `cargo test service::team_service` で既存テストの動作確認

##### Phase 14.4: DTOの再構成（30分）✅
- [x] `features/team/dto/requests/`ディレクトリを作成
- [x] `features/team/dto/responses/`ディレクトリを作成
- [x] 既存のdto/team.rs, dto/team_invitation.rsを分析
- [x] リクエストDTOをrequests/に分割配置
- [x] レスポンスDTOをresponses/に分割配置
- [x] dto/mod.rsで後方互換性のための再エクスポート
- [x] `cargo clippy --all-targets` で警告なし確認

##### Phase 14.5: Handlers層の移行（30分）✅
- [x] `features/team/handlers/`ディレクトリを作成
- [x] `api/handlers/team_handler.rs` → `features/team/handlers/team.rs`
- [x] handlers/mod.rsで公開APIを定義
- [x] servicesへのインポートを`super::services`に更新
- [x] DTOへのインポートを`super::dto`に更新
- [x] `team_router_with_state`関数の動作確認
- [x] `cargo test` で全テストがパスすることを確認

##### Phase 14.6: 最終統合とクリーンアップ（30分）
- [x] features/team/mod.rsで全モジュールを適切に公開
- [x] main.rsのインポートを`features::team::handlers`に更新
- [ ] 元ファイルを削除（後方互換性が不要な場合）
- [ ] または再エクスポートファイルとして維持
- [x] `make ci-check-fast` で全テストがパス
- [x] `cargo clippy --all-targets --all-features -- -D warnings`

#### 📝 Phase 14 完了時の残課題

Phase 14の実装において、以下の一時的な対処を行いました。Phase 19で古い参照を削除する際に、これらの対処も合わせて削除してください。

1. **未使用インポートの警告抑制**
   - `src/features/team/models/mod.rs`
     - `#[allow(unused_imports)]` on re-exports (lines 6-7)
   - `src/features/team/dto/mod.rs`
     - `#[allow(unused_imports)]` on multiple re-exports
   - `src/features/team/dto/team.rs`
     - `#[allow(unused_imports)]` on `SafeUser` import (line 2)

2. **Dead codeの警告抑制**
   - `src/features/team/repositories/team.rs`
     - `#[allow(dead_code)]` on `TeamRepository` struct (lines 23-24)
   - `src/features/team/repositories/team_invitation.rs`
     - `#[allow(dead_code)]` on `TeamInvitationRepository` struct (lines 20-21)
   - `src/features/team/services/team.rs`
     - `#[allow(dead_code)]` on `TeamService` struct (lines 29-30)
   - `src/features/team/services/team_invitation.rs`
     - `#[allow(dead_code)]` on `TeamInvitationService` struct (lines 13-14)
   - `src/features/team/models/team_invitation.rs`
     - `#[allow(dead_code)]` on `update_message` method (line 180)
   - `src/features/team/services/team_invitation.rs`
     - `#[allow(dead_code)]` on `cancelled` field in `TeamInvitationStatistics` (line 527)

3. **旧ハンドラーのdead code警告抑制**
   - `src/api/handlers/team_handler.rs`
     - `#[allow(dead_code)]` on all handler functions and `team_router` functions
     - このファイル自体がPhase 19で削除予定

4. **DTOの未使用フィールド警告抑制**
   - `src/features/team/dto/requests/team.rs`
     - `#[allow(dead_code)]` on `organization_id` field in `CreateTeamRequest`
     - `#[allow(dead_code)]` on `role` field in `UpdateTeamMemberRoleRequest`
   - `src/features/team/dto/requests/team_invitation.rs`
     - `#[allow(dead_code)]` on `invitation_id` field in `ResendInvitationRequest`
     - `#[allow(dead_code)]` on `team_id` field in `CreateInvitationRequest`
     - `#[allow(dead_code)]` on fields in `BulkUpdateStatusRequest`
     - `#[allow(dead_code)]` on `validate_emails` function
   - `src/features/team/dto/responses/team.rs`
     - `#[allow(dead_code)]` on `TeamPaginationResponse::new` method
   - `src/features/team/dto/responses/team_invitation.rs`
     - `#[allow(dead_code)]` on `InvitationPaginationResponse::new` method

5. **再エクスポートの未使用警告抑制**
   - `src/features/team/repositories/mod.rs`
     - `#[allow(unused_imports)]` on re-exports
   - `src/features/team/services/mod.rs`
     - `#[allow(unused_imports)]` on re-exports
   - `src/features/team/mod.rs`
     - `#[allow(unused_imports)]` on `team_router_with_state` re-export

**対処方針**:
- これらの警告は、移行期間中の後方互換性維持のために発生しています
- Phase 19で旧ディレクトリ構造（domain/, repository/, service/, api/handlers/）からの参照を削除する際に、これらの`#[allow]`アノテーションも削除します
- 各TODOコメントには「Phase 19で古い参照を削除後、#[allow]を削除予定」または「Phase 19で本来の使用箇所が移行されたら#[allow(dead_code)]を削除」と記載済み

#### 📋 Phase 14 積み残し事項

以下の項目はPhase 14.6で未実施となっており、Phase 19での対応が必要です：

1. **旧ファイルの削除または再エクスポート化**
   - `src/domain/team_model.rs` - 削除またはfeatures/team/modelsへの再エクスポートに変更
   - `src/domain/team_member_model.rs` - 削除またはfeatures/team/modelsへの再エクスポートに変更
   - `src/domain/team_invitation_model.rs` - 削除またはfeatures/team/modelsへの再エクスポートに変更
   - `src/repository/team_repository.rs` - 削除またはfeatures/team/repositoriesへの再エクスポートに変更
   - `src/repository/team_member_repository.rs` - 削除またはfeatures/team/repositoriesへの再エクスポートに変更
   - `src/repository/team_invitation_repository.rs` - 削除またはfeatures/team/repositoriesへの再エクスポートに変更
   - `src/service/team_service.rs` - 削除またはfeatures/team/servicesへの再エクスポートに変更
   - `src/service/team_invitation_service.rs` - 削除またはfeatures/team/servicesへの再エクスポートに変更
   - `src/api/handlers/team_handler.rs` - 削除またはfeatures/team/handlersへの再エクスポートに変更

2. **TeamInvitationServiceの追加移行**
   - `src/service/team_invitation_service.rs`を`features/team/services/team_invitation.rs`へ移行済み
   - ただし、旧ファイルがまだ存在している状態

3. **型の重複問題**
   - `TeamRole`が`domain::team_model`と`features::team::models::team`の両方に存在
   - 現在は型エイリアスで対処しているが、Phase 19で統一が必要

**対応方針**:
- Phase 19.1で旧ファイルの調査時に、これらのファイルの参照状況を確認
- 参照がある場合は再エクスポートファイルとして変更
- 参照がない場合は削除
- 型の重複は、旧domainモジュールの削除と同時に解消

#### 🏢 Phase 15: Organization機能の完全実装

**現状**: DTOのみ存在
**目標**: 階層構造を持つ組織管理機能として再構築

##### Phase 15.1: Models層の移行（30分）
- [ ] `features/organization/models/`ディレクトリを作成
- [ ] `domain/organization_model.rs` → `features/organization/models/organization.rs`
- [ ] `domain/department_model.rs` → `features/organization/models/department.rs`
- [ ] `domain/department_member_model.rs` → `features/organization/models/department_member.rs`
- [ ] models/mod.rsで公開APIを定義
- [ ] 循環依存チェック：他モデルへの参照を確認
- [ ] `cargo test --lib` でビルド確認

##### Phase 15.2: Repositories層の移行（30分）
- [ ] `features/organization/repositories/`ディレクトリを作成
- [ ] `repository/organization_repository.rs` → `features/organization/repositories/organization.rs`
- [ ] `repository/department_repository.rs` → `features/organization/repositories/department.rs`
- [ ] `repository/department_member_repository.rs` → `features/organization/repositories/department_member.rs`
- [ ] repositories/mod.rsで公開APIを定義
- [ ] modelsへのインポートパスを`super::models`に更新
- [ ] `cargo test --lib` でビルド確認

##### Phase 15.3: Services層の移行（45分）
- [ ] `features/organization/services/`ディレクトリを作成
- [ ] `service/organization_service.rs` → `features/organization/services/organization.rs`
- [ ] `service/organization_hierarchy_service.rs` → `features/organization/services/hierarchy.rs`
- [ ] services/mod.rsで公開APIを定義
- [ ] repositoriesへのインポートを`super::repositories`に更新
- [ ] modelsへのインポートを`super::models`に更新
- [ ] 外部依存（team等）の確認と整理
- [ ] `cargo test service::organization` で既存テストの動作確認

##### Phase 15.4: Usecases層の作成（30分）
- [ ] `features/organization/usecases/`ディレクトリを作成
- [ ] 階層構造操作の複雑なロジックを`hierarchy_operations.rs`に抽出
- [ ] usecases/mod.rsで公開APIを定義
- [ ] servicesから複雑なロジックを移動
- [ ] `cargo test` でテスト確認

##### Phase 15.5: DTOの再構成（30分）
- [ ] `features/organization/dto/requests/`ディレクトリを作成
- [ ] `features/organization/dto/responses/`ディレクトリを作成
- [ ] 既存のdto/organization.rs, dto/organization_hierarchy.rsを分析
- [ ] リクエストDTOをrequests/に分割配置
- [ ] レスポンスDTOをresponses/に分割配置
- [ ] 階層構造用のDTOを`responses/hierarchy.rs`に整理
- [ ] `cargo clippy --all-targets` で警告なし確認

##### Phase 15.6: Handlers層の移行と統合（45分）
- [ ] `features/organization/handlers/`ディレクトリを作成
- [ ] `api/handlers/organization_handler.rs` → `features/organization/handlers/organization.rs`
- [ ] handlers/mod.rsで公開APIを定義
- [ ] servicesへのインポートを`super::services`に更新
- [ ] usecasesへのインポートを`super::usecases`に更新
- [ ] main.rsのインポートを更新
- [ ] `make ci-check-fast` で全テストがパス

#### 🔐 Phase 16: Security機能の完全実装

**現状**: DTOのみ存在、role関連はshared/dto/role_types.rsに分離済み
**目標**: ロール・権限管理機能として再構築

##### Phase 16.1: Models層の移行（30分）
- [ ] `features/security/models/`ディレクトリを作成
- [ ] `domain/role_model.rs` → `features/security/models/role.rs`
- [ ] `domain/role_permission_model.rs` → `features/security/models/role_permission.rs`
- [ ] `domain/user_permission_model.rs` → `features/security/models/user_permission.rs`
- [ ] models/mod.rsで公開APIを定義
- [ ] shared/dto/role_types.rsへの依存を確認
- [ ] `cargo test --lib` でビルド確認

##### Phase 16.2: Repositories層の移行（30分）
- [ ] `features/security/repositories/`ディレクトリを作成
- [ ] `repository/role_repository.rs` → `features/security/repositories/role.rs`
- [ ] `repository/role_permission_repository.rs` → `features/security/repositories/role_permission.rs`
- [ ] `repository/user_permission_repository.rs` → `features/security/repositories/user_permission.rs`
- [ ] repositories/mod.rsで公開APIを定義
- [ ] modelsへのインポートパスを`super::models`に更新
- [ ] `cargo test --lib` でビルド確認

##### Phase 16.3: Services層の移行（45分）
- [ ] `features/security/services/`ディレクトリを作成
- [ ] `service/security_service.rs` → `features/security/services/security.rs`
- [ ] `service/role_service.rs` → `features/security/services/role.rs`
- [ ] `service/permission_service.rs` → `features/security/services/permission.rs`
- [ ] services/mod.rsで公開APIを定義
- [ ] shared/dto/role_typesのインポートを維持
- [ ] repositoriesへのインポートを`super::repositories`に更新
- [ ] `cargo test service::security` で既存テストの動作確認

##### Phase 16.4: Usecases層の作成（30分）
- [ ] `features/security/usecases/`ディレクトリを作成
- [ ] 権限チェックロジックを`permission_checker.rs`として抽出
- [ ] ロール階層処理を`role_hierarchy.rs`として抽出
- [ ] usecases/mod.rsで公開APIを定義
- [ ] infrastructure/utils/permissionとの連携を確認
- [ ] `cargo test` でテスト確認

##### Phase 16.5: DTOの再構成（30分）
- [ ] `features/security/dto/requests/`ディレクトリを作成
- [ ] `features/security/dto/responses/`ディレクトリを作成
- [ ] 既存のdto/security.rs, dto/permission.rsを分析
- [ ] shared/dto/role_types.rsは共通型として維持
- [ ] リクエストDTOをrequests/に配置
- [ ] レスポンスDTOをresponses/に配置
- [ ] `cargo clippy --all-targets` で警告なし確認

##### Phase 16.6: Handlers層の移行と統合（45分）
- [ ] `features/security/handlers/`ディレクトリを作成
- [ ] `api/handlers/security_handler.rs` → `features/security/handlers/security.rs`
- [ ] `api/handlers/role_handler.rs` → `features/security/handlers/role.rs`
- [ ] `api/handlers/permission_handler.rs` → `features/security/handlers/permission.rs`
- [ ] handlers/mod.rsで統合ルーターを提供
- [ ] main.rsのインポートを更新
- [ ] `make ci-check-fast` で全テストがパス

#### 👨‍💼 Phase 17: Admin機能の完全実装

**現状**: DTOのみ存在（最も複雑）、複数のサービスが分散
**目標**: 管理者向け統合機能として再構築

##### Phase 17.1: Services層の分析と移行（45分）
- [ ] `features/admin/services/`ディレクトリを作成
- [ ] `service/admin_organization_service.rs` → `features/admin/services/organization.rs`
- [ ] `service/analytics_service.rs` → `features/admin/services/analytics.rs`
- [ ] services/mod.rsで公開APIを定義
- [ ] 依存関係の分析（他featureのサービスへの依存を確認）
- [ ] 循環依存がないことを確認
- [ ] `cargo test --lib` でビルド確認

##### Phase 17.2: Repositories層の確認（15分）
- [ ] Admin専用のリポジトリが必要か確認
- [ ] 既存の他featureのリポジトリを再利用するパターンを確認
- [ ] 必要に応じて`features/admin/repositories/`を作成
- [ ] 統計情報用の専用リポジトリが必要な場合は作成

##### Phase 17.3: Usecases層の作成（45分）
- [ ] `features/admin/usecases/`ディレクトリを作成
- [ ] 組織管理操作を`organization_management.rs`に整理
- [ ] 統計・分析処理を`analytics_operations.rs`に整理
- [ ] ユーザー管理操作を`user_management.rs`に整理
- [ ] サブスクリプション管理を`subscription_management.rs`に整理
- [ ] usecases/mod.rsで公開APIを定義
- [ ] `cargo test` でテスト確認

##### Phase 17.4: DTOの整理とサブモジュール化（45分）
- [ ] `features/admin/dto/requests/`ディレクトリを作成
- [ ] `features/admin/dto/responses/`ディレクトリを作成
- [ ] 機能別サブディレクトリを作成:
  - [ ] `dto/requests/organization/`
  - [ ] `dto/requests/analytics/`
  - [ ] `dto/requests/subscription/`
  - [ ] `dto/responses/organization/`
  - [ ] `dto/responses/analytics/`
  - [ ] `dto/responses/subscription/`
- [ ] 既存のDTOを適切なサブディレクトリに配置
- [ ] admin_operations.rsのインラインDTOも整理
- [ ] `cargo clippy --all-targets` で警告なし確認

##### Phase 17.5: Handlers層の移行と統合（45分）
- [ ] `features/admin/handlers/`ディレクトリを作成
- [ ] `api/handlers/admin_handler.rs` → `features/admin/handlers/admin.rs`
- [ ] `api/handlers/analytics_handler.rs` → `features/admin/handlers/analytics.rs`
- [ ] handlers/mod.rsで統合ルーターを提供
- [ ] servicesへのインポートを`super::services`に更新
- [ ] usecasesへのインポートを`super::usecases`に更新
- [ ] 他featureへの依存を整理（features::team::services等）
- [ ] `cargo test` で既存テストの動作確認

##### Phase 17.6: 最終統合とテスト（45分）
- [ ] features/admin/mod.rsで全モジュールを適切に公開
- [ ] main.rsのインポートを`features::admin::handlers`に更新
- [ ] 管理者権限のミドルウェアとの連携確認
- [ ] 統合テストの実行と確認
- [ ] `make ci-check-fast` で全テストがパス
- [ ] `cargo clippy --all-targets --all-features -- -D warnings`

#### 💳 Phase 18: Subscription機能の完全実装

**現状**: DTOのみ存在、core/subscription_tier.rsとの連携必要
**目標**: サブスクリプション管理機能として再構築

##### Phase 18.1: Models層の移行（30分）
- [ ] `features/subscription/models/`ディレクトリを作成
- [ ] `domain/subscription_history_model.rs` → `features/subscription/models/history.rs`
- [ ] models/mod.rsで公開APIを定義
- [ ] core::subscription_tierへの依存を確認
- [ ] `cargo test --lib` でビルド確認

##### Phase 18.2: Repositories層の移行（30分）
- [ ] `features/subscription/repositories/`ディレクトリを作成
- [ ] `repository/subscription_history_repository.rs` → `features/subscription/repositories/history.rs`
- [ ] repositories/mod.rsで公開APIを定義
- [ ] modelsへのインポートパスを`super::models`に更新
- [ ] `cargo test --lib` でビルド確認

##### Phase 18.3: Services層の移行（30分）
- [ ] `features/subscription/services/`ディレクトリを作成
- [ ] `service/subscription_service.rs` → `features/subscription/services/subscription.rs`
- [ ] services/mod.rsで公開APIを定義
- [ ] repositoriesへのインポートを`super::repositories`に更新
- [ ] core::subscription_tierの使用を確認
- [ ] `cargo test service::subscription` で既存テストの動作確認

##### Phase 18.4: DTOの再構成（30分）
- [ ] `features/subscription/dto/requests/`ディレクトリを作成
- [ ] `features/subscription/dto/responses/`ディレクトリを作成
- [ ] 既存のdto/subscription.rsを分析
- [ ] リクエストDTOをrequests/に配置
- [ ] レスポンスDTOをresponses/に配置
- [ ] `cargo clippy --all-targets` で警告なし確認

##### Phase 18.5: Handlers層の移行（30分）
- [ ] `features/subscription/handlers/`ディレクトリを作成
- [ ] `api/handlers/subscription_handler.rs` → `features/subscription/handlers/subscription.rs`
- [ ] handlers/mod.rsで公開APIを定義
- [ ] servicesへのインポートを`super::services`に更新
- [ ] main.rsのインポートを更新
- [ ] `make ci-check-fast` で全テストがパス

#### 🔄 Phase 19: 残存ファイルの整理と移行

**目標**: api/, service/, repository/, domain/ディレクトリに残存するファイルを適切に移行

##### Phase 19.1: 残存ファイルの調査（30分）
- [ ] `find src/api -name "*.rs" | grep -v mod.rs` で残存ハンドラーをリスト
- [ ] `find src/service -name "*.rs" | grep -v mod.rs` で残存サービスをリスト
- [ ] `find src/repository -name "*.rs" | grep -v mod.rs` で残存リポジトリをリスト
- [ ] `find src/domain -name "*.rs" | grep -v mod.rs` で残存モデルをリスト
- [ ] 各ファイルの機能と依存関係を分析
- [ ] 移行先の決定（既存feature or 新規feature or infrastructure）

##### Phase 19.2: Health機能の移行（30分）
- [ ] `infrastructure/health/`ディレクトリを作成
- [ ] `api/handlers/health_handler.rs` → `infrastructure/health/handler.rs`
- [ ] ヘルスチェック関連のロジックを整理
- [ ] main.rsのインポートを更新
- [ ] `cargo test` でテスト確認

##### Phase 19.3: User関連の統合（30分）
- [ ] user_model.rsの移行先を決定（features/auth/models/へ）
- [ ] user関連の残存ファイルをfeatures/authに統合
- [ ] 依存関係の更新
- [ ] `cargo test` でテスト確認

##### Phase 19.4: その他の残存ファイル処理（30分）
- [ ] 汎用的なユーティリティは`shared/`へ
- [ ] インフラ系は`infrastructure/`へ
- [ ] ビジネスロジックは適切なfeatureへ
- [ ] 不要なファイルは削除
- [ ] `cargo clippy --all-targets` で警告なし確認

##### Phase 19.5: ディレクトリクリーンアップ（30分）
- [ ] 空になったディレクトリの削除
- [ ] mod.rsファイルの整理
- [ ] 不要な再エクスポートの削除
- [ ] `make ci-check-fast` で全テストがパス

#### 🏗️ Phase 20: Workspace構成への移行準備

**目標**: 各featureモジュールを独立したクレートとして分離可能にする

##### Phase 20.1: 依存関係の分析（45分）
- [ ] 各featureモジュールの外部依存をリストアップ
- [ ] feature間の依存関係をグラフ化
- [ ] 循環依存がないことを確認
- [ ] 共通依存の最小化案を作成

##### Phase 20.2: インターフェース定義（45分）
- [ ] 各featureの公開APIを明確化
- [ ] trait定義による抽象化の検討
- [ ] featureプラグインシステムの設計
- [ ] 依存注入パターンの適用箇所を特定

##### Phase 20.3: Cargo.toml案の作成（45分）
- [ ] ワークスペースレベルのCargo.toml案
- [ ] 各featureクレートのCargo.toml案
- [ ] 共通依存の管理方法を決定
- [ ] ビルド最適化設定の検討

##### Phase 20.4: 移行計画の策定（45分）
- [ ] 段階的移行のロードマップ作成
- [ ] 各段階でのテスト計画
- [ ] ロールバック計画
- [ ] ドキュメント更新計画

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

#### 📊 実装効果の測定

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

#### 🎯 移行戦略の原則

1. **後方互換性の維持**
   - 既存のインポートが動作し続けるよう、段階的に移行
   - 一度に全てを変更せず、小さなステップで実施

2. **テスト駆動での移行**
   - 各変更前後でテストスイートが通ることを確認
   - 新しいインポートパスでのテストを先に作成

3. **影響範囲の最小化**
   - 一度に1つのモジュールのみを変更
   - 依存関係の少ないものから着手

4. **ドキュメント化**
   - 各Phaseの実施内容と結果を記録
   - 新しいモジュール構造の使用方法を文書化

#### 🚫 循環依存を防ぐための設計原則

1. **レイヤー間の依存方向**
   ```
   handlers → services → repositories → models
      ↓          ↓           ↓            ↓
     dto     usecases      dto         core
   ```
   - 上位層は下位層に依存（逆は禁止）
   - 同一層内での相互依存も避ける

2. **Feature間の依存関係**
   - 直接的な相互依存は禁止
   - 共通機能は`shared/`または`core/`に抽出
   - インターフェース（trait）による疎結合化

3. **DTO設計の原則**
   - DTOはその機能内で完結（他featureのDTOを参照しない）
   - 共通型は`shared/types/`に配置
   - Service層からDTO層への逆依存は絶対禁止

4. **依存関係のチェック方法**
   ```bash
   # 各サブフェーズ完了時に実行
   cargo test --lib
   cargo clippy --all-targets
   
   # 循環依存の確認
   cargo deps --all-features | grep -E "cycle|circular"
   ```

5. **問題が発生した場合の対処**
   - 共通型の抽出：`shared/types/`へ
   - インターフェースの導入：trait定義
   - イベント駆動：直接呼び出しを避ける
   - 依存性注入：コンストラクタでの注入

#### 🛡️ リファクタリング時のリスク軽減方針

**1. Feature間の相互依存への対処**

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

**2. 共通機能の抽出タイミング**

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

**3. ビルド時間増加への対処**

**並列ビルド戦略**:
```bash
# 1. 変更したfeatureのみをテスト（開発中）
cargo test -p feature_team

# 2. 関連featureも含めてテスト（サブフェーズ完了時）
cargo test -p feature_team -p feature_organization

# 3. 全体テスト（Phase完了時のみ）
make ci-check-fast
```
**4. 段階的移行のチェックポイント**

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
```
