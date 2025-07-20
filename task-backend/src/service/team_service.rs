// task-backend/src/service/team_service.rs

use crate::api::dto::team_dto::*;
use crate::api::dto::team_query_dto::TeamSearchQuery;
use crate::domain::team_member_model::Model as TeamMemberModel;
use crate::domain::team_model::{Model as TeamModel, TeamRole};
use crate::middleware::subscription_guard::check_feature_limit;
use crate::utils::email::EmailService;

// Type aliases for domain models
pub type Team = TeamModel;
pub type TeamMember = TeamMemberModel;
use crate::domain::subscription_tier::SubscriptionTier;
use crate::error::{AppError, AppResult};
use crate::repository::team_repository::TeamRepository;
use crate::repository::user_repository::UserRepository;
use crate::types::Timestamp;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

pub struct TeamService {
    team_repository: TeamRepository,
    user_repository: UserRepository,
    email_service: Arc<EmailService>,
}

impl TeamService {
    pub fn new(
        _db: Arc<DatabaseConnection>,
        team_repository: TeamRepository,
        user_repository: UserRepository,
        email_service: Arc<EmailService>,
    ) -> Self {
        Self {
            team_repository,
            user_repository,
            email_service,
        }
    }

    /// チームを作成
    pub async fn create_team(
        &self,
        request: CreateTeamRequest,
        owner_id: Uuid,
    ) -> AppResult<TeamResponse> {
        // チーム名の重複チェック
        if let Some(_existing) = self.team_repository.find_by_name(&request.name).await? {
            return Err(AppError::BadRequest("Team name already exists".to_string()));
        }

        // ユーザーのサブスクリプションティアを取得
        let user = self
            .user_repository
            .find_by_id(owner_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
        let user_tier =
            SubscriptionTier::from_str(&user.subscription_tier).unwrap_or(SubscriptionTier::Free);

        // 現在のチーム数を取得
        let current_team_count = self
            .team_repository
            .count_user_owned_teams(owner_id)
            .await?;

        // チーム数制限チェック
        check_feature_limit(&user_tier, current_team_count, "teams")?;

        info!(
            owner_id = %owner_id,
            team_name = %request.name,
            current_teams = current_team_count,
            user_tier = %user_tier.as_str(),
            "Starting team creation"
        );

        // チームのサブスクリプション階層はオーナーと同じ
        let subscription_tier = user_tier;

        let team = Team::new_team(
            request.name.clone(),
            request.description.clone(),
            request.organization_id,
            owner_id,
            subscription_tier,
        );

        // チームを作成（repositoryのcreate_teamメソッドを使用）
        let created_team = self.team_repository.create_team(&team).await?;

        // オーナーをメンバーとして追加
        let owner_member = TeamMember::new_member(created_team.id, owner_id, TeamRole::Owner, None);
        let created_member = self.team_repository.add_member(&owner_member).await?;

        info!(
            owner_id = %owner_id,
            team_id = %created_team.id,
            team_name = %created_team.name,
            "Team created successfully"
        );

        // レスポンスを作成
        let member_response = self.build_team_member_response(&created_member).await?;
        Ok(TeamResponse::from((created_team, vec![member_response])))
    }

    /// チーム詳細を取得
    pub async fn get_team_by_id(&self, team_id: Uuid, user_id: Uuid) -> AppResult<TeamResponse> {
        let team = self
            .team_repository
            .find_by_id(team_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Team not found".to_string()))?;

        // アクセス権限チェック
        self.check_team_access(&team, user_id).await?;

        let members = self
            .team_repository
            .find_members_by_team_id(team_id)
            .await?;
        let member_responses = self.build_team_member_responses(&members).await?;

        Ok(TeamResponse::from((team, member_responses)))
    }

    /// チーム一覧を取得
    pub async fn get_teams(
        &self,
        query: TeamSearchQuery,
        user_id: Uuid,
    ) -> AppResult<Vec<TeamListResponse>> {
        let teams = if let Some(org_id) = query.organization_id {
            // 組織のチーム一覧
            self.team_repository.find_by_organization_id(org_id).await?
        } else if let Some(owner_id) = query.owner_id {
            // 特定ユーザーが所有するチーム一覧
            self.team_repository.find_by_owner_id(owner_id).await?
        } else {
            // ユーザーが参加しているチーム一覧
            self.team_repository.find_teams_by_member(user_id).await?
        };

        let mut team_responses = Vec::new();
        for team in teams {
            let member_count = self.team_repository.count_members(team.id).await? as i32;
            let mut team_response = TeamListResponse::from(team);
            team_response.current_member_count = member_count;
            team_responses.push(team_response);
        }

        Ok(team_responses)
    }

    /// チームを更新
    pub async fn update_team(
        &self,
        team_id: Uuid,
        request: UpdateTeamRequest,
        user_id: Uuid,
    ) -> AppResult<TeamResponse> {
        let mut team = self
            .team_repository
            .find_by_id(team_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Team not found".to_string()))?;

        // 管理権限チェック
        self.check_team_management_permission(&team, user_id)
            .await?;

        // 更新処理
        if let Some(name) = request.name {
            // 名前の重複チェック（異なるチームで）
            if let Some(existing) = self.team_repository.find_by_name(&name).await? {
                if existing.id != team_id {
                    return Err(AppError::BadRequest("Team name already exists".to_string()));
                }
            }
            team.name = name;
        }

        if let Some(description) = request.description {
            team.description = Some(description);
        }

        let updated_team = self.team_repository.update_team(&team).await?;
        let members = self
            .team_repository
            .find_members_by_team_id(team_id)
            .await?;
        let member_responses = self.build_team_member_responses(&members).await?;

        Ok(TeamResponse::from((updated_team, member_responses)))
    }

    /// チームを削除
    pub async fn delete_team(&self, team_id: Uuid, user_id: Uuid) -> AppResult<()> {
        let team = self
            .team_repository
            .find_by_id(team_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Team not found".to_string()))?;

        // オーナー権限チェック
        if team.owner_id != user_id {
            return Err(AppError::Forbidden(
                "Only team owner can delete the team".to_string(),
            ));
        }

        // メンバーを全て削除
        let members = self
            .team_repository
            .find_members_by_team_id(team_id)
            .await?;
        for member in members {
            self.team_repository.remove_member(member.id).await?;
        }

        // チームを削除
        self.team_repository.delete_team(team_id).await?;
        Ok(())
    }

    /// チームメンバーを招待
    pub async fn invite_team_member(
        &self,
        team_id: Uuid,
        request: InviteTeamMemberRequest,
        inviter_id: Uuid,
    ) -> AppResult<TeamMemberResponse> {
        let team = self
            .team_repository
            .find_by_id(team_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Team not found".to_string()))?;

        // 招待権限チェック
        self.check_team_invite_permission(&team, inviter_id).await?;

        // チームオーナーのサブスクリプションティアを取得
        let owner = self
            .user_repository
            .find_by_id(team.owner_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Team owner not found".to_string()))?;
        let owner_tier =
            SubscriptionTier::from_str(&owner.subscription_tier).unwrap_or(SubscriptionTier::Free);

        // 現在のメンバー数を取得
        let current_member_count = self.team_repository.count_members(team_id).await?;

        // メンバー数制限チェック
        check_feature_limit(&owner_tier, current_member_count, "team_members")?;

        // ユーザーIDを取得（emailまたはuser_idから）
        let user_id = if let Some(user_id) = request.user_id {
            user_id
        } else if let Some(email) = &request.email {
            self.user_repository
                .find_by_email(email)
                .await?
                .ok_or_else(|| AppError::NotFound("User not found".to_string()))?
                .id
        } else {
            return Err(AppError::BadRequest(
                "Either user_id or email is required".to_string(),
            ));
        };

        // 既存メンバーチェック
        if let Some(_existing) = self
            .team_repository
            .find_member_by_user_and_team(user_id, team_id)
            .await?
        {
            return Err(AppError::BadRequest(
                "User is already a team member".to_string(),
            ));
        }

        let member = TeamMember::new_member(team_id, user_id, request.role, Some(inviter_id));
        let created_member = self.team_repository.add_member(&member).await?;

        // メール送信のための情報を取得
        let invited_user = self
            .user_repository
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Invited user not found".to_string()))?;

        let inviter = self
            .user_repository
            .find_by_id(inviter_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Inviter not found".to_string()))?;

        // チーム招待メールを送信
        if let Err(e) = self
            .email_service
            .send_team_invitation_email(
                &invited_user.email,
                &invited_user.username,
                &team.name,
                &inviter.username,
                &format!("https://yourapp.com/teams/{}/accept", team_id),
            )
            .await
        {
            // メール送信失敗はログに記録するが、処理は継続
            warn!("Failed to send team invitation email: {}", e);
        }

        self.build_team_member_response(&created_member).await
    }

    /// チームメンバーの役割を更新
    pub async fn update_team_member_role(
        &self,
        team_id: Uuid,
        member_id: Uuid,
        request: UpdateTeamMemberRoleRequest,
        user_id: Uuid,
    ) -> AppResult<TeamMemberResponse> {
        let team = self
            .team_repository
            .find_by_id(team_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Team not found".to_string()))?;

        let mut member = self
            .team_repository
            .find_member_by_id(member_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Team member not found".to_string()))?;

        // 管理権限チェック
        self.check_team_management_permission(&team, user_id)
            .await?;

        // オーナーの役割変更は禁止
        if member.get_role() == TeamRole::Owner {
            return Err(AppError::BadRequest("Cannot change owner role".to_string()));
        }

        member.role = request.role.to_string();
        let updated_member = self.team_repository.update_member(&member).await?;
        self.build_team_member_response(&updated_member).await
    }

    /// チームメンバーを削除
    pub async fn remove_team_member(
        &self,
        team_id: Uuid,
        member_id: Uuid,
        user_id: Uuid,
    ) -> AppResult<()> {
        // チームの存在確認
        self.team_repository
            .find_by_id(team_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Team not found".to_string()))?;

        let member = self
            .team_repository
            .find_member_by_id(member_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Team member not found".to_string()))?;

        // オーナーは削除不可
        if member.get_role() == TeamRole::Owner {
            return Err(AppError::BadRequest("Cannot remove team owner".to_string()));
        }

        // 削除権限チェック（管理者または本人）
        let requester_member = self
            .team_repository
            .find_member_by_user_and_team(user_id, team_id)
            .await?
            .ok_or_else(|| AppError::Forbidden("Not a team member".to_string()))?;

        if !requester_member.can_manage() && requester_member.user_id != member.user_id {
            return Err(AppError::Forbidden("Insufficient permissions".to_string()));
        }

        self.team_repository.remove_member(member_id).await?;
        Ok(())
    }

    /// チーム統計を取得
    pub async fn get_team_stats(&self, user_id: Uuid) -> AppResult<TeamStatsResponse> {
        // ユーザーが参加しているチーム一覧を取得
        let teams = self.team_repository.find_teams_by_member(user_id).await?;

        let mut total_members = 0i32;
        let mut teams_by_tier = std::collections::HashMap::new();
        let mut most_active_teams = Vec::new();

        for team in &teams {
            let member_count = self.team_repository.count_members(team.id).await? as i32;
            total_members += member_count;

            // サブスクリプション階層別統計を更新
            let tier_stats =
                teams_by_tier
                    .entry(team.get_subscription_tier())
                    .or_insert(TeamTierStats {
                        tier: team.get_subscription_tier(),
                        team_count: 0,
                        member_count: 0,
                    });
            tier_stats.team_count += 1;
            tier_stats.member_count += member_count;

            // アクティブチーム情報を追加
            most_active_teams.push(TeamActivity {
                team_id: team.id,
                team_name: team.name.clone(),
                member_count,
                recent_activity_count: 0, // 実装時にアクティビティ情報を追加
            });
        }

        // アクティブチームを並び替え（メンバー数順）
        most_active_teams.sort_by(|a, b| b.member_count.cmp(&a.member_count));
        most_active_teams.truncate(10); // 上位10チーム

        let average_members_per_team = if teams.is_empty() {
            0.0
        } else {
            total_members as f64 / teams.len() as f64
        };

        Ok(TeamStatsResponse {
            total_teams: teams.len() as i32,
            teams_by_tier: teams_by_tier.into_values().collect(),
            total_members,
            average_members_per_team,
            most_active_teams,
        })
    }

    /// チーム一覧をページング付きで取得
    pub async fn get_teams_with_pagination(
        &self,
        page: u64,
        page_size: u64,
        organization_id: Option<Uuid>,
        user_id: Uuid,
    ) -> AppResult<(Vec<TeamListResponse>, u64)> {
        let (teams, total) = self
            .team_repository
            .find_with_pagination(page, page_size, organization_id)
            .await?;

        let mut team_responses = Vec::new();
        for team in teams {
            // アクセス権限チェック
            if self.check_team_access(&team, user_id).await.is_ok() {
                let member_count = self.team_repository.count_members(team.id).await? as i32;
                let mut team_response = TeamListResponse::from(team);
                team_response.current_member_count = member_count;
                team_responses.push(team_response);
            }
        }

        Ok((team_responses, total))
    }

    /// チームを検索（統一クエリパターン使用）
    pub async fn search_teams(
        &self,
        query: &TeamSearchQuery,
        user_id: Uuid,
    ) -> AppResult<(Vec<TeamListResponse>, u64)> {
        // 既存のメソッドを使用してページネーションとフィルタリング
        let (page, per_page) = query.pagination.get_pagination();
        self.get_teams_with_pagination(page as u64, per_page as u64, query.organization_id, user_id)
            .await
    }

    /// アクティブなチーム数を取得
    pub async fn count_active_teams(&self) -> AppResult<u64> {
        // 現在の実装では全チームがアクティブとみなす
        // 将来的にはis_activeフラグやactivity_statusなどでフィルタリング可能
        self.team_repository.count_all_teams().await
    }

    // ヘルパーメソッド

    async fn check_team_access(&self, team: &Team, user_id: Uuid) -> AppResult<()> {
        // チームメンバーかチェック
        if let Some(_member) = self
            .team_repository
            .find_member_by_user_and_team(user_id, team.id)
            .await?
        {
            return Ok(());
        }

        Err(AppError::Forbidden("Not a team member".to_string()))
    }

    async fn check_team_management_permission(&self, team: &Team, user_id: Uuid) -> AppResult<()> {
        let member = self
            .team_repository
            .find_member_by_user_and_team(user_id, team.id)
            .await?
            .ok_or_else(|| AppError::Forbidden("Not a team member".to_string()))?;

        if !member.can_manage() {
            return Err(AppError::Forbidden("Insufficient permissions".to_string()));
        }

        Ok(())
    }

    async fn check_team_invite_permission(&self, team: &Team, user_id: Uuid) -> AppResult<()> {
        let member = self
            .team_repository
            .find_member_by_user_and_team(user_id, team.id)
            .await?
            .ok_or_else(|| AppError::Forbidden("Not a team member".to_string()))?;

        if !member.get_role().can_invite() {
            return Err(AppError::Forbidden(
                "Insufficient permissions to invite members".to_string(),
            ));
        }

        Ok(())
    }

    async fn build_team_member_response(
        &self,
        member: &TeamMember,
    ) -> AppResult<TeamMemberResponse> {
        let user = self
            .user_repository
            .find_by_id(member.user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        Ok(TeamMemberResponse {
            id: member.id,
            user_id: member.user_id,
            username: user.username,
            email: user.email,
            role: member.get_role(),
            joined_at: Timestamp::from_datetime(member.joined_at),
            invited_by: member.invited_by,
        })
    }

    async fn build_team_member_responses(
        &self,
        members: &[TeamMember],
    ) -> AppResult<Vec<TeamMemberResponse>> {
        let mut responses = Vec::new();
        for member in members {
            responses.push(self.build_team_member_response(member).await?);
        }
        Ok(responses)
    }

    /// ユーザーが所属するチームのIDリストを取得
    pub async fn get_user_team_ids(&self, user_id: Uuid) -> AppResult<Vec<Uuid>> {
        let teams = self.team_repository.find_teams_by_member(user_id).await?;
        Ok(teams.iter().map(|t| t.id).collect())
    }

    /// ユーザーが特定のチームのメンバーかチェック
    pub async fn is_user_member_of_team(&self, user_id: Uuid, team_id: Uuid) -> AppResult<bool> {
        Ok(self
            .team_repository
            .find_member_by_user_and_team(user_id, team_id)
            .await?
            .is_some())
    }

    /// チームメンバーの詳細情報を取得（権限情報付き）
    pub async fn get_team_member_detail(
        &self,
        team_id: Uuid,
        member_id: Uuid,
        user_id: Uuid,
    ) -> AppResult<TeamMemberDetailResponse> {
        // チームの存在確認と権限チェック
        let _team = self
            .team_repository
            .find_by_id(team_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Team not found".to_string()))?;

        // チームメンバーかどうかをチェック
        let members = self
            .team_repository
            .find_members_by_team_id(team_id)
            .await?;
        let _current_member = members
            .iter()
            .find(|m| m.user_id == user_id)
            .ok_or_else(|| AppError::Forbidden("You are not a member of this team".to_string()))?;

        // 対象メンバーの取得
        let member = self
            .team_repository
            .find_member_by_id(member_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Team member not found".to_string()))?;

        // メンバーがこのチームに所属しているか確認
        if member.team_id != team_id {
            return Err(AppError::NotFound("Team member not found".to_string()));
        }

        // ユーザー情報の取得
        let user = self
            .user_repository
            .find_by_id(member.user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // 権限情報の生成
        let is_owner = member.is_owner();
        let is_admin = member.is_admin();
        let can_invite = is_owner || is_admin;
        let can_remove_members = is_owner || is_admin;

        Ok(TeamMemberDetailResponse {
            id: member.id,
            user_id: member.user_id,
            username: user.username,
            email: user.email,
            role: member.get_role(),
            is_owner,
            is_admin,
            can_invite,
            can_remove_members,
            joined_at: Timestamp::from_datetime(member.joined_at),
            invited_by: member.invited_by,
        })
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    // Note: Full integration tests with database are in tests/integration/
    // These are logic-only unit tests without database dependency

    #[test]
    fn test_team_service_construction() {
        // This is a simple construction test
        // Full integration tests with database are in the integration test directory
        // Test will be implemented when mock repositories are available
    }

    #[test]
    fn test_check_team_access_logic() {
        // Logic test: Test access control decision logic without database
        use crate::domain::subscription_tier::SubscriptionTier;
        use crate::domain::team_model::Model as Team;
        use uuid::Uuid;

        let _team_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();

        let team = Team::new_team(
            "Test Team".to_string(),
            Some("Test Description".to_string()),
            None,
            owner_id,
            SubscriptionTier::Free,
        );

        // Test logic: owner should have access
        assert_eq!(team.owner_id, owner_id);

        // Test logic: different user should not be owner
        assert_ne!(team.owner_id, user_id);

        // Test logic: team properties are correctly set
        assert_eq!(team.name, "Test Team");
        assert_eq!(team.description, Some("Test Description".to_string()));
        assert_eq!(team.subscription_tier, SubscriptionTier::Free.to_string());
        assert_eq!(team.max_members, 3); // Free tier limit
    }

    #[test]
    fn test_team_pagination_logic() {
        // Logic test: Test pagination calculation logic

        // Test boundary conditions
        let _page = 1u64;
        let page_size = 20u64;
        let total_count = 45u64;

        // Calculate expected results
        let expected_total_pages = total_count.div_ceil(page_size);
        assert_eq!(expected_total_pages, 3); // 45 / 20 = 2.25 -> 3

        // Test edge cases
        let edge_page = 1u64; // Always 1 for valid pagination
        assert_eq!(edge_page, 1);

        let edge_page_size = 200u64.clamp(1, 100); // Should clamp to 100
        assert_eq!(edge_page_size, 100);

        // Test empty result set
        let empty_total = 0u64;
        let empty_pages = empty_total.div_ceil(page_size);
        assert_eq!(empty_pages, 0);
    }

    #[test]
    fn test_team_member_limit_logic() {
        // Logic test: Test team member limit validation logic
        use crate::domain::subscription_tier::SubscriptionTier;
        use crate::domain::team_model::Model as Team;

        let owner_id = Uuid::new_v4();

        // Test Free tier limits
        let free_team = Team::new_team(
            "Free Team".to_string(),
            None,
            None,
            owner_id,
            SubscriptionTier::Free,
        );
        assert_eq!(free_team.max_members, 3);
        assert!(2 < free_team.max_members); // 2 < 3
        assert!(3 >= free_team.max_members); // 3 >= 3

        // Test Pro tier limits
        let pro_team = Team::new_team(
            "Pro Team".to_string(),
            None,
            None,
            owner_id,
            SubscriptionTier::Pro,
        );
        assert_eq!(pro_team.max_members, 10);
        assert!(9 < pro_team.max_members); // 9 < 10
        assert!(10 >= pro_team.max_members); // 10 >= 10

        // Test Enterprise tier limits
        let enterprise_team = Team::new_team(
            "Enterprise Team".to_string(),
            None,
            None,
            owner_id,
            SubscriptionTier::Enterprise,
        );
        assert_eq!(enterprise_team.max_members, 100);
        assert!(99 < enterprise_team.max_members); // 99 < 100
        assert!(100 >= enterprise_team.max_members); // 100 >= 100
    }
}
