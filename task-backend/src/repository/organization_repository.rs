// task-backend/src/repository/organization_repository.rs

use crate::domain::organization_model::{Organization, OrganizationMember};
use crate::error::AppResult;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

pub struct OrganizationRepository {
    _db: DatabaseConnection,
    // In-memory storage for testing
    organizations: Arc<Mutex<Vec<Organization>>>,
    members: Arc<Mutex<Vec<OrganizationMember>>>,
}

impl OrganizationRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            _db: db,
            organizations: Arc::new(Mutex::new(Vec::new())),
            members: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// 組織を作成
    pub async fn create_organization(
        &self,
        organization: &Organization,
    ) -> AppResult<Organization> {
        let mut orgs = self.organizations.lock().await;
        orgs.push(organization.clone());
        Ok(organization.clone())
    }

    /// IDで組織を検索
    pub async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Organization>> {
        let orgs = self.organizations.lock().await;
        Ok(orgs.iter().find(|o| o.id == id).cloned())
    }

    /// 名前で組織を検索
    pub async fn find_by_name(&self, name: &str) -> AppResult<Option<Organization>> {
        let orgs = self.organizations.lock().await;
        Ok(orgs.iter().find(|o| o.name == name).cloned())
    }

    /// オーナーIDで組織一覧を取得
    pub async fn find_by_owner_id(&self, owner_id: Uuid) -> AppResult<Vec<Organization>> {
        let orgs = self.organizations.lock().await;
        Ok(orgs
            .iter()
            .filter(|o| o.owner_id == owner_id)
            .cloned()
            .collect())
    }

    /// ユーザーが参加している組織一覧を取得
    pub async fn find_organizations_by_member(
        &self,
        user_id: Uuid,
    ) -> AppResult<Vec<Organization>> {
        let members = self.members.lock().await;
        let org_ids: Vec<Uuid> = members
            .iter()
            .filter(|m| m.user_id == user_id)
            .map(|m| m.organization_id)
            .collect();

        let orgs = self.organizations.lock().await;
        Ok(orgs
            .iter()
            .filter(|o| org_ids.contains(&o.id))
            .cloned()
            .collect())
    }

    /// 全組織を取得（管理者用）
    pub async fn find_all_organizations(&self) -> AppResult<Vec<Organization>> {
        let orgs = self.organizations.lock().await;
        Ok(orgs.clone())
    }

    /// 組織を更新
    pub async fn update_organization(
        &self,
        organization: &Organization,
    ) -> AppResult<Organization> {
        // 実装が必要
        Ok(organization.clone())
    }

    /// 組織を削除
    pub async fn delete_organization(&self, _id: Uuid) -> AppResult<()> {
        // 実装が必要
        Ok(())
    }

    /// 組織にメンバーを追加
    pub async fn add_member(&self, member: &OrganizationMember) -> AppResult<OrganizationMember> {
        let mut members = self.members.lock().await;
        members.push(member.clone());
        Ok(member.clone())
    }

    /// 組織メンバーを更新
    pub async fn update_member(
        &self,
        member: &OrganizationMember,
    ) -> AppResult<OrganizationMember> {
        // 実装が必要
        Ok(member.clone())
    }

    /// 組織メンバーを削除
    pub async fn remove_member(&self, _member_id: Uuid) -> AppResult<()> {
        // 実装が必要
        Ok(())
    }

    /// IDでメンバーを検索
    pub async fn find_member_by_id(&self, _id: Uuid) -> AppResult<Option<OrganizationMember>> {
        // 実装が必要
        Ok(None)
    }

    /// ユーザーと組織でメンバーを検索
    pub async fn find_member_by_user_and_organization(
        &self,
        _user_id: Uuid,
        _organization_id: Uuid,
    ) -> AppResult<Option<OrganizationMember>> {
        // 実装が必要
        Ok(None)
    }

    /// 組織のメンバー一覧を取得
    pub async fn find_members_by_organization_id(
        &self,
        _organization_id: Uuid,
    ) -> AppResult<Vec<OrganizationMember>> {
        // 実装が必要
        Ok(Vec::new())
    }

    /// 組織のメンバー数をカウント
    pub async fn count_members(&self, _organization_id: Uuid) -> AppResult<i64> {
        // 実装が必要
        Ok(0)
    }

    /// 全組織数を取得
    pub async fn count_all_organizations(&self) -> AppResult<u64> {
        // モック実装 - 実際の実装ではデータベースから取得
        Ok(10)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_organization_repository_creation() {
        // テスト実装が必要
    }
}
