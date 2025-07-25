# チーム管理エンドポイント

チームの作成、管理、メンバー招待などのAPIエンドポイント群です。チーム機能により、複数のユーザーが協力してタスクを管理できます。

## 認証必要エンドポイント

すべてのチーム管理エンドポイントにはJWT認証が必要です。

### 1. チーム作成 (POST /teams)

新しいチームを作成します。作成者は自動的にチームのオーナーになります。

**リクエスト例:**
```bash
curl -X POST http://localhost:5000/teams \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "開発チーム",
    "description": "メインプロダクト開発チーム",
    "is_public": false
  }'
```

**レスポンス例 (201 Created):**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440001",
  "name": "開発チーム",
  "description": "メインプロダクト開発チーム",
  "is_public": false,
  "owner_id": "550e8400-e29b-41d4-a716-446655440000",
  "member_count": 1,
  "created_at": "2025-06-12T10:00:00Z",
  "updated_at": "2025-06-12T10:00:00Z"
}
```

### 2. チーム一覧取得 (GET /teams)

ユーザーが所属するチーム一覧を取得します。

**リクエスト例:**
```bash
curl -X GET http://localhost:5000/teams \
  -H "Authorization: Bearer <access_token>"
```

**レスポンス例 (200 OK):**
```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440001",
    "name": "開発チーム",
    "description": "メインプロダクト開発チーム",
    "is_public": false,
    "owner_id": "550e8400-e29b-41d4-a716-446655440000",
    "member_count": 5,
    "user_role": "owner",
    "created_at": "2025-06-12T10:00:00Z",
    "updated_at": "2025-06-12T10:00:00Z"
  },
  {
    "id": "550e8400-e29b-41d4-a716-446655440002",
    "name": "QAチーム",
    "description": "品質保証チーム",
    "is_public": true,
    "owner_id": "550e8400-e29b-41d4-a716-446655440003",
    "member_count": 3,
    "user_role": "member",
    "created_at": "2025-06-12T11:00:00Z",
    "updated_at": "2025-06-12T11:00:00Z"
  }
]
```

### 3. チーム詳細取得 (GET /teams/{id})

指定したチームの詳細情報を取得します。

**リクエスト例:**
```bash
curl -X GET http://localhost:5000/teams/550e8400-e29b-41d4-a716-446655440001 \
  -H "Authorization: Bearer <access_token>"
```

**レスポンス例 (200 OK):**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440001",
  "name": "開発チーム",
  "description": "メインプロダクト開発チーム",
  "is_public": false,
  "owner": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "username": "team_owner",
    "display_name": "チームオーナー"
  },
  "members": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "username": "team_owner",
      "display_name": "チームオーナー",
      "role": "owner",
      "joined_at": "2025-06-12T10:00:00Z"
    },
    {
      "id": "550e8400-e29b-41d4-a716-446655440004",
      "username": "developer1",
      "display_name": "開発者1",
      "role": "member",
      "joined_at": "2025-06-12T12:00:00Z"
    }
  ],
  "member_count": 5,
  "created_at": "2025-06-12T10:00:00Z",
  "updated_at": "2025-06-12T10:00:00Z"
}
```

### 4. チーム更新 (PATCH /teams/{id})

チームの基本情報を更新します。オーナーまたは管理者権限が必要です。

**リクエスト例:**
```bash
curl -X PATCH http://localhost:5000/teams/550e8400-e29b-41d4-a716-446655440001 \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "メイン開発チーム",
    "description": "プライマリプロダクトの開発を担当するチーム",
    "is_public": true
  }'
```

### 5. チーム削除 (DELETE /teams/{id})

チームを削除します。オーナーまたは管理者権限が必要です。

**リクエスト例:**
```bash
curl -X DELETE http://localhost:5000/teams/550e8400-e29b-41d4-a716-446655440001 \
  -H "Authorization: Bearer <access_token>"
```

**成功レスポンス:** 204 No Content

## チームメンバー管理

