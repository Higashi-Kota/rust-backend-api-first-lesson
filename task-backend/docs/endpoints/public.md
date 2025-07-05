# パブリックエンドポイント

認証不要でアクセス可能なAPIエンドポイント群です。システム情報の取得やヘルスチェックなどの基本的な機能を提供します。

## 認証不要エンドポイント

### 1. API基本情報 (GET /)

APIの基本情報とバージョン情報を取得します。

**リクエスト例:**
```bash
curl -X GET http://localhost:5000/
```

**レスポンス例 (200 OK):**
```json
{
  "name": "Task Backend API",
  "version": "1.0.0",
  "description": "Rust製タスク管理API - 動的パーミッションシステム搭載",
  "environment": "development",
  "server_time": "2025-06-12T15:30:00Z",
  "uptime_seconds": 86400,
  "features": [
    "JWT認証",
    "動的パーミッションシステム",
    "サブスクリプション階層管理",
    "チーム・組織機能",
    "リアルタイム分析"
  ],
  "api_documentation": {
    "endpoints_documentation": "/docs/endpoints/",
    "openapi_spec": "/api/docs/openapi.json",
    "postman_collection": "/api/docs/postman.json"
  },
  "support": {
    "email": "support@taskapi.com",
    "documentation": "https://docs.taskapi.com",
    "status_page": "https://status.taskapi.com"
  }
}
```

### 2. システムヘルスチェック (GET /health)

システム全体の稼働状況を確認します。

**リクエスト例:**
```bash
curl -X GET http://localhost:5000/health
```

**レスポンス例 (200 OK):**
```json
{
  "status": "healthy",
  "timestamp": "2025-06-12T15:30:00Z",
  "version": "1.0.0",
  "environment": "development",
  "services": {
    "database": {
      "status": "healthy",
      "response_time_ms": 12,
      "connection_pool": {
        "active": 5,
        "idle": 10,
        "max": 20
      }
    },
    "redis": {
      "status": "healthy",
      "response_time_ms": 3,
      "memory_usage_mb": 45
    },
    "email_service": {
      "status": "healthy",
      "response_time_ms": 150,
      "queue_size": 3
    }
  },
  "metrics": {
    "uptime_seconds": 86400,
    "memory_usage_mb": 256,
    "cpu_usage_percent": 15.2,
    "active_connections": 47,
    "requests_per_minute": 125
  }
}
```

**異常時のレスポンス例 (503 Service Unavailable):**
```json
{
  "status": "degraded",
  "timestamp": "2025-06-12T15:30:00Z",
  "version": "1.0.0",
  "environment": "development",
  "services": {
    "database": {
      "status": "healthy",
      "response_time_ms": 12
    },
    "redis": {
      "status": "unhealthy",
      "error": "Connection timeout",
      "last_successful_check": "2025-06-12T15:25:00Z"
    },
    "email_service": {
      "status": "degraded",
      "response_time_ms": 5000,
      "warning": "High response time detected"
    }
  },
  "issues": [
    {
      "service": "redis",
      "severity": "high",
      "message": "Redis connection unavailable"
    },
    {
      "service": "email_service", 
      "severity": "medium",
      "message": "Email service response time exceeds threshold"
    }
  ]
}
```

### 3. 認証ステータス確認 (GET /auth/status)

認証システムの状態を確認します。

**リクエスト例:**
```bash
curl -X GET http://localhost:5000/auth/status
```

**レスポンス例 (200 OK):**
```json
{
  "authentication_service": "operational",
  "jwt_service": "healthy",
  "password_reset_service": "operational",
  "email_verification_service": "operational",
  "session_store": "healthy",
  "supported_features": [
    "jwt_authentication",
    "refresh_tokens",
    "password_reset",
    "email_verification",
    "multi_device_sessions"
  ],
  "rate_limits": {
    "login_attempts": {
      "max_attempts": 5,
      "window_minutes": 15
    },
    "password_reset": {
      "max_requests": 3,
      "window_hours": 1
    },
    "registration": {
      "max_registrations": 10,
      "window_hours": 1
    }
  },
  "security_features": {
    "argon2_hashing": true,
    "automatic_password_rehashing": true,
    "secure_session_management": true,
    "brute_force_protection": true
  }
}
```

### 4. ユーザーサービスのヘルスチェック (GET /users/health)

ユーザー管理サービスの稼働状況を確認します。

**リクエスト例:**
```bash
curl -X GET http://localhost:5000/users/health
```

**レスポンス例 (200 OK):**
```json
{
  "service": "user_service",
  "status": "healthy",
  "timestamp": "2025-06-12T15:30:00Z",
  "database_connection": "healthy",
  "email_service": "operational",
  "profile_service": "healthy",
  "statistics": {
    "total_users": 10000,
    "active_users_last_24h": 1250,
    "new_registrations_last_24h": 45,
    "email_verifications_pending": 123
  },
  "features_status": {
    "user_registration": "enabled",
    "email_verification": "enabled",
    "profile_updates": "enabled",
    "admin_operations": "enabled"
  }
}
```

### 5. 権限サービスのヘルスチェック (GET /permissions/health)

権限管理システムの稼働状況を確認します。

**リクエスト例:**
```bash
curl -X GET http://localhost:5000/permissions/health
```

**レスポンス例 (200 OK):**
```json
{
  "service": "permission_service",
  "status": "healthy",
  "timestamp": "2025-06-12T15:30:00Z",
  "dynamic_permission_engine": "operational",
  "subscription_tier_integration": "healthy",
  "role_based_access_control": "operational",
  "statistics": {
    "permission_checks_last_hour": 5420,
    "average_check_time_ms": 15,
    "cache_hit_rate": 85.6,
    "active_user_sessions": 1250
  },
  "features_status": {
    "dynamic_permissions": "enabled",
    "subscription_based_limits": "enabled",
    "role_inheritance": "enabled",
    "permission_caching": "enabled",
    "audit_logging": "enabled"
  },
  "subscription_tiers": {
    "Free": "operational",
    "Pro": "operational", 
    "Enterprise": "operational"
  }
}
```

