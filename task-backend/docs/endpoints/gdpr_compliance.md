# GDPR Compliance API

## Overview

GDPR (General Data Protection Regulation) compliance endpoints provide functionality for data subject rights, including data export, deletion, and consent management.

## Endpoints

### 1. Export User Data

Export all personal data for a specific user.

**Endpoint:** `POST /gdpr/users/{user_id}/export`

**Authentication:** Required (User can only export their own data)

**Request Body:**
```json
{
  "include_tasks": true,
  "include_teams": true,
  "include_activity_logs": true,
  "include_subscription_history": true
}
```

**Response:**
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

**Status Codes:**
- `200 OK`: Data exported successfully
- `401 Unauthorized`: Not authenticated
- `403 Forbidden`: Cannot export other user's data
- `404 Not Found`: User not found

### 2. Delete User Data

Permanently delete all user data (right to erasure).

**Endpoint:** `DELETE /gdpr/users/{user_id}/delete`

**Authentication:** Required (User can only delete their own data)

**Request Body:**
```json
{
  "confirm_deletion": true,
  "reason": "User requested account deletion"
}
```

**Response:**
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

**Status Codes:**
- `200 OK`: Data deleted successfully
- `400 Bad Request`: Deletion not confirmed
- `401 Unauthorized`: Not authenticated
- `403 Forbidden`: Cannot delete other user's data
- `404 Not Found`: User not found

### 3. Get Compliance Status

Check GDPR compliance status for a user.

**Endpoint:** `GET /gdpr/users/{user_id}/status`

**Authentication:** Required

**Response:**
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

### 4. Get User Consents

Retrieve all consent records for a user.

**Endpoint:** `GET /gdpr/users/{user_id}/consents`

**Authentication:** Required

**Response:**
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

### 5. Update User Consents

Update multiple consent preferences at once.

**Endpoint:** `POST /gdpr/users/{user_id}/consents`

**Authentication:** Required

**Request Body:**
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

**Response:**
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

### 6. Update Single Consent

Update a single consent preference.

**Endpoint:** `PATCH /gdpr/users/{user_id}/consents/single`

**Authentication:** Required

**Request Body:**
```json
{
  "consent_type": "marketing",
  "granted": false
}
```

**Response:**
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

### 7. Get Consent History

Retrieve the history of consent changes for a user.

**Endpoint:** `GET /gdpr/users/{user_id}/consents/history`

**Authentication:** Required

**Query Parameters:**
- `consent_type` (optional): Filter by consent type
- `from_date` (optional): Start date for history
- `to_date` (optional): End date for history

**Response:**
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

## Admin Endpoints

### 8. Admin Export User Data

Administrators can export any user's data.

**Endpoint:** `POST /admin/gdpr/users/{user_id}/export`

**Authentication:** Required (Admin only)

**Request Body:** Same as user export endpoint

**Response:** Same as user export endpoint

**Status Codes:**
- `200 OK`: Data exported successfully
- `401 Unauthorized`: Not authenticated
- `403 Forbidden`: Not an admin
- `404 Not Found`: User not found

### 9. Admin Delete User Data

Administrators can delete any user's data.

**Endpoint:** `DELETE /admin/gdpr/users/{user_id}/delete`

**Authentication:** Required (Admin only)

**Request Body:**
```json
{
  "confirm_deletion": true,
  "reason": "Administrative action - user violation",
  "notify_user": true
}
```

**Response:** Same as user delete endpoint

**Status Codes:**
- `200 OK`: Data deleted successfully
- `400 Bad Request`: Deletion not confirmed
- `401 Unauthorized`: Not authenticated
- `403 Forbidden`: Not an admin
- `404 Not Found`: User not found

## Consent Types

The system supports the following consent types:

1. **marketing**: Consent for marketing communications
2. **analytics**: Consent for analytics and performance tracking
3. **third_party**: Consent for sharing data with third parties

## Data Categories

When exporting data, the following categories are included:

1. **Personal Information**: Name, email, username
2. **Account Data**: Role, subscription, settings
3. **Usage Data**: Tasks, teams, activity logs
4. **Preferences**: User settings, UI preferences
5. **Communications**: Consent records, notifications

## Compliance Features

- **Data Minimization**: Only necessary data is collected
- **Purpose Limitation**: Data is used only for stated purposes
- **Storage Limitation**: Data retention policies are enforced
- **Accuracy**: Users can update their information
- **Security**: Data is encrypted and protected
- **Accountability**: All actions are logged for audit

## Error Responses

All endpoints follow the standard error response format:

```json
{
  "success": false,
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable error message",
    "details": {}
  }
}
```

Common error codes:
- `UNAUTHORIZED`: User not authenticated
- `FORBIDDEN`: Insufficient permissions
- `NOT_FOUND`: Resource not found
- `VALIDATION_ERROR`: Invalid request data
- `INTERNAL_ERROR`: Server error