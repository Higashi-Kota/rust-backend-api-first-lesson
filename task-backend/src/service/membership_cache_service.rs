// task-backend/src/service/membership_cache_service.rs
//
// メンバーシップ情報のキャッシュサービス
// チームや組織のメンバーシップ情報を効率的に取得するためのキャッシュ実装
// 将来的なパフォーマンス最適化のために保持

#![allow(dead_code)] // 将来のパフォーマンス最適化のために保持

use crate::error::AppResult;
use crate::log_with_context;
use crate::repository::organization_repository::OrganizationRepository;
use crate::repository::team_repository::TeamRepository;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// メンバーシップ情報のキャッシュ
#[derive(Clone, Debug)]
pub struct MembershipInfo {
    pub user_id: Uuid,
    pub team_ids: Vec<Uuid>,
    pub organization_ids: Vec<Uuid>,
    pub team_roles: HashMap<Uuid, String>, // team_id -> role
    pub organization_roles: HashMap<Uuid, String>, // org_id -> role
    pub cached_at: chrono::DateTime<chrono::Utc>,
}

/// メンバーシップキャッシュサービス
#[derive(Clone)]
pub struct MembershipCacheService {
    team_repository: Arc<TeamRepository>,
    organization_repository: Arc<OrganizationRepository>,
    cache: Arc<RwLock<HashMap<Uuid, MembershipInfo>>>,
    cache_ttl: chrono::Duration,
}

impl MembershipCacheService {
    pub fn new(
        team_repository: Arc<TeamRepository>,
        organization_repository: Arc<OrganizationRepository>,
    ) -> Self {
        Self {
            team_repository,
            organization_repository,
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: chrono::Duration::minutes(5), // 5分のキャッシュ
        }
    }

    /// ユーザーのメンバーシップ情報を取得（キャッシュ付き）
    pub async fn get_user_memberships(&self, user_id: Uuid) -> AppResult<MembershipInfo> {
        log_with_context!(
            tracing::Level::DEBUG,
            "Getting user memberships",
            "user_id" => user_id
        );
        // キャッシュをチェック
        {
            let cache = self.cache.read().await;
            if let Some(info) = cache.get(&user_id) {
                let age = chrono::Utc::now() - info.cached_at;
                if age < self.cache_ttl {
                    log_with_context!(
                        tracing::Level::DEBUG,
                        "Cache hit for user memberships",
                        "user_id" => user_id,
                        "cache_age_seconds" => age.num_seconds()
                    );
                    return Ok(info.clone());
                }
            }
        }

        // キャッシュミスの場合、DBから取得
        log_with_context!(
            tracing::Level::DEBUG,
            "Cache miss, fetching from database",
            "user_id" => user_id
        );
        let membership_info = self.fetch_membership_info(user_id).await?;

        // キャッシュを更新
        {
            let mut cache = self.cache.write().await;
            cache.insert(user_id, membership_info.clone());
        }

        log_with_context!(
            tracing::Level::INFO,
            "User memberships cached",
            "user_id" => user_id,
            "team_count" => membership_info.team_ids.len(),
            "org_count" => membership_info.organization_ids.len()
        );

        Ok(membership_info)
    }

    /// DBからメンバーシップ情報を取得
    async fn fetch_membership_info(&self, user_id: Uuid) -> AppResult<MembershipInfo> {
        // チームメンバーシップを取得
        let teams = self.team_repository.find_teams_by_member(user_id).await?;

        let mut team_ids = Vec::new();
        let mut team_roles = HashMap::new();

        for team in &teams {
            team_ids.push(team.id);
            // チームメンバーの役割を個別に取得
            if let Ok(Some(member)) = self
                .team_repository
                .find_member_by_user_and_team(user_id, team.id)
                .await
            {
                team_roles.insert(team.id, member.role);
            }
        }

        // 組織メンバーシップを取得
        let organizations = self
            .organization_repository
            .find_organizations_by_member(user_id)
            .await?;

        let mut organization_ids = Vec::new();
        let mut organization_roles = HashMap::new();

        for org in &organizations {
            organization_ids.push(org.id);
            // 組織メンバーの役割を個別に取得
            if let Ok(Some(member)) = self
                .organization_repository
                .find_member_by_user_and_organization(user_id, org.id)
                .await
            {
                organization_roles.insert(org.id, member.role.to_string());
            }
        }

        Ok(MembershipInfo {
            user_id,
            team_ids,
            organization_ids,
            team_roles,
            organization_roles,
            cached_at: chrono::Utc::now(),
        })
    }

    /// キャッシュをクリア（ユーザーのメンバーシップが変更された時に使用）
    pub async fn invalidate_user_cache(&self, user_id: Uuid) {
        log_with_context!(
            tracing::Level::DEBUG,
            "Invalidating user cache",
            "user_id" => user_id
        );
        let mut cache = self.cache.write().await;
        cache.remove(&user_id);
    }

    /// すべてのキャッシュをクリア
    pub async fn clear_all_cache(&self) {
        log_with_context!(tracing::Level::INFO, "Clearing all membership cache");
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// ユーザーが特定のチームのメンバーかチェック
    pub async fn is_team_member(&self, user_id: Uuid, team_id: Uuid) -> AppResult<bool> {
        let membership = self.get_user_memberships(user_id).await?;
        Ok(membership.team_ids.contains(&team_id))
    }

    /// ユーザーが特定の組織のメンバーかチェック
    pub async fn is_organization_member(&self, user_id: Uuid, org_id: Uuid) -> AppResult<bool> {
        let membership = self.get_user_memberships(user_id).await?;
        Ok(membership.organization_ids.contains(&org_id))
    }

    /// ユーザーのチーム内での役割を取得
    pub async fn get_team_role(&self, user_id: Uuid, team_id: Uuid) -> AppResult<Option<String>> {
        let membership = self.get_user_memberships(user_id).await?;
        Ok(membership.team_roles.get(&team_id).cloned())
    }

    /// ユーザーの組織内での役割を取得
    pub async fn get_organization_role(
        &self,
        user_id: Uuid,
        org_id: Uuid,
    ) -> AppResult<Option<String>> {
        let membership = self.get_user_memberships(user_id).await?;
        Ok(membership.organization_roles.get(&org_id).cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // モックリポジトリのテストは省略（実際の実装では必要）

    #[tokio::test]
    async fn test_membership_info_structure() {
        let user_id = Uuid::new_v4();
        let team_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();

        let mut team_roles = HashMap::new();
        team_roles.insert(team_id, "Owner".to_string());

        let mut org_roles = HashMap::new();
        org_roles.insert(org_id, "Admin".to_string());

        let info = MembershipInfo {
            user_id,
            team_ids: vec![team_id],
            organization_ids: vec![org_id],
            team_roles,
            organization_roles: org_roles,
            cached_at: chrono::Utc::now(),
        };

        assert_eq!(info.team_ids.len(), 1);
        assert_eq!(info.organization_ids.len(), 1);
        assert_eq!(info.team_roles.get(&team_id), Some(&"Owner".to_string()));
    }
}
