// task-backend/src/features/security/usecases/permission_checker.rs

use super::super::models::role::RoleWithPermissions;
use super::super::services::permission::PermissionService;
use crate::core::permission::{PermissionResult, PermissionScope};
use crate::error::AppResult;
use std::sync::Arc;
use uuid::Uuid;

/// 権限チェックのビジネスロジックを統合的に扱うUseCase
pub struct PermissionCheckerUseCase {
    permission_service: Arc<PermissionService>,
}

impl PermissionCheckerUseCase {
    pub fn new(permission_service: Arc<PermissionService>) -> Self {
        Self { permission_service }
    }

    /// リソースに対する包括的な権限チェック
    /// 単純な権限チェックではなく、複数の条件を組み合わせた複雑な権限判定を行う
    pub async fn check_complex_resource_permission(
        &self,
        user_id: Uuid,
        resource_type: &str,
        resource_id: Option<Uuid>,
        action: &str,
        additional_context: Option<ComplexPermissionContext>,
    ) -> AppResult<PermissionDecision> {
        // 基本的な権限チェック
        let basic_check = self
            .permission_service
            .check_resource_access(user_id, resource_type, resource_id, action)
            .await;

        if basic_check.is_ok() {
            return Ok(PermissionDecision::Allowed {
                reason: "Basic permission check passed".to_string(),
                conditions: vec![],
            });
        }

        // 追加のコンテキストがある場合は、より詳細なチェックを実行
        if let Some(context) = additional_context {
            // 組織レベルの権限チェック
            if let Some(org_id) = context.organization_id {
                if self
                    .permission_service
                    .check_organization_management_permission(user_id, org_id)
                    .await
                    .is_ok()
                {
                    return Ok(PermissionDecision::Allowed {
                        reason: "Organization-level permission granted".to_string(),
                        conditions: vec!["Must maintain organization membership".to_string()],
                    });
                }
            }

            // チームレベルの権限チェック
            if let Some(team_id) = context.team_id {
                if self
                    .permission_service
                    .check_team_management_permission(user_id, team_id)
                    .await
                    .is_ok()
                {
                    return Ok(PermissionDecision::Allowed {
                        reason: "Team-level permission granted".to_string(),
                        conditions: vec!["Must maintain team membership".to_string()],
                    });
                }
            }

            // 時間制限付き権限チェック
            if let Some(time_restriction) = context.time_restriction {
                if self.check_time_based_permission(time_restriction) {
                    return Ok(PermissionDecision::Allowed {
                        reason: "Time-based permission granted".to_string(),
                        conditions: vec![format!("Valid during: {}", time_restriction.description)],
                    });
                }
            }
        }

        Ok(PermissionDecision::Denied {
            reason: "No applicable permissions found".to_string(),
            required_permissions: self.get_required_permissions(resource_type, action),
        })
    }

    /// 階層的な権限継承チェック
    /// 上位階層（組織→チーム→個人）の権限を考慮した総合的な判定
    pub async fn check_hierarchical_permission(
        &self,
        user_id: Uuid,
        hierarchy: PermissionHierarchy,
    ) -> AppResult<HierarchicalPermissionResult> {
        let mut permissions = vec![];

        // 組織レベル
        if let Some(org_id) = hierarchy.organization_id {
            let org_perm = self
                .permission_service
                .check_organization_management_permission(user_id, org_id)
                .await;
            permissions.push(HierarchyLevel {
                level: "organization".to_string(),
                has_permission: org_perm.is_ok(),
                scope: if org_perm.is_ok() {
                    PermissionScope::Organization
                } else {
                    PermissionScope::Own
                },
            });
        }

        // チームレベル
        if let Some(team_id) = hierarchy.team_id {
            let team_perm = self
                .permission_service
                .check_team_management_permission(user_id, team_id)
                .await;
            permissions.push(HierarchyLevel {
                level: "team".to_string(),
                has_permission: team_perm.is_ok(),
                scope: if team_perm.is_ok() {
                    PermissionScope::Team
                } else {
                    PermissionScope::Own
                },
            });
        }

        // 個人レベル
        if let Some(target_user_id) = hierarchy.user_id {
            let user_perm = self
                .permission_service
                .check_user_access(user_id, target_user_id)
                .await;
            permissions.push(HierarchyLevel {
                level: "user".to_string(),
                has_permission: user_perm.is_ok(),
                scope: PermissionScope::Own,
            });
        }

        // 最高権限レベルを決定
        let highest_permission = permissions
            .iter()
            .filter(|p| p.has_permission)
            .max_by_key(|p| p.scope.level())
            .cloned();

        Ok(HierarchicalPermissionResult {
            hierarchy_levels: permissions,
            highest_permission,
            effective_scope: highest_permission
                .as_ref()
                .map(|p| p.scope.clone())
                .unwrap_or(PermissionScope::Own),
        })
    }

