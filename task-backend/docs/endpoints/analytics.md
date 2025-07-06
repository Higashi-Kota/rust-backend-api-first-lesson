# アナリティクス・統計エンドポイント

システムの利用統計、ユーザー行動分析、パフォーマンス指標などのAPIエンドポイント群です。サブスクリプション階層に応じて利用可能な機能が異なります。

## 認証必要エンドポイント（一般ユーザー）

### 1. ユーザー個人のアクティビティ統計を取得 (GET /analytics/activity)

現在のユーザーの個人的なアクティビティ統計を取得します。

**リクエスト例:**
```bash
curl -X GET "http://localhost:5000/analytics/activity?period=30d&timezone=Asia/Tokyo" \
  -H "Authorization: Bearer <access_token>"
```

**レスポンス例 (200 OK):**
```json
{
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "period": "30d",
  "timezone": "Asia/Tokyo",
  "summary": {
    "total_tasks_created": 45,
    "total_tasks_completed": 38,
    "completion_rate": 84.4,
    "average_completion_time_hours": 18.5,
    "most_active_day": "Tuesday",
    "most_active_hour": 14
  },
  "daily_activity": [
    {
      "date": "2025-06-12",
      "tasks_created": 3,
      "tasks_completed": 2,
      "time_spent_minutes": 120
    },
    {
      "date": "2025-06-11", 
      "tasks_created": 1,
      "tasks_completed": 4,
      "time_spent_minutes": 95
    }
  ],
  "productivity_trends": {
    "weekly_average": 8.5,
    "trend": "increasing",
    "improvement_percentage": 12.3
  },
  "task_status_breakdown": {
    "todo": 7,
    "in_progress": 5,
    "completed": 38,
    "cancelled": 2
  }
}
```

### 2. タスク統計詳細を取得 (GET /analytics/tasks)

ユーザーのタスクに関する詳細な統計データを取得します。

**リクエスト例:**
```bash
curl -X GET "http://localhost:5000/analytics/tasks?group_by=status&include_trends=true" \
  -H "Authorization: Bearer <access_token>"
```

**レスポンス例 (200 OK):**
```json
{
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "total_tasks": 52,
  "status_distribution": {
    "todo": {
      "count": 7,
      "percentage": 13.5,
      "average_age_days": 3.2
    },
    "in_progress": {
      "count": 5,
      "percentage": 9.6,
      "average_age_days": 1.8
    },
    "completed": {
      "count": 38,
      "percentage": 73.1,
      "average_completion_time_hours": 18.5
    },
    "cancelled": {
      "count": 2,
      "percentage": 3.8,
      "average_age_days": 12.5
    }
  },
  "priority_analysis": {
    "high_priority": {
      "count": 8,
      "completion_rate": 87.5
    },
    "medium_priority": {
      "count": 25,
      "completion_rate": 84.0
    },
    "low_priority": {
      "count": 19,
      "completion_rate": 78.9
    }
  },
  "monthly_trends": [
    {
      "month": "2025-06",
      "created": 15,
      "completed": 12,
      "completion_rate": 80.0
    },
    {
      "month": "2025-05",
      "created": 22,
      "completed": 19,
      "completion_rate": 86.4
    }
  ],
  "performance_metrics": {
    "average_tasks_per_week": 8.5,
    "peak_productivity_hour": "14:00",
    "most_productive_day": "Tuesday",
    "estimated_weekly_capacity": 12
  }
}
```

### 3. ユーザー行動分析を取得 (GET /analytics/behavior)

ユーザーの行動パターンと利用傾向を分析します（Pro階層以上）。

**リクエスト例:**
```bash
curl -X GET "http://localhost:5000/analytics/behavior?analyze_patterns=true" \
  -H "Authorization: Bearer <pro_access_token>"
```

