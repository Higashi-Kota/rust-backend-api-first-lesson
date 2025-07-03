# GDPR準拠API

## 概要

GDPR（一般データ保護規則）準拠エンドポイントは、データ主体の権利に関する機能を提供し、データエクスポート、削除、同意管理を含みます。

## エンドポイント

### 1. ユーザーデータのエクスポート

特定のユーザーの全個人データをエクスポートします。

**エンドポイント:** `POST /gdpr/users/{user_id}/export`

**認証:** 必須（ユーザーは自分のデータのみエクスポート可能）

**リクエストボディ:**
```json
{
  "include_tasks": true,
  "include_teams": true,
  "include_activity_logs": true,
  "include_subscription_history": true
}
```

**レスポンス:**
```json
{
  "success": true,
  "data": {
    "export_id": "550e8400-e29b-41d4-a716-446655440001",
    "user_data": {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "email": "user@example.com",
      "username": "johndoe",
      "role_name": "user",
      "subscription_tier": "pro",
      "created_at": "2024-01-01T00:00:00Z"
    },
    "tasks": [...],
    "teams": [...],
    "activity_logs": [...],
    "exported_at": "2024-06-20T10:00:00Z"
  }
}
```

**ステータスコード:**
- `200 OK`: データエクスポート成功
- `401 Unauthorized`: 認証されていません
- `403 Forbidden`: 他のユーザーのデータはエクスポートできません
- `404 Not Found`: ユーザーが見つかりません

### 2. ユーザーデータの削除

全ユーザーデータを完全に削除します（削除権）。

**エンドポイント:** `DELETE /gdpr/users/{user_id}/delete`

**認証:** 必須（ユーザーは自分のデータのみ削除可能）

**リクエストボディ:**
```json
{
  "confirm_deletion": true,
  "reason": "ユーザーがアカウント削除を要求"
}
```

**レスポンス:**
```json
{
  "success": true,
  "data": {
    "user_id": "550e8400-e29b-41d4-a716-446655440000",
    "deleted_at": "2024-06-20T10:00:00Z",
    "deleted_records": {
      "user_data": true,
      "tasks_count": 45,
      "teams_count": 3,
      "subscription_history_count": 5,
      "activity_logs_count": 1250,
      "refresh_tokens_count": 2
    }
  }
}
```

**ステータスコード:**
- `200 OK`: データ削除成功
- `400 Bad Request`: 削除が確認されていません
- `401 Unauthorized`: 認証されていません
- `403 Forbidden`: 他のユーザーのデータは削除できません
- `404 Not Found`: ユーザーが見つかりません

### 3. コンプライアンス状況の取得

ユーザーのGDPRコンプライアンス状況を確認します。

**エンドポイント:** `GET /gdpr/users/{user_id}/status`

**認証:** 必須

**レスポンス:**
```json
{
  "success": true,
  "data": {
    "user_id": "550e8400-e29b-41d4-a716-446655440000",
    "data_retention_days": 90,
    "deletion_requested": false,
    "deletion_scheduled_date": null,
    "consents": {
      "marketing": {
        "granted": true,
        "granted_at": "2024-01-15T10:00:00Z"
      },
      "analytics": {
        "granted": false,
        "revoked_at": "2024-03-01T14:30:00Z"
      },
      "third_party": {
        "granted": false,
        "granted_at": null
      }
    },
    "last_data_export": "2024-05-01T09:00:00Z",
    "data_categories": [
      "personal_information",
      "usage_data",
      "preferences",
      "communications"
    ]
  }
}
```

### 4. ユーザー同意の取得

ユーザーの全同意記録を取得します。

**エンドポイント:** `GET /gdpr/users/{user_id}/consents`

**認証:** 必須

**レスポンス:**
```json
{
  "success": true,
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440001",
      "consent_type": "marketing",
      "granted": true,
      "granted_at": "2024-01-15T10:00:00Z",
      "revoked_at": null,
      "ip_address": "192.168.1.1",
      "user_agent": "Mozilla/5.0..."
    },
    {
      "id": "550e8400-e29b-41d4-a716-446655440002",
      "consent_type": "analytics",
      "granted": false,
      "granted_at": "2024-01-15T10:00:00Z",
      "revoked_at": "2024-03-01T14:30:00Z",
      "ip_address": "192.168.1.1",
      "user_agent": "Mozilla/5.0..."
    }
  ]
}
```

### 5. ユーザー同意の更新

複数の同意設定を一度に更新します。

**エンドポイント:** `POST /gdpr/users/{user_id}/consents`

**認証:** 必須

**リクエストボディ:**
```json
{
  "consents": [
    {
      "consent_type": "marketing",
      "granted": true
    },
    {
      "consent_type": "analytics",
      "granted": false
    },
    {
      "consent_type": "third_party",
      "granted": true
    }
  ]
}
```

