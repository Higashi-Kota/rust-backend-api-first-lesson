// task-backend/src/features/team/repositories/team.rs

use crate::error::{AppError, AppResult};
use crate::features::team::models::team::{
    ActiveModel as TeamActiveModel, Column as TeamColumn, Entity as TeamEntity, Model as Team,
};
use crate::features::team::models::team_member::{
    ActiveModel as TeamMemberActiveModel, Column as TeamMemberColumn, Entity as TeamMemberEntity,
    Model as TeamMember,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait,
    QueryFilter, QueryOrder, Set,
};
use uuid::Uuid;

// Helper function to convert SeaORM errors to AppError
fn map_db_error(err: DbErr) -> AppError {
    AppError::InternalServerError(err.to_string())
}

pub struct TeamRepository {
    db: DatabaseConnection,
}

impl TeamRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// チームを作成
    pub async fn create_team(&self, team: &Team) -> AppResult<Team> {
        let active_model = TeamActiveModel {
            id: Set(team.id),
            name: Set(team.name.clone()),
            description: Set(team.description.clone()),
            organization_id: Set(team.organization_id),
            owner_id: Set(team.owner_id),
            subscription_tier: Set(team.subscription_tier.clone()),
            max_members: Set(team.max_members),
            created_at: Set(team.created_at),
            updated_at: Set(team.updated_at),
        };

        let _result = active_model.insert(&self.db).await.map_err(map_db_error)?;
        Ok(team.clone())
    }

    /// チームをIDで取得
    pub async fn find_by_id(&self, team_id: Uuid) -> AppResult<Option<Team>> {
        let model = TeamEntity::find_by_id(team_id)
            .one(&self.db)
            .await
            .map_err(map_db_error)?;
        Ok(model)
    }

    /// チームを名前で検索
    pub async fn find_by_name(&self, name: &str) -> AppResult<Option<Team>> {
        let model = TeamEntity::find()
            .filter(TeamColumn::Name.eq(name))
            .one(&self.db)
            .await
            .map_err(map_db_error)?;
        Ok(model)
    }

    /// 組織のチーム一覧を取得
    pub async fn find_by_organization_id(&self, org_id: Uuid) -> AppResult<Vec<Team>> {
        let models = TeamEntity::find()
            .filter(TeamColumn::OrganizationId.eq(org_id))
            .order_by_asc(TeamColumn::CreatedAt)
            .all(&self.db)
            .await
            .map_err(map_db_error)?;

        Ok(models)
    }

    /// ユーザーが所有するチーム一覧を取得
    pub async fn find_by_owner_id(&self, owner_id: Uuid) -> AppResult<Vec<Team>> {
        let models = TeamEntity::find()
            .filter(TeamColumn::OwnerId.eq(owner_id))
            .order_by_asc(TeamColumn::CreatedAt)
            .all(&self.db)
            .await
            .map_err(map_db_error)?;

        Ok(models)
    }

    /// ユーザーが所有するチーム数を取得
    pub async fn count_user_owned_teams(&self, owner_id: Uuid) -> AppResult<usize> {
        let count = TeamEntity::find()
            .filter(TeamColumn::OwnerId.eq(owner_id))
            .count(&self.db)
            .await
            .map_err(map_db_error)?;

        Ok(count as usize)
    }

    /// チーム一覧をページングで取得
    pub async fn find_with_pagination(
        &self,
        page: u64,
        page_size: u64,
        organization_id: Option<Uuid>,
    ) -> AppResult<(Vec<Team>, u64)> {
        let mut query = TeamEntity::find();

        if let Some(org_id) = organization_id {
            query = query.filter(TeamColumn::OrganizationId.eq(org_id));
        }

        let paginator = query
            .order_by_asc(TeamColumn::CreatedAt)
            .paginate(&self.db, page_size);

        let total_items = paginator.num_items().await.map_err(map_db_error)?;
        let models = paginator.fetch_page(page - 1).await.map_err(map_db_error)?;

        Ok((models, total_items))
    }

    /// チームを更新
    pub async fn update_team(&self, team: &Team) -> AppResult<Team> {
        let active_model = TeamActiveModel {
            id: Set(team.id),
            name: Set(team.name.clone()),
            description: Set(team.description.clone()),
            organization_id: Set(team.organization_id),
            owner_id: Set(team.owner_id),
            subscription_tier: Set(team.subscription_tier.to_string()),
            max_members: Set(team.max_members),
            created_at: Set(team.created_at),
            updated_at: Set(team.updated_at),
        };

        let _result = active_model.update(&self.db).await.map_err(map_db_error)?;
        Ok(team.clone())
    }

    /// チームを削除
    pub async fn delete_team(&self, team_id: Uuid) -> AppResult<bool> {
        let result = TeamEntity::delete_by_id(team_id)
            .exec(&self.db)
            .await
            .map_err(map_db_error)?;
        Ok(result.rows_affected > 0)
    }

    /// チームメンバーを追加
    pub async fn add_member(&self, member: &TeamMember) -> AppResult<TeamMember> {
        let active_model = TeamMemberActiveModel {
            id: Set(member.id),
            team_id: Set(member.team_id),
            user_id: Set(member.user_id),
            role: Set(member.role.to_string()),
            joined_at: Set(member.joined_at),
            invited_by: Set(member.invited_by),
        };

        let _result = active_model.insert(&self.db).await.map_err(map_db_error)?;
        Ok(member.clone())
    }

    /// チームメンバーを取得
    pub async fn find_member_by_id(&self, member_id: Uuid) -> AppResult<Option<TeamMember>> {
        let model = TeamMemberEntity::find_by_id(member_id)
            .one(&self.db)
            .await
            .map_err(map_db_error)?;
        Ok(model)
    }

    /// チームのメンバー一覧を取得
    pub async fn find_members_by_team_id(&self, team_id: Uuid) -> AppResult<Vec<TeamMember>> {
        let models = TeamMemberEntity::find()
            .filter(TeamMemberColumn::TeamId.eq(team_id))
            .order_by_asc(TeamMemberColumn::JoinedAt)
            .all(&self.db)
            .await
            .map_err(map_db_error)?;

        Ok(models)
    }

    /// ユーザーのチームメンバーシップを取得
    pub async fn find_member_by_user_and_team(
        &self,
        user_id: Uuid,
        team_id: Uuid,
    ) -> AppResult<Option<TeamMember>> {
        let model = TeamMemberEntity::find()
            .filter(TeamMemberColumn::UserId.eq(user_id))
            .filter(TeamMemberColumn::TeamId.eq(team_id))
            .one(&self.db)
            .await
            .map_err(map_db_error)?;
        Ok(model)
    }

    /// ユーザーが参加しているチーム一覧を取得
    pub async fn find_teams_by_member(&self, user_id: Uuid) -> AppResult<Vec<Team>> {
        let team_ids: Vec<Uuid> = TeamMemberEntity::find()
            .filter(TeamMemberColumn::UserId.eq(user_id))
            .all(&self.db)
            .await
            .map_err(map_db_error)?
            .into_iter()
            .map(|model| model.team_id)
            .collect();

        if team_ids.is_empty() {
            return Ok(vec![]);
        }

        let models = TeamEntity::find()
            .filter(TeamColumn::Id.is_in(team_ids))
            .order_by_asc(TeamColumn::CreatedAt)
            .all(&self.db)
            .await
            .map_err(map_db_error)?;

        Ok(models)
    }

    /// チームメンバーを更新
    pub async fn update_member(&self, member: &TeamMember) -> AppResult<TeamMember> {
        let active_model = TeamMemberActiveModel {
            id: Set(member.id),
            team_id: Set(member.team_id),
            user_id: Set(member.user_id),
            role: Set(member.role.to_string()),
            joined_at: Set(member.joined_at),
            invited_by: Set(member.invited_by),
        };

        let _result = active_model.update(&self.db).await.map_err(map_db_error)?;
        Ok(member.clone())
    }

    /// チームメンバーを削除
    pub async fn remove_member(&self, member_id: Uuid) -> AppResult<bool> {
        let result = TeamMemberEntity::delete_by_id(member_id)
            .exec(&self.db)
            .await
            .map_err(map_db_error)?;
        Ok(result.rows_affected > 0)
    }

    /// チームのメンバー数を取得
    pub async fn count_members(&self, team_id: Uuid) -> AppResult<usize> {
        let count = TeamMemberEntity::find()
            .filter(TeamMemberColumn::TeamId.eq(team_id))
            .count(&self.db)
            .await
            .map_err(map_db_error)?;
        Ok(count as usize)
    }

    /// 組織のチーム数を取得
    pub async fn count_teams_by_organization(&self, org_id: Uuid) -> AppResult<u64> {
        let count = TeamEntity::find()
            .filter(TeamColumn::OrganizationId.eq(org_id))
            .count(&self.db)
            .await
            .map_err(map_db_error)?;
        Ok(count)
    }
}
