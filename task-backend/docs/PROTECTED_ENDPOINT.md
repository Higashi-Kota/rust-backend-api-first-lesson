# 認証済みユーザー専用エンドポイント確認ガイド

本ドキュメントでは、認証後に使用できる保護されたAPIエンドポイントの確認方法を詳しく説明します。

## 🔐 認証について

すべてのエンドポイントはJWT認証が必要です。以下のいずれかの方法でアクセストークンを提供してください：

1. **Authorizationヘッダー** (推奨)
```bash
-H "Authorization: Bearer <access_token>"
```

2. **httpOnlyクッキー**
```bash
-H "Cookie: access_token=<access_token>"
```

## 👤 ユーザープロフィール管理エンドポイント

### 1. ユーザープロフィール取得 (GET /users/profile)

現在のユーザーのプロフィール情報を取得。

```bash
curl -X GET http://localhost:3000/users/profile \
  -H "Authorization: Bearer <access_token>" | jq
```

#### 成功レスポンス例 (200 OK):
```json
{
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "username": "testuser",
    "email": "test@example.com",
    "email_verified": false,
    "is_active": true,
    "created_at": "2025-06-12T10:00:00Z",
    "updated_at": "2025-06-12T10:00:00Z",
    "profile": {
      "display_name": "Test User",
      "bio": "Software Developer",
      "avatar_url": null
    }
  }
}
```

### 2. ユーザー名更新 (PATCH /users/username)

現在のユーザーのユーザー名を更新。

```bash
curl -X PATCH http://localhost:3000/users/username \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "username": "newusername"
  }' | jq
```

#### 成功レスポンス例 (200 OK):
```json
{
  "message": "Username updated successfully",
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "username": "newusername",
    "email": "test@example.com",
    "updated_at": "2025-06-12T11:00:00Z"
  }
}
```

#### バリデーションエラー例 (400 Bad Request):
```bash
curl -X PATCH http://localhost:3000/users/username \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "username": "ab"
  }' | jq
```

レスポンス:
```json
{
  "errors": [
    "username: Username must be at least 3 characters long"
  ],
  "error_type": "validation_errors"
}
```

## 📋 タスク管理エンドポイント

### 3. タスク作成 (POST /tasks)

新しいタスクを作成。

```bash
curl -X POST http://localhost:3000/tasks \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "重要なタスク",
    "description": "このタスクは最優先で完了する必要があります",
    "status": "todo",
    "due_date": "2025-06-30T23:59:59Z"
  }' | jq
```

#### 成功レスポンス例 (201 Created):
```json
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
```

### 4. タスク一覧取得 (GET /tasks)

ユーザーのすべてのタスクを取得。

```bash
curl -X GET http://localhost:3000/tasks \
  -H "Authorization: Bearer <access_token>" | jq
```

#### 成功レスポンス例 (200 OK):
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

### 5. 特定タスク取得 (GET /tasks/{id})

特定のタスクの詳細を取得。

```bash
curl -X GET http://localhost:3000/tasks/550e8400-e29b-41d4-a716-446655440001 \
  -H "Authorization: Bearer <access_token>" | jq
```

#### 成功レスポンス例 (200 OK):
```json
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
```

### 6. タスク更新 (PATCH /tasks/{id})

既存のタスクを部分的に更新。

```bash
curl -X PATCH http://localhost:3000/tasks/550e8400-e29b-41d4-a716-446655440001 \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "status": "in_progress",
    "description": "進行中のタスクです"
  }' | jq
```

#### 成功レスポンス例 (200 OK):
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440001",
  "title": "重要なタスク",
  "description": "進行中のタスクです",
  "status": "in_progress",
  "due_date": "2025-06-30T23:59:59Z",
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "created_at": "2025-06-12T10:00:00Z",
  "updated_at": "2025-06-12T11:30:00Z"
}
```

### 7. タスク削除 (DELETE /tasks/{id})

特定のタスクを削除。

```bash
curl -X DELETE http://localhost:3000/tasks/550e8400-e29b-41d4-a716-446655440001 \
  -H "Authorization: Bearer <access_token>" -v
```

#### 成功レスポンス (204 No Content):
- HTTPステータスコード: 204
- レスポンスボディ: なし

### 8. ページネーション付きタスク一覧 (GET /tasks/paginated)

ページネーション機能付きでタスクを取得。

```bash
curl -X GET "http://localhost:3000/tasks/paginated?page=1&page_size=10" \
  -H "Authorization: Bearer <access_token>" | jq
```

#### 成功レスポンス例 (200 OK):
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

### 9. タスクフィルタリング (GET /tasks/filter)

条件に基づいてタスクをフィルタリング。

```bash
# ステータスでフィルタリング
curl -X GET "http://localhost:3000/tasks/filter?status=todo&limit=5" \
  -H "Authorization: Bearer <access_token>" | jq

# タイトルで検索
curl -X GET "http://localhost:3000/tasks/filter?title_contains=重要&page=1&page_size=5" \
  -H "Authorization: Bearer <access_token>" | jq

# 期日でフィルタリング
curl -X GET "http://localhost:3000/tasks/filter?due_date_before=2025-07-01T00:00:00Z&sort_by=due_date&sort_order=asc" \
  -H "Authorization: Bearer <access_token>" | jq
