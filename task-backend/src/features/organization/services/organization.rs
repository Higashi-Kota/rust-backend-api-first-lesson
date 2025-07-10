use super::super::dto::requests::{
    CreateOrganizationRequest, InviteOrganizationMemberRequest, OrganizationSearchQuery,
    UpdateOrganizationMemberRoleRequest, UpdateOrganizationRequest,
    UpdateOrganizationSettingsRequest,
};
use super::super::dto::responses::{
    OrganizationCapacityResponse, OrganizationListResponse,
    OrganizationMemberDetailResponse, OrganizationMemberResponse, OrganizationResponse,
    OrganizationStatsResponse, OrganizationUsageInfo,
};
// TODO: Phase 19でOrganizationActivityを使用するようになったら#[allow(unused_imports)]を削除
#[allow(unused_imports)]
use super::super::dto::responses::OrganizationActivity;
use super::super::models::{Organization, OrganizationMember, OrganizationRole};
use super::super::repositories::OrganizationRepository;
use crate::error::{AppError, AppResult};
use crate::features::auth::repository::user_repository::UserRepository;
use crate::repository::subscription_history_repository::SubscriptionHistoryRepository;
use crate::repository::team_repository::TeamRepository;
use uuid::Uuid;

// TODO: Phase 19で本来の使用箇所が移行されたら#[allow(dead_code)]を削除
#[allow(dead_code)]
pub struct OrganizationService {
    organization_repository: OrganizationRepository,
    team_repository: TeamRepository,
    user_repository: UserRepository,
    subscription_history_repository: SubscriptionHistoryRepository,
}

impl OrganizationService {
    pub fn new(
        organization_repository: OrganizationRepository,
        team_repository: TeamRepository,
        user_repository: UserRepository,
        subscription_history_repository: SubscriptionHistoryRepository,
    ) -> Self {
        Self {
            organization_repository,
            team_repository,
            user_repository,
            subscription_history_repository,
        }
    }

    /// 組織を作成
    pub async fn create_organization(
        &self,
        request: CreateOrganizationRequest,
        owner_id: Uuid,
    ) -> AppResult<OrganizationResponse> {
        // 組織名の重複チェック
        if let Some(_existing) = self
            .organization_repository
            .find_by_name(&request.name)
            .await?
        {
            return Err(AppError::BadRequest(
                "Organization name already exists".to_string(),
            ));
        }

        let organization = Organization::new(
            request.name,
            request.description,
            owner_id,
            request.subscription_tier,
        );

        // 組織を作成
        let created_organization = self
            .organization_repository
            .create_organization(&organization)
            .await?;

        // オーナーをメンバーとして追加
        let owner_member = OrganizationMember::new(
            created_organization.id,
            owner_id,
            OrganizationRole::Owner,
            None,
        );
        self.organization_repository
            .add_member(&owner_member)
            .await?;

        // レスポンスを作成
        let member_response = self
            .build_organization_member_response(&owner_member)
            .await?;
        Ok(OrganizationResponse::from((
            created_organization,
            vec![member_response],
            0,
        )))
    }