    /// バッチ権限チェック
    /// 複数のリソースに対する権限を効率的にチェック
    pub async fn check_batch_permissions(
        &self,
        user_id: Uuid,
        requests: Vec<PermissionRequest>,
    ) -> AppResult<Vec<BatchPermissionResult>> {
        let mut results = Vec::new();

        for request in requests {
            let check_result = self
                .permission_service
                .check_resource_access(
                    user_id,
                    &request.resource_type,
                    request.resource_id,
                    &request.action,
                )
                .await;

            results.push(BatchPermissionResult {
                request_id: request.id,
                resource_type: request.resource_type,
                resource_id: request.resource_id,
                action: request.action,
                allowed: check_result.is_ok(),
                error: check_result.err().map(|e| e.to_string()),
            });
        }

        Ok(results)
    }

    // ヘルパーメソッド

    fn check_time_based_permission(&self, restriction: TimeRestriction) -> bool {
        use chrono::{Local, NaiveTime, Timelike};

        let now = Local::now();
        let current_time =
            NaiveTime::from_hms_opt(now.hour() as u32, now.minute() as u32, now.second() as u32)
                .unwrap();

        match restriction {
            TimeRestriction::BusinessHours => {
                let start = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
                let end = NaiveTime::from_hms_opt(18, 0, 0).unwrap();
                current_time >= start && current_time <= end
            }
            TimeRestriction::Custom { start, end, .. } => {
                current_time >= start && current_time <= end
            }
        }
    }

    fn get_required_permissions(&self, resource_type: &str, action: &str) -> Vec<String> {
        match (resource_type, action) {
            ("user", "view") => vec!["user:read".to_string(), "admin:read".to_string()],
            ("user", "update") => vec!["user:write".to_string(), "admin:write".to_string()],
            ("organization", _) => vec!["organization:manage".to_string(), "admin:*".to_string()],
            ("team", _) => vec!["team:manage".to_string(), "admin:*".to_string()],
            _ => vec![
                format!("{}:{}", resource_type, action),
                "admin:*".to_string(),
            ],
        }
    }
}

// データ構造

#[derive(Debug, Clone)]
pub struct ComplexPermissionContext {
    pub organization_id: Option<Uuid>,
    pub team_id: Option<Uuid>,
    pub time_restriction: Option<TimeRestriction>,
}

#[derive(Debug, Clone)]
pub enum TimeRestriction {
    BusinessHours,
    Custom {
        start: chrono::NaiveTime,
        end: chrono::NaiveTime,
        description: String,
    },
}

#[derive(Debug, Clone)]
pub enum PermissionDecision {
    Allowed {
        reason: String,
        conditions: Vec<String>,
    },
    Denied {
        reason: String,
        required_permissions: Vec<String>,
    },
}

#[derive(Debug, Clone)]
pub struct PermissionHierarchy {
    pub organization_id: Option<Uuid>,
    pub team_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
}

#[derive(Debug, Clone)]
pub struct HierarchyLevel {
    pub level: String,
    pub has_permission: bool,
    pub scope: PermissionScope,
}

#[derive(Debug, Clone)]
pub struct HierarchicalPermissionResult {
    pub hierarchy_levels: Vec<HierarchyLevel>,
    pub highest_permission: Option<HierarchyLevel>,
    pub effective_scope: PermissionScope,
}

#[derive(Debug, Clone)]
pub struct PermissionRequest {
    pub id: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub action: String,
}

#[derive(Debug, Clone)]
pub struct BatchPermissionResult {
    pub request_id: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub action: String,
    pub allowed: bool,
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_restriction() {
        let business_hours = TimeRestriction::BusinessHours;
        // 実際のテストは時刻に依存するため、モックが必要
    }

    #[test]
    fn test_permission_decision() {
        let allowed = PermissionDecision::Allowed {
            reason: "Test allowed".to_string(),
            conditions: vec!["Condition 1".to_string()],
        };

        match allowed {
            PermissionDecision::Allowed { reason, .. } => {
                assert_eq!(reason, "Test allowed");
            }
            _ => panic!("Expected allowed decision"),
        }
    }
}