### 6. チームメンバー追加 (POST /teams/{id}/members)

既存のユーザーをチームに追加します。オーナーまたは管理者権限が必要です。

**リクエスト例:**
```bash
curl -X POST http://localhost:5000/teams/550e8400-e29b-41d4-a716-446655440001/members \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "550e8400-e29b-41d4-a716-446655440005",
    "role": "member"
  }'
```

**レスポンス例 (201 Created):**
```json
{
  "message": "Member added successfully",
  "member": {
    "id": "550e8400-e29b-41d4-a716-446655440005",
    "username": "newmember",
    "display_name": "新しいメンバー",
    "role": "member",
    "joined_at": "2025-06-12T15:00:00Z"
  }
}
```

### 7. チームメンバー役割更新 (PATCH /teams/{team_id}/members/{member_id}/role)

チームメンバーの役割を変更します。オーナーまたは管理者権限が必要です。

**リクエスト例:**
```bash
curl -X PATCH http://localhost:5000/teams/550e8400-e29b-41d4-a716-446655440001/members/550e8400-e29b-41d4-a716-446655440005/role \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "role": "admin"
  }'
```

**可能な役割:**
- `member`: 一般メンバー
- `admin`: チーム管理者（メンバー招待・削除可能）
- `owner`: チームオーナー（全権限）

### 8. チームメンバー削除 (DELETE /teams/{team_id}/members/{member_id})

チームからメンバーを削除します。オーナーまたは管理者権限が必要です。

**リクエスト例:**
```bash
curl -X DELETE http://localhost:5000/teams/550e8400-e29b-41d4-a716-446655440001/members/550e8400-e29b-41d4-a716-446655440005 \
  -H "Authorization: Bearer <access_token>"
```

**成功レスポンス:** 204 No Content

## チーム招待システム

### 9. 単一メンバー招待 (POST /teams/{id}/invitations/single)

メールアドレスを使って新しいメンバーをチームに招待します。

**リクエスト例:**
```bash
curl -X POST http://localhost:5000/teams/550e8400-e29b-41d4-a716-446655440001/invitations/single \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "newmember@example.com",
    "message": "プロジェクトに参加してください！"
  }'
```

**レスポンス例 (201 Created):**
```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440006",
    "team_id": "550e8400-e29b-41d4-a716-446655440001",
    "email": "newmember@example.com",
    "status": "pending",
    "expires_at": "2025-06-19T15:00:00Z",
    "created_at": "2025-06-12T15:00:00Z"
  }
}
```

### 10. 一括メンバー招待 (POST /teams/{id}/invitations/bulk)

複数のメンバーを一度に招待します。

**リクエスト例:**
```bash
curl -X POST http://localhost:5000/teams/550e8400-e29b-41d4-a716-446655440001/invitations/bulk \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "invitations": [
      {
        "email": "developer1@example.com",
        "message": "開発チームへようこそ！"
      },
      {
        "email": "developer2@example.com",
        "message": "プロジェクトに参加してください"
      }
    ]
  }'
```

**レスポンス例 (201 Created):**
```json
{
  "success": true,
  "data": {
    "sent": 2,
    "failed": 0,
    "invitations": [
      {
        "email": "developer1@example.com",
        "status": "sent",
        "invitation_id": "550e8400-e29b-41d4-a716-446655440007"
      },
      {
        "email": "developer2@example.com",
        "status": "sent",
        "invitation_id": "550e8400-e29b-41d4-a716-446655440008"
      }
    ]
  }
}
```

### 11. 招待一覧取得 (GET /teams/{id}/invitations)

チームの招待状況を確認します。

**リクエスト例:**
```bash
curl -X GET http://localhost:5000/teams/550e8400-e29b-41d4-a716-446655440001/invitations \
  -H "Authorization: Bearer <access_token>"
```

