# 組織階層・部門管理エンドポイント

Enterprise階層の組織内での部門管理と階層構造の管理APIエンドポイント群です。

## 概要

組織階層・部門管理機能は Enterprise サブスクリプション専用機能で、大規模組織における部門構造、メンバーシップ、権限マトリックスを管理します。

## 組織階層管理

### 1. 組織階層取得 (GET /organizations/:organization_id/hierarchy)

組織の階層構造を取得します。

**リクエスト例:**
```bash
curl -X GET http://localhost:3000/organizations/550e8400-e29b-41d4-a716-446655440001/hierarchy \
  -H "Authorization: Bearer <access_token>"
```

**レスポンス例 (200 OK):**
```json
{
  "organization_id": "550e8400-e29b-41d4-a716-446655440001",
  "departments": [
    {
      "id": "dept-001",
      "name": "開発部",
      "parent_id": null,
      "level": 1,
      "members_count": 15,
      "children": [
        {
          "id": "dept-002",
          "name": "フロントエンド課",
          "parent_id": "dept-001",
          "level": 2,
          "members_count": 8
        }
      ]
    }
  ],
  "total_departments": 5,
  "max_depth": 3
}
```

## 部門管理

### 2. 部門一覧取得 (GET /organizations/:organization_id/departments)

組織内の全部門を取得します。

**リクエスト例:**
```bash
curl -X GET http://localhost:3000/organizations/550e8400-e29b-41d4-a716-446655440001/departments \
  -H "Authorization: Bearer <access_token>"
```

### 3. 部門作成 (POST /organizations/:organization_id/departments)

新しい部門を作成します。

**リクエスト例:**
```bash
curl -X POST http://localhost:3000/organizations/550e8400-e29b-41d4-a716-446655440001/departments \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "マーケティング部",
    "description": "マーケティング戦略を担当する部門",
    "parent_id": null,
    "manager_id": "550e8400-e29b-41d4-a716-446655440100"
  }'
```

### 4. 部門更新 (PUT /organizations/:organization_id/departments/:department_id)

既存の部門情報を更新します。

**リクエスト例:**
```bash
curl -X PUT http://localhost:3000/organizations/550e8400-e29b-41d4-a716-446655440001/departments/dept-001 \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "開発統括部",
    "description": "全社開発を統括する部門"
  }'
```

### 5. 部門削除 (DELETE /organizations/:organization_id/departments/:department_id)

部門を削除します。

**リクエスト例:**
```bash
curl -X DELETE http://localhost:3000/organizations/550e8400-e29b-41d4-a716-446655440001/departments/dept-001 \
  -H "Authorization: Bearer <access_token>"
```

**成功レスポンス:** 204 No Content

## 部門メンバー管理

### 6. 部門メンバー追加 (POST /organizations/:organization_id/departments/:department_id/members)

部門にメンバーを追加します。

**リクエスト例:**
```bash
curl -X POST http://localhost:3000/organizations/550e8400-e29b-41d4-a716-446655440001/departments/dept-001/members \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "550e8400-e29b-41d4-a716-446655440200",
    "role": "member",
    "start_date": "2025-07-01T00:00:00Z"
  }'
```

### 7. 部門メンバー削除 (DELETE /organizations/:organization_id/departments/:department_id/members/:user_id)

部門からメンバーを削除します。

**リクエスト例:**
```bash
curl -X DELETE http://localhost:3000/organizations/550e8400-e29b-41d4-a716-446655440001/departments/dept-001/members/550e8400-e29b-41d4-a716-446655440200 \
  -H "Authorization: Bearer <access_token>"
```

## 組織アナリティクス

### 8. 組織分析データ取得 (GET /organizations/:organization_id/analytics)

組織の分析データを取得します。

**リクエスト例:**
```bash
curl -X GET http://localhost:3000/organizations/550e8400-e29b-41d4-a716-446655440001/analytics \
  -H "Authorization: Bearer <access_token>"
```

**レスポンス例 (200 OK):**
```json
{
  "organization_id": "550e8400-e29b-41d4-a716-446655440001",
  "departments_count": 5,
  "total_members": 150,
  "active_projects": 25,
  "department_breakdown": [
    {
      "department_id": "dept-001",
      "department_name": "開発部",
      "members_count": 45,
      "active_tasks": 128,
      "completion_rate": 0.78
    }
  ],
  "performance_metrics": {
    "average_task_completion_time": "2.5 days",
    "department_efficiency_score": 0.82
  }
}
```

### 9. 分析メトリック作成 (POST /organizations/:organization_id/analytics)

新しい分析メトリックを作成します。