**レスポンス:**
```json
{
  "success": true,
  "data": {
    "updated_consents": [
      {
        "consent_type": "marketing",
        "granted": true,
        "updated_at": "2024-06-20T10:00:00Z"
      },
      {
        "consent_type": "analytics",
        "granted": false,
        "updated_at": "2024-06-20T10:00:00Z"
      },
      {
        "consent_type": "third_party",
        "granted": true,
        "updated_at": "2024-06-20T10:00:00Z"
      }
    ]
  }
}
```

### 6. 単一同意の更新

単一の同意設定を更新します。

**エンドポイント:** `PATCH /gdpr/users/{user_id}/consents/single`

**認証:** 必須

**リクエストボディ:**
```json
{
  "consent_type": "marketing",
  "granted": false
}
```

**レスポンス:**
```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440001",
    "consent_type": "marketing",
    "granted": false,
    "granted_at": null,
    "revoked_at": "2024-06-20T10:00:00Z"
  }
}
```

### 7. 同意履歴の取得

ユーザーの同意変更履歴を取得します。

**エンドポイント:** `GET /gdpr/users/{user_id}/consents/history`

**認証:** 必須

**クエリパラメータ:**
- `consent_type` (オプション): 同意タイプでフィルタ
- `from_date` (オプション): 履歴の開始日
- `to_date` (オプション): 履歴の終了日

**レスポンス:**
```json
{
  "success": true,
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440001",
      "consent_type": "marketing",
      "action": "granted",
      "timestamp": "2024-01-15T10:00:00Z",
      "ip_address": "192.168.1.1"
    },
    {
      "id": "550e8400-e29b-41d4-a716-446655440002",
      "consent_type": "marketing",
      "action": "revoked",
      "timestamp": "2024-06-20T10:00:00Z",
      "ip_address": "192.168.1.2"
    }
  ]
}
```

## 管理者エンドポイント

### 8. 管理者ユーザーデータエクスポート

管理者は任意のユーザーのデータをエクスポートできます。

**エンドポイント:** `POST /admin/gdpr/users/{user_id}/export`

**認証:** 必須（管理者のみ）

**リクエストボディ:** ユーザーエクスポートエンドポイントと同じ

**レスポンス:** ユーザーエクスポートエンドポイントと同じ

**ステータスコード:**
- `200 OK`: データエクスポート成功
- `401 Unauthorized`: 認証されていません
- `403 Forbidden`: 管理者ではありません
- `404 Not Found`: ユーザーが見つかりません

### 9. 管理者ユーザーデータ削除

管理者は任意のユーザーのデータを削除できます。

**エンドポイント:** `DELETE /admin/gdpr/users/{user_id}/delete`

**認証:** 必須（管理者のみ）

**リクエストボディ:**
```json
{
  "confirm_deletion": true,
  "reason": "管理措置 - ユーザー違反",
  "notify_user": true
}
```

**レスポンス:** ユーザー削除エンドポイントと同じ

**ステータスコード:**
- `200 OK`: データ削除成功
- `400 Bad Request`: 削除が確認されていません
- `401 Unauthorized`: 認証されていません
- `403 Forbidden`: 管理者ではありません
- `404 Not Found`: ユーザーが見つかりません

## 同意タイプ

システムは以下の同意タイプをサポートします：

1. **marketing**: マーケティング通信への同意
2. **analytics**: 分析およびパフォーマンス追跡への同意
3. **third_party**: 第三者とのデータ共有への同意

## データカテゴリ

データエクスポート時に、以下のカテゴリが含まれます：

1. **個人情報**: 氏名、メールアドレス、ユーザー名
2. **アカウントデータ**: 役割、サブスクリプション、設定
3. **使用データ**: タスク、チーム、活動ログ
4. **設定**: ユーザー設定、UI設定
5. **通信**: 同意記録、通知

## コンプライアンス機能

- **データ最小化**: 必要なデータのみを収集
- **目的制限**: データは述べられた目的のみに使用
- **保存制限**: データ保持ポリシーが実施
- **正確性**: ユーザーは情報を更新可能
- **セキュリティ**: データは暗号化され保護
- **説明責任**: 全アクションが監査のためログ記録

## エラーレスポンス

全エンドポイントは標準エラーレスポンス形式に従います：

```json
{
  "success": false,
  "error": {
    "code": "ERROR_CODE",
    "message": "人間が読めるエラーメッセージ",
    "details": {}
  }
}
```

一般的なエラーコード:
- `UNAUTHORIZED`: ユーザーが認証されていません
- `FORBIDDEN`: 権限が不十分です
- `NOT_FOUND`: リソースが見つかりません
- `VALIDATION_ERROR`: 無効なリクエストデータ
- `INTERNAL_ERROR`: サーバーエラー