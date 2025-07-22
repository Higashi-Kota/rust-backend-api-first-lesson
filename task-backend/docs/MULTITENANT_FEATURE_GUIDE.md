# マルチテナント機能ガイド

## 概要

Task Backend APIのマルチテナント機能は、チームや組織単位でタスクを共有・協業できる機能を提供します。既存の個人タスク機能と完全に互換性を保ちながら、チーム協業のための強力な機能を実装しています。

## 目次

1. [機能概要](#機能概要)
2. [アーキテクチャ](#アーキテクチャ)
3. [権限モデル](#権限モデル)
4. [API仕様](#api仕様)
5. [実装ガイド](#実装ガイド)
6. [セキュリティとパフォーマンス](#セキュリティとパフォーマンス)
7. [テスト戦略](#テスト戦略)

## 機能概要

### タスクの可視性レベル

- **Personal（個人）**: 作成者のみがアクセス可能（デフォルト）
- **Team（チーム）**: チームメンバー全員がアクセス可能
- **Organization（組織）**: 組織メンバー全員がアクセス可能

### 主要機能

1. **チーム共有タスク**: チーム単位でのタスク作成・管理
2. **タスク割り当て**: チームメンバーへのタスク割り当て
3. **タスク引き継ぎ**: 担当者変更時のスムーズな引き継ぎ
4. **監査ログ**: すべての重要操作の記録とコンプライアンス対応
5. **スコープベースフィルタリング**: 権限に基づくタスク一覧の取得

## アーキテクチャ

### コンポーネント構成

```
┌─────────────────────────────────────────────────┐
│                  API Layer                      │
│  ┌───────────┐ ┌───────────┐ ┌──────────────┐ │
│  │  Task     │ │   Team    │ │  Audit Log   │ │
│  │ Handlers  │ │ Handlers  │ │  Handlers    │ │
│  └───────────┘ └───────────┘ └──────────────┘ │
├─────────────────────────────────────────────────┤
│              Middleware Layer                   │
│  ┌───────────────────┐ ┌───────────────────┐  │
│  │  統一権限チェック  │ │  認証ミドルウェア │  │
│  │   ミドルウェア    │ │                   │  │
│  └───────────────────┘ └───────────────────┘  │
├─────────────────────────────────────────────────┤
│               Service Layer                     │
│  ┌───────────┐ ┌───────────┐ ┌──────────────┐ │
│  │   Task    │ │   Team    │ │  Audit Log   │ │
│  │  Service  │ │  Service  │ │   Service    │ │
│  └───────────┘ └───────────┘ └──────────────┘ │
├─────────────────────────────────────────────────┤
│             Repository Layer                    │
│  ┌───────────┐ ┌───────────┐ ┌──────────────┐ │
│  │   Task    │ │   Team    │ │  Audit Log   │ │
│  │Repository │ │Repository │ │ Repository   │ │
│  └───────────┘ └───────────┘ └──────────────┘ │
├─────────────────────────────────────────────────┤
│              Database Layer                     │
│  ┌───────────┐ ┌───────────┐ ┌──────────────┐ │
│  │   tasks   │ │   teams   │ │ audit_logs   │ │
│  │   table   │ │   table   │ │    table     │ │
│  └───────────┘ └───────────┘ └──────────────┘ │
└─────────────────────────────────────────────────┘
```

### データベース設計

#### tasksテーブル拡張

```sql
-- マルチテナント対応フィールド
ALTER TABLE tasks ADD COLUMN team_id UUID REFERENCES teams(id);
ALTER TABLE tasks ADD COLUMN organization_id UUID REFERENCES organizations(id);
ALTER TABLE tasks ADD COLUMN visibility task_visibility NOT NULL DEFAULT 'personal';
ALTER TABLE tasks ADD COLUMN assigned_to UUID REFERENCES users(id);

-- Enum型の定義
CREATE TYPE task_visibility AS ENUM ('personal', 'team', 'organization');
```

#### インデックス設計

```sql
-- 単一カラムインデックス
CREATE INDEX idx_tasks_team_id ON tasks(team_id);
CREATE INDEX idx_tasks_organization_id ON tasks(organization_id);
CREATE INDEX idx_tasks_visibility ON tasks(visibility);
CREATE INDEX idx_tasks_assigned_to ON tasks(assigned_to);

-- 複合インデックス（クエリパターンに基づく）
CREATE INDEX idx_tasks_user_visibility ON tasks(user_id, visibility);
CREATE INDEX idx_tasks_team_visibility ON tasks(team_id, visibility);
CREATE INDEX idx_tasks_org_visibility ON tasks(organization_id, visibility);

-- 全文検索用GINインデックス
CREATE INDEX idx_tasks_title_gin ON tasks USING gin(to_tsvector('simple', title));
CREATE INDEX idx_tasks_description_gin ON tasks USING gin(to_tsvector('simple', description));
```

## 権限モデル

### 権限制御マトリクス

#### システムレベル権限
| ロール | 説明 | 権限範囲 |
|--------|------|----------|
| Admin | システム管理者 | 全リソースへの完全なアクセス権 |
| Member | 一般ユーザー | 自身のリソースのみアクセス可能 |

#### 組織レベル権限
| ロール | 権限レベル | 組織管理 | メンバー管理 | チーム作成 | 分析閲覧 |
|--------|------------|----------|--------------|------------|----------|
| Owner | 最上位 | ✓ | ✓ | ✓ | ✓ |
| Admin | 管理者 | ✓ | ✓ | ✓ | ✓ |
| Member | 一般 | × | × | △※1 | △※2 |

※1 組織の設定により制限可能
※2 自身が関わるデータのみ

#### チームレベル権限
| ロール | 権限レベル | チーム設定 | メンバー管理 | リソース作成 | リソース閲覧 | リソース編集 | リソース削除 |
|--------|------------|------------|--------------|--------------|--------------|--------------|--------------|
| Owner | 最上位 | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Admin | 管理者 | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Member | 一般 | × | × | ✓ | ✓ | △※3 | △※3 |
| Viewer | 閲覧のみ | × | × | × | ✓ | × | × |

※3 自身が作成したリソースのみ、またはチーム設定による

### 権限チェックの実装

```rust
impl TaskService {
    pub async fn can_access_task(
        &self,
        user: &AuthenticatedUser,
        task: &task_model::Model,
    ) -> AppResult<bool> {
        match task.visibility {
            TaskVisibility::Personal => {
                // 個人タスクは所有者のみアクセス可能
                Ok(task.user_id == user.user_id())
            }
            TaskVisibility::Team => {
                if let Some(team_id) = task.team_id {
                    // チームメンバーかどうかを確認
                    self.team_service
                        .is_team_member(team_id, user.user_id())
                        .await
                } else {
                    Ok(false)
                }
            }
            TaskVisibility::Organization => {
                if let Some(org_id) = task.organization_id {
                    // 組織メンバーかどうかを確認
                    self.team_service
                        .is_organization_member(org_id, user.user_id())
                        .await
                } else {
                    Ok(false)
                }
            }
        }
    }
}
```

## API仕様

### 日時フィールドのフォーマット

すべての日時フィールドはUnix Timestamp（秒単位）で統一：

```json
{
  "created_at": 1736922123,  // 2025-01-15 05:55:23 UTC
  "updated_at": 1736922456
}
```

### チームタスクAPI

#### チームタスク作成

**エンドポイント**: `POST /teams/{team_id}/tasks`

**権限**: チームメンバーであること

**リクエストボディ**:
```json
{
  "title": "新機能の設計レビュー",
  "description": "Q1の新機能について設計レビューを実施",
  "status": "todo",
  "priority": 2,
  "due_date": 1737526800,  // Unix timestamp (optional)
  "assigned_to": "550e8400-e29b-41d4-a716-446655440001"  // UUID (optional)
}
```

**レスポンス**: `201 Created`
```json
{
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "title": "新機能の設計レビュー",
    "description": "Q1の新機能について設計レビューを実施",
    "status": "todo",
    "priority": 2,
    "due_date": 1737526800,
    "created_at": 1736922123,
    "updated_at": 1736922123,
    "team_id": "550e8400-e29b-41d4-a716-446655440002",
    "organization_id": null,
    "visibility": "team",
    "assigned_to": "550e8400-e29b-41d4-a716-446655440001",
    "user_id": "550e8400-e29b-41d4-a716-446655440003"
  },
  "meta": {
    "request_id": "req_12345",
    "timestamp": 1736922123
  }
}
```

#### スコープ指定タスク一覧取得

**エンドポイント**: `GET /tasks/scoped`

**クエリパラメータ**:
- `visibility`: `personal` | `team` | `organization` (required)
- `team_id`: チームID（visibility=teamの場合は必須）
- `organization_id`: 組織ID（visibility=organizationの場合は必須）
- `page`: ページ番号（デフォルト: 1）
- `per_page`: 1ページあたりの件数（デフォルト: 20、最大: 100）

**例**:
```
GET /tasks/scoped?visibility=team&team_id=550e8400-e29b-41d4-a716-446655440002&page=1&per_page=20
```

#### タスク引き継ぎ

**エンドポイント**: `POST /tasks/{id}/transfer`

**権限**: 
- 個人タスク: 所有者のみ
- チームタスク: チームメンバー
- 組織タスク: 組織メンバー

**リクエストボディ**:
```json
{
  "new_assignee": "550e8400-e29b-41d4-a716-446655440006",
  "reason": "プロジェクト担当者の変更のため"  // optional, max 500 characters
}
```

**レスポンス**: `200 OK`
```json
{
  "data": {
    "task_id": "550e8400-e29b-41d4-a716-446655440000",
    "previous_assignee": "550e8400-e29b-41d4-a716-446655440003",
    "new_assignee": "550e8400-e29b-41d4-a716-446655440006",
    "transferred_at": 1736923300,
    "transferred_by": "550e8400-e29b-41d4-a716-446655440003",
    "reason": "プロジェクト担当者の変更のため"
  },
  "meta": {
    "request_id": "req_12349",
    "timestamp": 1736923300
  }
}
```

### 監査ログAPI

#### 個人の監査ログ取得

**エンドポイント**: `GET /audit-logs/me`

**クエリパラメータ**:
- `page`: ページ番号（デフォルト: 1）
- `per_page`: 1ページあたりの件数（デフォルト: 20、最大: 100）
- `action`: フィルタするアクション（optional）
- `resource_type`: フィルタするリソースタイプ（optional）

**レスポンス**: `200 OK`
```json
{
  "data": {
    "items": [
      {
        "id": "550e8400-e29b-41d4-a716-446655440010",
        "user_id": "550e8400-e29b-41d4-a716-446655440003",
        "action": "TaskTransferred",
        "resource_type": "task",
        "resource_id": "550e8400-e29b-41d4-a716-446655440000",
        "team_id": "550e8400-e29b-41d4-a716-446655440002",
        "organization_id": null,
        "metadata": {
          "previous_assignee": "550e8400-e29b-41d4-a716-446655440003",
          "new_assignee": "550e8400-e29b-41d4-a716-446655440006",
          "reason": "プロジェクト担当者の変更のため"
        },
        "ip_address": "192.168.1.100",
        "user_agent": "Mozilla/5.0...",
        "created_at": 1736923300
      }
    ],
    "total": 150,
    "page": 1,
    "per_page": 20
  },
  "meta": {
    "request_id": "req_12351",
    "timestamp": 1736923500
  }
}
```

### エラーレスポンス

#### 権限エラー

**ステータス**: `403 Forbidden`
```json
{
  "error": {
    "code": "FORBIDDEN",
    "message": "You don't have permission to access this resource"
  },
  "meta": {
    "request_id": "req_12352",
    "timestamp": 1736923600
  }
}
```

#### バリデーションエラー

**ステータス**: `400 Bad Request`
```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Validation failed",
    "details": [
      {
        "field": "title",
        "code": "required",
        "message": "Title is required"
      }
    ]
  },
  "meta": {
    "request_id": "req_12354",
    "timestamp": 1736923800
  }
}
```

## 実装ガイド

### サービス層での実装パターン

```rust
pub async fn update_team_task(
    &self,
    user: &AuthenticatedUser,
    task_id: Uuid,
    update_dto: UpdateTaskDto,
) -> AppResult<TaskDto> {
    // 1. タスクの存在確認
    let task = self.repository
        .find_by_id(task_id)
        .await
        .map_err(|e| internal_server_error(e, "task_service::update_team_task", "Failed to find task"))?
        .ok_or_else(|| not_found_error("Task not found", "task_service::update_team_task"))?;

    // 2. アクセス権限の確認
    if !self.can_access_task(user, &task).await? {
        return Err(forbidden_error(
            "You don't have permission to update this task",
            "task_service::update_team_task",
        ));
    }

    // 3. 監査ログの記録
    self.audit_log_service
        .log_action(
            user.user_id(),
            AuditAction::TaskUpdated,
            "task",
            Some(task_id),
            task.team_id,
            task.organization_id,
            None,
        )
        .await?;

    // 4. 実際の更新処理
    let updated_task = self.repository
        .update(task_id, update_dto)
        .await
        .map_err(|e| internal_server_error(e, "task_service::update_team_task", "Failed to update task"))?;

    Ok(TaskDto::from(updated_task))
}
```

### DTOでのデータ変換

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct TeamTaskDto {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: Option<i32>,
    pub due_date: Option<i64>,  // Unix timestamp
    pub created_at: i64,
    pub updated_at: i64,
    // マルチテナントフィールド
    pub team_id: Uuid,
    pub organization_id: Option<Uuid>,
    pub visibility: TaskVisibility,
    pub assigned_to: Option<Uuid>,
    pub created_by: Uuid,
}

impl From<task_model::Model> for TeamTaskDto {
    fn from(model: task_model::Model) -> Self {
        Self {
            id: model.id,
            title: model.title,
            description: model.description,
            status: model.status,
            priority: model.priority,
            due_date: model.due_date.map(|dt| dt.timestamp()),
            created_at: model.created_at.timestamp(),
            updated_at: model.updated_at.timestamp(),
            team_id: model.team_id.expect("Team task must have team_id"),
            organization_id: model.organization_id,
            visibility: model.visibility,
            assigned_to: model.assigned_to,
            created_by: model.user_id,
        }
    }
}
```

## セキュリティとパフォーマンス

### セキュリティ対策

1. **データアクセス制御**
   - SQLインジェクション対策: すべてのクエリでパラメータバインディングを使用
   - 権限昇格の防止: タスク作成時の`team_id`検証
   - 監査ログの完全性: すべての重要操作を記録

2. **APIセキュリティ**
   ```rust
   // レート制限の適用
   .layer(RateLimitLayer::new(
       100,  // 100リクエスト
       Duration::from_secs(3600),  // 1時間あたり
   ))
   ```

3. **入力検証**
   ```rust
   #[derive(Validate)]
   pub struct CreateTeamTaskDto {
       #[validate(length(min = 1, max = 100))]
       pub title: String,
       #[validate(length(max = 1000))]
       pub description: Option<String>,
   }
   ```

### パフォーマンス最適化

1. **クエリ最適化**
   ```sql
   -- 効率的なクエリ例
   SELECT * FROM tasks 
   WHERE team_id = $1 AND visibility = 'team' 
   ORDER BY created_at DESC 
   LIMIT 20;
   ```

2. **N+1問題の回避**
   ```rust
   // 関連データを事前に結合して取得
   let tasks_with_users = task_model::Entity::find()
       .filter(task_model::Column::TeamId.eq(team_id))
       .find_also_related(user_model::Entity)
       .all(&self.db)
       .await?;
   ```

3. **ベンチマーク結果**
   - 100タスクの一括作成: 約2-3秒
   - 1000タスクからのフィルタリング: 50ms以下
   - 同時アクセス（10ユーザー）: 各リクエスト100ms以下

## テスト戦略

### 統合テストの構成（AAAパターン）

```rust
#[tokio::test]
async fn test_team_task_isolation() {
    let (app, _schema, _db) = setup_full_app().await;
    
    // Arrange: 2つのチームと各チームのメンバーを作成
    let team_a = create_test_team(&app, "Team A").await;
    let team_b = create_test_team(&app, "Team B").await;
    
    let user_a = create_and_add_to_team(&app, &team_a).await;
    let user_b = create_and_add_to_team(&app, &team_b).await;
    
    // Act: Team Aのメンバーがタスクを作成
    let task_a = create_team_task(&app, &user_a, &team_a, "Task for Team A").await;
    
    // Assert: Team Bのメンバーはアクセス不可
    let response = app
        .clone()
        .oneshot(create_authenticated_request(
            "GET",
            &format!("/tasks/{}", task_a.id),
            &user_b.token,
            None,
        ))
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
```

### テストカバレッジ

各APIエンドポイントに対して最低限以下のケースをテスト:
- 正常系
- バリデーションエラー
- 認証エラー
- 認可エラー
- リソース不在

### パフォーマンステスト

```rust
#[tokio::test]
async fn test_large_scale_team_task_query_performance() {
    let (app, _schema, _db) = setup_full_app().await;
    
    // Arrange: 大量のタスクを作成
    let team = create_test_team(&app, "Performance Test Team").await;
    let user = create_and_add_to_team(&app, &team).await;
    
    // 1000件のチームタスクを作成
    for i in 0..1000 {
        create_team_task(&app, &user, &team, &format!("Task {}", i)).await;
    }
    
    // Act: タスク一覧を取得して時間を測定
    let start = std::time::Instant::now();
    
    let response = app
        .clone()
        .oneshot(create_authenticated_request(
            "GET",
            &format!("/tasks/scoped?visibility=team&team_id={}&per_page=100", team.id),
            &user.token,
            None,
        ))
        .await
        .unwrap();
    
    let duration = start.elapsed();
    
    // Assert: レスポンスタイムが許容範囲内
    assert!(duration.as_millis() < 500, "Query took too long: {:?}", duration);
    assert_eq!(response.status(), StatusCode::OK);
}
```

## 注意事項

1. **後方互換性**: 既存の個人タスクAPIは引き続き動作し、デフォルトで`visibility=personal`が設定されます
2. **権限チェック**: すべてのマルチテナントAPIは統一権限チェックミドルウェアにより保護されています
3. **監査ログ**: タスクの作成、更新、削除、引き継ぎなどの重要な操作はすべて監査ログに記録されます
4. **パフォーマンス**: 大量のチームタスクを扱う場合は、適切なページネーションを使用してください

## 使用例

### チームタスクの作成から引き継ぎまでの流れ

```bash
# 1. チームタスクを作成
curl -X POST https://api.example.com/teams/550e8400-e29b-41d4-a716-446655440002/tasks \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "新機能の実装",
    "description": "ユーザー管理機能の追加",
    "status": "todo",
    "assigned_to": "550e8400-e29b-41d4-a716-446655440003"
  }'

# 2. タスクを別のメンバーに引き継ぎ
curl -X POST https://api.example.com/tasks/TASK_ID/transfer \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "new_assignee": "550e8400-e29b-41d4-a716-446655440006",
    "reason": "担当変更のため"
  }'

# 3. 監査ログを確認
curl -X GET "https://api.example.com/audit-logs/me?action=TaskTransferred" \
  -H "Authorization: Bearer YOUR_TOKEN"
```