**リクエスト例:**
```bash
curl -X POST http://localhost:3000/organizations/550e8400-e29b-41d4-a716-446655440001/analytics \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "metric_type": "department_efficiency",
    "department_id": "dept-001",
    "value": 0.85,
    "measurement_date": "2025-06-29T00:00:00Z"
  }'
```

## 権限マトリックス管理

### 10. 権限マトリックス取得 (GET /organizations/:organization_id/permission-matrix)

組織の権限マトリックスを取得します。

**リクエスト例:**
```bash
curl -X GET http://localhost:3000/organizations/550e8400-e29b-41d4-a716-446655440001/permission-matrix \
  -H "Authorization: Bearer <access_token>"
```

### 11. 権限マトリックス設定 (PUT /organizations/:organization_id/permission-matrix)

組織の権限マトリックスを設定します。

**リクエスト例:**
```bash
curl -X PUT http://localhost:3000/organizations/550e8400-e29b-41d4-a716-446655440001/permission-matrix \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "department_permissions": [
      {
        "department_id": "dept-001",
        "permissions": ["task_create", "task_read", "task_update", "user_read"]
      }
    ],
    "role_permissions": [
      {
        "role": "department_manager",
        "permissions": ["department_manage", "member_manage", "analytics_read"]
      }
    ]
  }'
```

### 12. 有効権限取得 (GET /organizations/:organization_id/effective-permissions)

ユーザーの有効権限を取得します。

**リクエスト例:**
```bash
curl -X GET "http://localhost:3000/organizations/550e8400-e29b-41d4-a716-446655440001/effective-permissions?user_id=550e8400-e29b-41d4-a716-446655440200" \
  -H "Authorization: Bearer <access_token>"
```

## データエクスポート

### 13. 組織データエクスポート (POST /organizations/:organization_id/data-export)

組織データを指定形式でエクスポートします。

**リクエスト例:**
```bash
curl -X POST http://localhost:3000/organizations/550e8400-e29b-41d4-a716-446655440001/data-export \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "export_type": "hierarchy_structure",
    "format": "json",
    "include_members": true,
    "include_analytics": false
  }'
```

**レスポンス例 (200 OK):**
```json
{
  "export_id": "export-001",
  "status": "processing",
  "estimated_completion": "2025-06-29T10:30:00Z",
  "download_url": null
}
```

## 権限要件

### 必要な権限レベル

- **基本操作**: Enterprise サブスクリプション + 組織メンバー
- **部門管理**: 組織管理者またはEnterprise管理者
- **権限マトリックス**: 組織オーナーまたはシステム管理者
- **データエクスポート**: Enterprise管理者権限

### 部門役割

- **department_manager**: 部門管理、メンバー管理権限
- **department_member**: 部門参照権限
- **organization_admin**: 全部門管理権限

## エラーレスポンス例

### サブスクリプション制限 (402 Payment Required)
```json
{
  "error": "Enterprise subscription required for organization hierarchy features",
  "error_type": "subscription_required",
  "required_tier": "enterprise"
}
```

### 権限不足 (403 Forbidden)
```json
{
  "error": "Insufficient permissions to manage department structure",
  "error_type": "permission_denied",
  "required_permissions": ["department_manage"]
}
```

### 部門が見つからない (404 Not Found)
```json
{
  "error": "Department with id dept-999 not found",
  "error_type": "not_found"
}
```

## 使用例

### 完整な部門管理ワークフロー

```bash
# Enterprise管理者トークンを取得済みと仮定
ADMIN_TOKEN="enterprise_admin_token_here"
ORG_ID="550e8400-e29b-41d4-a716-446655440001"

# 1. 組織階層を確認
curl -s -X GET http://localhost:3000/organizations/$ORG_ID/hierarchy \
  -H "Authorization: Bearer $ADMIN_TOKEN"

# 2. 新しい部門を作成
DEPT_RESPONSE=$(curl -s -X POST http://localhost:3000/organizations/$ORG_ID/departments \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "AI研究部",
    "description": "AI技術の研究開発を行う部門",
    "parent_id": "dept-001"
  }')

DEPT_ID=$(echo $DEPT_RESPONSE | jq -r '.id')

# 3. 部門にメンバーを追加
curl -s -X POST http://localhost:3000/organizations/$ORG_ID/departments/$DEPT_ID/members \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "550e8400-e29b-41d4-a716-446655440300",
    "role": "department_manager"
  }'

# 4. 部門の分析データを確認
curl -s -X GET http://localhost:3000/organizations/$ORG_ID/analytics \
  -H "Authorization: Bearer $ADMIN_TOKEN"

# 5. 組織データをエクスポート
curl -s -X POST http://localhost:3000/organizations/$ORG_ID/data-export \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "export_type": "full_hierarchy",
    "format": "csv",
    "include_members": true
  }'
```