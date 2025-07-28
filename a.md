# クレート分割前の事前整理手順書

## 概要

本ドキュメントは、クレート分割を実施する前に既存のtask-backendコードベース内で実施すべきリファクタリング手順を詳細に記載します。この事前整理により、クレート分割時の作業をパス変更のみに限定し、リスクを最小化します。

## 設計方針

**「Clean Architecture + Rust慣用句」アプローチ**を採用し、以下の原則に従います：

1. **レイヤーの責務を明確化**
   - Handler層: HTTP処理とDTO
   - Service層: ビジネスロジックとDTO⇔Entity変換
   - Repository層: 純粋なデータアクセス（Entityのみ）

2. **Rustのベストプラクティスに準拠**
   - ジェネリクスによるゼロコスト抽象化
   - 型安全性の維持
   - 動的ディスパッチの最小化

3. **統一的な実装パターン**
   - すべてのFeatureで同じアーキテクチャ
   - 明確で一貫したデータフロー

## 目的

1. **依存関係の整理**: DTO依存を適切な層に限定
2. **疎結合の実現**: ジェネリクスベースの依存性注入
3. **移行リスクの軽減**: 段階的かつ統一的な変更により各ステップでの動作確認を可能に

## やらないこと

以下の作業はこの事前整理の範囲外とし、実施しません：

1. **新機能の追加**: 依存関係の整理に集中し、新しい機能は追加しない
2. **ビジネスロジックの変更**: 既存のロジックは変更せず、構造のみを変更
3. **過度な最適化**: 必要以上のパフォーマンスチューニングは行わない
4. **データベーススキーマの変更**: マイグレーションは実施しない
5. **APIインターフェースの変更**: 外部に公開されているAPIは一切変更しない
6. **テストロジックの変更**: 既存テストの修正は最小限に留める（importパスのみ）
7. **エラーハンドリングの改善**: 既存のエラー処理は変更しない
8. **ログ出力の追加・変更**: デバッグ用のログも追加しない
9. **コメントやドキュメントの追加**: 必要最小限の変更のみ
10. **不要コードの削除**: dead_codeがあってもこの段階では削除しない

## Phase 1: Mapper層の導入（推定作業時間: 1日）

### 1.1 目的

DTO-Entity間の変換ロジックを専用のMapper層に集約し、各層の責務を明確化します。

### 1.2 Mapperモジュールの作成

**新規ファイル**: `src/mappers/mod.rs`

```rust
pub mod task_mapper;
pub mod user_mapper;
pub mod team_mapper;
// 他のmapperモジュールも同様に追加
```

**新規ファイル**: `src/mappers/task_mapper.rs`

```rust
use crate::api::dto::task_dto::{CreateTaskDto, UpdateTaskDto, TaskDto};
use crate::domain::task_model::{ActiveModel, Model};
use crate::domain::task_status::TaskStatus;
use crate::domain::task_visibility::TaskVisibility;
use chrono::Utc;
use sea_orm::Set;
use uuid::Uuid;

pub struct TaskMapper;

impl TaskMapper {
    /// CreateTaskDto → ActiveModel変換
    pub fn create_dto_to_entity(dto: CreateTaskDto) -> ActiveModel {
        ActiveModel {
            id: Set(Uuid::new_v4()),
            title: Set(dto.title),
            description: Set(dto.description),
            status: Set(dto.status.unwrap_or(TaskStatus::Todo).to_string()),
            priority: Set(dto.priority.unwrap_or_default()),
            user_id: Set(dto.user_id),
            team_id: Set(dto.team_id),
            organization_id: Set(dto.organization_id),
            visibility: Set(dto.visibility.unwrap_or(TaskVisibility::Private)),
            due_date: Set(dto.due_date),
            assigned_to: Set(dto.assigned_to),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            completed_at: Set(None),
            completion_duration_hours: Set(None),
        }
    }
    
    /// UpdateTaskDto → ActiveModel変換（部分更新）
    pub fn update_dto_to_entity(id: Uuid, dto: UpdateTaskDto) -> ActiveModel {
        let mut active_model = ActiveModel {
            id: Set(id),
            ..Default::default()
        };
        
        if let Some(title) = dto.title {
            active_model.title = Set(title);
        }
        if let Some(description) = dto.description {
            active_model.description = Set(Some(description));
        }
        // 他のフィールドも同様に処理
        
        active_model.updated_at = Set(Utc::now());
        active_model
    }
    
    /// Model → TaskDto変換
    pub fn entity_to_dto(entity: Model) -> TaskDto {
        TaskDto::from(entity) // 既存のFrom実装を利用
    }
}
```

