# ロール管理エンドポイント

システム内のロール（役割）管理とユーザーへのロール割り当てを行うAPIエンドポイント群です。すべてのエンドポイントは管理者権限が必要です。

## 管理者専用エンドポイント

すべてのロール管理エンドポイントは管理者権限が必要です。

### 1. ロール一覧取得 (GET /admin/roles)

システム内のすべてのロール一覧を取得します。

**リクエスト例:**
```bash
curl -X GET http://localhost:5000/admin/roles \
  -H "Authorization: Bearer <admin_access_token>"
```

**レスポンス例 (200 OK):**
```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440001",
    "name": "admin",
    "display_name": "管理者",
    "description": "システム全体の管理権限",
    "permissions": [
      {
        "resource": "tasks",
        "action": "admin",
        "scope": "Global"
      },
      {
        "resource": "users", 
        "action": "admin",
        "scope": "Global"
      }
    ],
    "created_at": "2025-06-12T10:00:00Z",
    "updated_at": "2025-06-12T10:00:00Z"
  },
  {
    "id": "550e8400-e29b-41d4-a716-446655440002",
    "name": "user",
    "display_name": "一般ユーザー",
    "description": "基本的なユーザー権限",
    "permissions": [
      {
        "resource": "tasks",
        "action": "read",
        "scope": "Own"
      },
      {
        "resource": "tasks",
        "action": "write", 
        "scope": "Own"
      }
    ],
    "created_at": "2025-06-12T10:00:00Z",
    "updated_at": "2025-06-12T10:00:00Z"
  }
]
```

### 2. ロール作成 (POST /admin/roles)

新しいロールを作成します。

**リクエスト例:**
```bash
curl -X POST http://localhost:5000/admin/roles \
  -H "Authorization: Bearer <admin_access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "team_leader",
    "display_name": "チームリーダー",
    "description": "チーム管理権限を持つロール",
    "permissions": [
      {
        "resource": "tasks",
        "action": "read",
        "scope": "Team"
      },
      {
        "resource": "tasks",
        "action": "write",
        "scope": "Team"
      },
      {
        "resource": "users",
        "action": "read",
        "scope": "Team"
      }
    ]
  }'
```

**レスポンス例 (201 Created):**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440003",
  "name": "team_leader",
  "display_name": "チームリーダー",
  "description": "チーム管理権限を持つロール",
  "permissions": [
    {
      "resource": "tasks",
      "action": "read",
      "scope": "Team"
    },
    {
      "resource": "tasks", 
      "action": "write",
      "scope": "Team"
    },
    {
      "resource": "users",
      "action": "read",
      "scope": "Team"
    }
  ],
  "created_at": "2025-06-12T15:00:00Z",
  "updated_at": "2025-06-12T15:00:00Z"
}
```

### 3. 特定ロール取得 (GET /admin/roles/{id})

指定したIDのロール詳細を取得します。

**リクエスト例:**
```bash
curl -X GET http://localhost:5000/admin/roles/550e8400-e29b-41d4-a716-446655440001 \
  -H "Authorization: Bearer <admin_access_token>"
```

### 4. ロール更新 (PATCH /admin/roles/{id})

既存のロール情報を更新します。

**リクエスト例:**
```bash
curl -X PATCH http://localhost:5000/admin/roles/550e8400-e29b-41d4-a716-446655440003 \
  -H "Authorization: Bearer <admin_access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "display_name": "上級チームリーダー",
    "description": "拡張されたチーム管理権限",
    "permissions": [
      {
        "resource": "tasks",
        "action": "admin",
        "scope": "Team"
      },
      {
        "resource": "users",
        "action": "write",
        "scope": "Team"
      }
    ]
  }'
```

### 5. ロール削除 (DELETE /admin/roles/{id})

指定したロールを削除します。

**リクエスト例:**
```bash
curl -X DELETE http://localhost:5000/admin/roles/550e8400-e29b-41d4-a716-446655440003 \
  -H "Authorization: Bearer <admin_access_token>"