    /// 組織詳細を取得
    pub async fn get_organization_by_id(
        &self,
        organization_id: Uuid,
        user_id: Uuid,
    ) -> AppResult<OrganizationResponse> {
        let organization = self
            .organization_repository
            .find_by_id(organization_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Organization not found".to_string()))?;

        // アクセス権限チェック
        self.check_organization_access(&organization, user_id)
            .await?;

        let members = self
            .organization_repository
            .find_members_by_organization_id(organization_id)
            .await?;
        let member_responses = self.build_organization_member_responses(&members).await?;
        let team_count = self
            .team_repository
            .count_teams_by_organization(organization_id)
            .await? as u32;

        Ok(OrganizationResponse::from((
            organization,
            member_responses,
            team_count,
        )))
    }

    /// 組織一覧をページネーション付きで取得
    pub async fn get_organizations_paginated(
        &self,
        query: OrganizationSearchQuery,
        user_id: Uuid,
    ) -> AppResult<(Vec<OrganizationListResponse>, usize)> {
        self.get_organizations_internal(query, Some(user_id)).await
    }

    /// 組織一覧取得の内部実装（共通ロジック）
    async fn get_organizations_internal(
        &self,
        query: OrganizationSearchQuery,
        user_id: Option<Uuid>,
    ) -> AppResult<(Vec<OrganizationListResponse>, usize)> {
        let page = query.page.unwrap_or(1) as i32;
        let page_size = query.page_size.unwrap_or(20) as i32;
        let page_size = std::cmp::min(page_size, 100); // 最大100件に制限

        // 組織を取得
        let all_organizations = if user_id.is_none() {
            // 管理者用: 全組織を取得
            self.organization_repository
                .find_all_organizations()
                .await?
        } else if let Some(owner_id) = query.owner_id {
            // 特定ユーザーが所有する組織
            self.organization_repository
                .find_by_owner_id(owner_id)
                .await?
        } else {
            // ユーザーが参加している組織
            self.organization_repository
                .find_organizations_by_member(user_id.unwrap())
                .await?
        };

        // サブスクリプション階層でフィルタリング（指定されている場合）
        let filtered_organizations = if let Some(tier) = query.subscription_tier {
            all_organizations
                .into_iter()
                .filter(|org| org.subscription_tier == tier)
                .collect()
        } else {
            all_organizations
        };

        let total_count = filtered_organizations.len();
        let offset = ((page - 1) * page_size) as usize;
        let limit = page_size as usize;

        // ページネーション適用
        let organizations = filtered_organizations
            .into_iter()
            .skip(offset)
            .take(limit)
            .collect::<Vec<_>>();

        // レスポンスの構築
        let mut organization_responses = Vec::new();
        for organization in organizations {
            let member_count = self
                .organization_repository
                .count_members(organization.id)
                .await? as u32;
            let team_count = self
                .team_repository
                .count_teams_by_organization(organization.id)
                .await? as u32;

            let mut org_response = OrganizationListResponse::from(organization);
            org_response.current_member_count = member_count;
            org_response.current_team_count = team_count;
            organization_responses.push(org_response);
        }

        Ok((organization_responses, total_count))
    }

    /// 組織を更新
    pub async fn update_organization(
        &self,
        organization_id: Uuid,
        request: UpdateOrganizationRequest,
        user_id: Uuid,
    ) -> AppResult<OrganizationResponse> {
        let mut organization = self
            .organization_repository
            .find_by_id(organization_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Organization not found".to_string()))?;

        // 管理権限チェック
        self.check_organization_management_permission(&organization, user_id)
            .await?;

        // 更新処理
        if let Some(name) = request.name {
            // 名前の重複チェック（異なる組織で）
            if let Some(existing) = self.organization_repository.find_by_name(&name).await? {
                if existing.id != organization_id {
                    return Err(AppError::BadRequest(
                        "Organization name already exists".to_string(),
                    ));
                }
            }
            organization.update_name(name);
        }

        if let Some(description) = request.description {
            organization.update_description(Some(description));
        }

        let updated_organization = self
            .organization_repository
            .update_organization(&organization)
            .await?;
        let members = self
            .organization_repository
            .find_members_by_organization_id(organization_id)
            .await?;
        let member_responses = self.build_organization_member_responses(&members).await?;
        let team_count = self
            .team_repository
            .count_teams_by_organization(organization_id)
            .await? as u32;

        Ok(OrganizationResponse::from((
            updated_organization,
            member_responses,
            team_count,
        )))
    }

    /// 組織設定を更新
    pub async fn update_organization_settings(
        &self,
        organization_id: Uuid,
        request: UpdateOrganizationSettingsRequest,
        user_id: Uuid,
    ) -> AppResult<OrganizationResponse> {
        let mut organization = self
            .organization_repository
            .find_by_id(organization_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Organization not found".to_string()))?;

        // 設定変更権限チェック
        let member = self
            .organization_repository
            .find_member_by_user_and_organization(user_id, organization_id)
            .await?
            .ok_or_else(|| AppError::Forbidden("Not an organization member".to_string()))?;

        if !member.role.can_change_settings() {
            return Err(AppError::Forbidden(
                "Insufficient permissions to change organization settings".to_string(),
            ));
        }

        // 設定を更新
        let mut settings = organization.settings.clone();
        if let Some(allow_public_teams) = request.allow_public_teams {
            settings.allow_public_teams = allow_public_teams;
        }
        if let Some(require_approval) = request.require_approval_for_new_members {
            settings.require_approval_for_new_members = require_approval;
        }
        if let Some(enable_sso) = request.enable_single_sign_on {
            settings.enable_single_sign_on = enable_sso;
        }
        if let Some(default_tier) = request.default_team_subscription_tier {
            settings.default_team_subscription_tier = default_tier;
        }

        organization.update_settings(settings);

        let updated_organization = self
            .organization_repository
            .update_organization(&organization)
            .await?;

        let members = self
            .organization_repository
            .find_members_by_organization_id(organization_id)
            .await?;
        let member_responses = self.build_organization_member_responses(&members).await?;
        let team_count = self
            .team_repository
            .count_teams_by_organization(organization_id)
            .await? as u32;

        Ok(OrganizationResponse::from((
            updated_organization,
            member_responses,
            team_count,
        )))
    }

    /// 組織を削除
    pub async fn delete_organization(&self, organization_id: Uuid, user_id: Uuid) -> AppResult<()> {
        let organization = self
            .organization_repository
            .find_by_id(organization_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Organization not found".to_string()))?;

        // オーナーのみ削除可能
        if organization.owner_id != user_id {
            return Err(AppError::Forbidden(
                "Only the owner can delete the organization".to_string(),
            ));
        }

        // 関連データの存在チェック
        let team_count = self
            .team_repository
            .count_teams_by_organization(organization_id)
            .await?;
        if team_count > 0 {
            return Err(AppError::BadRequest(
                "Cannot delete organization with existing teams".to_string(),
            ));
        }

        self.organization_repository
            .delete_organization(organization_id)
            .await?;
        Ok(())
    }

    /// 組織メンバーを招待
    pub async fn invite_member(
        &self,
        organization_id: Uuid,
        request: InviteOrganizationMemberRequest,
        inviter_id: Uuid,
    ) -> AppResult<OrganizationMemberDetailResponse> {
        let organization = self
            .organization_repository
            .find_by_id(organization_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Organization not found".to_string()))?;

        // 招待権限チェック
        let inviter_member = self
            .organization_repository
            .find_member_by_user_and_organization(inviter_id, organization_id)
            .await?
            .ok_or_else(|| AppError::Forbidden("Not an organization member".to_string()))?;

        if !inviter_member.role.can_invite_members() {
            return Err(AppError::Forbidden(
                "Insufficient permissions to invite members".to_string(),
            ));
        }

        // ユーザーの存在確認
        let user = self
            .user_repository
            .find_by_id(request.user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // 既存メンバーチェック
        if let Some(_existing) = self
            .organization_repository
            .find_member_by_user_and_organization(request.user_id, organization_id)
            .await?
        {
            return Err(AppError::BadRequest(
                "User is already a member of this organization".to_string(),
            ));
        }

        // メンバー数制限チェック
        let current_member_count = self
            .organization_repository
            .count_members(organization_id)
            .await? as u32;
        if !organization.can_add_member(current_member_count) {
            return Err(AppError::BadRequest(format!(
                "Organization member limit reached ({}/{})",
                current_member_count, organization.max_members
            )));
        }

        // メンバーを追加
        let new_member = OrganizationMember::new(
            organization_id,
            request.user_id,
            request.role,
            Some(inviter_id),
        );
        let added_member = self.organization_repository.add_member(&new_member).await?;

        Ok(OrganizationMemberDetailResponse::from((added_member, user)))
    }

    /// 組織メンバーの役割を更新
    pub async fn update_member_role(
        &self,
        organization_id: Uuid,
        member_id: Uuid,
        request: UpdateOrganizationMemberRoleRequest,
        updater_id: Uuid,
    ) -> AppResult<OrganizationMemberDetailResponse> {
        let organization = self
            .organization_repository
            .find_by_id(organization_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Organization not found".to_string()))?;

        // 権限チェック
        let updater_member = self
            .organization_repository
            .find_member_by_user_and_organization(updater_id, organization_id)
            .await?
            .ok_or_else(|| AppError::Forbidden("Not an organization member".to_string()))?;

        if !updater_member.role.can_manage() {
            return Err(AppError::Forbidden(
                "Insufficient permissions to update member roles".to_string(),
            ));
        }

        // メンバーの取得
        let mut member = self
            .organization_repository
            .find_member_by_id(member_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Member not found".to_string()))?;

        if member.organization_id != organization_id {
            return Err(AppError::BadRequest(
                "Member not in this organization".to_string(),
            ));
        }

        // オーナーの役割は変更不可
        if member.user_id == organization.owner_id {
            return Err(AppError::BadRequest(
                "Cannot change the role of the organization owner".to_string(),
            ));
        }

        member.update_role(request.role);
        let updated_member = self.organization_repository.update_member(&member).await?;

        let user = self
            .user_repository
            .find_by_id(updated_member.user_id)
            .await?
            .unwrap();

        Ok(OrganizationMemberDetailResponse::from((
            updated_member,
            user,
        )))
    }

    /// 組織メンバーを削除
    pub async fn remove_member(
        &self,
        organization_id: Uuid,
        member_id: Uuid,
        remover_id: Uuid,
    ) -> AppResult<()> {
        let organization = self
            .organization_repository
            .find_by_id(organization_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Organization not found".to_string()))?;

        // 権限チェック
        let remover_member = self
            .organization_repository
            .find_member_by_user_and_organization(remover_id, organization_id)
            .await?
            .ok_or_else(|| AppError::Forbidden("Not an organization member".to_string()))?;

        // メンバーの取得
        let member = self
            .organization_repository
            .find_member_by_id(member_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Member not found".to_string()))?;

        if member.organization_id != organization_id {
            return Err(AppError::BadRequest(
                "Member not in this organization".to_string(),
            ));
        }

        // オーナーは削除不可
        if member.user_id == organization.owner_id {
            return Err(AppError::BadRequest(
                "Cannot remove the organization owner".to_string(),
            ));
        }

        // 自分自身を削除する場合または管理権限がある場合のみ可能
        if member.user_id != remover_id && !remover_member.role.can_manage() {
            return Err(AppError::Forbidden(
                "Insufficient permissions to remove members".to_string(),
            ));
        }

        self.organization_repository
            .remove_member(member_id)
            .await?;
        Ok(())
    }

    /// 組織の容量情報を取得
    pub async fn get_organization_capacity(
        &self,
        organization_id: Uuid,
        user_id: Uuid,
    ) -> AppResult<OrganizationCapacityResponse> {
        let organization = self
            .organization_repository
            .find_by_id(organization_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Organization not found".to_string()))?;

        // アクセス権限チェック
        self.check_organization_access(&organization, user_id)
            .await?;

        let current_member_count = self
            .organization_repository
            .count_members(organization_id)
            .await? as u32;
        let current_team_count = self
            .team_repository
            .count_teams_by_organization(organization_id)
            .await? as u32;

        Ok(OrganizationCapacityResponse {
            organization_id,
            max_teams: organization.max_teams,
            current_team_count,
            max_members: organization.max_members,
            current_member_count,
            can_add_team: organization.can_add_team(current_team_count),
            can_add_member: organization.can_add_member(current_member_count),
        })
    }

    /// 組織の統計情報を取得
    pub async fn get_organization_stats(
        &self,
        organization_id: Uuid,
        user_id: Uuid,
    ) -> AppResult<OrganizationStatsResponse> {
        let organization = self
            .organization_repository
            .find_by_id(organization_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Organization not found".to_string()))?;

        // 管理権限チェック
        self.check_organization_management_permission(&organization, user_id)
            .await?;

        let members = self
            .organization_repository
            .find_members_by_organization_id(organization_id)
            .await?;
        let team_count = self
            .team_repository
            .count_teams_by_organization(organization_id)
            .await? as u32;

        // ロール別のメンバー数をカウント
        let mut owner_count = 0;
        let mut admin_count = 0;
        let mut member_count = 0;

        for member in &members {
            match member.role {
                OrganizationRole::Owner => owner_count += 1,
                OrganizationRole::Admin => admin_count += 1,
                OrganizationRole::Member => member_count += 1,
            }
        }

        // サブスクリプション履歴から最新のアクティビティを取得
        // TODO: Phase 19でfind_by_entity_idメソッドを実装後、コメントを解除
        let recent_activity = None;
        // let recent_activity = self
        //     .subscription_history_repository
        //     .find_by_entity_id(organization_id, "organization")
        //     .await?
        //     .first()
        //     .map(|history| OrganizationActivity {
        //         activity_type: "subscription_change".to_string(),
        //         description: format!(
        //             "Subscription changed from {:?} to {:?}",
        //             history.previous_tier, history.new_tier
        //         ),
        //         user_id: history.changed_by,
        //         timestamp: history.created_at,
        //     });

        Ok(OrganizationStatsResponse {
            organization_id,
            total_members: members.len() as u32,
            total_teams: team_count,
            owner_count,
            admin_count,
            member_count,
            tier_info: OrganizationUsageInfo {
                current_tier: organization.subscription_tier,
                max_teams_allowed: organization.max_teams,
                max_members_allowed: organization.max_members,
                teams_usage_percentage: (team_count as f32 / organization.max_teams as f32 * 100.0),
                members_usage_percentage: (members.len() as f32 / organization.max_members as f32
                    * 100.0),
            },
            recent_activity,
            created_at: organization.created_at,
            updated_at: organization.updated_at,
        })
    }

    // ヘルパーメソッド

    /// 組織へのアクセス権限をチェック
    async fn check_organization_access(
        &self,
        organization: &Organization,
        user_id: Uuid,
    ) -> AppResult<()> {
        // オーナーまたはメンバーであればアクセス可能
        if organization.owner_id != user_id {
            let is_member = self
                .organization_repository
                .find_member_by_user_and_organization(user_id, organization.id)
                .await?
                .is_some();
            if !is_member {
                return Err(AppError::Forbidden(
                    "No access to this organization".to_string(),
                ));
            }
        }
        Ok(())
    }

    /// 組織の管理権限をチェック
    async fn check_organization_management_permission(
        &self,
        organization: &Organization,
        user_id: Uuid,
    ) -> AppResult<()> {
        if organization.owner_id == user_id {
            return Ok(());
        }

        let member = self
            .organization_repository
            .find_member_by_user_and_organization(user_id, organization.id)
            .await?
            .ok_or_else(|| AppError::Forbidden("Not an organization member".to_string()))?;

        if !member.role.can_manage() {
            return Err(AppError::Forbidden(
                "Insufficient permissions to manage organization".to_string(),
            ));
        }
        Ok(())
    }

    /// OrganizationMemberからOrganizationMemberResponseを構築
    async fn build_organization_member_response(
        &self,
        member: &OrganizationMember,
    ) -> AppResult<OrganizationMemberResponse> {
        let user = self
            .user_repository
            .find_by_id(member.user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
        Ok(OrganizationMemberResponse::from((member.clone(), user)))
    }

    /// 複数のOrganizationMemberからレスポンスを構築
    async fn build_organization_member_responses(
        &self,
        members: &[OrganizationMember],
    ) -> AppResult<Vec<OrganizationMemberResponse>> {
        let mut responses = Vec::new();
        for member in members {
            responses.push(self.build_organization_member_response(member).await?);
        }
        Ok(responses)
    }

    // 管理者向けAPIメソッド

    /// 全組織を取得（管理者用）
    pub async fn get_all_organizations_for_admin(
        &self,
        query: OrganizationSearchQuery,
    ) -> AppResult<(Vec<OrganizationListResponse>, usize)> {
        self.get_organizations_internal(query, None).await
    }
}