### 1.3 lib.rsへの追加

```rust
// src/lib.rs に追加
pub mod mappers;
```

## Phase 2: Repository層の純粋化とDTO依存の削除（推定作業時間: 1-2日）

### 2.1 Repository層のインターフェース変更

**対象ファイル例**: `src/repository/task_repository.rs`

```rust
// === 変更前 ===
use crate::api::dto::task_dto::{CreateTaskDto, UpdateTaskDto};

impl TaskRepository {
    pub async fn create(&self, dto: CreateTaskDto) -> Result<task_model::Model, DbErr> {
        let task = task_model::ActiveModel {
            id: Set(Uuid::new_v4()),
            title: Set(dto.title),
            description: Set(dto.description),
            // ... DTOから直接マッピング
        };
        // ...
    }
}

// === 変更後 ===
// DTOのimportを削除
use crate::domain::task_model::{ActiveModel, Model};

impl TaskRepository {
    pub async fn create(&self, task: ActiveModel) -> Result<Model, DbErr> {
        // 純粋なデータ永続化のみ
        self.prepare_connection().await?;
        TaskEntity::insert(task)
            .exec(&self.db)
            .await
    }
    
    pub async fn update(&self, id: Uuid, task: ActiveModel) -> Result<Model, DbErr> {
        self.prepare_connection().await?;
        TaskEntity::update(task)
            .filter(task_model::Column::Id.eq(id))
            .exec(&self.db)
            .await
    }
}
```

### 2.2 Service層でのMapper使用

**対象ファイル例**: `src/service/task_service.rs`

```rust
// === 変更前 ===
use crate::repository::task_repository::TaskRepository;

pub struct TaskService {
    repo: Arc<TaskRepository>,
    // ...
}

pub async fn create_task(&self, payload: CreateTaskDto) -> AppResult<TaskDto> {
    let created_task = self.repo.create(payload).await?;
    Ok(created_task.into())
}

// === 変更後 ===
use crate::repository::task_repository::TaskRepository;
use crate::mappers::task_mapper::TaskMapper;

pub struct TaskService {
    repo: Arc<TaskRepository>,
    mapper: TaskMapper, // Mapperを追加
    // ...
}

impl TaskService {
    pub fn new(db_pool: DbPool) -> Self {
        Self {
            repo: Arc::new(TaskRepository::new(db_pool)),
            mapper: TaskMapper,
            // ...
        }
    }
    
    pub async fn create_task(&self, payload: CreateTaskDto) -> AppResult<TaskDto> {
        // Mapperを使用してDTO → Entity変換
        let entity = TaskMapper::create_dto_to_entity(payload);
        let created_task = self.repo.create(entity).await?;
        Ok(TaskMapper::entity_to_dto(created_task))
    }
    
    pub async fn update_task(&self, id: Uuid, payload: UpdateTaskDto) -> AppResult<TaskDto> {
        // 部分更新もMapperで処理
        let entity = TaskMapper::update_dto_to_entity(id, payload);
        let updated_task = self.repo.update(id, entity).await?;
        Ok(TaskMapper::entity_to_dto(updated_task))
    }
}
```

### 2.3 変更対象ファイル一覧

1. **Repository層**（DTOのimportを削除、Entityのみを扱う）
   - `task_repository.rs`
   - `user_repository.rs`
   - `team_repository.rs`
   - `organization_repository.rs`
   - その他すべてのrepositoryファイル

2. **Service層**（Mapperを追加し、変換処理を実装）
   - `task_service.rs`
   - `user_service.rs`
   - `team_service.rs`
   - `organization_service.rs`
   - その他すべてのserviceファイル

### 2.4 テスト方法

各変更後に以下のテストを実施：

```bash
# 単体テストの実行
cargo test --package task-backend --lib repository::task_repository::tests
cargo test --package task-backend --lib service::task_service::tests

# 統合テストの実行
cargo test --package task-backend --test integration::tasks

# 全体のテスト
cargo test
```

## Phase 3: Repository/Service traitの定義（推定作業時間: 1日）

### 3.1 トレイトモジュールの作成

**新規ファイル**: `src/traits/mod.rs`

```rust
pub mod repositories;
pub mod services;

pub use repositories::*;
pub use services::*;
```

### 3.2 Repository traitの定義

**新規ファイル**: `src/traits/repositories.rs`