```

## 🔄 一括操作エンドポイント

### 10. 一括タスク作成 (POST /tasks/batch/create)

複数のタスクを一度に作成。

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
  }' | jq
```

#### 成功レスポンス例 (201 Created):
```json
{
  "created_tasks": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440002",
      "title": "一括タスク1",
      "description": "一括作成テスト1",
      "status": "todo",
      "due_date": null,
      "user_id": "550e8400-e29b-41d4-a716-446655440000",
      "created_at": "2025-06-12T12:00:00Z",
      "updated_at": "2025-06-12T12:00:00Z"
    },
    {
      "id": "550e8400-e29b-41d4-a716-446655440003",
      "title": "一括タスク2",
      "description": "一括作成テスト2",
      "status": "todo",
      "due_date": "2025-07-01T12:00:00Z",
      "user_id": "550e8400-e29b-41d4-a716-446655440000",
      "created_at": "2025-06-12T12:00:00Z",
      "updated_at": "2025-06-12T12:00:00Z"
    }
  ]
}
```

### 11. 一括タスク更新 (PATCH /tasks/batch/update)

複数のタスクを一度に更新。

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
  }' | jq
```

#### 成功レスポンス例 (200 OK):
```json
{
  "updated_count": 2
}
```

### 12. 一括タスク削除 (POST /tasks/batch/delete)

複数のタスクを一度に削除。

```bash
curl -X POST http://localhost:3000/tasks/batch/delete \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "ids": [
      "550e8400-e29b-41d4-a716-446655440002",
      "550e8400-e29b-41d4-a716-446655440003"
    ]
  }' | jq
```

#### 成功レスポンス例 (200 OK):
```json
{
  "deleted_count": 2
}
```

## 🔐 認証関連保護エンドポイント

### 13. 現在のユーザー情報取得 (GET /auth/me)

認証済みユーザーの詳細情報を取得。

```bash
curl -X GET http://localhost:3000/auth/me \
  -H "Authorization: Bearer <access_token>" | jq
```

### 14. パスワード変更 (PUT /auth/change-password)

現在のユーザーのパスワードを変更。

```bash
curl -X PUT http://localhost:3000/auth/change-password \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "current_password": "CurrentPass123!",
    "new_password": "NewSecurePass456!",
    "new_password_confirmation": "NewSecurePass456!"
  }' | jq
```

### 15. ログアウト (POST /auth/signout)

現在のセッションを終了。

```bash
curl -X POST http://localhost:3000/auth/signout \
  -H "Authorization: Bearer <access_token>" | jq
```

### 16. 全デバイスからログアウト (POST /auth/signout-all)

すべてのデバイスからログアウト。

```bash
curl -X POST http://localhost:3000/auth/signout-all \
  -H "Authorization: Bearer <access_token>" | jq
```

### 17. アカウント削除 (DELETE /auth/account)

ユーザーアカウントを完全に削除。

```bash
curl -X DELETE http://localhost:3000/auth/account \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "password": "CurrentPass123!",
    "confirmation": "DELETE"
  }' | jq
```

## 🛠️ 実用的な使用例

### 完全なワークフロー例

```bash
# 1. 認証してアクセストークンを取得
SIGNIN_RESPONSE=$(curl -s -X POST http://localhost:3000/auth/signin \
  -H "Content-Type: application/json" \
  -d '{
    "identifier": "user@example.com",
    "password": "SecurePass123!"
  }')

ACCESS_TOKEN=$(echo $SIGNIN_RESPONSE | jq -r '.tokens.access_token')

# 2. 新しいタスクを作成
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

# 3. タスクを進行中に更新
curl -s -X PATCH http://localhost:3000/tasks/$TASK_ID \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "status": "in_progress"
  }' | jq

# 4. 現在のすべてのタスクを確認
curl -s -X GET http://localhost:3000/tasks \
  -H "Authorization: Bearer $ACCESS_TOKEN" | jq

# 5. 進行中のタスクのみフィルタリング
curl -s -X GET "http://localhost:3000/tasks/filter?status=in_progress" \
  -H "Authorization: Bearer $ACCESS_TOKEN" | jq
```

## ⚠️ エラーハンドリング

### よくあるエラーレスポンス

#### 1. 認証エラー (401 Unauthorized)
```json
{
  "error": "Missing authentication token",
  "error_type": "unauthorized"
}
```

#### 2. 権限エラー (403 Forbidden)
```json
{
  "error": "Access denied to this resource",
  "error_type": "forbidden"
}
```

#### 3. リソースが見つからない (404 Not Found)
```json
{
  "error": "Task with id 550e8400-e29b-41d4-a716-446655440999 not found",
  "error_type": "not_found"
}
```

#### 4. バリデーションエラー (400 Bad Request)
```json
{
  "errors": [
    "Title cannot be empty",
    "Invalid status value"
  ],
  "error_type": "validation_errors"
}
```

## 📝 注意事項

1. **アクセストークンの有効期限**: 15分で自動期限切れ
2. **ユーザー固有データ**: すべてのタスクは認証済みユーザーに関連付けられ、他のユーザーからは見えません
3. **レート制限**: 一部のエンドポイントにはレート制限があります
4. **データ検証**: すべての入力データは厳密に検証されます
5. **セキュリティ**: 本番環境では必ずHTTPS接続を使用してください