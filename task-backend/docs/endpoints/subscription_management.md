# サブスクリプション管理エンドポイント

サブスクリプション階層の管理、アップグレード・ダウングレード、履歴管理などのAPIエンドポイント群です。Free/Pro/Enterpriseの階層システムを提供します。

## 認証必要エンドポイント（一般ユーザー）

### 1. 現在のサブスクリプション情報取得 (GET /subscriptions/current)

現在のユーザーのサブスクリプション情報を取得します。

**リクエスト例:**
```bash
curl -X GET http://localhost:5000/subscriptions/current \
  -H "Authorization: Bearer <access_token>"
```

**レスポンス例 (200 OK):**
```json
{
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "current_tier": "Pro",
  "tier_details": {
    "name": "Pro",
    "display_name": "プロプラン",
    "description": "個人・小規模チーム向けの高機能プラン",
    "monthly_price": 1980,
    "annual_price": 19800,
    "features": [
      "チーム機能（最大3チーム）",
      "高度なタスクフィルタリング",
      "データエクスポート機能",
      "優先サポート"
    ],
    "limits": {
      "max_tasks": 10000,
      "max_teams": 3,
      "max_team_members": 10,
      "storage_gb": 10
    }
  },
  "subscription_start_date": "2025-06-01T00:00:00Z",
  "next_billing_date": "2025-07-01T00:00:00Z",
  "is_trial": false,
  "trial_ends_at": null,
  "is_active": true,
  "auto_renew": true
}
```

### 2. サブスクリプションアップグレード (POST /subscriptions/upgrade)

現在のサブスクリプションを上位プランにアップグレードします。

**リクエスト例:**
```bash
# FreeからProへアップグレード
curl -X POST http://localhost:5000/subscriptions/upgrade \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "target_tier": "Pro",
    "billing_cycle": "monthly",
    "payment_method_id": "pm_1234567890"
  }'

# ProからEnterpriseへアップグレード
curl -X POST http://localhost:5000/subscriptions/upgrade \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "target_tier": "Enterprise",
    "billing_cycle": "annual",
    "payment_method_id": "pm_1234567890"
  }'
```

**レスポンス例 (200 OK):**
```json
{
  "message": "Subscription upgraded successfully",
  "previous_tier": "Free",
  "new_tier": "Pro",
  "billing_cycle": "monthly",
  "next_billing_date": "2025-07-12T00:00:00Z",
  "prorated_amount": 1320,
  "transaction_id": "txn_upgrade_1234567890"
}
```

### 3. サブスクリプションダウングレード (POST /subscriptions/downgrade)

現在のサブスクリプションを下位プランにダウングレードします。

**リクエスト例:**
```bash
curl -X POST http://localhost:5000/subscriptions/downgrade \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "target_tier": "Free",
    "effective_date": "2025-07-01T00:00:00Z",
    "reason": "cost_reduction"
  }'
```

**レスポンス例 (200 OK):**
```json
{
  "message": "Subscription downgrade scheduled successfully",
  "current_tier": "Pro",
  "target_tier": "Free",
  "effective_date": "2025-07-01T00:00:00Z",
  "refund_amount": 0,
  "data_retention_period": "2025-08-01T00:00:00Z",
  "features_lost": [
    "チーム機能",
    "高度なフィルタリング",
    "データエクスポート"
  ]
}
```

### 4. サブスクリプション履歴取得 (GET /subscriptions/history)

ユーザーのサブスクリプション変更履歴を取得します。

**リクエスト例:**
```bash
curl -X GET "http://localhost:5000/subscriptions/history?limit=10&offset=0" \
  -H "Authorization: Bearer <access_token>"
```

**レスポンス例 (200 OK):**
```json
{
  "history": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440001",
      "user_id": "550e8400-e29b-41d4-a716-446655440000",
      "from_tier": "Free",
      "to_tier": "Pro",
      "change_type": "upgrade",
      "billing_cycle": "monthly",
      "amount": 1980,
      "effective_date": "2025-06-01T00:00:00Z",
      "payment_method": "credit_card",
      "transaction_id": "txn_upgrade_1234567890",
      "reason": null,
      "created_at": "2025-06-01T00:00:00Z"
    },
    {
      "id": "550e8400-e29b-41d4-a716-446655440002",
      "user_id": "550e8400-e29b-41d4-a716-446655440000",
      "from_tier": null,
      "to_tier": "Free",
      "change_type": "initial",
      "billing_cycle": null,
      "amount": 0,
      "effective_date": "2025-05-15T00:00:00Z",
      "payment_method": null,
      "transaction_id": null,
      "reason": "account_creation",
      "created_at": "2025-05-15T00:00:00Z"
    }
  ],
  "pagination": {
    "current_page": 1,
    "page_size": 10,
    "total_items": 2,
    "total_pages": 1,
    "has_next_page": false,
    "has_previous_page": false
  }
}
```

## 管理者専用エンドポイント

### 5. 管理者用サブスクリプション統計取得 (GET /admin/subscriptions/stats)

システム全体のサブスクリプション統計を取得します（管理者用）。

**リクエスト例:**
```bash
curl -X GET http://localhost:5000/admin/subscriptions/stats \
  -H "Authorization: Bearer <admin_access_token>"
```

