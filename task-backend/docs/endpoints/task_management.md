# タスク管理エンドポイント

タスクのCRUD操作、フィルタリング、一括操作などのAPIエンドポイント群です。動的パーミッションシステムにより、ユーザーのサブスクリプション階層に応じて異なる機能を提供します。

## 基本的なタスク操作

### 1. タスク一覧取得 (GET /tasks)

ユーザーのすべてのタスクを取得します。

**リクエスト例:**
```bash
curl -X GET http://localhost:3000/tasks \
  -H "Authorization: Bearer <access_token>"
```

**レスポンス例 (200 OK):**
```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440001",
    "title": "重要なタスク",
    "description": "このタスクは最優先で完了する必要があります",
    "status": "todo",
    "due_date": "2025-06-30T23:59:59Z",
    "user_id": "550e8400-e29b-41d4-a716-446655440000",
    "created_at": "2025-06-12T10:00:00Z",
    "updated_at": "2025-06-12T10:00:00Z"
  }
]
```

### 2. タスク作成 (POST /tasks)

新しいタスクを作成します。

**リクエスト例:**
```bash
curl -X POST http://localhost:3000/tasks \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "重要なタスク",
    "description": "このタスクは最優先で完了する必要があります",
    "status": "todo",
    "due_date": "2025-06-30T23:59:59Z"
  }'
```

### 3. 特定タスク取得 (GET /tasks/{id})

指定したIDのタスクの詳細を取得します。

**リクエスト例:**
```bash
curl -X GET http://localhost:3000/tasks/550e8400-e29b-41d4-a716-446655440001 \
  -H "Authorization: Bearer <access_token>"
```

### 4. タスク更新 (PATCH /tasks/{id})

既存のタスクを部分的に更新します。

**リクエスト例:**
```bash
curl -X PATCH http://localhost:3000/tasks/550e8400-e29b-41d4-a716-446655440001 \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "status": "in_progress",
    "description": "進行中のタスクです"
  }'
```

### 5. タスク削除 (DELETE /tasks/{id})

指定したタスクを削除します。

**リクエスト例:**
```bash
curl -X DELETE http://localhost:3000/tasks/550e8400-e29b-41d4-a716-446655440001 \
  -H "Authorization: Bearer <access_token>"
```

**成功レスポンス:** 204 No Content

## ページネーション・フィルタリング

### 6. ページネーション付きタスク一覧 (GET /tasks/paginated)

ページネーション機能付きでタスクを取得します。

**リクエスト例:**
```bash
curl -X GET "http://localhost:3000/tasks/paginated?page=1&page_size=10" \
  -H "Authorization: Bearer <access_token>"
```

**レスポンス例 (200 OK):**
```json
{
  "tasks": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440001",
      "title": "重要なタスク",
      "description": "このタスクは最優先で完了する必要があります",
      "status": "todo",
      "due_date": "2025-06-30T23:59:59Z",
      "user_id": "550e8400-e29b-41d4-a716-446655440000",
      "created_at": "2025-06-12T10:00:00Z",
      "updated_at": "2025-06-12T10:00:00Z"
    }
  ],
  "pagination": {
    "current_page": 1,
    "page_size": 10,
    "total_items": 15,
    "total_pages": 2,
    "has_next_page": true,
    "has_previous_page": false
  }
}
```

### 7. タスクフィルタリング (GET /tasks/filter)

条件に基づいてタスクをフィルタリングします。

**リクエスト例:**
```bash
# ステータスでフィルタリング
curl -X GET "http://localhost:3000/tasks/filter?status=todo&limit=5" \
  -H "Authorization: Bearer <access_token>"

# タイトルで検索
curl -X GET "http://localhost:3000/tasks/filter?title_contains=重要&page=1&page_size=5" \
  -H "Authorization: Bearer <access_token>"

# 期日でフィルタリング
curl -X GET "http://localhost:3000/tasks/filter?due_date_before=2025-07-01T00:00:00Z&sort_by=due_date&sort_order=asc" \
  -H "Authorization: Bearer <access_token>"
```

## 動的パーミッション付きエンドポイント

これらのエンドポイントは、ユーザーのサブスクリプション階層（Free/Pro/Enterprise）に応じて異なる機能やデータアクセス範囲を提供します。

### 8. 動的権限付きタスク一覧取得 (GET /tasks/dynamic)

ユーザーの権限レベルに応じたタスク一覧を取得します。

**リクエスト例:**
```bash
curl -X GET http://localhost:3000/tasks/dynamic \
  -H "Authorization: Bearer <access_token>"
```

**権限レベル別の挙動:**
- **Free**: 自分のタスクのみ、最大100件
- **Pro**: チーム内のタスク、最大10,000件、高度なフィルタ機能
- **Enterprise**: 全体アクセス、無制限、すべての機能

### 9. 動的権限付きフィルタリング (GET /tasks/dynamic/filter)

