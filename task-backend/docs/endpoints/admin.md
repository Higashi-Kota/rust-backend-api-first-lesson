# 管理者機能エンドポイント

システム管理者専用の高度な管理機能APIエンドポイント群です。タスク・ユーザー・チーム・組織の一括管理、システム統計、セキュリティ管理機能を提供します。

## 概要

管理者機能は、システム管理者権限を持つユーザーのみがアクセス可能な高度な管理機能群です。一般ユーザーでは実行できない特権操作を含みます。

## 管理者用タスク管理

### 1. 管理者用タスク詳細取得 (GET /admin/tasks/{task_id})

任意のタスクに制限なくアクセスします。

**リクエスト例:**
```bash
curl -X GET http://localhost:3000/admin/tasks/550e8400-e29b-41d4-a716-446655440001 \
  -H "Authorization: Bearer <admin_access_token>"
```

**レスポンス例 (200 OK):**
```json
{
  "message": "Task retrieved successfully",
  "data": {
    "tasks": [
      {
        "id": "550e8400-e29b-41d4-a716-446655440001",
        "title": "重要なタスク",
        "description": "管理者がアクセスしたタスク",
        "status": "in_progress",
        "user_id": "550e8400-e29b-41d4-a716-446655440100",
        "created_at": "2025-06-29T08:00:00Z",
        "updated_at": "2025-06-29T09:00:00Z"
      }
    ],
    "bulk_operations": true,
    "unlimited_access": true
  }
}
```

### 2. 全タスク一覧取得 (GET /admin/tasks)

システム内の全ユーザーのタスクを取得します。

**リクエスト例:**
```bash
curl -X GET http://localhost:3000/admin/tasks \
  -H "Authorization: Bearer <admin_access_token>"
```

### 3. ページネーション付きタスク一覧 (GET /admin/tasks/paginated)

管理者用のページネーション機能付きタスク一覧です。

**リクエスト例:**
```bash
curl -X GET "http://localhost:3000/admin/tasks/paginated?page=1&page_size=50" \
  -H "Authorization: Bearer <admin_access_token>"
```

### 4. 特定ユーザーのタスク取得 (GET /admin/users/{user_id}/tasks)

指定したユーザーのすべてのタスクを取得します。

**リクエスト例:**
```bash
curl -X GET http://localhost:3000/admin/users/550e8400-e29b-41d4-a716-446655440100/tasks \
  -H "Authorization: Bearer <admin_access_token>"
```

## 管理者用タスク操作

### 5. 管理者用タスク作成 (POST /admin/tasks)

管理者権限でタスクを作成します。

**リクエスト例:**
```bash
curl -X POST http://localhost:3000/admin/tasks \
  -H "Authorization: Bearer <admin_access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "システムメンテナンスタスク",
    "description": "管理者が作成したシステムタスク",
    "status": "todo",
    "due_date": "2025-07-01T00:00:00Z"
  }'
```

### 6. 管理者用タスク更新 (PUT /admin/tasks/{task_id})

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

### 7. 管理者用タスク削除 (DELETE /admin/tasks/{task_id})

任意のタスクを管理者権限で削除します。

**リクエスト例:**
```bash
curl -X DELETE http://localhost:3000/admin/tasks/550e8400-e29b-41d4-a716-446655440001 \
  -H "Authorization: Bearer <admin_access_token>"
```

## 一括操作管理

### 8. 一括タスク作成 (POST /admin/tasks/bulk/create)

複数のタスクを一度に作成します（最大100件）。

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

**レスポンス例 (200 OK):**
```json
{
  "message": "Bulk task creation completed",
  "data": {
    "success_count": 2,
    "failed_count": 0,
    "total_requested": 2,
    "errors": []
  }
}
```

### 9. 一括タスク更新 (PUT /admin/tasks/bulk/update)

複数のタスクを一度に更新します。

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

### 10. 一括タスク削除 (DELETE /admin/tasks/bulk/delete)

複数のタスクを一度に削除します。

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

## 統計・分析管理

### 11. タスク統計取得 (GET /admin/tasks/statistics)

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

## 招待・メンバーシップ管理

### 12. 期限切れ招待クリーンアップ (POST /admin/invitations/cleanup)

期限切れのチーム招待をクリーンアップします。

**リクエスト例:**
```bash
curl -X POST http://localhost:3000/admin/invitations/cleanup \
  -H "Authorization: Bearer <admin_access_token>"
```

