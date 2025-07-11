use super::super::models::{
    department::{self, Model as Department},
    department_member::{self, DepartmentRole},
};
use super::super::repositories::{
    DepartmentMemberRepository, DepartmentRepository, OrganizationRepository,
};
// TODO: Phase 19でOrganizationHierarchyServiceの使用箇所が実装されたら#[allow(unused_imports)]を削除
#[allow(unused_imports)]
use super::super::services::hierarchy::OrganizationHierarchyService;
use crate::error::{AppError, AppResult};
use crate::features::auth::repository::user_repository::UserRepository;
use chrono::Utc;
use sea_orm::{DatabaseConnection, Set};
use std::sync::Arc;
use uuid::Uuid;

/// 組織階層の再編成を行うUseCase
/// 複数の部門を跨ぐ大規模な組織再編を扱う
#[allow(dead_code)] // Organization hierarchy feature - will be used when integrated
pub struct ReorganizeDepartmentsUseCase {
    db: Arc<DatabaseConnection>,
    organization_repository: Arc<OrganizationRepository>,
    #[allow(dead_code)] // False positive - used in add_members_to_department
    user_repository: Arc<UserRepository>,
}

#[allow(dead_code)] // TODO: Will be used when organization reorganization features are integrated
impl ReorganizeDepartmentsUseCase {
    pub fn new(
        db: Arc<DatabaseConnection>,
        organization_repository: Arc<OrganizationRepository>,
        user_repository: Arc<UserRepository>,
    ) -> Self {
        Self {
            db,
            organization_repository,
            user_repository,
        }
    }

