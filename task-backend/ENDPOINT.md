# API エンドポイント確認ガイド

以下は、各 API エンドポイントの動作確認方法です。curl コマンドを使用して、各 API を簡単にテストできます。

## 基本的な CRUD 操作

### 1. タスク作成 (POST /tasks)

```bash
curl -X POST http://localhost:3000/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "title": "テストタスク1",
    "description": "これはテスト用のタスクです",
    "status": "todo",
    "due_date": "2025-06-01T00:00:00Z"
  }' | jq
```

レスポンス例:

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "title": "テストタスク1",
  "description": "これはテスト用のタスクです",
  "status": "todo",
  "due_date": "2025-06-01T00:00:00Z",
  "created_at": "2025-05-11T07:36:38.000000Z",
  "updated_at": "2025-05-11T07:36:38.000000Z"
}
```

### 2. タスク一覧取得 (GET /tasks)

```bash
curl -X GET http://localhost:3000/tasks | jq
```

レスポンス例:

```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "title": "テストタスク1",
    "description": "これはテスト用のタスクです",
    "status": "todo",
    "due_date": "2025-06-01T00:00:00Z",
    "created_at": "2025-05-11T07:36:38.000000Z",
    "updated_at": "2025-05-11T07:36:38.000000Z"
  }
]
```

### 3. 特定タスク取得 (GET /tasks/{id})

```bash
curl -X GET http://localhost:3000/tasks/550e8400-e29b-41d4-a716-446655440000 | jq
```

レスポンス例:

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "title": "テストタスク1",
  "description": "これはテスト用のタスクです",
  "status": "todo",
  "due_date": "2025-06-01T00:00:00Z",
  "created_at": "2025-05-11T07:36:38.000000Z",
  "updated_at": "2025-05-11T07:36:38.000000Z"
}
```

### 4. タスク更新 (PATCH /tasks/{id})

```bash
curl -X PATCH http://localhost:3000/tasks/550e8400-e29b-41d4-a716-446655440000 \
  -H "Content-Type: application/json" \
  -d '{
    "status": "in_progress",
    "description": "更新されたタスクの説明"
  }' | jq
```

レスポンス例:

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "title": "テストタスク1",
  "description": "更新されたタスクの説明",
  "status": "in_progress",
  "due_date": "2025-06-01T00:00:00Z",
  "created_at": "2025-05-11T07:36:38.000000Z",
  "updated_at": "2025-05-11T07:37:42.000000Z"
}
```

### 5. タスク削除 (DELETE /tasks/{id})

```bash
curl -X DELETE http://localhost:3000/tasks/550e8400-e29b-41d4-a716-446655440000 -v
```

成功時のレスポンス:

- HTTP ステータスコード: 204 No Content
- レスポンスボディなし

## 一括操作

### 6. 一括タスク作成 (POST /tasks/batch/create)

```bash
curl -X POST http://localhost:3000/tasks/batch/create \
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
        "status": "todo"
      }
    ]
  }' | jq
```

レスポンス例:

```json
{
  "created_tasks": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440001",
      "title": "一括タスク1",
      "description": "一括作成テスト1",
      "status": "todo",
      "due_date": null,
      "created_at": "2025-05-11T07:38:15.000000Z",
      "updated_at": "2025-05-11T07:38:15.000000Z"
    },
    {
      "id": "550e8400-e29b-41d4-a716-446655440002",
      "title": "一括タスク2",
      "description": "一括作成テスト2",
      "status": "todo",
      "due_date": null,
      "created_at": "2025-05-11T07:38:15.000000Z",
      "updated_at": "2025-05-11T07:38:15.000000Z"
    }
  ]
}
```

### 7. 一括タスク更新 (PATCH /tasks/batch/update)

```bash
curl -X PATCH http://localhost:3000/tasks/batch/update \
  -H "Content-Type: application/json" \
  -d '{
    "tasks": [
      {
        "id": "550e8400-e29b-41d4-a716-446655440001",
        "status": "completed"
      },
      {
        "id": "550e8400-e29b-41d4-a716-446655440002",
        "status": "in_progress"
      }
    ]
  }' | jq
```

レスポンス例:

```json
{
  "updated_count": 2
}
```

### 8. 一括タスク削除 (POST /tasks/batch/delete)

```bash
curl -X POST http://localhost:3000/tasks/batch/delete \
  -H "Content-Type: application/json" \
  -d '{
    "ids": [
      "550e8400-e29b-41d4-a716-446655440001",
      "550e8400-e29b-41d4-a716-446655440002"
    ]
  }' | jq
```

レスポンス例:

```json
{
  "deleted_count": 2
}
```

## その他

### 9. ヘルスチェック (GET /health)

```bash
curl -X GET http://localhost:3000/health | jq
```

レスポンス例:

```
OK
```

## エラー処理の検証

### バリデーションエラーの例 (空のタイトル)

```bash
curl -X POST http://localhost:3000/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "title": "",
    "description": "空のタイトルでエラーになるはず",
    "status": "todo"
  }' | jq
```

レスポンス例:

```json
{
  "errors": ["Title cannot be empty"],
  "error_type": "validation_errors"
}
```

### 存在しないリソースへのアクセス

```bash
curl -X GET http://localhost:3000/tasks/00000000-0000-0000-0000-000000000000 | jq
```

レスポンス例:

```json
{
  "error": "Task with id 00000000-0000-0000-0000-000000000000 not found",
  "error_type": "not_found"
}
```

### 不正な UUID フォーマット

```bash
curl -X GET http://localhost:3000/tasks/invalid-uuid | jq
```

レスポンス例:

```json
{
  "error": "Invalid UUID: invalid UUID format",
  "error_type": "invalid_uuid"
}
```

これらの curl コマンドを使用して、API の各エンドポイントが正しく動作していることを確認できます。また、エラー処理も適切に実装されているかを検証できます。