**レスポンス例 (200 OK):**
```json
{
  "success": true,
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440006",
      "email": "newmember@example.com",
      "status": "pending",
      "invited_by": {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "username": "team_owner"
      },
      "created_at": "2025-06-12T15:00:00Z",
      "expires_at": "2025-06-19T15:00:00Z"
    },
    {
      "id": "550e8400-e29b-41d4-a716-446655440007",
      "email": "developer1@example.com",
      "status": "accepted",
      "accepted_at": "2025-06-13T10:00:00Z",
      "invited_by": {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "username": "team_owner"
      },
      "created_at": "2025-06-12T16:00:00Z"
    }
  ]
}
```

### 12. 招待承認 (POST /teams/{team_id}/invitations/{invitation_id}/accept)

受信した招待を承認します。

**リクエスト例:**
```bash
curl -X POST http://localhost:5000/teams/550e8400-e29b-41d4-a716-446655440001/invitations/550e8400-e29b-41d4-a716-446655440006/accept \
  -H "Authorization: Bearer <invitee_access_token>"
```

**レスポンス例 (200 OK):**
```json
{
  "success": true,
  "data": {
    "message": "Invitation accepted successfully",
    "team": {
      "id": "550e8400-e29b-41d4-a716-446655440001",
      "name": "開発チーム",
      "role": "member"
    }
  }
}
```

### 13. 招待拒否 (POST /teams/{team_id}/invitations/{invitation_id}/decline)

受信した招待を拒否します。

**リクエスト例:**
```bash
curl -X POST http://localhost:5000/teams/550e8400-e29b-41d4-a716-446655440001/invitations/550e8400-e29b-41d4-a716-446655440006/decline \
  -H "Authorization: Bearer <invitee_access_token>"
```

### 14. 招待キャンセル (DELETE /teams/{team_id}/invitations/{invitation_id}/cancel)

送信した招待をキャンセルします。

**リクエスト例:**
```bash
curl -X DELETE http://localhost:5000/teams/550e8400-e29b-41d4-a716-446655440001/invitations/550e8400-e29b-41d4-a716-446655440006/cancel \
  -H "Authorization: Bearer <access_token>"
```

### 15. 招待再送信 (POST /teams/{team_id}/invitations/{invitation_id}/resend)

期限切れになっていない招待を再送信します。

**リクエスト例:**
```bash
curl -X POST http://localhost:5000/teams/550e8400-e29b-41d4-a716-446655440001/invitations/550e8400-e29b-41d4-a716-446655440006/resend \
  -H "Authorization: Bearer <access_token>"
```

### 16. 招待統計取得 (GET /teams/{id}/invitations/statistics)

チームの招待に関する統計情報を取得します。

**リクエスト例:**
```bash
curl -X GET http://localhost:5000/teams/550e8400-e29b-41d4-a716-446655440001/invitations/statistics \
  -H "Authorization: Bearer <access_token>"
```

**レスポンス例 (200 OK):**
```json
{
  "success": true,
  "data": {
    "total_sent": 15,
    "pending": 3,
    "accepted": 10,
    "declined": 1,
    "expired": 1,
    "acceptance_rate": 66.67,
    "avg_response_time_hours": 24.5
  }
}
```

### 17. ユーザーの招待一覧 (GET /users/invitations)

ユーザーが受信したチーム招待の一覧を取得します。

**リクエスト例:**
```bash
curl -X GET http://localhost:5000/users/invitations \
  -H "Authorization: Bearer <access_token>"
```

**レスポンス例 (200 OK):**
```json
{
  "success": true,
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440009",
      "team": {
        "id": "550e8400-e29b-41d4-a716-446655440002",
        "name": "QAチーム",
        "description": "品質保証チーム"
      },
      "invited_by": {
        "id": "550e8400-e29b-41d4-a716-446655440003",
        "username": "qa_lead"
      },
      "message": "QAチームに参加しませんか？",
      "status": "pending",
      "created_at": "2025-06-14T10:00:00Z",
      "expires_at": "2025-06-21T10:00:00Z"
    }
  ]
}
```

## 統計・分析

### 9. チーム統計取得 (GET /teams/stats)

