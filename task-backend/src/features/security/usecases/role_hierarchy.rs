// task-backend/src/features/security/usecases/role_hierarchy.rs

use super::super::models::role::{RoleName, RoleWithPermissions};
use super::super::services::role::RoleService;
use crate::error::AppResult;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use uuid::Uuid;

/// ロール階層の管理と処理を行うUseCase
#[allow(dead_code)] // Security feature usecase - will be used when integrated
pub struct RoleHierarchyUseCase {
    role_service: Arc<RoleService>,
}

#[allow(dead_code)] // TODO: Will be used when role hierarchy features are integrated
impl RoleHierarchyUseCase {
    pub fn new(role_service: Arc<RoleService>) -> Self {
        Self { role_service }
    }

    /// ロール階層ツリーを構築
    pub async fn build_role_hierarchy(&self) -> AppResult<RoleHierarchyTree> {
        let all_roles = self.role_service.list_all_roles().await?;

        // ロール階層を定義（現在はシンプルな2階層）
        let mut hierarchy = RoleHierarchyTree {
            root: RoleNode {
                role: None, // ルートノード
                children: vec![],
            },
        };

        // Admin ロールをトップレベルに配置
        let admin_roles: Vec<_> = all_roles
            .iter()
            .filter(|r| r.name == RoleName::Admin)
            .cloned()
            .collect();

        // Member ロールとカスタムロールを第2レベルに配置
        let member_roles: Vec<_> = all_roles
            .iter()
            .filter(|r| r.name == RoleName::Member)
            .cloned()
            .collect();

        let custom_roles: Vec<_> = all_roles
            .iter()
            .filter(|r| r.name != RoleName::Admin && r.name != RoleName::Member)
            .cloned()
            .collect();

        // 階層構造を構築
        for admin_role in admin_roles {
            let mut admin_node = RoleNode {
                role: Some(admin_role),
                children: vec![],
            };

            // Adminの下にMemberとカスタムロールを配置
            for member_role in &member_roles {
                admin_node.children.push(RoleNode {
                    role: Some(member_role.clone()),
                    children: custom_roles
                        .iter()
                        .map(|r| RoleNode {
                            role: Some(r.clone()),
                            children: vec![],
                        })
                        .collect(),
                });
            }

            hierarchy.root.children.push(admin_node);
        }

        // Adminが存在しない場合は、Memberをトップレベルに配置
        if hierarchy.root.children.is_empty() {
            for member_role in member_roles {
                hierarchy.root.children.push(RoleNode {
                    role: Some(member_role),
                    children: custom_roles
                        .iter()
                        .map(|r| RoleNode {
                            role: Some(r.clone()),
                            children: vec![],
                        })
                        .collect(),
                });
            }
        }

        Ok(hierarchy)
    }

    /// ロールの継承権限を計算
    /// 上位ロールは下位ロールのすべての権限を継承する
    pub async fn calculate_inherited_permissions(
        &self,
        role_id: Uuid,
    ) -> AppResult<InheritedPermissions> {
        let role = self.role_service.get_role_by_id(role_id).await?;
        let all_roles = self.role_service.list_all_roles().await?;

        let mut inherited_permissions = InheritedPermissions {
            base_role: role.clone(),
            inherited_from: vec![],
            all_permissions: HashSet::new(),
        };

        // 基本権限を追加
        self.add_role_permissions(&role, &mut inherited_permissions.all_permissions);

        // 階層に基づいて権限を継承
        match role.name {
            RoleName::Admin => {
                // Adminはすべてのロールの権限を継承
                for other_role in &all_roles {
                    if other_role.id != role.id {
                        self.add_role_permissions(
                            other_role,
                            &mut inherited_permissions.all_permissions,
                        );
                        inherited_permissions.inherited_from.push(InheritedFrom {
                            role_id: other_role.id,
                            role_name: other_role.name.to_string(),
                            permissions_count: self.count_role_permissions(other_role),
                        });
                    }
                }
            }
            RoleName::Member => {
                // Memberは基本権限のみ（カスタムロールからは継承しない）
            }
        }

        Ok(inherited_permissions)
    }

    /// ロールの統計情報を取得
    pub async fn get_role_statistics(&self) -> AppResult<RoleStatistics> {
        let all_roles = self.role_service.list_all_roles().await?;
        let active_roles = self.role_service.list_active_roles().await?;

        let mut role_distribution = HashMap::new();
        let mut permission_distribution = HashMap::new();

        for role in &all_roles {
            // ロール名の分布
            *role_distribution.entry(role.name.to_string()).or_insert(0) += 1;

            // 権限レベルの分布
            let level = role.name.permission_level();
            *permission_distribution
                .entry(format!("Level {}", level))
                .or_insert(0) += 1;
        }

        Ok(RoleStatistics {
            total_roles: all_roles.len(),
            active_roles: active_roles.len(),
            inactive_roles: all_roles.len() - active_roles.len(),
            system_roles: all_roles
                .iter()
                .filter(|r| matches!(r.name, RoleName::Admin | RoleName::Member))
                .count(),
            custom_roles: all_roles
                .iter()
                .filter(|r| !matches!(r.name, RoleName::Admin | RoleName::Member))
                .count(),
            role_distribution,
            permission_distribution,
        })
    }

