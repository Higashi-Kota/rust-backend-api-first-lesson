# セキュリティ管理エンドポイント

システムセキュリティの監視と管理APIエンドポイント群です。管理者専用機能として、トークン管理、セッション監視、セキュリティ監査機能を提供します。

## 概要

セキュリティ管理機能は管理者専用で、システム全体のセキュリティ状況を監視し、潜在的な脅威に対処するための機能群です。

## トークン監視・管理

### 1. トークン統計取得 (GET /admin/security/token-stats)

システム全体のトークン使用統計を取得します。

**リクエスト例:**
```bash
curl -X GET http://localhost:3000/admin/security/token-stats \
  -H "Authorization: Bearer <admin_access_token>"
```

**レスポンス例 (200 OK):**
```json
{
  "active_access_tokens": 1250,
  "active_refresh_tokens": 850,
  "expired_tokens_last_24h": 320,
  "token_creation_rate": {
    "last_hour": 45,
    "last_24h": 890
  },
  "suspicious_activity": {
    "multiple_device_logins": 12,
    "unusual_token_requests": 3
  },
  "token_distribution": {
    "mobile_app": 600,
    "web_browser": 550,
    "api_client": 100
  }
}
```

### 2. リフレッシュトークン監視 (GET /admin/security/refresh-tokens)

リフレッシュトークンの詳細監視データを取得します。

**リクエスト例:**
```bash
curl -X GET "http://localhost:3000/admin/security/refresh-tokens?limit=50&include_suspicious=true" \
  -H "Authorization: Bearer <admin_access_token>"
```

**レスポンス例 (200 OK):**
```json
{
  "refresh_tokens": [
    {
      "token_id": "rt_550e8400e29b41d4a716446655440001",
      "user_id": "550e8400-e29b-41d4-a716-446655440100",
      "created_at": "2025-06-29T08:00:00Z",
      "last_used": "2025-06-29T09:45:00Z",
      "device_info": "Chrome 126.0 on Windows 10",
      "ip_address": "192.168.1.100",
      "is_suspicious": false,
      "usage_count": 15
    }
  ],
  "suspicious_tokens": [
    {
      "token_id": "rt_suspicious_001",
      "user_id": "550e8400-e29b-41d4-a716-446655440200",
      "flags": ["unusual_location", "high_frequency_usage"],
      "risk_score": 0.85
    }
  ],
  "total_count": 850,
  "suspicious_count": 8
}
```

### 3. トークンクリーンアップ実行 (POST /admin/security/cleanup-tokens)

期限切れ・不正なトークンのクリーンアップを実行します。

**リクエスト例:**
```bash
curl -X POST http://localhost:3000/admin/security/cleanup-tokens \
  -H "Authorization: Bearer <admin_access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "cleanup_expired": true,
    "cleanup_unused_days": 30,
    "revoke_suspicious": true,
    "dry_run": false
  }'
```

**レスポンス例 (200 OK):**
```json
{
  "cleanup_summary": {
    "expired_tokens_removed": 145,
    "unused_tokens_removed": 67,
    "suspicious_tokens_revoked": 8,
    "total_cleaned": 220
  },
  "execution_time": "1.2s",
  "next_cleanup_recommended": "2025-07-06T00:00:00Z"
}
```

## パスワードリセット監視

### 4. パスワードリセット監視 (GET /admin/security/password-resets)

パスワードリセット要求の監視データを取得します。

**リクエスト例:**
```bash
curl -X GET "http://localhost:3000/admin/security/password-resets?days=7&include_failed=true" \
  -H "Authorization: Bearer <admin_access_token>"
```

**レスポンス例 (200 OK):**
```json
{
  "reset_requests": [
    {
      "user_id": "550e8400-e29b-41d4-a716-446655440100",
      "email": "user@example.com",
      "requested_at": "2025-06-29T08:30:00Z",
      "completed_at": "2025-06-29T08:45:00Z",
      "ip_address": "192.168.1.100",
      "status": "completed",
      "attempts": 1
    }
  ],
  "statistics": {
    "total_requests": 45,
    "successful_resets": 42,
    "failed_attempts": 3,
    "average_completion_time": "12 minutes"
  },
  "suspicious_activity": {
    "high_frequency_requests": 2,
    "unusual_ip_patterns": 1
  }
}
```

## セッション・アクセス管理

### 5. 全トークン取り消し (POST /admin/security/revoke-all-tokens)

特定ユーザーまたは全ユーザーのトークンを取り消します。

**リクエスト例:**
```bash
# 特定ユーザーのトークンを全て取り消し
curl -X POST http://localhost:3000/admin/security/revoke-all-tokens \
  -H "Authorization: Bearer <admin_access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "550e8400-e29b-41d4-a716-446655440100",
    "reason": "security_incident",
    "notify_user": true
  }'

# 全ユーザーのトークンを取り消し（緊急時）
curl -X POST http://localhost:3000/admin/security/revoke-all-tokens \
  -H "Authorization: Bearer <admin_access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "revoke_all_users": true,
    "reason": "system_compromise",
    "emergency": true
  }'
```

### 6. セッション分析 (GET /admin/security/session-analytics)

ユーザーセッションの詳細分析データを取得します。

**リクエスト例:**
```bash
curl -X GET "http://localhost:3000/admin/security/session-analytics?period=7d&include_geo=true" \
  -H "Authorization: Bearer <admin_access_token>"
```

