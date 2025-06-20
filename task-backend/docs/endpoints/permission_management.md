# 権限管理エンドポイント

動的パーミッションシステムの権限チェック、権限情報取得、機能アクセス管理などのAPIエンドポイント群です。

## 認証必要エンドポイント（一般ユーザー）

### 1. 特定の権限をチェック (POST /permissions/check)

指定したリソースとアクションに対する権限をチェックします。

**リクエスト例:**
```bash
curl -X POST http://localhost:3000/permissions/check \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "resource": "tasks",
    "action": "read",
    "context": {
      "target_user_id": "550e8400-e29b-41d4-a716-446655440001"
    }
  }'
```

**レスポンス例 (200 OK):**
```json
{
  "allowed": true,
  "permission_result": "Allowed",
  "scope": "Team",
  "privilege": {
    "name": "team_task_access",
    "subscription_tier": "Pro",
    "quota": {
      "max_items": 10000,
      "current_usage": 245
    },
    "features": [
      "advanced_filtering",
      "data_export",
      "team_collaboration"
    ]
  },
  "reason": "User has team-level access to tasks",
  "expires_at": null
}
```

**権限が拒否された場合 (200 OK):**
```json
{
  "allowed": false,
  "permission_result": "Denied",
  "scope": null,
  "privilege": null,
  "reason": "Insufficient subscription tier for team access",
  "required_tier": "Pro",
  "current_tier": "Free",
  "upgrade_url": "/subscriptions/upgrade"
}
```

### 2. 複数の権限を一括検証 (POST /permissions/validate)

複数のリソース・アクションの組み合わせを一度に検証します。

**リクエスト例:**
```bash
curl -X POST http://localhost:3000/permissions/validate \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "permissions": [
      {
        "resource": "tasks",
        "action": "read"
      },
      {
        "resource": "tasks",
        "action": "write"
      },
      {
        "resource": "teams",
        "action": "admin"
      },
      {
        "resource": "analytics",
        "action": "read"
      }
    ]
  }'
```

**レスポンス例 (200 OK):**
```json
{
  "results": [
    {
      "resource": "tasks",
      "action": "read",
      "allowed": true,
      "scope": "Own",
      "privilege": {
        "name": "basic_task_access",
        "subscription_tier": "Free"
      }
    },
    {
      "resource": "tasks",
      "action": "write",
      "allowed": true,
      "scope": "Own",
      "privilege": {
        "name": "basic_task_access",
        "subscription_tier": "Free"
      }
    },
    {
      "resource": "teams",
      "action": "admin",
      "allowed": false,
      "reason": "Team features require Pro subscription"
    },
    {
      "resource": "analytics",
      "action": "read",
      "allowed": false,
      "reason": "Analytics features require Pro subscription"
    }
  ],
  "summary": {
    "total_permissions": 4,
    "allowed_count": 2,
    "denied_count": 2,
    "overall_access_level": "Limited"
  }
}
```

### 3. ユーザーの権限情報を取得 (GET /permissions/user/{id})

指定したユーザーの権限情報を取得します（自分の情報または管理権限が必要）。

**リクエスト例:**
```bash
# 自分の権限情報を取得
curl -X GET http://localhost:3000/permissions/user/me \
  -H "Authorization: Bearer <access_token>"

# 特定ユーザーの権限情報を取得（管理者権限必要）
curl -X GET http://localhost:3000/permissions/user/550e8400-e29b-41d4-a716-446655440001 \
  -H "Authorization: Bearer <access_token>"
```

**レスポンス例 (200 OK):**
```json
{
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "subscription_tier": "Pro",
  "role": {
    "name": "user",
    "display_name": "一般ユーザー",
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
      }
    ]
  },
  "effective_permissions": {
    "tasks": {
      "scope": "Team",
      "actions": ["read", "write", "delete"]
    },
    "teams": {
      "scope": "Team",
      "actions": ["read", "write"]
    },
    "analytics": {
      "scope": "Own",
      "actions": ["read"]
    }
  },
  "current_quotas": {
    "tasks": {
      "limit": 10000,
      "used": 245,
      "remaining": 9755
    },
    "teams": {
      "limit": 3,
      "used": 1,
      "remaining": 2
    }
  },
  "features_enabled": [
    "advanced_filtering",
    "data_export",
    "team_collaboration",
    "priority_support"
  ]
}
```