    /// ロール移行の影響を分析
    /// あるロールから別のロールへユーザーを移行した場合の影響を事前に評価
    pub async fn analyze_role_migration_impact(
        &self,
        from_role_id: Uuid,
        to_role_id: Uuid,
    ) -> AppResult<RoleMigrationImpact> {
        let from_role = self.role_service.get_role_by_id(from_role_id).await?;
        let to_role = self.role_service.get_role_by_id(to_role_id).await?;

        let mut gained_permissions = HashSet::new();
        let mut lost_permissions = HashSet::new();

        // 権限の比較
        let from_permissions = self.get_role_permissions(&from_role);
        let to_permissions = self.get_role_permissions(&to_role);

        for perm in &to_permissions {
            if !from_permissions.contains(perm) {
                gained_permissions.insert(perm.clone());
            }
        }

        for perm in &from_permissions {
            if !to_permissions.contains(perm) {
                lost_permissions.insert(perm.clone());
            }
        }

        // リスク評価
        let risk_level = if lost_permissions.is_empty() {
            RiskLevel::None
        } else if from_role.name == RoleName::Admin && to_role.name != RoleName::Admin {
            RiskLevel::High
        } else if !gained_permissions.is_empty() && !lost_permissions.is_empty() {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        };

        Ok(RoleMigrationImpact {
            from_role: from_role.clone(),
            to_role: to_role.clone(),
            gained_permissions: gained_permissions.into_iter().collect(),
            lost_permissions: lost_permissions.into_iter().collect(),
            risk_level,
            warnings: self.generate_migration_warnings(&from_role, &to_role),
        })
    }

    // ヘルパーメソッド

    fn add_role_permissions(&self, role: &RoleWithPermissions, permissions: &mut HashSet<String>) {
        // 基本的な権限を追加
        permissions.insert(format!("role:{}", role.name));

        if role.is_admin() {
            permissions.insert("admin:*".to_string());
            permissions.insert("user:*".to_string());
            permissions.insert("role:*".to_string());
            permissions.insert("organization:*".to_string());
            permissions.insert("team:*".to_string());
        } else if role.is_member() {
            permissions.insert("user:read:self".to_string());
            permissions.insert("user:write:self".to_string());
            permissions.insert("team:read:member".to_string());
            permissions.insert("organization:read:member".to_string());
        }
    }

    fn count_role_permissions(&self, role: &RoleWithPermissions) -> usize {
        let mut permissions = HashSet::new();
        self.add_role_permissions(role, &mut permissions);
        permissions.len()
    }

    fn get_role_permissions(&self, role: &RoleWithPermissions) -> HashSet<String> {
        let mut permissions = HashSet::new();
        self.add_role_permissions(role, &mut permissions);
        permissions
    }

    fn generate_migration_warnings(
        &self,
        from_role: &RoleWithPermissions,
        to_role: &RoleWithPermissions,
    ) -> Vec<String> {
        let mut warnings = vec![];

        if from_role.name == RoleName::Admin && to_role.name != RoleName::Admin {
            warnings.push("User will lose administrative privileges".to_string());
        }

        if !from_role.is_active && to_role.is_active {
            warnings.push("User is being moved from an inactive to an active role".to_string());
        }

        if from_role.is_active && !to_role.is_active {
            warnings.push("User is being moved to an inactive role".to_string());
        }

        warnings
    }
}

// データ構造

#[derive(Debug, Clone)]
#[allow(dead_code)] // Will be used for role hierarchy visualization
pub struct RoleHierarchyTree {
    pub root: RoleNode,
}

#[derive(Debug, Clone)]
#[allow(dead_code)] // Will be used for role hierarchy visualization
pub struct RoleNode {
    pub role: Option<RoleWithPermissions>,
    pub children: Vec<RoleNode>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)] // Will be used for permission inheritance analysis
pub struct InheritedPermissions {
    pub base_role: RoleWithPermissions,
    pub inherited_from: Vec<InheritedFrom>,
    pub all_permissions: HashSet<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)] // Will be used for permission inheritance analysis
pub struct InheritedFrom {
    pub role_id: Uuid,
    pub role_name: String,
    pub permissions_count: usize,
}

#[derive(Debug, Clone)]
#[allow(dead_code)] // Will be used for role analytics
pub struct RoleStatistics {
    pub total_roles: usize,
    pub active_roles: usize,
    pub inactive_roles: usize,
    pub system_roles: usize,
    pub custom_roles: usize,
    pub role_distribution: HashMap<String, usize>,
    pub permission_distribution: HashMap<String, usize>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)] // Will be used for role migration analysis
pub struct RoleMigrationImpact {
    pub from_role: RoleWithPermissions,
    pub to_role: RoleWithPermissions,
    pub gained_permissions: Vec<String>,
    pub lost_permissions: Vec<String>,
    pub risk_level: RiskLevel,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)] // Will be used for risk assessment
pub enum RiskLevel {
    None,
    Low,
    Medium,
    High,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_level() {
        assert_eq!(RiskLevel::None, RiskLevel::None);
        assert_ne!(RiskLevel::Low, RiskLevel::High);
    }
}
