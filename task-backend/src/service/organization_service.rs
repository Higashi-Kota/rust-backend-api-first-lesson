// task-backend/src/service/organization_service.rs

use crate::api::dto::organization_dto::*;
use crate::domain::organization_model::{Organization, OrganizationMember, OrganizationRole};
use crate::domain::subscription_tier::SubscriptionTier;
use crate::error::{AppError, AppResult};
use crate::repository::organization_repository::OrganizationRepository;
use crate::repository::subscription_history_repository::SubscriptionHistoryRepository;
use crate::repository::team_repository::TeamRepository;
use crate::repository::user_repository::UserRepository;
use uuid::Uuid;

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

    /// 組織のサブスクリプション階層を更新
    pub async fn update_organization_subscription(
        &self,
        organization_id: Uuid,
        new_tier: SubscriptionTier,
        user_id: Uuid,
    ) -> AppResult<OrganizationResponse> {
        let mut organization = self
            .organization_repository
            .find_by_id(organization_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Organization not found".to_string()))?;

        // 管理権限チェック
        if organization.owner_id != user_id {
            return Err(AppError::Forbidden(
                "Only organization owner can update subscription".to_string(),
            ));
        }

        // 現在の階層を記録
        let previous_tier = organization.subscription_tier.as_str().to_string();

        // ダウングレード時の制約チェック
        if new_tier.level() < organization.subscription_tier.level() {
            // 現在のチーム数を確認
            let current_team_count = self
                .team_repository
                .count_teams_by_organization(organization_id)
                .await? as u32;

            // 現在のメンバー数を確認
            let current_member_count = self
                .organization_repository
                .count_members(organization_id)
                .await? as u32;

            // 新しいプランの制限を取得
            let (new_max_teams, new_max_members) = match new_tier {
                SubscriptionTier::Free => (3, 10),
                SubscriptionTier::Pro => (20, 100),
                SubscriptionTier::Enterprise => (100, 1000),
            };

            // チーム数が制限を超えているかチェック
            if current_team_count > new_max_teams {
                return Err(AppError::BadRequest(format!(
                    "Cannot downgrade: Current team count ({}) exceeds {} plan limit ({})",
                    current_team_count,
                    new_tier.as_str(),
                    new_max_teams
                )));
            }

            // メンバー数が制限を超えているかチェック
            if current_member_count > new_max_members {
                return Err(AppError::BadRequest(format!(
                    "Cannot downgrade: Current member count ({}) exceeds {} plan limit ({})",
                    current_member_count,
                    new_tier.as_str(),
                    new_max_members
                )));
            }
        }

        // サブスクリプション階層を更新
        organization.update_subscription_tier(new_tier);

        let updated_organization = self
            .organization_repository
            .update_organization(&organization)
            .await?;

        // サブスクリプション履歴を記録（組織オーナーの履歴として）
        self.subscription_history_repository
            .create(
                organization.owner_id,
                Some(previous_tier),
                new_tier.as_str().to_string(),
                Some(user_id),
                Some("Organization subscription change".to_string()),
            )
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

        // オーナー権限チェック
        if organization.owner_id != user_id {
            return Err(AppError::Forbidden(
                "Only organization owner can delete the organization".to_string(),
            ));
        }

        // 組織に属するチームを削除
        let teams = self
            .team_repository
            .find_by_organization_id(organization_id)
            .await?;
        for team in teams {
            // チームメンバーを削除
            let members = self
                .team_repository
                .find_members_by_team_id(team.id)
                .await?;
            for member in members {
                self.team_repository.remove_member(member.id).await?;
            }
            // チームを削除
            self.team_repository.delete_team(team.id).await?;
        }

        // 組織メンバーを全て削除
        let members = self
            .organization_repository
            .find_members_by_organization_id(organization_id)
            .await?;
        for member in members {
            self.organization_repository
                .remove_member(member.id)
                .await?;
        }

        // 組織を削除
        self.organization_repository
            .delete_organization(organization_id)
            .await?;
        Ok(())
    }

    /// 組織メンバーを招待
    pub async fn invite_organization_member(
        &self,
        organization_id: Uuid,
        request: InviteOrganizationMemberRequest,
        inviter_id: Uuid,
    ) -> AppResult<OrganizationMemberResponse> {
        let organization = self
            .organization_repository
            .find_by_id(organization_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Organization not found".to_string()))?;

        // 招待権限チェック
        self.check_organization_invite_permission(&organization, inviter_id)
            .await?;

        // メンバー数制限チェック
        let current_member_count = self
            .organization_repository
            .count_members(organization_id)
            .await? as u32;
        if !organization.can_add_member(current_member_count) {
            return Err(AppError::BadRequest(
                "Organization member limit exceeded".to_string(),
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
            return Err(AppError::BadRequest(
                "Either user_id or email is required".to_string(),
            ));
        };

        // 既存メンバーチェック
        if let Some(_existing) = self
            .organization_repository
            .find_member_by_user_and_organization(user_id, organization_id)
            .await?
        {
            return Err(AppError::BadRequest(
                "User is already an organization member".to_string(),
            ));
        }

        let member =
            OrganizationMember::new(organization_id, user_id, request.role, Some(inviter_id));
        let created_member = self.organization_repository.add_member(&member).await?;
        self.build_organization_member_response(&created_member)
            .await
    }

    /// 組織メンバーの役割を更新
    pub async fn update_organization_member_role(
        &self,
        organization_id: Uuid,
        member_id: Uuid,
        request: UpdateOrganizationMemberRoleRequest,
        user_id: Uuid,
    ) -> AppResult<OrganizationMemberResponse> {
        let organization = self
            .organization_repository
            .find_by_id(organization_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Organization not found".to_string()))?;

        let mut member = self
            .organization_repository
            .find_member_by_id(member_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Organization member not found".to_string()))?;

        // 管理権限チェック
        self.check_organization_management_permission(&organization, user_id)
            .await?;

        // オーナーの役割変更は禁止
        if member.role == OrganizationRole::Owner {
            return Err(AppError::BadRequest("Cannot change owner role".to_string()));
        }

        member.update_role(request.role);
        let updated_member = self.organization_repository.update_member(&member).await?;
        self.build_organization_member_response(&updated_member)
            .await
    }

    /// 組織メンバーを削除
    pub async fn remove_organization_member(
        &self,
        organization_id: Uuid,
        member_id: Uuid,
        user_id: Uuid,
    ) -> AppResult<()> {
        let _organization = self
            .organization_repository
            .find_by_id(organization_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Organization not found".to_string()))?;

        let member = self
            .organization_repository
            .find_member_by_id(member_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Organization member not found".to_string()))?;

        // オーナーは削除不可
        if member.role == OrganizationRole::Owner {
            return Err(AppError::BadRequest(
                "Cannot remove organization owner".to_string(),
            ));
        }

        // 削除権限チェック（管理者または本人）
        let requester_member = self
            .organization_repository
            .find_member_by_user_and_organization(user_id, organization_id)
            .await?
            .ok_or_else(|| AppError::Forbidden("Not an organization member".to_string()))?;

        if !requester_member.can_manage() && requester_member.user_id != member.user_id {
            return Err(AppError::Forbidden("Insufficient permissions".to_string()));
        }

        self.organization_repository
            .remove_member(member_id)
            .await?;
        Ok(())
    }

    /// 組織統計を取得
    pub async fn get_organization_stats(
        &self,
        user_id: Uuid,
    ) -> AppResult<OrganizationStatsResponse> {
        // ユーザーが参加している組織一覧を取得
        let organizations = self
            .organization_repository
            .find_organizations_by_member(user_id)
            .await?;

        let mut total_teams = 0u32;
        let mut total_members = 0u32;
        let mut organizations_by_tier = std::collections::HashMap::new();
        let mut most_active_organizations = Vec::new();

        for organization in &organizations {
            let member_count = self
                .organization_repository
                .count_members(organization.id)
                .await? as u32;
            let team_count = self
                .team_repository
                .count_teams_by_organization(organization.id)
                .await? as u32;

            total_teams += team_count;
            total_members += member_count;

            // サブスクリプション階層別統計を更新
            let tier_stats = organizations_by_tier
                .entry(organization.subscription_tier)
                .or_insert(OrganizationTierStats {
                    tier: organization.subscription_tier,
                    organization_count: 0,
                    team_count: 0,
                    member_count: 0,
                });
            tier_stats.organization_count += 1;
            tier_stats.team_count += team_count;
            tier_stats.member_count += member_count;

            // アクティブ組織情報を追加
            most_active_organizations.push(OrganizationActivity {
                organization_id: organization.id,
                organization_name: organization.name.clone(),
                team_count,
                member_count,
                recent_activity_count: 0, // 実装時にアクティビティ情報を追加
            });
        }

        // アクティブ組織を並び替え（チーム数 + メンバー数順）
        most_active_organizations
            .sort_by(|a, b| (b.team_count + b.member_count).cmp(&(a.team_count + a.member_count)));
        most_active_organizations.truncate(10); // 上位10組織

        let average_teams_per_organization = if organizations.is_empty() {
            0.0
        } else {
            total_teams as f64 / organizations.len() as f64
        };

        let average_members_per_organization = if organizations.is_empty() {
            0.0
        } else {
            total_members as f64 / organizations.len() as f64
        };

        Ok(OrganizationStatsResponse {
            total_organizations: organizations.len() as u32,
            organizations_by_tier: organizations_by_tier.into_values().collect(),
            total_teams,
            total_members,
            average_teams_per_organization,
            average_members_per_organization,
            most_active_organizations,
        })
    }

    /// 全組織数を取得
    pub async fn count_all_organizations(&self) -> AppResult<u64> {
        self.organization_repository.count_all_organizations().await
    }

    // ヘルパーメソッド

    async fn check_organization_access(
        &self,
        organization: &Organization,
        user_id: Uuid,
    ) -> AppResult<()> {
        // 組織メンバーかチェック
        if let Some(_member) = self
            .organization_repository
            .find_member_by_user_and_organization(user_id, organization.id)
            .await?
        {
            return Ok(());
        }

        Err(AppError::Forbidden(
            "Not an organization member".to_string(),
        ))
    }

    async fn check_organization_management_permission(
        &self,
        organization: &Organization,
        user_id: Uuid,
    ) -> AppResult<()> {
        let member = self
            .organization_repository
            .find_member_by_user_and_organization(user_id, organization.id)
            .await?
            .ok_or_else(|| AppError::Forbidden("Not an organization member".to_string()))?;

        if !member.can_manage() {
            return Err(AppError::Forbidden("Insufficient permissions".to_string()));
        }

        Ok(())
    }

    async fn check_organization_invite_permission(
        &self,
        organization: &Organization,
        user_id: Uuid,
    ) -> AppResult<()> {
        let member = self
            .organization_repository
            .find_member_by_user_and_organization(user_id, organization.id)
            .await?
            .ok_or_else(|| AppError::Forbidden("Not an organization member".to_string()))?;

        if !member.role.can_invite_members() {
            return Err(AppError::Forbidden(
                "Insufficient permissions to invite members".to_string(),
            ));
        }

        Ok(())
    }

    async fn build_organization_member_response(
        &self,
        member: &OrganizationMember,
    ) -> AppResult<OrganizationMemberResponse> {
        let user = self
            .user_repository
            .find_by_id(member.user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        Ok(OrganizationMemberResponse {
            id: member.id,
            user_id: member.user_id,
            username: user.username,
            email: user.email,
            role: member.role.clone(),
            joined_at: member.joined_at,
            invited_by: member.invited_by,
        })
    }

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

    /// 組織メンバーの詳細情報を取得（権限情報付き）
    pub async fn get_organization_member_detail(
        &self,
        organization_id: Uuid,
        member_id: Uuid,
        user_id: Uuid,
    ) -> AppResult<OrganizationMemberDetailResponse> {
        // 組織の存在確認と権限チェック
        let organization = self
            .organization_repository
            .find_by_id(organization_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Organization not found".to_string()))?;

        // 組織メンバーかどうかをチェック
        self.check_organization_access(&organization, user_id)
            .await?;

        // 対象メンバーの取得
        let member = self
            .organization_repository
            .find_member_by_id(member_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Organization member not found".to_string()))?;

        // メンバーがこの組織に所属しているか確認
        if member.organization_id != organization_id {
            return Err(AppError::NotFound(
                "Organization member not found".to_string(),
            ));
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
        let can_manage = member.can_manage();
        let can_create_teams = member.role.can_create_teams();
        let can_invite_members = member.role.can_invite_members();
        let can_change_settings = member.role.can_change_settings();

        Ok(OrganizationMemberDetailResponse {
            id: member.id,
            user_id: member.user_id,
            username: user.username,
            email: user.email,
            role: member.role,
            is_owner,
            is_admin,
            can_manage,
            can_create_teams,
            can_invite_members,
            can_change_settings,
            joined_at: member.joined_at,
            invited_by: member.invited_by,
        })
    }

    /// 組織容量チェック
    pub async fn check_organization_capacity(
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

        let current_team_count = self
            .team_repository
            .count_teams_by_organization(organization_id)
            .await? as u32;

        let current_member_count = self
            .organization_repository
            .count_members(organization_id)
            .await? as u32;

        let can_add_teams = organization.can_add_team(current_team_count);
        let can_add_members = organization.can_add_member(current_member_count);

        let utilization_percentage = {
            let team_utilization = current_team_count as f64 / organization.max_teams as f64;
            let member_utilization = current_member_count as f64 / organization.max_members as f64;
            ((team_utilization + member_utilization) / 2.0 * 100.0).round()
        };

        Ok(OrganizationCapacityResponse {
            organization_id: organization.id,
            organization_name: organization.name,
            subscription_tier: organization.subscription_tier,
            max_teams: organization.max_teams,
            current_team_count,
            can_add_teams,
            max_members: organization.max_members,
            current_member_count,
            can_add_members,
            utilization_percentage,
        })
    }
}

#[cfg(test)]
mod tests {

    // Note: Full integration tests with database are in tests/integration/
    // These are logic-only unit tests without database dependency

    #[test]
    fn test_organization_service_construction() {
        // This is a simple construction test
        // Full integration tests with database are in the integration test directory
        // Test will be implemented when mock repositories are available
    }
}