**レスポンス例 (200 OK):**
```json
{
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "subscription_tier": "Pro",
  "behavior_analysis": {
    "login_patterns": {
      "most_active_hours": ["09:00", "14:00", "16:00"],
      "most_active_days": ["Monday", "Tuesday", "Wednesday"],
      "average_session_duration_minutes": 45,
      "sessions_per_week": 12
    },
    "task_creation_patterns": {
      "preferred_creation_time": "09:30",
      "batch_creation_tendency": "low",
      "planning_vs_immediate_ratio": 0.7,
      "average_description_length": 85
    },
    "completion_patterns": {
      "preferred_completion_time": "16:30",
      "completion_streak_record": 7,
      "completion_consistency_score": 8.2,
      "procrastination_index": 2.1
    },
    "feature_usage": {
      "most_used_features": [
        "task_filtering",
        "due_date_setting",
        "status_updates"
      ],
      "least_used_features": [
        "task_sharing",
        "advanced_search"
      ],
      "feature_adoption_rate": 65.5
    }
  },
  "personalized_insights": [
    {
      "type": "productivity_tip",
      "message": "午後2時台の生産性が最も高いため、重要なタスクをこの時間に集中させることをお勧めします。"
    },
    {
      "type": "improvement_suggestion",
      "message": "火曜日の完了率が他の曜日より10%高いパターンが見られます。"
    }
  ]
}
```

## 管理者専用エンドポイント

### 4. システム全体の統計を取得 (GET /admin/analytics/system)

システム全体の利用統計と健全性指標を取得します（管理者権限必要）。

**リクエスト例:**
```bash
curl -X GET "http://localhost:5000/admin/analytics/system?include_details=true" \
  -H "Authorization: Bearer <admin_access_token>"
```

**レスポンス例 (200 OK):**
```json
{
  "system_overview": {
    "total_users": 10000,
    "active_users_last_30d": 7500,
    "new_users_last_30d": 1200,
    "total_tasks": 250000,
    "tasks_created_last_30d": 45000,
    "tasks_completed_last_30d": 38000
  },
  "subscription_analytics": {
    "conversion_rates": {
      "free_to_pro": 12.5,
      "pro_to_enterprise": 8.3,
      "overall_paid_conversion": 25.0
    },
    "revenue_metrics": {
      "monthly_recurring_revenue": 6460000,
      "average_revenue_per_user": 2584,
      "lifetime_value": 85600
    },
    "churn_analysis": {
      "monthly_churn_rate": 2.5,
      "annual_churn_rate": 15.8,
      "churn_by_tier": {
        "Pro": 2.1,
        "Enterprise": 1.2
      }
    }
  },
  "feature_usage_stats": {
    "most_popular_features": [
      {
        "feature": "task_creation",
        "usage_percentage": 98.5
      },
      {
        "feature": "task_filtering",
        "usage_percentage": 76.3
      },
      {
        "feature": "team_collaboration",
        "usage_percentage": 45.2
      }
    ],
    "underutilized_features": [
      {
        "feature": "advanced_analytics",
        "usage_percentage": 12.1
      },
      {
        "feature": "api_integration",
        "usage_percentage": 8.7
      }
    ]
  },
  "performance_metrics": {
    "average_response_time_ms": 245,
    "api_success_rate": 99.7,
    "peak_concurrent_users": 2500,
    "system_uptime_percentage": 99.95
  },
  "growth_metrics": {
    "user_growth_rate_monthly": 8.5,
    "revenue_growth_rate_monthly": 12.3,
    "feature_adoption_rate": 65.5,
    "customer_satisfaction_score": 4.2
  }
}
```

### 5. 管理者用ユーザーアクティビティ統計を取得 (GET /admin/analytics/users/{id}/activity)

特定ユーザーの詳細なアクティビティ統計を取得します（管理者権限必要）。

**リクエスト例:**
```bash
curl -X GET "http://localhost:5000/admin/analytics/users/550e8400-e29b-41d4-a716-446655440000/activity?period=90d" \
  -H "Authorization: Bearer <admin_access_token>"
```

### 6. バルクユーザー操作を実行 (POST /admin/users/bulk-operations)

管理者権限で複数ユーザーに対する一括操作を実行します。

**リクエスト例:**
```bash
curl -X POST http://localhost:5000/admin/users/bulk-operations \
  -H "Authorization: Bearer <admin_access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "operation": "export_activity",
    "user_filters": {
      "subscription_tier": ["Pro", "Enterprise"],
      "last_login_after": "2025-05-01T00:00:00Z",
      "task_count_min": 10
    },
    "export_format": "csv",
    "include_fields": [
      "user_id", "username", "email", "subscription_tier",
      "task_count", "last_login", "completion_rate"
    ]
  }'
```

## 高度なエクスポート機能