### 4. 利用可能なリソース一覧を取得 (GET /permissions/resources)

システムで利用可能なリソースとアクションの一覧を取得します。

**リクエスト例:**
```bash
curl -X GET http://localhost:3000/permissions/resources \
  -H "Authorization: Bearer <access_token>"
```

**レスポンス例 (200 OK):**
```json
{
  "resources": [
    {
      "name": "tasks",
      "display_name": "タスク",
      "description": "タスクの作成、編集、削除",
      "actions": [
        {
          "name": "read",
          "display_name": "読み取り",
          "description": "タスクの閲覧"
        },
        {
          "name": "write",
          "display_name": "書き込み",
          "description": "タスクの作成・編集"
        },
        {
          "name": "delete",
          "display_name": "削除",
          "description": "タスクの削除"
        },
        {
          "name": "admin",
          "display_name": "管理",
          "description": "すべてのタスク操作"
        }
      ]
    },
    {
      "name": "teams",
      "display_name": "チーム",
      "description": "チームの作成、管理",
      "actions": [
        {
          "name": "read",
          "display_name": "読み取り",
          "description": "チーム情報の閲覧"
        },
        {
          "name": "write",
          "display_name": "書き込み",
          "description": "チームの作成・編集"
        },
        {
          "name": "admin",
          "display_name": "管理",
          "description": "チームの全管理権限"
        }
      ]
    },
    {
      "name": "analytics",
      "display_name": "アナリティクス",
      "description": "統計・分析データ",
      "actions": [
        {
          "name": "read",
          "display_name": "読み取り",
          "description": "分析データの閲覧"
        }
      ]
    }
  ]
}
```

### 5. リソース固有権限チェック (GET /permissions/resources/{resource}/actions/{action}/check)

特定のリソース・アクションの組み合わせに対する権限を直接チェックします。

**リクエスト例:**
```bash
curl -X GET "http://localhost:3000/permissions/resources/tasks/actions/admin/check?target_id=550e8400-e29b-41d4-a716-446655440001" \
  -H "Authorization: Bearer <access_token>"
```

### 6. バルク権限チェック (POST /permissions/bulk-check)

大量の権限チェックを効率的に実行します。

**リクエスト例:**
```bash
curl -X POST http://localhost:3000/permissions/bulk-check \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "checks": [
      {
        "id": "check_1",
        "resource": "tasks",
        "action": "read",
        "target_id": "task_123"
      },
      {
        "id": "check_2",
        "resource": "teams",
        "action": "admin",
        "target_id": "team_456"
      },
      {
        "id": "check_3",
        "resource": "analytics",
        "action": "read"
      }
    ]
  }'
```

### 7. ユーザー有効権限取得 (GET /permissions/user/{id}/effective)

ユーザーの実際に有効な権限（継承や動的計算を含む）を取得します。

**リクエスト例:**
```bash
curl -X GET http://localhost:3000/permissions/user/me/effective \
  -H "Authorization: Bearer <access_token>"
```

## 機能アクセス管理

### 8. 利用可能な機能を取得 (GET /features/available)

現在のユーザーが利用可能な機能一覧を取得します。

**リクエスト例:**
```bash
curl -X GET http://localhost:3000/features/available \
  -H "Authorization: Bearer <access_token>"
```

**レスポンス例 (200 OK):**
```json
{
  "subscription_tier": "Pro",
  "available_features": [
    {
      "name": "advanced_filtering",
      "display_name": "高度なフィルタリング",
      "description": "複雑な条件でのタスク検索",
      "enabled": true
    },
    {
      "name": "data_export",
      "display_name": "データエクスポート",
      "description": "タスクデータのCSV/PDF出力",
      "enabled": true
    },
    {
      "name": "team_collaboration",
      "display_name": "チーム協業",
      "description": "チーム機能全般",
      "enabled": true,
      "quota": {
        "max_teams": 3,
        "used_teams": 1
      }
    },
    {
      "name": "organization_management",
      "display_name": "組織管理",
      "description": "組織レベルの管理機能",
      "enabled": false,
      "required_tier": "Enterprise"
    }
  ],
  "feature_summary": {
    "total_features": 15,
    "enabled_features": 8,
    "disabled_features": 7
  }
}
```

### 9. 管理者機能アクセス情報を取得 (GET /features/admin)

管理者機能へのアクセス情報を取得します（管理者権限必要）。

