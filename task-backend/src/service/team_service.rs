// task-backend/src/service/team_service.rs

use crate::api::dto::team_dto::*;
use crate::domain::team_member_model::Model as TeamMemberModel;
use crate::domain::team_model::{Model as TeamModel, TeamRole};

// Type aliases for domain models
#[allow(dead_code)]
pub type Team = TeamModel;
#[allow(dead_code)]
pub type TeamMember = TeamMemberModel;
use crate::domain::subscription_tier::SubscriptionTier;
use crate::error::{AppError, AppResult};
use crate::repository::team_repository::TeamRepository;
use crate::repository::user_repository::UserRepository;
use uuid::Uuid;

#[allow(dead_code)]
pub struct TeamService {
    team_repository: TeamRepository,
    user_repository: UserRepository,
}

#[allow(dead_code)]
impl TeamService {
    pub fn new(team_repository: TeamRepository, user_repository: UserRepository) -> Self {
        Self {
            team_repository,
            user_repository,
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
            return Err(AppError::ValidationError(
                "Team name already exists".to_string(),
            ));
        }

        // デフォルトのサブスクリプション階層（ユーザーのサブスクリプションに基づいて決定可能）
        let subscription_tier = SubscriptionTier::Free;

        let team = Team::new_team(
            request.name,
            request.description,
            request.organization_id,
            owner_id,
            subscription_tier,
        );

        // チームを作成
        let created_team = self.team_repository.create_team(&team).await?;

        // オーナーをメンバーとして追加
        let owner_member = TeamMember::new_member(created_team.id, owner_id, TeamRole::Owner, None);
        self.team_repository.add_member(&owner_member).await?;

        // レスポンスを作成
        let member_response = self.build_team_member_response(&owner_member).await?;
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
                    return Err(AppError::ValidationError(
                        "Team name already exists".to_string(),
                    ));
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

        // メンバー数制限チェック
        let current_member_count = self.team_repository.count_members(team_id).await? as i32;
        if !team.can_add_member(current_member_count) {
            return Err(AppError::ValidationError(
                "Team member limit exceeded".to_string(),
            ));
        }

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
            return Err(AppError::ValidationError(
                "Either user_id or email is required".to_string(),
            ));
        };

        // 既存メンバーチェック
        if let Some(_existing) = self
            .team_repository
            .find_member_by_user_and_team(user_id, team_id)
            .await?
        {
            return Err(AppError::ValidationError(
                "User is already a team member".to_string(),
            ));
        }

        let member = TeamMember::new_member(team_id, user_id, request.role, Some(inviter_id));
        let created_member = self.team_repository.add_member(&member).await?;
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
            return Err(AppError::ValidationError(
                "Cannot change owner role".to_string(),
            ));
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
            return Err(AppError::ValidationError(
                "Cannot remove team owner".to_string(),
            ));
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
            joined_at: member.joined_at,
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
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    // Note: These would be integration tests with actual database connections
    // For unit tests, we would need to mock the repositories

    #[test]
    fn test_team_service_creation() {
        // Mock dependencies would be created here
        // let team_repo = MockTeamRepository::new();
        // let user_repo = MockUserRepository::new();
        // let service = TeamService::new(team_repo, user_repo);
        // assert!(service is created successfully);
    }
}