```rust
use crate::domain::{task_model, user_model, team_model};
use crate::error::AppResult;
use async_trait::async_trait;
use sea_orm::{ActiveModel, DbErr};
use uuid::Uuid;

#[async_trait]
pub trait TaskRepositoryTrait: Send + Sync {
    async fn create(&self, task: task_model::ActiveModel) -> Result<task_model::Model, DbErr>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<task_model::Model>, DbErr>;
    async fn update(&self, id: Uuid, task: task_model::ActiveModel) -> Result<task_model::Model, DbErr>;
    async fn delete(&self, id: Uuid) -> Result<(), DbErr>;
}

#[async_trait]
pub trait UserRepositoryTrait: Send + Sync {
    async fn create(&self, user: user_model::ActiveModel) -> Result<user_model::Model, DbErr>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<user_model::Model>, DbErr>;
    async fn find_by_email(&self, email: &str) -> Result<Option<user_model::Model>, DbErr>;
}

#[async_trait]
pub trait TeamRepositoryTrait: Send + Sync {
    async fn create(&self, team: team_model::ActiveModel) -> Result<team_model::Model, DbErr>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<team_model::Model>, DbErr>;
}
```

### 3.3 既存Repositoryへのトレイト実装

**変更ファイル**: `src/repository/task_repository.rs`

```rust
use crate::traits::TaskRepositoryTrait;
use async_trait::async_trait;

// 既存のimpl TaskRepositoryブロックの後に追加
#[async_trait]
impl TaskRepositoryTrait for TaskRepository {
    async fn create(&self, task: task_model::ActiveModel) -> Result<task_model::Model, DbErr> {
        self.create(task).await // 既存のメソッドを呼び出す
    }
    
    async fn find_by_id(&self, id: Uuid) -> Result<Option<task_model::Model>, DbErr> {
        self.find_by_id(id).await
    }
    
    async fn update(&self, id: Uuid, task: task_model::ActiveModel) -> Result<task_model::Model, DbErr> {
        self.update(id, task).await
    }
    
    async fn delete(&self, id: Uuid) -> Result<(), DbErr> {
        self.delete(id).await
    }
}
```

### 3.4 lib.rsへの追加

**変更ファイル**: `src/lib.rs`

```rust
// === 追加 ===
pub mod traits;
```

## Phase 4: ジェネリクスベースの依存性注入（推定作業時間: 1-2日）

### 4.1 ジェネリクスベースのService定義

**変更ファイル**: `src/service/task_service.rs`

```rust
// === 変更前 ===
use crate::repository::user_repository::UserRepository;
use crate::repository::team_repository::TeamRepository;
use crate::service::team_service::TeamService;

pub struct TaskService {
    repo: Arc<TaskRepository>,
    user_repo: Arc<UserRepository>,
    team_repo: Arc<TeamRepository>,
    team_service: Arc<TeamService>,
}

// === 変更後 ===
use crate::traits::{UserRepositoryTrait, TeamRepositoryTrait};
use crate::mappers::task_mapper::TaskMapper;

pub struct TaskService<UR, TR> 
where 
    UR: UserRepositoryTrait,
    TR: TeamRepositoryTrait,
{
    task_repo: Arc<TaskRepository>,
    user_repo: Arc<UR>,
    team_repo: Arc<TR>,
    mapper: TaskMapper,
}

impl<UR, TR> TaskService<UR, TR> 
where 
    UR: UserRepositoryTrait,
    TR: TeamRepositoryTrait,
{
    pub fn new(
        db_pool: DbPool,
        user_repo: Arc<UR>,
        team_repo: Arc<TR>,
    ) -> Self {
        Self {
            task_repo: Arc::new(TaskRepository::new(db_pool)),
            user_repo,
            team_repo,
            mapper: TaskMapper,
        }
    }
    
    pub async fn create_team_task(&self, team_id: Uuid, dto: CreateTaskDto) -> AppResult<TaskDto> {
        // チームの存在確認
        let team = self.team_repo.find_by_id(team_id).await?
            .ok_or_else(|| AppError::NotFound("Team not found".to_string()))?;
        
        // DTOをEntityに変換
        let mut entity = TaskMapper::create_dto_to_entity(dto);
        entity.team_id = Set(Some(team_id));
        entity.organization_id = Set(team.organization_id);
        
        // 作成
        let created = self.task_repo.create(entity).await?;
        Ok(TaskMapper::entity_to_dto(created))
    }
}
```

### 4.2 main.rsでの依存性注入

**変更ファイル**: `src/main.rs`

```rust
// === 変更後 ===
// Repositoryの作成
let user_repo = Arc::new(UserRepository::new(db_pool.clone()));
let team_repo = Arc::new(TeamRepository::new(db_pool.clone()));
let task_repo = Arc::new(TaskRepository::new(db_pool.clone()));

// ジェネリクスを使用したServiceの作成
let task_service = Arc::new(TaskService::new(
    db_pool.clone(),
    user_repo.clone(),
    team_repo.clone(),
));

// 型は自動的に推論される: TaskService<UserRepository, TeamRepository>
```

