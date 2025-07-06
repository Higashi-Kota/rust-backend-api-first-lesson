# 組織管理エンドポイント

組織（Organization）の作成、管理、メンバー招待などのAPIエンドポイント群です。組織機能により、複数のチームを統括した大規模な組織管理が可能です。

## 認証必要エンドポイント

すべての組織管理エンドポイントにはJWT認証が必要です。

### 1. 組織作成 (POST /organizations)

新しい組織を作成します。作成者は自動的に組織のオーナーになります。

**リクエスト例:**
```bash
curl -X POST http://localhost:5000/organizations \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "株式会社サンプル",
    "description": "サンプル企業の組織アカウント",
    "subscription_tier": "pro"
  }'
```

**レスポンス例 (201 Created):**
```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440001",
    "name": "株式会社サンプル",
    "description": "サンプル企業の組織アカウント",
    "owner_id": "550e8400-e29b-41d4-a716-446655440000",
    "subscription_tier": "pro",
    "max_teams": 20,
    "max_members": 100,
    "settings": {
      "allow_public_teams": false,
      "require_approval_for_new_members": true,
      "enable_single_sign_on": false,
      "default_team_subscription_tier": "pro"
    },
    "members": [
      {
        "id": "550e8400-e29b-41d4-a716-446655440100",
        "user_id": "550e8400-e29b-41d4-a716-446655440000",
        "username": "owner@example.com",
        "email": "owner@example.com",
        "role": "Owner",
        "joined_at": "2025-06-12T10:00:00Z"
      }
    ],
    "current_teams": 0,
    "created_at": "2025-06-12T10:00:00Z",
    "updated_at": "2025-06-12T10:00:00Z"
  }
}
```

### 2. 組織一覧取得 (GET /organizations)

ユーザーが所属する組織一覧を取得します。

**リクエスト例:**
```bash
curl -X GET http://localhost:5000/organizations \
  -H "Authorization: Bearer <access_token>"
```

**レスポンス例 (200 OK):**
```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440001",
    "name": "株式会社サンプル", 
    "description": "サンプル企業の組織アカウント",
    "domain": "sample-corp.com",
    "is_public": false,
    "owner_id": "550e8400-e29b-41d4-a716-446655440000",
    "member_count": 25,
    "team_count": 5,
    "user_role": "owner",
    "created_at": "2025-06-12T10:00:00Z",
    "updated_at": "2025-06-12T10:00:00Z"
  },
  {
    "id": "550e8400-e29b-41d4-a716-446655440002",
    "name": "オープンソースプロジェクト",
    "description": "コミュニティ主導の開発プロジェクト",
    "domain": null,
    "is_public": true,
    "owner_id": "550e8400-e29b-41d4-a716-446655440003",
    "member_count": 150,
    "team_count": 12,
    "user_role": "admin",
    "created_at": "2025-06-10T08:00:00Z",
    "updated_at": "2025-06-12T14:00:00Z"
  }
]
```

### 3. 組織詳細取得 (GET /organizations/{id})

指定した組織の詳細情報を取得します。

**リクエスト例:**
```bash
curl -X GET http://localhost:5000/organizations/550e8400-e29b-41d4-a716-446655440001 \
  -H "Authorization: Bearer <access_token>"
```

**レスポンス例 (200 OK):**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440001",
  "name": "株式会社サンプル",
  "description": "サンプル企業の組織アカウント",
  "domain": "sample-corp.com",
  "is_public": false,
  "owner": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "username": "ceo",
    "display_name": "CEO"
  },
  "members": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "username": "ceo",
      "display_name": "CEO",
      "role": "owner",
      "joined_at": "2025-06-12T10:00:00Z"
    },
    {
      "id": "550e8400-e29b-41d4-a716-446655440004",
      "username": "manager1",
      "display_name": "マネージャー1",
      "role": "admin",
      "joined_at": "2025-06-12T11:00:00Z"
    }
  ],
  "teams": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440101",
      "name": "開発チーム",
      "member_count": 8
    },
    {
      "id": "550e8400-e29b-41d4-a716-446655440102", 
      "name": "マーケティングチーム",
      "member_count": 5
    }
  ],
  "member_count": 25,
  "team_count": 5,
  "settings": {
    "allow_public_join": false,
    "require_domain_verification": true,
    "default_member_role": "member"
  },
  "created_at": "2025-06-12T10:00:00Z",
  "updated_at": "2025-06-12T10:00:00Z"
}
```

### 4. 組織更新 (PATCH /organizations/{id})

組織の基本情報を更新します。オーナーまたは管理者権限が必要です。

**リクエスト例:**
```bash
curl -X PATCH http://localhost:5000/organizations/550e8400-e29b-41d4-a716-446655440001 \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "株式会社サンプル（更新）",
    "description": "グローバル展開中のテクノロジー企業",
    "is_public": true
  }'