ユーザーが所属するチームの統計情報を取得します。

**リクエスト例:**
```bash
curl -X GET http://localhost:5000/teams/stats \
  -H "Authorization: Bearer <access_token>"
```

**レスポンス例 (200 OK):**
```json
{
  "total_teams": 2,
  "owned_teams": 1,
  "member_of_teams": 1,
  "total_team_members": 8,
  "teams_breakdown": [
    {
      "team_id": "550e8400-e29b-41d4-a716-446655440001",
      "team_name": "開発チーム",
      "role": "owner",
      "member_count": 5,
      "active_tasks": 12
    },
    {
      "team_id": "550e8400-e29b-41d4-a716-446655440002",
      "team_name": "QAチーム",
      "role": "member",
      "member_count": 3,
      "active_tasks": 8
    }
  ]
}
```

## チーム権限システム

### 役割別権限

**Owner (オーナー):**
- チーム設定の変更
- チームの削除
- メンバーの招待・削除
- メンバー役割の変更
- すべてのチームタスクへのアクセス

**Admin (管理者):**
- メンバーの招待・削除
- 一般メンバーの役割変更
- すべてのチームタスクへのアクセス

**Member (メンバー):**
- チーム情報の閲覧
- チームタスクの作成・編集
- チーム内での協力作業

### 動的パーミッションとの連携

チーム機能は動的パーミッションシステムと連携しており、サブスクリプション階層によって以下の制限があります：

- **Free**: チーム機能利用不可
- **Pro**: 最大3チーム作成、チーム当たり最大10名
- **Enterprise**: 無制限のチーム作成とメンバー数

## 使用例

### チーム作成からメンバー招待までの完整な流れ

```bash
# アクセストークンを取得済みと仮定
ACCESS_TOKEN="your_access_token_here"

# 1. 新しいチームを作成
TEAM_RESPONSE=$(curl -s -X POST http://localhost:5000/teams \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "新プロジェクトチーム",
    "description": "新製品開発プロジェクト専用チーム",
    "is_public": false
  }')

TEAM_ID=$(echo $TEAM_RESPONSE | jq -r '.id')

# 2. チームにメンバーを招待
curl -s -X POST http://localhost:5000/teams/$TEAM_ID/members \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "developer@example.com",
    "role": "member"
  }'

# 3. 別のメンバーを管理者として招待
curl -s -X POST http://localhost:5000/teams/$TEAM_ID/members \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "teamlead@example.com",
    "role": "admin"
  }'

# 4. チーム詳細を確認
curl -s -X GET http://localhost:5000/teams/$TEAM_ID \
  -H "Authorization: Bearer $ACCESS_TOKEN"

# 5. チーム統計を確認
curl -s -X GET http://localhost:5000/teams/stats \
  -H "Authorization: Bearer $ACCESS_TOKEN"
```

## エラーレスポンス例

### 権限エラー (403 Forbidden)
```json  
{
  "error": "Only team owner or admin can invite members",
  "error_type": "forbidden"
}
```

### リソースが見つからない (404 Not Found)
```json
{
  "error": "Team with id 550e8400-e29b-41d4-a716-446655440999 not found",
  "error_type": "not_found"
}
```

### サブスクリプション制限 (402 Payment Required)
```json
{
  "error": "Team creation limit reached for Free tier. Upgrade to Pro for more teams.",
  "error_type": "subscription_limit"
}
```

### バリデーションエラー (400 Bad Request)
```json
{
  "errors": [
    "Team name must be at least 3 characters long",
    "Invalid member role specified"
  ],
  "error_type": "validation_errors"
}
```

## 注意事項

1. **チーム削除**: チームを削除すると、関連するタスクもすべて削除されます
2. **オーナー変更**: チームオーナーの変更は現在未対応（今後実装予定）
3. **招待メール**: メンバー招待時は自動的に招待メールが送信されます
4. **プライベートチーム**: `is_public: false`のチームは招待されたメンバーのみアクセス可能です