### 7. 高度なエクスポートを実行 (POST /exports/advanced)

詳細なデータエクスポートを実行します（Pro階層以上）。

**リクエスト例:**
```bash
curl -X POST http://localhost:5000/exports/advanced \
  -H "Authorization: Bearer <pro_access_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "export_type": "comprehensive_analytics",
    "format": "pdf",
    "data_range": {
      "start_date": "2025-05-01T00:00:00Z",
      "end_date": "2025-06-12T23:59:59Z"
    },
    "include_sections": [
      "productivity_summary",
      "task_analytics", 
      "behavior_patterns",
      "goal_tracking",
      "recommendations"
    ],
    "visualization_options": {
      "include_charts": true,
      "chart_types": ["line", "bar", "pie"],
      "color_scheme": "professional"
    },
    "delivery_method": "email"
  }'
```

**レスポンス例 (202 Accepted):**
```json
{
  "export_id": "exp_550e8400-e29b-41d4-a716-446655440001",
  "status": "processing",
  "estimated_completion": "2025-06-12T16:05:00Z",
  "download_url": null,
  "message": "Export request has been queued for processing. You will receive an email when ready."
}
```

## サブスクリプション階層別機能

### Free階層
- 基本的な個人統計のみ
- 直近30日間のデータ
- 簡単なグラフ表示

### Pro階層  
- 詳細な行動分析
- 90日間のデータ履歴
- PDFエクスポート機能
- カスタムレポート作成

### Enterprise階層
- 無制限の履歴データ
- 高度な予測分析
- カスタムダッシュボード
- API統合による外部分析ツール連携

## 使用例

### 包括的な分析レポート作成

```bash
# Pro階層ユーザーのアクセストークンを使用
PRO_TOKEN="your_pro_access_token_here"

# 1. 個人のアクティビティ統計を確認
curl -s -X GET "http://localhost:5000/analytics/activity?period=30d" \
  -H "Authorization: Bearer $PRO_TOKEN"

# 2. 詳細なタスク統計を取得
curl -s -X GET "http://localhost:5000/analytics/tasks?group_by=status&include_trends=true" \
  -H "Authorization: Bearer $PRO_TOKEN"

# 3. 行動パターン分析を実行
curl -s -X GET "http://localhost:5000/analytics/behavior?analyze_patterns=true" \
  -H "Authorization: Bearer $PRO_TOKEN"

# 4. 包括的なレポートをPDFでエクスポート
EXPORT_RESPONSE=$(curl -s -X POST http://localhost:5000/exports/advanced \
  -H "Authorization: Bearer $PRO_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "export_type": "comprehensive_analytics",
    "format": "pdf",
    "data_range": {
      "start_date": "2025-05-01T00:00:00Z",
      "end_date": "2025-06-12T23:59:59Z"
    },
    "include_sections": [
      "productivity_summary",
      "task_analytics",
      "behavior_patterns"
    ]
  }')

EXPORT_ID=$(echo $EXPORT_RESPONSE | jq -r '.export_id')
echo "Export ID: $EXPORT_ID"
```

## エラーレスポンス例

### サブスクリプション制限 (402 Payment Required)
```json
{
  "error": "Advanced analytics require Pro subscription",
  "error_type": "subscription_required",
  "feature": "behavior_analysis",
  "required_tier": "Pro",
  "current_tier": "Free"
}
```

### データ範囲エラー (400 Bad Request)
```json
{
  "error": "Data range exceeds subscription limits",
  "error_type": "data_range_exceeded",
  "max_range_days": 30,
  "requested_range_days": 90,
  "required_tier": "Pro"
}
```

### エクスポート容量制限 (429 Too Many Requests)
```json
{
  "error": "Export quota exceeded for this month",
  "error_type": "export_quota_exceeded",
  "quota": {
    "monthly_limit": 5,
    "used": 5,
    "reset_date": "2025-07-01T00:00:00Z"
  }
}
```

## 注意事項

1. **データ保持期間**: サブスクリプション階層により利用可能なデータ履歴期間が異なります
2. **リアルタイム更新**: 統計データは1時間ごとに更新されます
3. **エクスポート制限**: Pro階層は月5回、Enterprise階層は無制限のエクスポートが可能です
4. **プライバシー**: 個人データは暗号化され、適切な権限チェックが行われます