    /// 部門を別の親部門に移動し、関連するすべての子部門も再帰的に更新
    pub async fn move_department_with_children(
        &self,
        department_id: Uuid,
        new_parent_id: Option<Uuid>,
        moved_by: Uuid,
    ) -> AppResult<Vec<Department>> {
        // 権限チェック
        let department = DepartmentRepository::find_by_id(&self.db, department_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Department not found".to_string()))?;

        // 組織の管理権限を確認
        let organization = self
            .organization_repository
            .find_by_id(department.organization_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Organization not found".to_string()))?;

        let member = self
            .organization_repository
            .find_member_by_user_and_organization(moved_by, organization.id)
            .await?
            .ok_or_else(|| AppError::Forbidden("Not an organization member".to_string()))?;

        if !member.role.can_manage() {
            return Err(AppError::Forbidden(
                "Insufficient permissions to reorganize departments".to_string(),
            ));
        }

        // 循環参照チェック
        if let Some(parent_id) = new_parent_id {
            if self
                .would_create_circular_dependency(department_id, parent_id)
                .await?
            {
                return Err(AppError::BadRequest(
                    "This move would create a circular dependency".to_string(),
                ));
            }
        }

        // 移動処理
        let mut affected_departments = Vec::new();

        // 1. 対象部門を更新
        let new_hierarchy_path = if let Some(parent_id) = new_parent_id {
            let parent = DepartmentRepository::find_by_id(&self.db, parent_id)
                .await?
                .ok_or_else(|| AppError::NotFound("Parent department not found".to_string()))?;

            let mut active_model: department::ActiveModel = department.clone().into();
            active_model.parent_department_id = Set(Some(parent_id));
            active_model.hierarchy_level = Set(parent.hierarchy_level + 1);
            active_model.hierarchy_path =
                Set(format!("{}/{}", parent.hierarchy_path, department_id));
            active_model.updated_at = Set(Utc::now());

            let updated =
                DepartmentRepository::update_by_id(&self.db, department_id, active_model).await?;
            affected_departments.push(updated.clone());
            updated.hierarchy_path
        } else {
            let mut active_model: department::ActiveModel = department.clone().into();
            active_model.parent_department_id = Set(None);
            active_model.hierarchy_level = Set(0);
            active_model.hierarchy_path = Set(format!("/{}", department_id));
            active_model.updated_at = Set(Utc::now());

            let updated =
                DepartmentRepository::update_by_id(&self.db, department_id, active_model).await?;
            affected_departments.push(updated.clone());
            updated.hierarchy_path
        };

        // 2. すべての子部門を再帰的に更新
        let children = self
            .update_children_hierarchy(department_id, &new_hierarchy_path)
            .await?;
        affected_departments.extend(children);

        Ok(affected_departments)
    }

    /// 複数部門の一括再編成
    pub async fn bulk_reorganize(
        &self,
        organization_id: Uuid,
        moves: Vec<(Uuid, Option<Uuid>)>, // (department_id, new_parent_id)
        reorganized_by: Uuid,
    ) -> AppResult<Vec<Department>> {
        // 権限チェック
        let member = self
            .organization_repository
            .find_member_by_user_and_organization(reorganized_by, organization_id)
            .await?
            .ok_or_else(|| AppError::Forbidden("Not an organization member".to_string()))?;

        if !member.role.can_manage() {
            return Err(AppError::Forbidden(
                "Insufficient permissions to reorganize departments".to_string(),
            ));
        }

        // 移動の妥当性を事前チェック
        for (dept_id, new_parent_id) in &moves {
            if let Some(parent_id) = new_parent_id {
                if self
                    .would_create_circular_dependency(*dept_id, *parent_id)
                    .await?
                {
                    return Err(AppError::BadRequest(format!(
                        "Move of department {} would create circular dependency",
                        dept_id
                    )));
                }
            }
        }

        // すべての移動を実行
        let mut all_affected = Vec::new();
        for (dept_id, new_parent_id) in moves {
            let affected = self
                .move_department_with_children(dept_id, new_parent_id, reorganized_by)
                .await?;
            all_affected.extend(affected);
        }

        Ok(all_affected)
    }

    // ヘルパーメソッド

    async fn would_create_circular_dependency(
        &self,
        department_id: Uuid,
        target_parent_id: Uuid,
    ) -> AppResult<bool> {
        if department_id == target_parent_id {
            return Ok(true);
        }

        // target_parent_idの祖先を辿り、department_idが含まれていないか確認
        let mut current_id = Some(target_parent_id);
        while let Some(id) = current_id {
            if id == department_id {
                return Ok(true);
            }
            let dept = DepartmentRepository::find_by_id(&self.db, id).await?;
            current_id = dept.and_then(|d| d.parent_department_id);
        }

        Ok(false)
    }

    fn update_children_hierarchy(
        &self,
        parent_id: Uuid,
        parent_path: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = AppResult<Vec<Department>>> + Send + '_>>
    {
        let parent_path = parent_path.to_string();
        Box::pin(async move {
            let mut updated_departments = Vec::new();
            let children =
                DepartmentRepository::find_children_by_parent_id(&self.db, parent_id).await?;

            for child in children {
                let new_path = format!("{}/{}", parent_path, child.id);
                let parent_level = parent_path.matches('/').count() as i32;

                let mut active_model: department::ActiveModel = child.clone().into();
                active_model.hierarchy_path = Set(new_path.clone());
                active_model.hierarchy_level = Set(parent_level + 1);
                active_model.updated_at = Set(Utc::now());

                let updated =
                    DepartmentRepository::update_by_id(&self.db, child.id, active_model).await?;
                updated_departments.push(updated);

                // 再帰的に子部門を更新
                let grand_children = self.update_children_hierarchy(child.id, &new_path).await?;
                updated_departments.extend(grand_children);
            }

            Ok(updated_departments)
        })
    }
}

/// 部門メンバーの一括管理UseCase
#[allow(dead_code)] // Organization hierarchy feature - will be used when integrated
pub struct ManageDepartmentMembersUseCase {
    db: Arc<DatabaseConnection>,
    organization_repository: Arc<OrganizationRepository>,
    user_repository: Arc<UserRepository>,
}

#[allow(dead_code)] // TODO: Will be used when department member management features are integrated
impl ManageDepartmentMembersUseCase {
    pub fn new(
        db: Arc<DatabaseConnection>,
        organization_repository: Arc<OrganizationRepository>,
        user_repository: Arc<UserRepository>,
    ) -> Self {
        Self {
            db,
            organization_repository,
            user_repository,
        }
    }

    /// 複数ユーザーを一括で部門に追加
    pub async fn bulk_add_members(
        &self,
        department_id: Uuid,
        user_ids: Vec<Uuid>,
        role: DepartmentRole,
        added_by: Uuid,
    ) -> AppResult<Vec<department_member::Model>> {
        // 部門の存在確認
        let department = DepartmentRepository::find_by_id(&self.db, department_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Department not found".to_string()))?;

        // 権限チェック
        let member = self
            .organization_repository
            .find_member_by_user_and_organization(added_by, department.organization_id)
            .await?
            .ok_or_else(|| AppError::Forbidden("Not an organization member".to_string()))?;

        if !member.role.can_manage() {
            return Err(AppError::Forbidden(
                "Insufficient permissions to manage department members".to_string(),
            ));
        }

        let mut added_members = Vec::new();

        for user_id in user_ids {
            // ユーザーの存在確認
            let _user = self
                .user_repository
                .find_by_id(user_id)
                .await?
                .ok_or_else(|| AppError::NotFound(format!("User {} not found", user_id)))?;

            // 組織メンバーであることを確認
            let is_org_member = self
                .organization_repository
                .find_member_by_user_and_organization(user_id, department.organization_id)
                .await?
                .is_some();

            if !is_org_member {
                continue; // スキップまたはエラー（ポリシーによる）
            }

            // 既存メンバーチェック
            if DepartmentMemberRepository::find_by_department_and_user(
                &self.db,
                department_id,
                user_id,
            )
            .await?
            .is_some()
            {
                continue; // 既にメンバーの場合はスキップ
            }

            // メンバー追加
            let member = department_member::ActiveModel {
                id: Set(Uuid::new_v4()),
                department_id: Set(department_id),
                user_id: Set(user_id),
                role: Set(role.to_string()),
                is_active: Set(true),
                joined_at: Set(Utc::now()),
                added_by: Set(added_by),
                created_at: Set(Utc::now()),
                updated_at: Set(Utc::now()),
            };

            let created = DepartmentMemberRepository::create(&self.db, member).await?;
            added_members.push(created);
        }

        Ok(added_members)
    }

    /// 部門間でメンバーを移動
    pub async fn move_members_between_departments(
        &self,
        from_department_id: Uuid,
        to_department_id: Uuid,
        user_ids: Vec<Uuid>,
        moved_by: Uuid,
    ) -> AppResult<Vec<department_member::Model>> {
        // 両部門の存在確認
        let from_dept = DepartmentRepository::find_by_id(&self.db, from_department_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Source department not found".to_string()))?;

        let to_dept = DepartmentRepository::find_by_id(&self.db, to_department_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Target department not found".to_string()))?;

        // 同じ組織であることを確認
        if from_dept.organization_id != to_dept.organization_id {
            return Err(AppError::BadRequest(
                "Cannot move members between different organizations".to_string(),
            ));
        }

        // 権限チェック
        let member = self
            .organization_repository
            .find_member_by_user_and_organization(moved_by, from_dept.organization_id)
            .await?
            .ok_or_else(|| AppError::Forbidden("Not an organization member".to_string()))?;

        if !member.role.can_manage() {
            return Err(AppError::Forbidden(
                "Insufficient permissions to move department members".to_string(),
            ));
        }

        let mut moved_members = Vec::new();

        for user_id in user_ids {
            // 元部門のメンバーであることを確認
            let from_member = DepartmentMemberRepository::find_by_department_and_user(
                &self.db,
                from_department_id,
                user_id,
            )
            .await?;

            if let Some(member) = from_member {
                // 元部門から削除
                DepartmentMemberRepository::deactivate_by_id(&self.db, member.id).await?;

                // 新部門に追加
                let new_member = department_member::ActiveModel {
                    id: Set(Uuid::new_v4()),
                    department_id: Set(to_department_id),
                    user_id: Set(user_id),
                    role: Set(member.role), // 元の役割を維持
                    is_active: Set(true),
                    joined_at: Set(Utc::now()),
                    added_by: Set(moved_by),
                    created_at: Set(Utc::now()),
                    updated_at: Set(Utc::now()),
                };

                let created = DepartmentMemberRepository::create(&self.db, new_member).await?;
                moved_members.push(created);
            }
        }

        Ok(moved_members)
    }
}