**リクエスト例:**
```bash
curl -X GET http://localhost:3000/features/admin \
  -H "Authorization: Bearer <admin_access_token>"
```

### 10. アナリティクス機能アクセス情報を取得 (GET /features/analytics)

アナリティクス機能へのアクセス情報を取得します。

**リクエスト例:**
```bash
curl -X GET http://localhost:3000/features/analytics \
  -H "Authorization: Bearer <access_token>"
```

## 管理者専用エンドポイント

### 11. システム権限監査 (GET /admin/permissions/audit)

システムの権限設定と使用状況を監査します（管理者権限必要）。

**リクエスト例:**
```bash
curl -X GET http://localhost:3000/admin/permissions/audit \
  -H "Authorization: Bearer <admin_access_token>"
```

**レスポンス例 (200 OK):**
```json
{
  "audit_timestamp": "2025-06-12T15:30:00Z",
  "system_overview": {
    "total_users": 10000,
    "active_sessions": 1250,
    "permission_checks_last_24h": 50000
  },
  "subscription_breakdown": {
    "Free": 7500,
    "Pro": 2000,
    "Enterprise": 500
  },
  "role_distribution": {
    "user": 9800,
    "admin": 195,
    "owner": 5
  },
  "permission_violations": [
    {
      "user_id": "550e8400-e29b-41d4-a716-446655440123",
      "violation_type": "unauthorized_access_attempt",
      "resource": "admin/users",
      "timestamp": "2025-06-12T14:25:00Z"
    }
  ],
  "quota_usage": {
    "near_limit_users": 45,
    "over_limit_users": 3
  }
}
```

## パブリックエンドポイント

### 12. 権限サービスのヘルスチェック (GET /permissions/health)

権限システムの稼働状況を確認します。

**リクエスト例:**
```bash
curl -X GET http://localhost:3000/permissions/health
```

## 使用例

### 動的パーミッションの完整な活用例

```bash
# アクセストークンを取得済みと仮定
ACCESS_TOKEN="your_access_token_here"

# 1. 現在のユーザーの権限情報を確認
curl -s -X GET http://localhost:3000/permissions/user/me \
  -H "Authorization: Bearer $ACCESS_TOKEN"

# 2. 利用可能な機能を確認
curl -s -X GET http://localhost:3000/features/available \
  -H "Authorization: Bearer $ACCESS_TOKEN"

# 3. 特定のアクションが可能かチェック
curl -s -X POST http://localhost:3000/permissions/check \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "resource": "teams",
    "action": "admin"
  }'

# 4. 複数の権限を一括チェック
curl -s -X POST http://localhost:3000/permissions/validate \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "permissions": [
      {"resource": "tasks", "action": "read"},
      {"resource": "analytics", "action": "read"},
      {"resource": "teams", "action": "write"}
    ]
  }'

# 5. 権限に基づいて動的にタスクを取得
curl -s -X GET http://localhost:3000/tasks/dynamic \
  -H "Authorization: Bearer $ACCESS_TOKEN"
```

## エラーレスポンス例

### 権限不足 (403 Forbidden)
```json
{
  "error": "Insufficient permissions for this action",
  "error_type": "forbidden",
  "required_permission": {
    "resource": "teams",
    "action": "admin",
    "scope": "Team"
  },
  "current_permission": {
    "resource": "teams",
    "action": "read",
    "scope": "Own"
  }
}
```

### サブスクリプション制限 (402 Payment Required)
```json
{
  "error": "Feature requires higher subscription tier",
  "error_type": "subscription_required",
  "feature": "team_collaboration",
  "required_tier": "Pro",
  "current_tier": "Free",
  "upgrade_url": "/subscriptions/upgrade"
}
```

### クォータ超過 (429 Too Many Requests)
```json
{
  "error": "Resource quota exceeded",
  "error_type": "quota_exceeded",
  "resource": "tasks",
  "quota": {
    "limit": 100,
    "used": 100,
    "reset_date": "2025-07-01T00:00:00Z"
  }
}
```

## 注意事項

1. **動的計算**: 権限は要求時に動的に計算され、サブスクリプション階層と役割の両方を考慮します
2. **キャッシュ**: 権限チェック結果は短時間キャッシュされますが、重要な変更は即座に反映されます  
3. **監査ログ**: すべての権限チェックと違反は監査ログに記録されます
4. **リアルタイム更新**: サブスクリプション変更や役割変更は即座に権限システムに反映されます