```

**成功レスポンス:** 204 No Content

### 6. ユーザーにロール割り当て (POST /admin/users/{id}/role)

指定したユーザーにロールを割り当てます。

**リクエスト例:**
```bash
curl -X POST http://localhost:5000/admin/users/550e8400-e29b-41d4-a716-446655440000/role \
  -H "Authorization: Bearer <admin_access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "role_id": "550e8400-e29b-41d4-a716-446655440003"
  }'
```

**レスポンス例 (200 OK):**
```json
{
  "message": "Role assigned successfully",
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "username": "testuser",
    "email": "test@example.com",
    "role": {
      "id": "550e8400-e29b-41d4-a716-446655440003",
      "name": "team_leader",
      "display_name": "チームリーダー"
    }
  }
}
```

## パーミッションスコープ

システムで使用されるパーミッションスコープの説明：

### スコープレベル
- **Own**: 自分のデータのみアクセス可能
- **Team**: 所属チーム内のデータにアクセス可能
- **Organization**: 所属組織内のデータにアクセス可能  
- **Global**: システム全体のデータにアクセス可能（管理者のみ）

### アクション種類
- **read**: データの読み取り権限
- **write**: データの作成・更新権限
- **delete**: データの削除権限
- **admin**: 管理権限（すべての操作が可能）

### リソース種類
- **tasks**: タスク管理
- **users**: ユーザー管理
- **teams**: チーム管理
- **organizations**: 組織管理
- **roles**: ロール管理
- **analytics**: 分析データ

## 使用例

### 新しいロール作成からユーザー割り当てまでの流れ

```bash
# 管理者のアクセストークンを使用
ADMIN_TOKEN="admin_access_token_here"

# 1. 新しいロール「プロジェクトマネージャー」を作成
ROLE_RESPONSE=$(curl -s -X POST http://localhost:5000/admin/roles \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "project_manager",
    "display_name": "プロジェクトマネージャー",
    "description": "プロジェクト管理権限を持つロール",
    "permissions": [
      {
        "resource": "tasks",
        "action": "admin",
        "scope": "Team"
      },
      {
        "resource": "users",
        "action": "read",
        "scope": "Team"
      },
      {
        "resource": "analytics",
        "action": "read",
        "scope": "Team"
      }
    ]
  }')

ROLE_ID=$(echo $ROLE_RESPONSE | jq -r '.id')

# 2. ユーザーにロールを割り当て
curl -s -X POST http://localhost:5000/admin/users/550e8400-e29b-41d4-a716-446655440000/role \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"role_id\": \"$ROLE_ID\"}"

# 3. ロール一覧で確認
curl -s -X GET http://localhost:5000/admin/roles \
  -H "Authorization: Bearer $ADMIN_TOKEN"

# 4. 特定ロールの詳細確認
curl -s -X GET http://localhost:5000/admin/roles/$ROLE_ID \
  -H "Authorization: Bearer $ADMIN_TOKEN"
```

## エラーレスポンス例

### 権限エラー (403 Forbidden)
```json
{
  "error": "Admin access required",
  "error_type": "forbidden"
}
```

### リソースが見つからない (404 Not Found)
```json
{
  "error": "Role with id 550e8400-e29b-41d4-a716-446655440999 not found",
  "error_type": "not_found"
}
```

### バリデーションエラー (400 Bad Request)
```json
{
  "errors": [
    "Role name must be unique",
    "Permission scope is invalid"
  ],
  "error_type": "validation_errors"
}
```

### 競合エラー (409 Conflict)
```json
{
  "error": "Role name 'admin' already exists",
  "error_type": "conflict"
}
```

## 注意事項

1. **システムロール**: `admin`と`user`は予約済みのシステムロールで削除できません
2. **パーミッション継承**: より高いスコープのロールは、低いスコープの権限も自動的に含みます
3. **ロール削除**: ユーザーに割り当てられているロールは削除前にユーザーから割り当て解除が必要です
4. **権限変更**: ロールの権限を変更すると、そのロールを持つすべてのユーザーに即座に適用されます