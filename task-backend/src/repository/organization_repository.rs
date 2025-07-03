// task-backend/src/repository/organization_repository.rs

use crate::domain::organization_model::{
    Organization, OrganizationMember, OrganizationRole, OrganizationSettings,
};
use crate::domain::subscription_tier::SubscriptionTier;
use crate::error::{AppError, AppResult};
use chrono::{DateTime, Utc};
use sea_orm::sea_query::{Expr, PostgresQueryBuilder, Query};
use sea_orm::{ConnectionTrait, DatabaseConnection, QueryResult};
use serde_json;
use uuid::Uuid;

#[derive(Clone)]
pub struct OrganizationRepository {
    db: DatabaseConnection,
}

impl OrganizationRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// 組織を作成
    pub async fn create_organization(
        &self,
        organization: &Organization,
    ) -> AppResult<Organization> {
        let settings_json = serde_json::to_string(&organization.settings).map_err(|e| {
            AppError::InternalServerError(format!("Failed to serialize settings: {}", e))
        })?;

        let query = Query::insert()
            .into_table(Alias::new("organizations"))
            .columns([
                Alias::new("id"),
                Alias::new("name"),
                Alias::new("description"),
                Alias::new("owner_id"),
                Alias::new("subscription_tier"),
                Alias::new("max_teams"),
                Alias::new("max_members"),
                Alias::new("settings_json"),
                Alias::new("created_at"),
                Alias::new("updated_at"),
            ])
            .values_panic([
                organization.id.into(),
                organization.name.clone().into(),
                organization.description.clone().into(),
                organization.owner_id.into(),
                organization.subscription_tier.as_str().into(),
                (organization.max_teams as i32).into(),
                (organization.max_members as i32).into(),
                settings_json.into(),
                organization.created_at.into(),
                organization.updated_at.into(),
            ])
            .to_string(PostgresQueryBuilder);

        self.db.execute_unprepared(&query).await.map_err(|e| {
            AppError::InternalServerError(format!("Failed to create organization: {}", e))
        })?;

        Ok(organization.clone())
    }

    /// IDで組織を検索
    pub async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Organization>> {
        let query = Query::select()
            .columns([
                Alias::new("id"),
                Alias::new("name"),
                Alias::new("description"),
                Alias::new("owner_id"),
                Alias::new("subscription_tier"),
                Alias::new("max_teams"),
                Alias::new("max_members"),
                Alias::new("settings_json"),
                Alias::new("created_at"),
                Alias::new("updated_at"),
            ])
            .from(Alias::new("organizations"))
            .and_where(Expr::col(Alias::new("id")).eq(id))
            .to_string(PostgresQueryBuilder);

        let result: Option<QueryResult> = self
            .db
            .query_one(sea_orm::Statement::from_string(
                self.db.get_database_backend(),
                query,
            ))
            .await
            .map_err(|e| {
                AppError::InternalServerError(format!("Failed to find organization: {}", e))
            })?;

        match result {
            Some(row) => {
                let org = self.row_to_organization(row)?;
                Ok(Some(org))
            }
            None => Ok(None),
        }
    }

    /// 名前で組織を検索
    pub async fn find_by_name(&self, name: &str) -> AppResult<Option<Organization>> {
        let query = Query::select()
            .columns([
                Alias::new("id"),
                Alias::new("name"),
                Alias::new("description"),
                Alias::new("owner_id"),
                Alias::new("subscription_tier"),
                Alias::new("max_teams"),
                Alias::new("max_members"),
                Alias::new("settings_json"),
                Alias::new("created_at"),
                Alias::new("updated_at"),
            ])
            .from(Alias::new("organizations"))
            .and_where(Expr::col(Alias::new("name")).eq(name))
            .to_string(PostgresQueryBuilder);

        let result: Option<QueryResult> = self
            .db
            .query_one(sea_orm::Statement::from_string(
                self.db.get_database_backend(),
                query,
            ))
            .await
            .map_err(|e| {
                AppError::InternalServerError(format!("Failed to find organization by name: {}", e))
            })?;

        match result {
            Some(row) => {
                let org = self.row_to_organization(row)?;
                Ok(Some(org))
            }
            None => Ok(None),
        }
    }

    /// オーナーIDで組織一覧を取得
    pub async fn find_by_owner_id(&self, owner_id: Uuid) -> AppResult<Vec<Organization>> {
        let query = Query::select()
            .columns([
                Alias::new("id"),
                Alias::new("name"),
                Alias::new("description"),
                Alias::new("owner_id"),
                Alias::new("subscription_tier"),
                Alias::new("max_teams"),
                Alias::new("max_members"),
                Alias::new("settings_json"),
                Alias::new("created_at"),
                Alias::new("updated_at"),
            ])
            .from(Alias::new("organizations"))
            .and_where(Expr::col(Alias::new("owner_id")).eq(owner_id))
            .to_string(PostgresQueryBuilder);

        let results = self
            .db
            .query_all(sea_orm::Statement::from_string(
                self.db.get_database_backend(),
                query,
            ))
            .await
            .map_err(|e| {
                AppError::InternalServerError(format!(
                    "Failed to find organizations by owner: {}",
                    e
                ))
            })?;

        let mut organizations = Vec::new();
        for row in results {
            organizations.push(self.row_to_organization(row)?);
        }

        Ok(organizations)
    }

    /// ユーザーが参加している組織一覧を取得
    pub async fn find_organizations_by_member(
        &self,
        user_id: Uuid,
    ) -> AppResult<Vec<Organization>> {
        let query = r#"
            SELECT DISTINCT o.id, o.name, o.description, o.owner_id, o.subscription_tier,
                   o.max_teams, o.max_members, o.settings_json, o.created_at, o.updated_at
            FROM organizations o
            LEFT JOIN organization_members om ON o.id = om.organization_id
            WHERE o.owner_id = $1 OR om.user_id = $1
            "#;

        let results = self
            .db
            .query_all(sea_orm::Statement::from_sql_and_values(
                self.db.get_database_backend(),
                query,
                vec![user_id.into()],
            ))
            .await
            .map_err(|e| {
                AppError::InternalServerError(format!(
                    "Failed to find organizations by member: {}",
                    e
                ))
            })?;

        let mut organizations = Vec::new();
        for row in results {
            organizations.push(self.row_to_organization(row)?);
        }

        Ok(organizations)
    }

    /// 全組織を取得（管理者用）
    pub async fn find_all_organizations(&self) -> AppResult<Vec<Organization>> {
        let query = Query::select()
            .columns([
                Alias::new("id"),
                Alias::new("name"),
                Alias::new("description"),
                Alias::new("owner_id"),
                Alias::new("subscription_tier"),
                Alias::new("max_teams"),
                Alias::new("max_members"),
                Alias::new("settings_json"),
                Alias::new("created_at"),
                Alias::new("updated_at"),
            ])
            .from(Alias::new("organizations"))
            .to_string(PostgresQueryBuilder);

        let results = self
            .db
            .query_all(sea_orm::Statement::from_string(
                self.db.get_database_backend(),
                query,
            ))
            .await
            .map_err(|e| {
                AppError::InternalServerError(format!("Failed to find all organizations: {}", e))
            })?;

        let mut organizations = Vec::new();
        for row in results {
            organizations.push(self.row_to_organization(row)?);
        }

        Ok(organizations)
    }

    /// 組織を更新
    pub async fn update_organization(
        &self,
        organization: &Organization,
    ) -> AppResult<Organization> {
        let settings_json = serde_json::to_string(&organization.settings).map_err(|e| {
            AppError::InternalServerError(format!("Failed to serialize settings: {}", e))
        })?;

        let query = Query::update()
            .table(Alias::new("organizations"))
            .values([
                (Alias::new("name"), organization.name.clone().into()),
                (
                    Alias::new("description"),
                    organization.description.clone().into(),
                ),
                (
                    Alias::new("subscription_tier"),
                    organization.subscription_tier.as_str().into(),
                ),
                (
                    Alias::new("max_teams"),
                    (organization.max_teams as i32).into(),
                ),
                (
                    Alias::new("max_members"),
                    (organization.max_members as i32).into(),
                ),
                (Alias::new("settings_json"), settings_json.into()),
                (Alias::new("updated_at"), organization.updated_at.into()),
            ])
            .and_where(Expr::col(Alias::new("id")).eq(organization.id))
            .to_string(PostgresQueryBuilder);

        let result = self.db.execute_unprepared(&query).await.map_err(|e| {
            AppError::InternalServerError(format!("Failed to update organization: {}", e))
        })?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Organization not found".to_string()));
        }

        Ok(organization.clone())
    }

    /// 組織を削除
    pub async fn delete_organization(&self, id: Uuid) -> AppResult<()> {
        let query = Query::delete()
            .from_table(Alias::new("organizations"))
            .and_where(Expr::col(Alias::new("id")).eq(id))
            .to_string(PostgresQueryBuilder);

        self.db.execute_unprepared(&query).await.map_err(|e| {
            AppError::InternalServerError(format!("Failed to delete organization: {}", e))
        })?;

        Ok(())
    }

    /// 組織にメンバーを追加
    pub async fn add_member(&self, member: &OrganizationMember) -> AppResult<OrganizationMember> {
        let query = Query::insert()
            .into_table(Alias::new("organization_members"))
            .columns([
                Alias::new("id"),
                Alias::new("organization_id"),
                Alias::new("user_id"),
                Alias::new("role"),
                Alias::new("joined_at"),
                Alias::new("invited_by"),
            ])
            .values_panic([
                member.id.into(),
                member.organization_id.into(),
                member.user_id.into(),
                member.role.to_string().into(),
                member.joined_at.into(),
                member.invited_by.into(),
            ])
            .to_string(PostgresQueryBuilder);

        self.db
            .execute_unprepared(&query)
            .await
            .map_err(|e| AppError::InternalServerError(format!("Failed to add member: {}", e)))?;

        Ok(member.clone())
    }

    /// 組織メンバーを更新
    pub async fn update_member(
        &self,
        member: &OrganizationMember,
    ) -> AppResult<OrganizationMember> {
        let query = Query::update()
            .table(Alias::new("organization_members"))
            .values([(Alias::new("role"), member.role.to_string().into())])
            .and_where(Expr::col(Alias::new("id")).eq(member.id))
            .to_string(PostgresQueryBuilder);

        let result = self.db.execute_unprepared(&query).await.map_err(|e| {
            AppError::InternalServerError(format!("Failed to update member: {}", e))
        })?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(
                "Organization member not found".to_string(),
            ));
        }

        Ok(member.clone())
    }

    /// 組織メンバーを削除
    pub async fn remove_member(&self, member_id: Uuid) -> AppResult<()> {
        let query = Query::delete()
            .from_table(Alias::new("organization_members"))
            .and_where(Expr::col(Alias::new("id")).eq(member_id))
            .to_string(PostgresQueryBuilder);

        self.db.execute_unprepared(&query).await.map_err(|e| {
            AppError::InternalServerError(format!("Failed to remove member: {}", e))
        })?;

        Ok(())
    }

    /// IDでメンバーを検索
    pub async fn find_member_by_id(&self, id: Uuid) -> AppResult<Option<OrganizationMember>> {
        let query = Query::select()
            .columns([
                Alias::new("id"),
                Alias::new("organization_id"),
                Alias::new("user_id"),
                Alias::new("role"),
                Alias::new("joined_at"),
                Alias::new("invited_by"),
            ])
            .from(Alias::new("organization_members"))
            .and_where(Expr::col(Alias::new("id")).eq(id))
            .to_string(PostgresQueryBuilder);

        let result: Option<QueryResult> = self
            .db
            .query_one(sea_orm::Statement::from_string(
                self.db.get_database_backend(),
                query,
            ))
            .await
            .map_err(|e| AppError::InternalServerError(format!("Failed to find member: {}", e)))?;

        match result {
            Some(row) => {
                let member = self.row_to_member(row)?;
                Ok(Some(member))
            }
            None => Ok(None),
        }
    }

    /// ユーザーと組織でメンバーを検索
    pub async fn find_member_by_user_and_organization(
        &self,
        user_id: Uuid,
        organization_id: Uuid,
    ) -> AppResult<Option<OrganizationMember>> {
        let query = Query::select()
            .columns([
                Alias::new("id"),
                Alias::new("organization_id"),
                Alias::new("user_id"),
                Alias::new("role"),
                Alias::new("joined_at"),
                Alias::new("invited_by"),
            ])
            .from(Alias::new("organization_members"))
            .and_where(Expr::col(Alias::new("user_id")).eq(user_id))
            .and_where(Expr::col(Alias::new("organization_id")).eq(organization_id))
            .to_string(PostgresQueryBuilder);

        let result: Option<QueryResult> = self
            .db
            .query_one(sea_orm::Statement::from_string(
                self.db.get_database_backend(),
                query,
            ))
            .await
            .map_err(|e| {
                AppError::InternalServerError(format!(
                    "Failed to find member by user and org: {}",
                    e
                ))
            })?;

        match result {
            Some(row) => {
                let member = self.row_to_member(row)?;
                Ok(Some(member))
            }
            None => Ok(None),
        }
    }

    /// 組織のメンバー一覧を取得
    pub async fn find_members_by_organization_id(
        &self,
        organization_id: Uuid,
    ) -> AppResult<Vec<OrganizationMember>> {
        let query = Query::select()
            .columns([
                Alias::new("id"),
                Alias::new("organization_id"),
                Alias::new("user_id"),
                Alias::new("role"),
                Alias::new("joined_at"),
                Alias::new("invited_by"),
            ])
            .from(Alias::new("organization_members"))
            .and_where(Expr::col(Alias::new("organization_id")).eq(organization_id))
            .to_string(PostgresQueryBuilder);

        let results = self
            .db
            .query_all(sea_orm::Statement::from_string(
                self.db.get_database_backend(),
                query,
            ))
            .await
            .map_err(|e| {
                AppError::InternalServerError(format!(
                    "Failed to find members by organization: {}",
                    e
                ))
            })?;

        let mut members = Vec::new();
        for row in results {
            members.push(self.row_to_member(row)?);
        }

        Ok(members)
    }

    /// 組織のメンバー数をカウント
    pub async fn count_members(&self, organization_id: Uuid) -> AppResult<i64> {
        let query = "SELECT COUNT(*) as count FROM organization_members WHERE organization_id = $1";

        let result: QueryResult = self
            .db
            .query_one(sea_orm::Statement::from_sql_and_values(
                self.db.get_database_backend(),
                query,
                vec![organization_id.into()],
            ))
            .await
            .map_err(|e| AppError::InternalServerError(format!("Failed to count members: {}", e)))?
            .ok_or_else(|| {
                AppError::InternalServerError("Count query returned no result".to_string())
            })?;

        let count: i64 = result.try_get("", "count").map_err(|e| {
            AppError::InternalServerError(format!("Failed to extract count: {}", e))
        })?;

        Ok(count)
    }

    /// 全組織数を取得
    pub async fn count_all_organizations(&self) -> AppResult<u64> {
        let query = "SELECT COUNT(*) as count FROM organizations";

        let result: QueryResult = self
            .db
            .query_one(sea_orm::Statement::from_string(
                self.db.get_database_backend(),
                query.to_string(),
            ))
            .await
            .map_err(|e| {
                AppError::InternalServerError(format!("Failed to count organizations: {}", e))
            })?
            .ok_or_else(|| {
                AppError::InternalServerError("Count query returned no result".to_string())
            })?;

        let count: i64 = result.try_get("", "count").map_err(|e| {
            AppError::InternalServerError(format!("Failed to extract count: {}", e))
        })?;

        Ok(count as u64)
    }

    // Helper methods
    fn row_to_organization(&self, row: QueryResult) -> AppResult<Organization> {
        let id: Uuid = row
            .try_get("", "id")
            .map_err(|e| AppError::InternalServerError(format!("Failed to get id: {}", e)))?;
        let name: String = row
            .try_get("", "name")
            .map_err(|e| AppError::InternalServerError(format!("Failed to get name: {}", e)))?;
        let description: Option<String> = row.try_get("", "description").map_err(|e| {
            AppError::InternalServerError(format!("Failed to get description: {}", e))
        })?;
        let owner_id: Uuid = row
            .try_get("", "owner_id")
            .map_err(|e| AppError::InternalServerError(format!("Failed to get owner_id: {}", e)))?;
        let subscription_tier_str: String = row.try_get("", "subscription_tier").map_err(|e| {
            AppError::InternalServerError(format!("Failed to get subscription_tier: {}", e))
        })?;
        let max_teams: i32 = row.try_get("", "max_teams").map_err(|e| {
            AppError::InternalServerError(format!("Failed to get max_teams: {}", e))
        })?;
        let max_members: i32 = row.try_get("", "max_members").map_err(|e| {
            AppError::InternalServerError(format!("Failed to get max_members: {}", e))
        })?;
        let settings_json: String = row.try_get("", "settings_json").map_err(|e| {
            AppError::InternalServerError(format!("Failed to get settings_json: {}", e))
        })?;
        let created_at: DateTime<Utc> = row.try_get("", "created_at").map_err(|e| {
            AppError::InternalServerError(format!("Failed to get created_at: {}", e))
        })?;
        let updated_at: DateTime<Utc> = row.try_get("", "updated_at").map_err(|e| {
            AppError::InternalServerError(format!("Failed to get updated_at: {}", e))
        })?;

        let subscription_tier = subscription_tier_str
            .parse::<SubscriptionTier>()
            .map_err(|e| {
                AppError::InternalServerError(format!("Invalid subscription tier: {}", e))
            })?;

        let settings: OrganizationSettings = serde_json::from_str(&settings_json).map_err(|e| {
            AppError::InternalServerError(format!("Failed to parse settings: {}", e))
        })?;

        Ok(Organization {
            id,
            name,
            description,
            owner_id,
            subscription_tier,
            max_teams: max_teams as u32,
            max_members: max_members as u32,
            settings,
            created_at,
            updated_at,
        })
    }

    fn row_to_member(&self, row: QueryResult) -> AppResult<OrganizationMember> {
        let id: Uuid = row
            .try_get("", "id")
            .map_err(|e| AppError::InternalServerError(format!("Failed to get id: {}", e)))?;
        let organization_id: Uuid = row.try_get("", "organization_id").map_err(|e| {
            AppError::InternalServerError(format!("Failed to get organization_id: {}", e))
        })?;
        let user_id: Uuid = row
            .try_get("", "user_id")
            .map_err(|e| AppError::InternalServerError(format!("Failed to get user_id: {}", e)))?;
        let role_str: String = row
            .try_get("", "role")
            .map_err(|e| AppError::InternalServerError(format!("Failed to get role: {}", e)))?;
        let joined_at: DateTime<Utc> = row.try_get("", "joined_at").map_err(|e| {
            AppError::InternalServerError(format!("Failed to get joined_at: {}", e))
        })?;
        let invited_by: Option<Uuid> = row.try_get("", "invited_by").map_err(|e| {
            AppError::InternalServerError(format!("Failed to get invited_by: {}", e))
        })?;

        let role = role_str
            .parse::<OrganizationRole>()
            .map_err(|e| AppError::InternalServerError(format!("Invalid role: {}", e)))?;

        Ok(OrganizationMember {
            id,
            organization_id,
            user_id,
            role,
            joined_at,
            invited_by,
        })
    }
}

use sea_orm::sea_query::Alias;

#[cfg(test)]
mod tests {
    #[test]
    fn test_organization_repository_creation() {
        // テスト実装が必要
    }
}