権限レベルに応じたフィルタリング機能を提供します。

**リクエスト例:**
```bash
curl -X GET "http://localhost:3000/tasks/dynamic/filter?status=in_progress&advanced_filter=true" \
  -H "Authorization: Bearer <access_token>"
```

### 10. 動的権限付きページネーション (GET /tasks/dynamic/paginated)

権限レベルに応じたページネーション機能を提供します。

**リクエスト例:**
```bash
curl -X GET "http://localhost:3000/tasks/dynamic/paginated?page=1&page_size=50" \
  -H "Authorization: Bearer <access_token>"
```

## 一括操作エンドポイント

### 11. 一括タスク作成 (POST /tasks/batch/create)

複数のタスクを一度に作成します（最大100件）。

**リクエスト例:**
```bash
curl -X POST http://localhost:3000/tasks/batch/create \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "tasks": [
      {
        "title": "一括タスク1",
        "description": "一括作成テスト1",
        "status": "todo"
      },
      {
        "title": "一括タスク2",
        "description": "一括作成テスト2",
        "status": "todo",
        "due_date": "2025-07-01T12:00:00Z"
      }
    ]
  }'
```

### 12. 一括タスク更新 (PATCH /tasks/batch/update)

複数のタスクを一度に更新します。

**リクエスト例:**
```bash
curl -X PATCH http://localhost:3000/tasks/batch/update \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "tasks": [
      {
        "id": "550e8400-e29b-41d4-a716-446655440002",
        "status": "completed"
      },
      {
        "id": "550e8400-e29b-41d4-a716-446655440003",
        "status": "in_progress"
      }
    ]
  }'
```

### 13. 一括タスク削除 (POST /tasks/batch/delete)

複数のタスクを一度に削除します。

**リクエスト例:**
```bash
curl -X POST http://localhost:3000/tasks/batch/delete \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "ids": [
      "550e8400-e29b-41d4-a716-446655440002",
      "550e8400-e29b-41d4-a716-446655440003"
    ]
  }'
```

### 14. タスクステータス一括更新 (PATCH /tasks/batch/status)

複数のタスクのステータスを一度に更新します。

**リクエスト例:**
```bash
curl -X PATCH http://localhost:3000/tasks/batch/status \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "task_ids": ["id1", "id2", "id3"],
    "status": "completed"
  }'
```

## 統計・分析

### 15. ユーザーのタスク統計情報取得 (GET /tasks/stats)

現在のユーザーのタスク統計を取得します。

**リクエスト例:**
```bash
curl -X GET http://localhost:3000/tasks/stats \
  -H "Authorization: Bearer <access_token>"
```

## 管理者専用エンドポイント

### 16. 全ユーザーのタスク取得 (GET /admin/tasks)

システム内のすべてのタスクを取得します（管理者用）。

**リクエスト例:**
```bash
curl -X GET http://localhost:3000/admin/tasks \
  -H "Authorization: Bearer <admin_access_token>"
```

### 17. 管理者用ページネーション付きタスク一覧 (GET /admin/tasks/paginated)

管理者用のページネーション機能付きタスク一覧です。

**リクエスト例:**
```bash
curl -X GET "http://localhost:3000/admin/tasks/paginated?page=1&page_size=50" \
  -H "Authorization: Bearer <admin_access_token>"
```

### 18. 管理者用タスク詳細取得 (GET /admin/tasks/{task_id})

任意のタスクに制限なくアクセスします。

**リクエスト例:**
```bash
curl -X GET http://localhost:3000/admin/tasks/550e8400-e29b-41d4-a716-446655440001 \
  -H "Authorization: Bearer <admin_access_token>"
```

### 19. 管理者用タスク作成 (POST /admin/tasks)

管理者権限でタスクを作成します。

**リクエスト例:**
```bash
curl -X POST http://localhost:3000/admin/tasks \
  -H "Authorization: Bearer <admin_access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "システムメンテナンスタスク",
    "description": "管理者が作成したシステムタスク",
    "status": "todo"
  }'
```

### 20. 管理者用タスク更新 (PUT /admin/tasks/{task_id})

任意のタスクを管理者権限で更新します。

**リクエスト例:**
```bash
curl -X PUT http://localhost:3000/admin/tasks/550e8400-e29b-41d4-a716-446655440001 \
  -H "Authorization: Bearer <admin_access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "更新されたタスク",
    "status": "completed"
  }'
```

### 21. 特定ユーザーのタスク取得 (GET /admin/users/{user_id}/tasks)

指定したユーザーのタスクを取得します（管理者用）。

**リクエスト例:**
```bash
curl -X GET http://localhost:3000/admin/users/550e8400-e29b-41d4-a716-446655440000/tasks \
  -H "Authorization: Bearer <admin_access_token>"
```

### 22. 任意のタスクを削除 (DELETE /admin/tasks/{task_id})

管理者権限で任意のタスクを削除します。