**レスポンス例 (200 OK):**
```json
{
  "session_metrics": {
    "total_sessions": 2800,
    "average_duration": "45 minutes",
    "concurrent_peaks": [
      {
        "timestamp": "2025-06-29T09:00:00Z",
        "concurrent_users": 185
      }
    ]
  },
  "geographic_distribution": [
    {
      "country": "Japan",
      "sessions": 1200,
      "percentage": 42.8
    },
    {
      "country": "United States", 
      "sessions": 800,
      "percentage": 28.6
    }
  ],
  "device_breakdown": {
    "desktop": 1680,
    "mobile": 980,
    "tablet": 140
  },
  "security_indicators": {
    "login_failures": 45,
    "suspicious_locations": 8,
    "brute_force_attempts": 2
  }
}
```

## セキュリティ監査・レポート

### 7. セキュリティ監査レポート生成 (POST /admin/security/audit-report)

包括的なセキュリティ監査レポートを生成します。

**リクエスト例:**
```bash
curl -X POST http://localhost:3000/admin/security/audit-report \
  -H "Authorization: Bearer <admin_access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "report_type": "comprehensive",
    "period_days": 30,
    "include_sections": [
      "authentication_events",
      "token_usage",
      "failed_attempts",
      "geographic_analysis",
      "recommendations"
    ],
    "format": "json",
    "email_report": true
  }'
```

**レスポンス例 (200 OK):**
```json
{
  "report_id": "audit_2025_06_29_001",
  "status": "generating",
  "estimated_completion": "2025-06-29T10:15:00Z",
  "sections": [
    {
      "section": "authentication_events",
      "total_events": 15420,
      "success_rate": 0.97,
      "anomalies_detected": 12
    },
    {
      "section": "token_usage",
      "active_tokens": 850,
      "token_abuse_detected": 3,
      "cleanup_recommended": true
    }
  ],
  "security_score": 0.89,
  "critical_issues": 0,
  "recommendations": [
    "Enable additional MFA for admin accounts",
    "Review tokens with unusual usage patterns",
    "Consider implementing geo-blocking for high-risk regions"
  ],
  "download_url": "https://api.taskbackend.com/admin/reports/audit_2025_06_29_001"
}
```

## セキュリティ設定・アクション

### セキュリティ設定の推奨事項

```bash
# 定期的なトークンクリーンアップの自動化設定
curl -X POST http://localhost:3000/admin/security/cleanup-tokens \
  -H "Authorization: Bearer <admin_access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "schedule": "daily",
    "cleanup_expired": true,
    "cleanup_unused_days": 7,
    "auto_revoke_suspicious": true
  }'
```

## 権限要件

### 必要な権限レベル

- **すべてのエンドポイント**: システム管理者権限が必要
- **トークン取り消し**: 高レベル管理者権限
- **監査レポート**: セキュリティ管理者権限

### セキュリティレベル

- **Level 1**: 統計・監視データの閲覧
- **Level 2**: トークンクリーンアップの実行
- **Level 3**: トークン取り消し・緊急対応
- **Level 4**: システム全体の監査レポート生成

## エラーレスポンス例

### 権限不足 (403 Forbidden)
```json
{
  "error": "Administrator access required for security management",
  "error_type": "insufficient_privileges",
  "required_role": "security_admin"
}
```

### 操作失敗 (500 Internal Server Error)
```json
{
  "error": "Token cleanup operation failed",
  "error_type": "operation_failed",
  "details": "Database connection timeout during cleanup"
}
```

## 使用例

### セキュリティ監視ワークフロー

```bash
# セキュリティ管理者トークンを取得済みと仮定
SECURITY_TOKEN="security_admin_token_here"

# 1. システム全体のトークン状況を確認
curl -s -X GET http://localhost:3000/admin/security/token-stats \
  -H "Authorization: Bearer $SECURITY_TOKEN"

# 2. 疑わしいリフレッシュトークンを調査
curl -s -X GET "http://localhost:3000/admin/security/refresh-tokens?include_suspicious=true" \
  -H "Authorization: Bearer $SECURITY_TOKEN"

# 3. パスワードリセットの異常を確認
curl -s -X GET "http://localhost:3000/admin/security/password-resets?days=1" \
  -H "Authorization: Bearer $SECURITY_TOKEN"

# 4. 必要に応じてトークンクリーンアップを実行
curl -s -X POST http://localhost:3000/admin/security/cleanup-tokens \
  -H "Authorization: Bearer $SECURITY_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "cleanup_expired": true,
    "revoke_suspicious": true,
    "dry_run": false
  }'

# 5. 週次セキュリティ監査レポート生成
curl -s -X POST http://localhost:3000/admin/security/audit-report \
  -H "Authorization: Bearer $SECURITY_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "report_type": "weekly",
    "period_days": 7,
    "email_report": true
  }'
```

## セキュリティベストプラクティス

### 推奨監視間隔

- **トークン統計**: 1時間毎
- **疑わしいアクティビティ**: リアルタイム
- **パスワードリセット**: 24時間毎
- **包括的監査**: 週1回

### アラート設定

```json
{
  "alert_thresholds": {
    "suspicious_tokens": 5,
    "failed_login_rate": 0.1,
    "unusual_geographic_access": 3,
    "brute_force_attempts": 1
  }
}
```