```

### 5. 組織削除 (DELETE /organizations/{id})

組織を削除します。オーナー権限が必要です。

**リクエスト例:**
```bash
curl -X DELETE http://localhost:5000/organizations/550e8400-e29b-41d4-a716-446655440001 \
  -H "Authorization: Bearer <access_token>"
```

**成功レスポンス:** 204 No Content

### 6. 組織設定更新 (PATCH /organizations/{id}/settings)

組織の設定を更新します。オーナーまたは管理者権限が必要です。

**リクエスト例:**
```bash
curl -X PATCH http://localhost:5000/organizations/550e8400-e29b-41d4-a716-446655440001/settings \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "allow_public_join": true,
    "require_domain_verification": false,
    "default_member_role": "member",
    "max_teams_per_member": 5
  }'
```

## 組織メンバー管理

### 7. 組織メンバー招待 (POST /organizations/{id}/members)

組織に新しいメンバーを招待します。オーナーまたは管理者権限が必要です。

**リクエスト例:**
```bash
curl -X POST http://localhost:5000/organizations/550e8400-e29b-41d4-a716-446655440001/members \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "newemployee@sample-corp.com",
    "role": "member"
  }'
```

**またはユーザーIDで招待:**
```bash
curl -X POST http://localhost:5000/organizations/550e8400-e29b-41d4-a716-446655440001/members \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "550e8400-e29b-41d4-a716-446655440005",
    "role": "admin"
  }'
```

**レスポンス例 (201 Created):**
```json
{
  "message": "Member invited successfully",
  "member": {
    "id": "550e8400-e29b-41d4-a716-446655440005",
    "username": "newemployee",
    "display_name": "新入社員",
    "role": "member",
    "joined_at": "2025-06-12T15:00:00Z"
  }
}
```

### 8. 組織メンバー役割更新 (PATCH /organizations/{id}/members/{member_id})

組織メンバーの役割を変更します。オーナーまたは管理者権限が必要です。

**リクエスト例:**
```bash
curl -X PATCH http://localhost:5000/organizations/550e8400-e29b-41d4-a716-446655440001/members/550e8400-e29b-41d4-a716-446655440005 \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "role": "admin"
  }'
```

**可能な役割:**
- `member`: 一般メンバー
- `admin`: 組織管理者（メンバー管理、チーム作成可能）
- `owner`: 組織オーナー（全権限）

### 9. 組織メンバー削除 (DELETE /organizations/{id}/members/{member_id})

組織からメンバーを削除します。オーナーまたは管理者権限が必要です。

**リクエスト例:**
```bash
curl -X DELETE http://localhost:5000/organizations/550e8400-e29b-41d4-a716-446655440001/members/550e8400-e29b-41d4-a716-446655440005 \
  -H "Authorization: Bearer <access_token>"
```

**成功レスポンス:** 204 No Content

## 統計・分析

### 10. 組織統計取得 (GET /organizations/stats)

ユーザーが所属する組織の統計情報を取得します。

**リクエスト例:**
```bash
curl -X GET http://localhost:5000/organizations/stats \
  -H "Authorization: Bearer <access_token>"