**リクエスト例:**
```bash
curl -X DELETE http://localhost:3000/admin/tasks/550e8400-e29b-41d4-a716-446655440001 \
  -H "Authorization: Bearer <admin_access_token>"
```

## 管理者用一括操作

### 23. 管理者用一括タスク作成 (POST /admin/tasks/bulk/create)

複数のタスクを一度に作成します（管理者用、最大100件）。

**リクエスト例:**
```bash
curl -X POST http://localhost:3000/admin/tasks/bulk/create \
  -H "Authorization: Bearer <admin_access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "tasks": [
      {
        "title": "一括作成タスク1",
        "description": "管理者による一括作成",
        "status": "todo"
      },
      {
        "title": "一括作成タスク2",
        "description": "システム管理タスク",
        "status": "in_progress"
      }
    ],
    "assign_to_user": "550e8400-e29b-41d4-a716-446655440100"
  }'
```

### 24. 管理者用一括タスク更新 (PUT /admin/tasks/bulk/update)

複数のタスクを一度に更新します（管理者用）。

**リクエスト例:**
```bash
curl -X PUT http://localhost:3000/admin/tasks/bulk/update \
  -H "Authorization: Bearer <admin_access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "updates": [
      {
        "id": "550e8400-e29b-41d4-a716-446655440001",
        "status": "completed",
        "title": "完了したタスク"
      },
      {
        "id": "550e8400-e29b-41d4-a716-446655440002",
        "status": "in_progress"
      }
    ]
  }'
```

### 25. 管理者用一括タスク削除 (DELETE /admin/tasks/bulk/delete)

複数のタスクを一度に削除します（管理者用）。

**リクエスト例:**
```bash
curl -X DELETE http://localhost:3000/admin/tasks/bulk/delete \
  -H "Authorization: Bearer <admin_access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "task_ids": [
      "550e8400-e29b-41d4-a716-446655440001",
      "550e8400-e29b-41d4-a716-446655440002"
    ]
  }'
```

### 26. 管理者用タスク統計取得 (GET /admin/tasks/statistics)

システム全体のタスク統計を取得します。

**リクエスト例:**
```bash
curl -X GET http://localhost:3000/admin/tasks/statistics \
  -H "Authorization: Bearer <admin_access_token>"
```

**レスポンス例 (200 OK):**
```json
{
  "message": "Task statistics retrieved successfully",
  "data": {
    "total_tasks": 15420,
    "tasks_by_status": [
      {
        "status": "todo",
        "count": 3200
      },
      {
        "status": "in_progress",
        "count": 4800
      },
      {
        "status": "completed",
        "count": 7420
      }
    ],
    "tasks_by_user": [
      {
        "user_id": "550e8400-e29b-41d4-a716-446655440100",
        "task_count": 145,
        "completed_count": 89
      }
    ],
    "recent_activity": [
      {
        "date": "2025-06-29",
        "created_count": 89,
        "completed_count": 124
      }
    ]
  }
}
```

## パブリックエンドポイント

### 27. ヘルスチェック (GET /health)

タスクサービスの稼働状況を確認します。

**リクエスト例:**
```bash
curl -X GET http://localhost:3000/health
```

## 使用例

### 完整なタスク管理ワークフロー

```bash
# アクセストークンを取得済みと仮定
ACCESS_TOKEN="your_access_token_here"

# 1. 新しいタスクを作成
TASK_RESPONSE=$(curl -s -X POST http://localhost:3000/tasks \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "プロジェクト開始",
    "description": "新しいプロジェクトの企画を開始する",
    "status": "todo",
    "due_date": "2025-07-01T17:00:00Z"
  }')

TASK_ID=$(echo $TASK_RESPONSE | jq -r '.id')

# 2. タスクを進行中に更新
curl -s -X PATCH http://localhost:3000/tasks/$TASK_ID \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"status": "in_progress"}'

# 3. 現在のすべてのタスクを確認
curl -s -X GET http://localhost:3000/tasks \
  -H "Authorization: Bearer $ACCESS_TOKEN"

# 4. 進行中のタスクのみフィルタリング
curl -s -X GET "http://localhost:3000/tasks/filter?status=in_progress" \
  -H "Authorization: Bearer $ACCESS_TOKEN"

# 5. タスク統計を確認
curl -s -X GET http://localhost:3000/tasks/stats \
  -H "Authorization: Bearer $ACCESS_TOKEN"
```

## エラーレスポンス例

### 認証エラー (401 Unauthorized)
```json
{
  "error": "Missing authentication token",
  "error_type": "unauthorized"
}
```

### リソースが見つからない (404 Not Found)
```json
{
  "error": "Task with id 550e8400-e29b-41d4-a716-446655440999 not found",
  "error_type": "not_found"
}
```

### バリデーションエラー (400 Bad Request)
```json
{
  "errors": [
    "Title cannot be empty",
    "Invalid status value"
  ],
  "error_type": "validation_errors"
}
```