**レスポンス例 (200 OK):**
```json
{
  "total_users": 10000,
  "subscription_breakdown": {
    "Free": {
      "count": 7500,
      "percentage": 75.0
    },
    "Pro": {
      "count": 2000,
      "percentage": 20.0
    },
    "Enterprise": {
      "count": 500,
      "percentage": 5.0
    }
  },
  "monthly_revenue": {
    "Pro": 3960000,
    "Enterprise": 2500000,
    "total": 6460000
  },
  "recent_changes": {
    "upgrades_last_30_days": 150,
    "downgrades_last_30_days": 45,
    "new_subscriptions_last_30_days": 320
  },
  "churn_rate": {
    "monthly": 2.5,
    "annual": 15.8
  }
}
```

### 6. 管理者用ユーザーのサブスクリプション変更 (PATCH /admin/users/{id}/subscription)

管理者権限で特定ユーザーのサブスクリプションを変更します。

**リクエスト例:**
```bash
curl -X PATCH http://localhost:5000/admin/users/550e8400-e29b-41d4-a716-446655440000/subscription \
  -H "Authorization: Bearer <admin_access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "target_tier": "Enterprise",
    "billing_cycle": "annual",
    "effective_date": "2025-06-15T00:00:00Z",
    "reason": "customer_support_upgrade",
    "override_billing": true
  }'
```

**レスポンス例 (200 OK):**
```json
{
  "message": "User subscription updated successfully",
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "previous_tier": "Pro",
  "new_tier": "Enterprise",
  "effective_date": "2025-06-15T00:00:00Z",
  "admin_override": true,
  "reason": "customer_support_upgrade"
}
```

## サブスクリプション階層の詳細

### Free階層
- **月額料金**: ¥0
- **制限事項**:
  - 最大100タスク
  - チーム機能なし
  - 基本サポートのみ
  - エクスポート機能なし

### Pro階層
- **月額料金**: ¥1,980 / **年額料金**: ¥19,800
- **機能**:
  - 最大10,000タスク
  - チーム機能（最大3チーム、チーム当たり最大10名）
  - 高度なフィルタリング・検索
  - データエクスポート機能
  - 優先サポート
  - 10GB ストレージ

### Enterprise階層
- **月額料金**: ¥5,000 / **年額料金**: ¥50,000
- **機能**:
  - 無制限タスク
  - 無制限チーム・組織機能
  - 高度なアナリティクス
  - API統合
  - SSOサポート
  - 専用サポート
  - 100GB ストレージ

## 動的パーミッションとの連携

サブスクリプション階層は動的パーミッションシステムと密接に連携します：

```bash
# Free階層ユーザーの動的タスク取得（制限付き）
curl -X GET http://localhost:5000/tasks/dynamic \
  -H "Authorization: Bearer <free_user_token>"
# → 最大100件、基本機能のみ

# Pro階層ユーザーの動的タスク取得（拡張機能）
curl -X GET http://localhost:5000/tasks/dynamic \
  -H "Authorization: Bearer <pro_user_token>"
# → 最大10,000件、高度なフィルタリング可能

# Enterprise階層ユーザーの動的タスク取得（無制限）
curl -X GET http://localhost:5000/tasks/dynamic \
  -H "Authorization: Bearer <enterprise_user_token>"
# → 無制限、すべての機能利用可能
```

## 使用例

### サブスクリプション管理の完整な流れ

```bash
# アクセストークンを取得済みと仮定
ACCESS_TOKEN="your_access_token_here"

# 1. 現在のサブスクリプション状況を確認
curl -s -X GET http://localhost:5000/subscriptions/current \
  -H "Authorization: Bearer $ACCESS_TOKEN"

# 2. Proプランにアップグレード
UPGRADE_RESPONSE=$(curl -s -X POST http://localhost:5000/subscriptions/upgrade \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "target_tier": "Pro",
    "billing_cycle": "monthly",
    "payment_method_id": "pm_1234567890"
  }')

# 3. アップグレード後の状況確認
curl -s -X GET http://localhost:5000/subscriptions/current \
  -H "Authorization: Bearer $ACCESS_TOKEN"

# 4. サブスクリプション履歴を確認
curl -s -X GET http://localhost:5000/subscriptions/history \
  -H "Authorization: Bearer $ACCESS_TOKEN"

# 5. 動的パーミッション機能をテスト
curl -s -X GET http://localhost:5000/tasks/dynamic \
  -H "Authorization: Bearer $ACCESS_TOKEN"
```

## エラーレスポンス例

### 支払い情報エラー (402 Payment Required)
```json
{
  "error": "Payment method required for subscription upgrade",
  "error_type": "payment_required"
}
```

### 無効なサブスクリプション変更 (400 Bad Request)
```json
{
  "error": "Cannot downgrade from Pro to Enterprise",
  "error_type": "invalid_subscription_change"
}
```

### サブスクリプション制限 (403 Forbidden)
```json
{
  "error": "Feature requires Pro or Enterprise subscription",
  "error_type": "subscription_required",
  "required_tier": "Pro",
  "current_tier": "Free"
}
```

### 請求エラー (422 Unprocessable Entity)
```json
{
  "error": "Payment processing failed",
  "error_type": "billing_error",
  "details": {
    "payment_method": "Credit card expired",
    "retry_after": "2025-06-15T00:00:00Z"
  }
}
```

## 注意事項

1. **即座の適用**: アップグレードは即座に適用されますが、ダウングレードは次回請求日に適用されます
2. **日割り計算**: アップグレード時は日割り計算で課金されます
3. **データ保持**: ダウングレード時、制限を超えるデータは一定期間保持されますが、最終的に削除されます
4. **自動更新**: すべての有料プランは自動更新が設定されています
5. **クーリングオフ**: アップグレード後24時間以内のキャンセルは全額返金されます