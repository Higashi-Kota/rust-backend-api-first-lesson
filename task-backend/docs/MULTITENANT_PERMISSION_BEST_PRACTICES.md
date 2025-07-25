# マルチテナント機能における権限統一チェック機構のベストプラクティス

## 概要

本ドキュメントでは、マルチテナント機能実装における権限統一チェック機構のベストプラクティスと、実際のコードベースでの適用内容について詳細に説明します。

## 目次

1. [マルチテナント権限モデル](#マルチテナント権限モデル)
2. [権限チェック機構の実装](#権限チェック機構の実装)
3. [実装されている権限チェックパターン](#実装されている権限チェックパターン)
4. [ベストプラクティス](#ベストプラクティス)
5. [実装例](#実装例)
6. [パフォーマンス最適化](#パフォーマンス最適化)

## マルチテナント権限モデル

### 権限階層の概要

マルチテナント環境では、以下の階層で権限が管理されます：

```
システム (System)
├── 組織 (Organization)
│   ├── チーム (Team)
│   │   └── タスク (Task)
│   └── 部署 (Department)
```

### 権限マトリクス

#### 1. システムレベル権限
| ロール | 説明 | 権限範囲 |
|--------|------|----------|
| Admin | システム管理者 | 全リソースへの完全なアクセス権 |
| Member | 一般ユーザー | 自身のリソースのみアクセス可能 |

#### 2. 組織レベル権限
| ロール | 権限レベル | 組織管理 | メンバー管理 | チーム作成 | 分析閲覧 |
|--------|------------|----------|--------------|------------|----------|
| Owner | 最上位 | ✓ | ✓ | ✓ | ✓ |
| Admin | 管理者 | ✓ | ✓ | ✓ | ✓ |
| Member | 一般 | × | × | △※1 | △※2 |

※1 組織の設定により制限可能
※2 自身が関わるデータのみ

#### 3. チームレベル権限
| ロール | 権限レベル | チーム設定 | メンバー管理 | リソース作成 | リソース閲覧 | リソース編集 | リソース削除 |
|--------|------------|------------|--------------|--------------|--------------|--------------|--------------|
| Owner | 最上位 | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Admin | 管理者 | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Member | 一般 | × | × | ✓ | ✓ | △※3 | △※3 |
| Viewer | 閲覧のみ | × | × | × | ✓ | × | × |

※3 自身が作成したリソースのみ、またはチーム設定による

### 権限の優先順位

1. **システムAdmin** > すべての権限（最優先）
2. **組織Owner** > 組織内のすべての権限
3. **組織Admin** ≈ 組織Owner（一部制限あり）
4. **チームOwner** > チーム内のすべての権限
5. **チームAdmin** ≈ チームOwner（一部制限あり）
6. **部署Manager** > 部署内の管理権限
7. その他のロールは上記の階層に従う

## 権限チェック機構の実装

### 1. ミドルウェアレベルのチェック

権限チェックは主に以下の3つのレベルで実装されています：

```rust
// 1. 認証ミドルウェア（AuthMiddleware）
// すべてのAPIリクエストに対してJWTトークンを検証

// 2. 権限ミドルウェア（PermissionMiddleware）
// リソースとアクションに基づいた権限チェック

// 3. サービスレベルのチェック
// より詳細なビジネスロジックに基づく権限検証
```

### 2. PermissionContext の活用

```rust
#[derive(Clone, Debug)]
pub struct PermissionContext {
    pub user_id: Uuid,
    pub role: RoleWithPermissions,
    pub resource: &'static str,
    pub action: Action,
}
```

PermissionContextは権限チェック後にリクエストに付加され、ハンドラーやサービス層で追加の権限情報として利用できます。

### 3. 統一権限チェックマクロ

```rust
#[macro_export]
macro_rules! require_permission {
    ($resource:expr, $action:expr) => {
        axum::middleware::from_fn(move |req, next| {
            let resource = $resource;
            let action = $action;
            $crate::middleware::authorization::permission_middleware(resource, action)(req, next)
        })
    };
}
```

## 実装されている権限チェックパターン

### 1. タスクリソースの権限チェック

```rust
// task_handler.rs での実装例
.route(
    "/tasks",
    get(list_tasks)  // 自分のタスクのみ表示（権限チェック不要）
        .post(create_task)
        .route_layer(require_permission!(resources::TASK, Action::Create))
)
.route(
    "/tasks/{id}",
    get(get_task)
        .route_layer(require_permission!(resources::TASK, Action::View))
        .patch(update_task)
        .route_layer(require_permission!(resources::TASK, Action::Update))
        .delete(delete_task)
        .route_layer(require_permission!(resources::TASK, Action::Delete))
)
```

### 2. チーム・組織レベルの権限チェック

```rust
// team_service.rs での実装
pub async fn check_team_access_by_id(
    &self,
    user_id: Uuid,
    team_id: Uuid,
    required_role: Option<TeamRole>,
) -> AppResult<TeamMember> {
    let member = self
        .repo
        .find_team_member(team_id, user_id)
        .await
        .map_err(|e| {
            internal_server_error(
                e,
                "team_service::check_team_access_by_id",
                "Failed to find team member",
            )
        })?
        .ok_or_else(|| {
            not_found_error(
                anyhow::anyhow!("Team member not found"),
                "team_service::check_team_access_by_id",
                "You are not a member of this team",
            )
        })?;

    // ロールのチェック
    if let Some(required_role) = required_role {
        if !member.role.has_permission_for(&required_role) {
            return Err(forbidden_error(
                anyhow::anyhow!("Insufficient team role"),
                "team_service::check_team_access_by_id",
                "You don't have the required role in this team",
            ));
        }
    }

    Ok(member)
}
```

### 3. マルチテナントタスクアクセス

```rust
// task_service.rs での実装
pub async fn filter_tasks_for_user(
    &self,
    filters: TaskFilter,
    user_id: Uuid,
    is_admin: bool,
) -> AppResult<PagedResponse<Task>> {
    // 管理者は全タスクにアクセス可能
    if is_admin {
        return self
            .repo
            .find_all_with_pagination(filters)
            .await
            .map_err(|e| {
                internal_server_error(
                    e,
                    "task_service::filter_tasks_for_user:admin",
                    "Failed to fetch tasks",
                )
            });
    }

    // ユーザーのチームIDを取得
    let team_ids = self
        .team_service
        .get_user_team_ids(user_id)
        .await
        .map_err(|e| {
            internal_server_error(
                e,
                "task_service::filter_tasks_for_user:teams",
                "Failed to get user teams",
            )
        })?;

    // ユーザーの組織IDを取得
    let organization_ids = self
        .organization_service
        .get_user_organization_ids(user_id)
        .await
        .map_err(|e| {
            internal_server_error(
                e,
                "task_service::filter_tasks_for_user:orgs",
                "Failed to get user organizations",
            )
        })?;

    // フィルター条件に基づいてタスクを取得
    self.repo
        .find_tasks_for_user_with_filters(
            user_id,
            team_ids,
            organization_ids,
            filters,
        )
        .await
        .map_err(|e| {
            internal_server_error(
                e,
                "task_service::filter_tasks_for_user:query",
                "Failed to fetch filtered tasks",
            )
        })
}
```

## ベストプラクティス

### 1. 権限チェックの一元化

**推奨事項：**
- すべての権限チェックは統一ミドルウェアを通じて実施
- 直接的な権限チェック（`is_admin()`など）の使用は避ける
- サービス層での追加チェックが必要な場合は、明確な理由をコメントで記載

```rust
// ✅ 良い例：ミドルウェアで権限チェック
.route_layer(require_permission!(resources::TASK, Action::Update))

// ❌ 避けるべき例：ハンドラー内での直接チェック
if !user.is_admin() {
    return Err(AppError::Forbidden("Admin access required".to_string()));
}
```

### 2. エラーハンドリングの統一

**エラーコンテキストの命名規則：**
```rust
// 形式: "モジュール名::関数名[:詳細]"
convert_validation_errors(e, "task_handler::create_task")
internal_server_error(e, "team_service::get_team", "Failed to fetch team")
```

### 3. リソース固有の権限チェック

**サービス層での実装例：**
```rust
pub async fn check_task_access(
    &self,
    user_id: Uuid,
    task_id: Uuid,
    action: &str,
) -> AppResult<Task> {
    let task = self.get_task_by_id(task_id).await?;
    
    // 個人タスクの場合
    if task.visibility == TaskVisibility::Personal {
        if task.user_id != user_id {
            return Err(forbidden_error(
                anyhow::anyhow!("Access denied"),
                "task_service::check_task_access",
                "You don't have access to this task",
            ));
        }
    }
    
    // チームタスクの場合
    if task.visibility == TaskVisibility::Team {
        if let Some(team_id) = task.team_id {
            self.team_service
                .check_team_access_by_id(user_id, team_id, None)
                .await?;
        }
    }
    
    // 組織タスクの場合
    if task.visibility == TaskVisibility::Organization {
        if let Some(org_id) = task.organization_id {
            let is_member = self
                .organization_service
                .is_organization_member(org_id, user_id)
                .await?;
            if !is_member {
                return Err(forbidden_error(
                    anyhow::anyhow!("Not an organization member"),
                    "task_service::check_task_access",
                    "You must be an organization member to access this task",
                ));
            }
        }
    }
    
    Ok(task)
}
```

### 4. テストの実施

**権限テストの必須パターン：**
```rust
// 1. 正常系（権限あり）
#[tokio::test]
async fn test_update_task_as_owner() {
    let task = create_personal_task(&user).await;
    let response = update_task(&app, &user.token, task.id, &payload).await;
    assert_eq!(response.status(), StatusCode::OK);
}

// 2. 権限なしエラー（403 Forbidden）
#[tokio::test]
async fn test_update_task_forbidden() {
    let other_user = create_and_authenticate_user(&app).await;
    let task = create_personal_task(&user).await;
    let response = update_task(&app, &other_user.token, task.id, &payload).await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

// 3. 認証なしエラー（401 Unauthorized）
#[tokio::test]
async fn test_update_task_unauthorized() {
    let task = create_personal_task(&user).await;
    let response = update_task(&app, "", task.id, &payload).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
```

## 実装例

### 1. マルチテナント対応タスクハンドラー

```rust
pub async fn create_task_with_team(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<CreateTaskWithTeamDto>,
) -> AppResult<ApiResponse<TaskDto>> {
    // チームメンバーシップの確認
    if let Some(team_id) = payload.team_id {
        app_state
            .team_service
            .check_team_access_by_id(user.user_id(), team_id, None)
            .await?;
    }
    
    // タスクの作成
    let task = app_state
        .task_service
        .create_task_with_visibility(
            user.user_id(),
            payload.into(),
            payload.team_id,
            payload.organization_id,
            payload.visibility,
        )
        .await?;
    
    Ok(ApiResponse::created(task.into()))
}
```

### 2. スコープ付きタスク一覧取得

```rust
pub async fn list_tasks_with_scope(
    State(app_state): State<AppState>,
    user: AuthenticatedUser,
    Query(query): Query<TaskSearchQuery>,
) -> AppResult<ApiResponse<PagedResponse<TaskDto>>> {
    let mut filters = TaskFilter::from(query);
    
    // スコープに基づいてフィルターを調整
    match query.visibility {
        Some(TaskVisibility::Team) => {
            if let Some(team_id) = query.team_id {
                // 特定チームのタスク
                filters.team_id = Some(team_id);
            }
        }
        Some(TaskVisibility::Organization) => {
            if let Some(org_id) = query.organization_id {
                // 特定組織のタスク
                filters.organization_id = Some(org_id);
            }
        }
        _ => {
            // デフォルトは個人タスク
            filters.user_id = Some(user.user_id());
        }
    }
    
    let tasks = app_state
        .task_service
        .filter_tasks_for_user(filters, user.user_id(), user.is_admin())
        .await?;
    
    Ok(ApiResponse::ok(tasks.map_items(TaskDto::from)))
}
```

## パフォーマンス最適化

### 1. データベースインデックス

マルチテナント機能のパフォーマンスを最適化するため、以下のインデックスが実装されています：

```sql
-- 単一カラムインデックス
CREATE INDEX idx_tasks_team_id ON tasks(team_id);
CREATE INDEX idx_tasks_organization_id ON tasks(organization_id);
CREATE INDEX idx_tasks_visibility ON tasks(visibility);
CREATE INDEX idx_tasks_assigned_to ON tasks(assigned_to);

-- 複合インデックス（別のマイグレーションファイルで実装予定）
CREATE INDEX idx_tasks_team_visibility ON tasks(team_id, visibility);
CREATE INDEX idx_tasks_org_visibility ON tasks(organization_id, visibility);
```

### 2. クエリ最適化

```rust
// 効率的なマルチテナントクエリ
pub async fn find_tasks_for_user_with_filters(
    &self,
    user_id: Uuid,
    team_ids: Vec<Uuid>,
    organization_ids: Vec<Uuid>,
    filters: TaskFilter,
) -> AppResult<PagedResponse<Task>> {
    let mut query = Task::find();
    
    // OR条件を使用して1回のクエリで取得
    query = query.filter(
        Condition::any()
            // 個人タスク
            .add(
                sea_orm::Condition::all()
                    .add(Column::UserId.eq(user_id))
                    .add(Column::Visibility.eq(TaskVisibility::Personal))
            )
            // チームタスク
            .add_option(if !team_ids.is_empty() {
                Some(
                    sea_orm::Condition::all()
                        .add(Column::TeamId.is_in(team_ids))
                        .add(Column::Visibility.eq(TaskVisibility::Team))
                )
            } else {
                None
            })
            // 組織タスク
            .add_option(if !organization_ids.is_empty() {
                Some(
                    sea_orm::Condition::all()
                        .add(Column::OrganizationId.is_in(organization_ids))
                        .add(Column::Visibility.eq(TaskVisibility::Organization))
                )
            } else {
                None
            })
            // 自分に割り当てられたタスク
            .add_option(if filters.include_assigned {
                Some(Column::AssignedTo.eq(user_id))
            } else {
                None
            })
    );
    
    // その他のフィルター条件を適用
    self.apply_filters(query, filters).await
}
```

### 3. キャッシュ戦略

```rust
// ユーザーの所属情報をキャッシュ
pub struct UserMembershipCache {
    team_ids: Vec<Uuid>,
    organization_ids: Vec<Uuid>,
    cached_at: Instant,
    ttl: Duration,
}

impl UserMembershipCache {
    pub fn is_valid(&self) -> bool {
        self.cached_at.elapsed() < self.ttl
    }
}
```

## まとめ

マルチテナント機能における権限統一チェック機構は、以下の原則に基づいて実装されています：

1. **階層的な権限モデル**: システム > 組織 > チーム > 個人の優先順位
2. **統一ミドルウェア**: すべての権限チェックを一元化
3. **明示的な権限設定**: 各エンドポイントで必要な権限を明確に定義
4. **パフォーマンス最適化**: 適切なインデックスとクエリ最適化
5. **包括的なテスト**: 各権限パターンに対する統合テスト

これらのベストプラクティスに従うことで、セキュアで拡張可能なマルチテナントシステムを構築できます。