```

**レスポンス例 (200 OK):**
```json
{
  "total_organizations": 2,
  "owned_organizations": 1,
  "member_of_organizations": 1,
  "total_organization_members": 175,
  "total_teams_in_organizations": 17,
  "organizations_breakdown": [
    {
      "organization_id": "550e8400-e29b-41d4-a716-446655440001",
      "organization_name": "株式会社サンプル",
      "role": "owner",
      "member_count": 25,
      "team_count": 5,
      "active_projects": 12
    },
    {
      "organization_id": "550e8400-e29b-41d4-a716-446655440002",
      "organization_name": "オープンソースプロジェクト",
      "role": "admin",
      "member_count": 150,
      "team_count": 12,
      "active_projects": 35
    }
  ]
}
```

## 組織権限システム

### 役割別権限

**Owner (オーナー):**
- 組織設定の変更
- 組織の削除
- メンバーの招待・削除・役割変更
- チームの作成・削除・管理
- 全組織データへのアクセス

**Admin (管理者):**
- メンバーの招待・削除
- 一般メンバーの役割変更
- チームの作成・管理
- 組織内データへのアクセス

**Member (メンバー):**
- 組織情報の閲覧
- 所属チームでの活動
- 組織内コラボレーション

### サブスクリプション制限

組織機能の利用制限はサブスクリプション階層によって決まります：

- **Free**: 最大3チーム、最大10メンバー
- **Pro**: 最大20チーム、最大100メンバー
- **Enterprise**: 最大100チーム、最大1000メンバー

### 6. 組織設定更新 (PATCH /organizations/{id}/settings)

組織の詳細設定を更新します。オーナーまたは管理者権限が必要です。

**リクエスト例:**
```bash
curl -X PATCH http://localhost:5000/organizations/550e8400-e29b-41d4-a716-446655440001/settings \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "allow_public_teams": true,
    "require_approval_for_new_members": false,
    "enable_single_sign_on": true,
    "default_team_subscription_tier": "pro"
  }'
```

**レスポンス例 (200 OK):**
```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440001",
    "name": "株式会社サンプル",
    "description": "サンプル企業の組織アカウント",
    "owner_id": "550e8400-e29b-41d4-a716-446655440000",
    "subscription_tier": "pro",
    "settings": {
      "allow_public_teams": true,
      "require_approval_for_new_members": false,
      "enable_single_sign_on": true,
      "default_team_subscription_tier": "pro"
    },
    "updated_at": "2025-06-12T15:30:00Z"
  }
}
```

### 7. 組織サブスクリプション更新 (PATCH /organizations/{id}/subscription)

組織のサブスクリプション階層を変更します。オーナー権限が必要です。

**リクエスト例:**
```bash
curl -X PATCH http://localhost:5000/organizations/550e8400-e29b-41d4-a716-446655440001/subscription \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "subscription_tier": "enterprise"
  }'
```

**レスポンス例 (200 OK):**
```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440001",
    "name": "株式会社サンプル",
    "subscription_tier": "enterprise",
    "max_teams": 100,
    "max_members": 1000,
    "previous_tier": "pro",
    "upgraded_at": "2025-06-12T16:00:00Z"
  }
}
```

**ダウングレード時の制約:**
- 現在のチーム数が新しいプランの上限を超えている場合はエラー
- 現在のメンバー数が新しいプランの上限を超えている場合はエラー

### 8. 組織サブスクリプション履歴 (GET /organizations/{id}/subscription/history)

組織のサブスクリプション変更履歴を取得します。

**リクエスト例:**
```bash
curl -X GET http://localhost:5000/organizations/550e8400-e29b-41d4-a716-446655440001/subscription/history \
  -H "Authorization: Bearer <access_token>"
```

**レスポンス例 (200 OK):**
```json
{
  "success": true,
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440100",
      "user_id": "550e8400-e29b-41d4-a716-446655440000",
      "old_tier": "free",
      "new_tier": "pro",
      "changed_at": "2025-06-12T10:00:00Z",
      "changed_by": "550e8400-e29b-41d4-a716-446655440000",
      "reason": "Initial organization creation"
    },
    {
      "id": "550e8400-e29b-41d4-a716-446655440101",
      "user_id": "550e8400-e29b-41d4-a716-446655440000",
      "old_tier": "pro",
      "new_tier": "enterprise",
      "changed_at": "2025-06-12T16:00:00Z",
      "changed_by": "550e8400-e29b-41d4-a716-446655440000",
      "reason": "Upgraded for increased capacity"
    }
  ]
}
```

### 9. 組織容量確認 (GET /organizations/{id}/capacity)

組織の現在の使用状況と制限を確認します。

**リクエスト例:**
```bash
curl -X GET http://localhost:5000/organizations/550e8400-e29b-41d4-a716-446655440001/capacity \
  -H "Authorization: Bearer <access_token>"
