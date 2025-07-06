# ユーザー管理エンドポイント

ユーザープロフィール管理、設定変更、統計情報などのAPIエンドポイント群です。

## 認証必要エンドポイント（一般ユーザー）

すべてのエンドポイントにはJWT認証が必要です。

### 1. ユーザープロフィール取得 (GET /users/profile)

現在のユーザーのプロフィール情報を取得します。

**リクエスト例:**
```bash
curl -X GET http://localhost:5000/users/profile \
  -H "Authorization: Bearer <access_token>"
```

**レスポンス例 (200 OK):**
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

### 2. ユーザー名更新 (PATCH /users/profile/username)

現在のユーザーのユーザー名を更新します。

**リクエスト例:**
```bash
curl -X PATCH http://localhost:5000/users/profile/username \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "username": "newusername"
  }'
```

### 3. メールアドレス更新 (PATCH /users/profile/email)

現在のユーザーのメールアドレスを更新します。

**リクエスト例:**
```bash
curl -X PATCH http://localhost:5000/users/profile/email \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "newemail@example.com"
  }'
```

### 4. プロフィール一括更新 (PATCH /users/profile)

ユーザープロフィールを一括で更新します。

**リクエスト例:**
```bash
curl -X PATCH http://localhost:5000/users/profile \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "display_name": "Updated Display Name",
    "bio": "Updated bio text",
    "avatar_url": "https://example.com/avatar.jpg"
  }'
```

### 5. ユーザー統計情報取得 (GET /users/stats)

現在のユーザーのアクティビティ統計を取得します。

**リクエスト例:**
```bash
curl -X GET http://localhost:5000/users/stats \
  -H "Authorization: Bearer <access_token>"
```

### 6. ユーザー設定取得 (GET /users/settings)

現在のユーザーの設定情報を取得します。

**リクエスト例:**
```bash
curl -X GET http://localhost:5000/users/settings \
  -H "Authorization: Bearer <access_token>"
```

### 7. メール認証実行 (POST /users/verify-email)

メールアドレスの認証を実行します。

**リクエスト例:**
```bash
curl -X POST http://localhost:5000/users/verify-email \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "verification_token": "token_from_email"
  }'
```

### 8. メール認証再送信 (POST /users/resend-verification)

メール認証用のメールを再送信します。

**リクエスト例:**
```bash
curl -X POST http://localhost:5000/users/resend-verification \
  -H "Authorization: Bearer <access_token>"
```

### 9. 最終ログイン時刻更新 (POST /users/update-last-login)

ユーザーの最終ログイン時刻を更新します。

**リクエスト例:**
```bash
curl -X POST http://localhost:5000/users/update-last-login \
  -H "Authorization: Bearer <access_token>"
```

## 管理者専用エンドポイント

以下のエンドポイントは管理者権限が必要です。

### 10. ユーザー一覧取得 (GET /admin/users)

システム内のすべてのユーザー一覧を取得します（管理者用）。

**リクエスト例:**
```bash
curl -X GET http://localhost:5000/admin/users \
  -H "Authorization: Bearer <admin_access_token>"
```

### 11. 特定ユーザー情報取得 (GET /admin/users/{id})

指定したユーザーの詳細情報を取得します（管理者用）。

**リクエスト例:**
```bash
curl -X GET http://localhost:5000/admin/users/550e8400-e29b-41d4-a716-446655440000 \
  -H "Authorization: Bearer <admin_access_token>"
```

### 12. 高度ユーザー検索 (GET /admin/users/advanced-search)

高度な検索条件でユーザーを検索します（管理者用）。

**リクエスト例:**
```bash
curl -X GET "http://localhost:5000/admin/users/advanced-search?subscription=pro&is_active=true&email_domain=example.com" \
  -H "Authorization: Bearer <admin_access_token>"
```

### 13. ユーザーアナリティクス (GET /admin/users/analytics)

ユーザーの分析データを取得します（管理者用）。

**リクエスト例:**
```bash
curl -X GET http://localhost:5000/admin/users/analytics \
  -H "Authorization: Bearer <admin_access_token>"
```

**レスポンス例 (200 OK):**
```json
{
  "total_users": 1250,
  "active_users": 1180,
  "subscription_breakdown": {
    "free": 800,
    "pro": 350,
    "enterprise": 100
  },
  "registration_trends": [
    {
      "date": "2025-06-29",
      "new_registrations": 15,
      "email_verifications": 12
    }
  ],
  "user_activity": {
    "daily_active_users": 450,
    "weekly_active_users": 820,
    "monthly_active_users": 1100
  }
}
```

### 14. ロール別ユーザー取得 (GET /admin/users/by-role/{role})

指定したロールのユーザー一覧を取得します（管理者用）。

**リクエスト例:**
```bash
curl -X GET http://localhost:5000/admin/users/by-role/admin \
  -H "Authorization: Bearer <admin_access_token>"
```

### 15. サブスクリプション別ユーザー取得 (GET /admin/users/by-subscription)

サブスクリプション階層別のユーザー一覧を取得します（管理者用）。

**リクエスト例:**
```bash
curl -X GET "http://localhost:5000/admin/users/by-subscription?tier=enterprise&include_stats=true" \
  -H "Authorization: Bearer <admin_access_token>"
```

### 16. 一括ユーザー操作 (POST /admin/users/bulk-operations)

複数ユーザーに対して一括操作を実行します（管理者用）。

**リクエスト例:**
```bash
curl -X POST http://localhost:5000/admin/users/bulk-operations \
  -H "Authorization: Bearer <admin_access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "operation": "update_subscription",
    "user_ids": ["user1", "user2", "user3"],
    "parameters": {
      "subscription_tier": "pro"
    }
  }'
```

### 17. アカウント状態更新 (PATCH /admin/users/{id}/status)

ユーザーアカウントの状態（アクティブ/非アクティブ）を更新します（管理者用）。

**リクエスト例:**
```bash
curl -X PATCH http://localhost:5000/admin/users/550e8400-e29b-41d4-a716-446655440000/status \
  -H "Authorization: Bearer <admin_access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "is_active": false,
    "reason": "Terms of service violation"
  }'
```

## パブリックエンドポイント

### 18. ヘルスチェック (GET /users/health)

ユーザーサービスの稼働状況を確認します。

**リクエスト例:**
```bash
curl -X GET http://localhost:5000/users/health
```

## エラーレスポンス例

### バリデーションエラー (400 Bad Request)
```json
{
  "errors": [
    "username: Username must be at least 3 characters long"
  ],
  "error_type": "validation_errors"
}
```

### 認証エラー (401 Unauthorized)
```json
{
  "error": "Missing authentication token",
  "error_type": "unauthorized"
}
```

### 権限エラー (403 Forbidden)
```json
{
  "error": "Admin access required",
  "error_type": "forbidden"
}
```

## 使用例

### プロフィール更新の完整な流れ

```bash
# 1. 現在のプロフィール確認
curl -X GET http://localhost:5000/users/profile \
  -H "Authorization: Bearer $ACCESS_TOKEN"

# 2. ユーザー名更新
curl -X PATCH http://localhost:5000/users/profile/username \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"username": "new_username"}'

# 3. プロフィール情報更新
curl -X PATCH http://localhost:5000/users/profile \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "display_name": "New Display Name",
    "bio": "Updated biography"
  }'

# 4. 更新後の確認
curl -X GET http://localhost:5000/users/profile \
  -H "Authorization: Bearer $ACCESS_TOKEN"
```