### 4.3 型エイリアスとファクトリ関数の活用

**変更ファイル**: `src/service/mod.rs`

```rust
// ユースケース層の型エイリアスを定義
pub type TaskUseCase = TaskService<UserRepository, TeamRepository>;
pub type UserUseCase = UserService<OrganizationRepository>;
pub type TeamUseCase = TeamService<UserRepository, OrganizationRepository>;
pub type AuthUseCase = AuthService<UserRepository, RefreshTokenRepository>;

// テスト用の型エイリアス
#[cfg(test)]
pub type MockTaskUseCase = TaskService<MockUserRepository, MockTeamRepository>;
#[cfg(test)]
pub type MockUserUseCase = UserService<MockOrganizationRepository>;
```

**変更ファイル**: `src/service/task_service.rs`

```rust
// ユースケース層のファクトリ関数を追加
impl TaskUseCase {
    /// ユースケース層の初期化関数
    pub fn new_usecase(db_pool: DbPool) -> Arc<Self> {
        let user_repo = Arc::new(UserRepository::new(db_pool.clone()));
        let team_repo = Arc::new(TeamRepository::new(db_pool.clone()));
        
        Arc::new(TaskService::new(db_pool, user_repo, team_repo))
    }
}

// APIハンドラーでの使用例
pub fn task_routes() -> Router {
    Router::new()
        .route("/tasks", post(create_task_handler))
        .route("/tasks/:id", get(get_task_handler))
}

async fn create_task_handler(
    State(usecase): State<Arc<TaskUseCase>>, // ユースケース層として明確
    Json(dto): Json<CreateTaskDto>,
) -> AppResult<Json<TaskDto>> {
    let task = usecase.create_task(dto).await?;
    Ok(Json(task))
}
```

**変更ファイル**: `src/main.rs`

```rust
// Clean Architectureの層構造を反映した初期化
let task_usecase = TaskUseCase::new_usecase(db_pool.clone());
let user_usecase = UserUseCase::new_usecase(db_pool.clone());
let team_usecase = TeamUseCase::new_usecase(db_pool.clone());
let auth_usecase = AuthUseCase::new_usecase(db_pool.clone());

// AppStateもユースケース層を明示
let app_state = AppState {
    task_usecase,
    user_usecase,
    team_usecase,
    auth_usecase,
    // ...
};
```

## 検証手順

### Phase完了ごとの確認項目

1. **コンパイル確認**
   ```bash
   cargo check
   cargo clippy --all-targets --all-features -- -D warnings
   ```

2. **テスト実行**
   ```bash
   cargo test
   ```

3. **ローカル動作確認**
   ```bash
   cargo run
   # 別ターミナルで
   curl http://localhost:3000/health
   ```

4. **パフォーマンステスト**
   - 主要なAPIエンドポイントのレスポンスタイムを計測
   - 変更前後で大きな性能劣化がないことを確認

## リスクと対策

### リスク1: 循環参照の発生
**対策**: 
- トレイトは`src/traits`に集約し、実装とは別モジュールにする
- Repositoryトレイトは純粋なデータアクセスのみを定義
- Serviceトレイトは高レベルのビジネスロジックのみを定義

### リスク2: ジェネリクスによる複雑性の増加

**問題点**:
- 型シグネチャが長くなり可読性が低下
- エラーメッセージが複雑化
- 新規開発者の学習コストが増加

**対策1: ユースケース層としての型エイリアス**
```rust
// src/service/mod.rs
// ユースケース層の具体的な実装
pub type TaskUseCase = TaskService<UserRepository, TeamRepository>;
pub type UserUseCase = UserService<OrganizationRepository>;
pub type TeamUseCase = TeamService<UserRepository, OrganizationRepository>;
pub type AuthUseCase = AuthService<UserRepository, RefreshTokenRepository>;

// これにより、Clean Architectureの意図が明確に
let task_usecase: Arc<TaskUseCase> = Arc::new(TaskService::new(...));
```

**対策2: テスト用の型エイリアス**
```rust
// src/service/mod.rs
#[cfg(test)]
pub type MockTaskUseCase = TaskService<MockUserRepository, MockTeamRepository>;
#[cfg(test)]
pub type MockUserUseCase = UserService<MockOrganizationRepository>;
```