```

**レスポンス例 (200 OK):**
```json
{
  "success": true,
  "data": {
    "subscription_tier": "enterprise",
    "teams": {
      "current": 15,
      "max": 100,
      "percentage_used": 15.0
    },
    "members": {
      "current": 85,
      "max": 1000,
      "percentage_used": 8.5
    },
    "storage": {
      "current_gb": 45.2,
      "max_gb": 500,
      "percentage_used": 9.04
    },
    "can_add_teams": true,
    "can_add_members": true
  }
}
```

## 組織階層構造

### 部門管理

組織は部門（Department）を持つことができ、階層的な権限管理が可能です。

### 10. 部門作成 (POST /organizations/{id}/departments)

**リクエスト例:**
```bash
curl -X POST http://localhost:5000/organizations/550e8400-e29b-41d4-a716-446655440001/departments \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "開発部門",
    "description": "ソフトウェア開発を担当する部門",
    "parent_department_id": null
  }'
```

### 11. 部門メンバー管理

部門へのメンバー割り当てと権限の継承が可能です。

## 使用例

### 組織作成から運用開始までの完整な流れ

```bash
# アクセストークンを取得済みと仮定
ACCESS_TOKEN="your_access_token_here"

# 1. 新しい組織を作成
ORG_RESPONSE=$(curl -s -X POST http://localhost:5000/organizations \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "テックスタートアップ",
    "description": "革新的なテクノロジースタートアップ",
    "subscription_tier": "pro"
  }')

ORG_ID=$(echo $ORG_RESPONSE | jq -r '.id')

# 2. 組織設定を調整
curl -s -X PATCH http://localhost:5000/organizations/$ORG_ID/settings \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "allow_public_teams": false,
    "require_approval_for_new_members": true,
    "enable_single_sign_on": false,
    "default_team_subscription_tier": "pro"
  }'

# 3. 管理者を招待
curl -s -X POST http://localhost:5000/organizations/$ORG_ID/members \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "cto@techstartup.com",
    "role": "admin"
  }'

# 4. 一般メンバーを招待
curl -s -X POST http://localhost:5000/organizations/$ORG_ID/members \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "developer@techstartup.com",
    "role": "member"
  }'

# 5. 組織詳細を確認
curl -s -X GET http://localhost:5000/organizations/$ORG_ID \
  -H "Authorization: Bearer $ACCESS_TOKEN"

# 6. 組織統計を確認
curl -s -X GET http://localhost:5000/organizations/stats \
  -H "Authorization: Bearer $ACCESS_TOKEN"
```

## エラーレスポンス例

### サブスクリプション制限 (402 Payment Required)
```json
{
  "error": "Organization features require Enterprise subscription",
  "error_type": "subscription_required"
}
```

### 権限エラー (403 Forbidden)
```json
{
  "error": "Only organization owner or admin can invite members",
  "error_type": "forbidden"
}
```

### ドメイン検証エラー (400 Bad Request)
```json
{
  "error": "Email domain does not match organization domain",
  "error_type": "domain_mismatch"
}
```

### リソースが見つからない (404 Not Found)
```json
{
  "error": "Organization with id 550e8400-e29b-41d4-a716-446655440999 not found",
  "error_type": "not_found"
}
```

## 注意事項

1. **サブスクリプション階層**: 組織の機能はサブスクリプション階層によって制限されます
2. **ダウングレード制約**: サブスクリプションをダウングレードする際は、現在のリソース使用量が新しいプランの制限内である必要があります
3. **組織削除**: 組織を削除すると、関連するチームとタスクもすべて削除されます
4. **階層構造**: 組織 > 部門 > チーム > ユーザー という階層構造でデータアクセス権限が管理されます
5. **権限継承**: 部門の権限は下位の部門とチームに継承されます