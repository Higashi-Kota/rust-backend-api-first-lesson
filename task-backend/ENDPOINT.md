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
curl -X GET http://localhost:3000/health | awk 4
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

## ページネーション付きタスク一覧の取得

### ページネーション付きタスク一覧取得 (GET /tasks/paginated)

```bash
curl -X GET "http://localhost:3000/tasks/paginated?page=1&page_size=5" | jq
```

レスポンス例:

```json
{
  "tasks": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "title": "テストタスク1",
      "description": "これはテスト用のタスクです",
      "status": "todo",
      "due_date": "2025-06-01T00:00:00Z",
      "created_at": "2025-05-11T07:36:38.000000Z",
      "updated_at": "2025-05-11T07:36:38.000000Z"
    }
    // ... 他のタスク（最大5件）
  ],
  "pagination": {
    "current_page": 1,
    "page_size": 5,
    "total_items": 12,
    "total_pages": 3,
    "has_next_page": true,
    "has_previous_page": false
  }
}
```

### 2 ページ目の取得

```bash
curl -X GET "http://localhost:3000/tasks/paginated?page=2&page_size=5" | jq
```

## フィルタリングの使用例

### ステータスでフィルタリング

```bash
curl -X GET "http://localhost:3000/tasks/filter?status=todo&limit=10" | jq
```

レスポンス例:

```json
{
  "tasks": [
    // statusが"todo"のタスク一覧
  ],
  "pagination": {
    "current_page": 1,
    "page_size": 10,
    "total_items": 7,
    "total_pages": 1,
    "has_next_page": false,
    "has_previous_page": false
  }
}
```

### タイトルで検索

```bash
curl -X GET "http://localhost:3000/tasks/filter?title_contains=重要&limit=10" | jq
```

レスポンス例:

```json
{
  "tasks": [
    // タイトルに"重要"を含むタスク一覧
  ],
  "pagination": {
    "current_page": 1,
    "page_size": 10,
    "total_items": 3,
    "total_pages": 1,
    "has_next_page": false,
    "has_previous_page": false
  }
}
```

### 期日で絞り込み

```bash
curl -X GET "http://localhost:3000/tasks/filter?due_date_before=2025-06-30T00:00:00Z&due_date_after=2025-06-01T00:00:00Z" | jq
```

レスポンス例:

```json
{
  "tasks": [
    // 2025年6月1日〜6月30日が期限のタスク一覧
  ],
  "pagination": {
    // ページネーション情報
  }
}
```

### 複合条件でフィルタリング

```bash
curl -X GET "http://localhost:3000/tasks/filter?status=in_progress&title_contains=開発&sort_by=due_date&sort_order=asc" | jq
```

レスポンス例:

```json
{
  "tasks": [
    // 「進行中」で、タイトルに「開発」を含むタスクを期日の早い順に表示
  ],
  "pagination": {
    // ページネーション情報
  }
}
```

### オフセットとリミットを指定した検索

```bash
curl -X GET "http://localhost:3000/tasks/filter?status=todo&offset=5&limit=5" | jq
```

レスポンス例:

```json
{
  "tasks": [
    // todoステータスのタスクのうち、6件目から5件を表示
  ],
  "pagination": {
    "current_page": 2,
    "page_size": 5,
    "total_items": 15,
    "total_pages": 3,
    "has_next_page": true,
    "has_previous_page": true
  }
}
```

## ソート機能のテスト

### 作成日の新しい順（デフォルト）

```bash
curl -X GET "http://localhost:3000/tasks/filter" | jq
```

### タイトルのアルファベット順

```bash
curl -X GET "http://localhost:3000/tasks/filter?sort_by=title&sort_order=asc" | jq
```

### 期日の迫っている順

```bash
curl -X GET "http://localhost:3000/tasks/filter?sort_by=due_date&sort_order=asc" | jq
```

## 複合テストケース

### 保守的なページサイズ制限のテスト

非常に大きな page_size を指定した場合でも、システム側で制限されることを確認します：

```bash
curl -X GET "http://localhost:3000/tasks/paginated?page=1&page_size=1000" | jq
```

結果として最大 100 件のみが返されるはずです。

### エラーケースのテスト

不正な並び替えフィールドを指定した場合：

```bash
curl -X GET "http://localhost:3000/tasks/filter?sort_by=invalid_field" | jq
```

この場合、デフォルトの並び順（作成日の降順）で結果が返されます。

## パフォーマンス関連テスト

### 大量のタスクを作成してフィルタリングのパフォーマンスを確認

まず、テスト用に多数のタスクを一括作成します：

```bash
curl -X POST http://localhost:3000/tasks/batch/create \
  -H "Content-Type: application/json" \
  -d '{
    "tasks": [
      {"title": "パフォーマンステスト1", "status": "todo"},
      {"title": "パフォーマンステスト2", "status": "todo"},
      {"title": "パフォーマンステスト3", "status": "in_progress"},
      {"title": "パフォーマンステスト4", "status": "in_progress"},
      {"title": "パフォーマンステスト5", "status": "completed"}
    ]
  }' | jq
```

次に、フィルタリングとページネーションを組み合わせてパフォーマンスを確認します：

```bash
curl -X GET "http://localhost:3000/tasks/filter?title_contains=パフォーマンス&page=1&page_size=2" | jq
```

レスポンス時間を確認するには、time コマンドを使用できます：

```bash
time curl -s -X GET "http://localhost:3000/tasks/filter?title_contains=パフォーマンス&page=1&page_size=2" > /dev/null
```

フィルタリング機能とページネーションが正常に動作し、効率的にクエリが実行されていることが確認できます。