## システム情報エンドポイント

### 6. API仕様情報 (GET /api/info)

API の技術仕様と制限事項の情報を取得します。

**リクエスト例:**
```bash
curl -X GET http://localhost:5000/api/info
```

**レスポンス例 (200 OK):**
```json
{
  "api_version": "1.0.0",
  "framework": "Axum",
  "language": "Rust",
  "database": "PostgreSQL",
  "cache": "Redis",
  "authentication": "JWT",
  "rate_limits": {
    "authenticated_requests": {
      "requests_per_minute": 1000,
      "burst_limit": 1500
    },
    "unauthenticated_requests": {
      "requests_per_minute": 100,
      "burst_limit": 200
    }
  },
  "request_limits": {
    "max_request_size_mb": 10,
    "max_response_size_mb": 50,
    "timeout_seconds": 30
  },
  "supported_formats": [
    "application/json",
    "application/x-www-form-urlencoded"
  ],
  "supported_methods": [
    "GET", "POST", "PATCH", "DELETE"
  ],
  "cors_enabled": true,
  "https_required": false,
  "api_documentation": {
    "swagger_ui": "/docs",
    "openapi_spec": "/api/docs/openapi.json"
  }
}
```

## 利用状況・統計情報

### 7. パブリック統計 (GET /public/stats)

一般公開可能なシステム統計情報を取得します。

**リクエスト例:**
```bash
curl -X GET http://localhost:5000/public/stats
```

**レスポンス例 (200 OK):**
```json
{
  "system_overview": {
    "total_registered_users": 10000,
    "total_tasks_created": 250000,
    "total_teams": 1250,
    "total_organizations": 150,
    "uptime_percentage_30d": 99.95
  },
  "feature_adoption": {
    "users_with_teams": 2500,
    "users_with_pro_subscription": 2000,
    "users_with_enterprise_subscription": 500,
    "api_integrations_active": 350
  },
  "performance_metrics": {
    "average_api_response_time_ms": 245,
    "api_success_rate_percentage": 99.7,
    "peak_concurrent_users": 2500
  },
  "growth_indicators": {
    "new_users_last_30d": 1200,
    "tasks_created_last_30d": 45000,
    "subscription_upgrades_last_30d": 150
  },
  "last_updated": "2025-06-12T15:30:00Z",
  "data_freshness_minutes": 60
}
```

## 使用例

### システム監視ダッシュボード用のデータ収集

```bash
# 1. API基本情報を取得
curl -s -X GET http://localhost:5000/ | jq '.version, .uptime_seconds'

# 2. システム全体のヘルスチェック
HEALTH_STATUS=$(curl -s -X GET http://localhost:5000/health)
echo $HEALTH_STATUS | jq '.status'

# 3. 各サービスの個別ヘルスチェック
curl -s -X GET http://localhost:5000/auth/status | jq '.authentication_service'
curl -s -X GET http://localhost:5000/users/health | jq '.status'
curl -s -X GET http://localhost:5000/permissions/health | jq '.status'

# 4. パブリック統計情報を取得
curl -s -X GET http://localhost:5000/public/stats | jq '.system_overview'

# 5. API制限情報を確認
curl -s -X GET http://localhost:5000/api/info | jq '.rate_limits'
```

### システム状態の監視スクリプト例

```bash
#!/bin/bash

# ヘルスチェック関数
check_health() {
    local service_name=$1
    local endpoint=$2
    
    response=$(curl -s -w "%{http_code}" -o /tmp/health_response $endpoint)
    http_code=${response: -3}
    
    if [ "$http_code" == "200" ]; then
        status=$(cat /tmp/health_response | jq -r '.status // "unknown"')
        echo "✅ $service_name: $status"
    else
        echo "❌ $service_name: HTTP $http_code"
    fi
    
    rm -f /tmp/health_response
}

echo "=== System Health Check ==="
check_health "Main API" "http://localhost:5000/health"
check_health "Authentication" "http://localhost:5000/auth/status"
check_health "User Service" "http://localhost:5000/users/health"
check_health "Permission Service" "http://localhost:5000/permissions/health"

echo ""
echo "=== System Statistics ==="
curl -s http://localhost:5000/public/stats | jq '{
  total_users: .system_overview.total_registered_users,
  uptime: .system_overview.uptime_percentage_30d,
  response_time: .performance_metrics.average_api_response_time_ms
}'
```

## エラーレスポンス例

### サービス利用不可 (503 Service Unavailable)
```json
{
  "status": "unavailable",
  "timestamp": "2025-06-12T15:30:00Z",
  "error": "Database connection failed",
  "retry_after_seconds": 30,
  "estimated_recovery": "2025-06-12T16:00:00Z"
}
```

### レート制限 (429 Too Many Requests)
```json
{
  "error": "Rate limit exceeded",
  "error_type": "rate_limit_exceeded",
  "retry_after_seconds": 60,
  "limit": 100,
  "remaining": 0,
  "reset_time": "2025-06-12T15:31:00Z"
}
```

## 注意事項

1. **キャッシュ**: パブリック統計は1時間ごとに更新されます
2. **レート制限**: 認証なしのエンドポイントには厳しいレート制限があります  
3. **監視**: これらのエンドポイントは外部監視システムからの定期的なチェックに適しています
4. **セキュリティ**: 個人情報は一切含まれず、集計データのみが提供されます