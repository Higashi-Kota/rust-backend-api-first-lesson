// task-backend/src/features/security/dto/requests/permission.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::core::subscription_tier::SubscriptionTier;

/// 権限チェックリクエスト
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CheckPermissionRequest {
    #[validate(length(
        min = 1,
        max = 50,
        message = "Resource must be between 1 and 50 characters"
    ))]
    pub resource: String,

    #[validate(length(
        min = 1,
        max = 50,
        message = "Action must be between 1 and 50 characters"
    ))]
    pub action: String,

    pub target_user_id: Option<Uuid>,
    pub context: Option<PermissionContext>,
}

/// 権限検証リクエスト
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct ValidatePermissionRequest {
    pub permissions: Vec<PermissionCheck>,
    pub require_all: Option<bool>, // true: AND logic, false: OR logic
}

/// 個別権限チェック
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct PermissionCheck {
    #[validate(length(
        min = 1,
        max = 50,
        message = "Resource must be between 1 and 50 characters"
    ))]
    pub resource: String,

    #[validate(length(
        min = 1,
        max = 50,
        message = "Action must be between 1 and 50 characters"
    ))]
    pub action: String,

    pub target_user_id: Option<Uuid>,
}

/// 権限コンテキスト
#[derive(Debug, Serialize, Deserialize)]
pub struct PermissionContext {
    pub organization_id: Option<Uuid>,
    pub team_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub additional_context: Option<serde_json::Value>,
}

/// 機能アクセスチェックリクエスト
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct FeatureAccessRequest {
    #[validate(length(
        min = 1,
        max = 100,
        message = "Feature name must be between 1 and 100 characters"
    ))]
    pub feature_name: String,

    pub required_tier: Option<SubscriptionTier>,
    pub context: Option<PermissionContext>,
}

/// リソース固有権限チェックリクエスト
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct ResourcePermissionRequest {
    #[validate(length(
        min = 1,
        max = 50,
        message = "Resource must be between 1 and 50 characters"
    ))]
    pub resource: String,

    #[validate(length(
        min = 1,
        max = 50,
        message = "Action must be between 1 and 50 characters"
    ))]
    pub action: String,

    pub target_user_id: Option<Uuid>,
    pub context: Option<PermissionContext>,
}

/// バルク権限チェックリクエスト
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct BulkPermissionCheckRequest {
    pub checks: Vec<PermissionCheck>,
    pub user_id: Option<Uuid>, // 対象ユーザー（省略時は実行者）
}

/// ユーザー有効権限取得リクエスト
#[derive(Debug, Serialize, Deserialize)]
pub struct UserEffectivePermissionsQuery {
    pub include_inherited: Option<bool>,
    pub resource_filter: Option<String>,
}

/// システム権限監査リクエスト
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemPermissionAuditQuery {
    pub user_id: Option<Uuid>,
    pub resource: Option<String>,
    pub action: Option<String>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
}

/// 複雑な操作の権限チェックリクエスト
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct ComplexOperationRequest {
    #[validate(length(
        min = 1,
        max = 50,
        message = "Operation must be between 1 and 50 characters"
    ))]
    pub operation: String,

    #[validate(length(
        min = 1,
        max = 50,
        message = "Resource type must be between 1 and 50 characters"
    ))]
    pub resource_type: String,

    pub resource_id: Option<Uuid>,
}