**レスポンス例 (200 OK):**
```json
{
  "message": "Expired invitations cleaned up successfully",
  "data": [
    {
      "id": "inv-001",
      "team_id": "team-001",
      "email": "expired@example.com",
      "expired_at": "2025-06-20T00:00:00Z",
      "status": "expired"
    }
  ]
}
```

### 13. 古い招待削除 (DELETE /admin/invitations/cleanup/old)

指定した日数より古い招待を削除します。

**リクエスト例:**
```bash
curl -X DELETE "http://localhost:3000/admin/invitations/cleanup/old?days=30" \
  -H "Authorization: Bearer <admin_access_token>"
```

**レスポンス例 (200 OK):**
```json
{
  "message": "Old invitations deleted successfully",
  "data": {
    "deleted_count": 45,
    "days": 30
  }
}
```

## 権限要件

### 必要な権限レベル

すべての管理者エンドポイントには以下が必要です：

- **ユーザーロール**: `admin`
- **認証**: 有効な管理者アクセストークン
- **権限チェック**: `user.is_admin()` が true

### 管理者権限レベル

- **基本管理者**: 閲覧・基本操作権限
- **システム管理者**: 一括操作・統計アクセス権限
- **上級管理者**: 削除・クリーンアップ権限

## エラーレスポンス例

### 権限不足 (403 Forbidden)
```json
{
  "error": "Administrator access required",
  "error_type": "forbidden"
}
```

### バリデーションエラー (400 Bad Request)
```json
{
  "error": "Must provide 1-100 tasks",
  "error_type": "validation_error"
}
```

### 一括操作エラー
```json
{
  "message": "Bulk task creation completed",
  "data": {
    "success_count": 8,
    "failed_count": 2,
    "total_requested": 10,
    "errors": [
      "Task title cannot be empty",
      "Invalid status value for task 9"
    ]
  }
}
```

## 使用例

### 管理者による一括タスク管理ワークフロー

```bash
# 管理者トークンを取得済みと仮定
ADMIN_TOKEN="admin_access_token_here"

# 1. システム全体のタスク統計を確認
curl -s -X GET http://localhost:3000/admin/tasks/statistics \
  -H "Authorization: Bearer $ADMIN_TOKEN"

# 2. 問題のあるユーザーのタスクを確認
USER_ID="550e8400-e29b-41d4-a716-446655440100"
curl -s -X GET http://localhost:3000/admin/users/$USER_ID/tasks \
  -H "Authorization: Bearer $ADMIN_TOKEN"

# 3. システムメンテナンス用タスクを一括作成
curl -s -X POST http://localhost:3000/admin/tasks/bulk/create \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "tasks": [
      {
        "title": "データベースバックアップ",
        "description": "週次データベースバックアップ",
        "status": "todo"
      },
      {
        "title": "ログ分析",
        "description": "システムログの分析と監視",
        "status": "todo"
      }
    ]
  }'

# 4. 完了したタスクを一括削除
curl -s -X DELETE http://localhost:3000/admin/tasks/bulk/delete \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "task_ids": ["completed_task_1", "completed_task_2"]
  }'

# 5. 期限切れ招待をクリーンアップ
curl -s -X POST http://localhost:3000/admin/invitations/cleanup \
  -H "Authorization: Bearer $ADMIN_TOKEN"

# 6. 古い招待データを削除
curl -s -X DELETE "http://localhost:3000/admin/invitations/cleanup/old?days=60" \
  -H "Authorization: Bearer $ADMIN_TOKEN"
```

### 緊急時のシステム管理

```bash
# 緊急時：特定ユーザーの全タスクを強制完了
curl -s -X PUT http://localhost:3000/admin/tasks/bulk/update \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "updates": [
      {
        "id": "emergency_task_1",
        "status": "completed",
        "description": "Emergency completion by admin"
      }
    ]
  }'

# システム全体の統計を即座に取得
curl -s -X GET http://localhost:3000/admin/tasks/statistics \
  -H "Authorization: Bearer $ADMIN_TOKEN" | jq
```

## セキュリティ考慮事項

### 管理者操作のログ記録

すべての管理者操作は以下の情報と共にログに記録されます：

- **管理者ID**: 操作を実行した管理者
- **操作タイプ**: 実行された操作の種類
- **対象リソース**: 操作対象のリソースID
- **タイムスタンプ**: 操作実行時刻
- **IPアドレス**: 操作元のIPアドレス

### 推奨セキュリティ設定

```json
{
  "admin_security": {
    "require_2fa": true,
    "session_timeout": "30 minutes",
    "audit_all_operations": true,
    "restrict_bulk_operations": {
      "max_items": 100,
      "require_confirmation": true
    }
  }
}
```