**対策3: ファクトリ関数の提供**
```rust
// src/service/task_service.rs
impl TaskUseCase {
    /// ユースケース層のファクトリ関数
    pub fn new_usecase(db_pool: DbPool) -> Arc<Self> {
        let user_repo = Arc::new(UserRepository::new(db_pool.clone()));
        let team_repo = Arc::new(TeamRepository::new(db_pool.clone()));
        
        Arc::new(TaskService::new(db_pool, user_repo, team_repo))
    }
}
```

**実際の使用例**:
```rust
// main.rs - Clean Architectureの層を意識した初期化
let task_usecase = TaskUseCase::new_usecase(db_pool.clone());
let user_usecase = UserUseCase::new_usecase(db_pool.clone());

// handler.rs - ユースケース層であることが明確
pub async fn create_task_handler(
    State(usecase): State<Arc<TaskUseCase>>,
    Json(payload): Json<CreateTaskDto>,
) -> Result<Json<TaskDto>> {
    let result = usecase.create_task(payload).await?;
    Ok(Json(result))
}

// AppStateもClean Architectureを反映
struct AppState {
    task_usecase: Arc<TaskUseCase>,
    user_usecase: Arc<UserUseCase>,
    team_usecase: Arc<TeamUseCase>,
    auth_usecase: Arc<AuthUseCase>,
}
```

### リスク3: テストの複雑化
**対策**: テスト用のモック実装を用意
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    struct MockUserRepository;
    
    #[async_trait]
    impl UserRepositoryTrait for MockUserRepository {
        async fn find_by_id(&self, _: Uuid) -> Result<Option<user_model::Model>, DbErr> {
            Ok(Some(test_user()))
        }
    }
    
    #[tokio::test]
    async fn test_with_mock() {
        let mock_user_repo = Arc::new(MockUserRepository);
        let service = TaskService::new(test_db_pool(), mock_user_repo, ...);
        // テスト実行
    }
}
```

### リスク4: コンパイル時間の増加
**対策**:
- ジェネリクスの使用は必要最小限に留める
- 頻繁に変更されない部分から順に実装

## チェックリスト

- [ ] Phase 1: Mapper層の導入
  - [ ] mappersモジュールを作成
  - [ ] 各エンティティ用のMapperを実装
  - [ ] lib.rsにmappersモジュールを追加
  - [ ] コンパイルエラーなし

- [ ] Phase 2: Repository層の純粋化とDTO依存の削除
  - [ ] すべてのRepositoryからDTO importを削除
  - [ ] RepositoryメソッドをEntityのみを扱うように変更
  - [ ] ServiceでMapperを使用するように変更
  - [ ] 各Repositoryのテストが通過

- [ ] Phase 3: Repository/Service traitの定義
  - [ ] traitsモジュールを作成
  - [ ] Repository traitを定義
  - [ ] 既存Repositoryにトレイトを実装
  - [ ] lib.rsにtraitsモジュールを追加

- [ ] Phase 4: ジェネリクスベースの依存性注入
  - [ ] Serviceをジェネリクスベースに変更
  - [ ] main.rsでの初期化を更新
  - [ ] 型エイリアスを定義（オプション）
  - [ ] すべてのテストが通過

## 完了基準

1. すべてのテストが通過すること
2. `cargo clippy`で警告が出ないこと
3. 主要なAPIの動作確認が完了していること
4. パフォーマンスの大幅な劣化がないこと

## アーキテクチャの最終形

事前整理が完了すると、以下の統一的なアーキテクチャが実現されます：

```
┌─────────────────────────────────────────────┐
│  Handler層                                  │
│  - HTTPリクエスト/レスポンス処理            │
│  - DTOの受け渡し                            │
└────────────────┬────────────────────────────┘
                 │
┌────────────────▼────────────────────────────┐
│  Service層 (ジェネリクス)                   │
│  - ビジネスロジック                         │
│  - Mapperによる変換                         │
│  - Repository traitへの依存                 │
└────────────────┬────────────────────────────┘
                 │
┌────────────────▼────────────────────────────┐
│  Repository層 (純粋)                        │
│  - Entityのみを扱う                         │
│  - データアクセスのみ                       │
│  - DTOへの依存なし                          │
└─────────────────────────────────────────────┘
```

## 次のステップ

この事前整理が完了したら、CLAUDE.mdに記載されているクレート分割作業に進むことができます。事前整理により：

1. **各層の責務が明確**になり、クレート分割が容易
2. **依存関係が整理**され、循環参照のリスクが排除
3. **統一的な実装パターン**により、新しいクレート構成でも同じアーキテクチャを維持

クレート分割時は、純粋にimport/exportパスの変更のみに集中でき、ビジネスロジックの変更リスクを完全に排除できます。