// task-backend/src/repository/team_invitation_repository.rs

use crate::domain::team_invitation_model::{
    ActiveModel as TeamInvitationActiveModel, Column as TeamInvitationColumn,
    Entity as TeamInvitationEntity, Model as TeamInvitation, TeamInvitationStatus,
};
use crate::error::{AppError, AppResult};
use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait,
    QueryFilter, QueryOrder, Set,
};
use uuid::Uuid;

fn map_db_error(err: DbErr) -> AppError {
    AppError::InternalServerError(err.to_string())
}

pub struct TeamInvitationRepository {
    db: DatabaseConnection,
}

#[allow(dead_code)]
impl TeamInvitationRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    #[allow(dead_code)]
    pub async fn create_invitation(
        &self,
        invitation: &TeamInvitation,
    ) -> AppResult<TeamInvitation> {
        let active_model = TeamInvitationActiveModel {
            id: Set(invitation.id),
            team_id: Set(invitation.team_id),
            invited_email: Set(invitation.invited_email.clone()),
            invited_user_id: Set(invitation.invited_user_id),
            invited_by_user_id: Set(invitation.invited_by_user_id),
            status: Set(invitation.status.clone()),
            message: Set(invitation.message.clone()),
            expires_at: Set(invitation.expires_at),
            accepted_at: Set(invitation.accepted_at),
            declined_at: Set(invitation.declined_at),
            decline_reason: Set(invitation.decline_reason.clone()),
            created_at: Set(invitation.created_at),
            updated_at: Set(invitation.updated_at),
        };

        let _result = active_model.insert(&self.db).await.map_err(map_db_error)?;
        Ok(invitation.clone())
    }

    pub async fn create_bulk_invitations(
        &self,
        invitations: &[TeamInvitation],
    ) -> AppResult<Vec<TeamInvitation>> {
        let mut active_models = Vec::new();
        for invitation in invitations {
            let active_model = TeamInvitationActiveModel {
                id: Set(invitation.id),
                team_id: Set(invitation.team_id),
                invited_email: Set(invitation.invited_email.clone()),
                invited_user_id: Set(invitation.invited_user_id),
                invited_by_user_id: Set(invitation.invited_by_user_id),
                status: Set(invitation.status.clone()),
                message: Set(invitation.message.clone()),
                expires_at: Set(invitation.expires_at),
                accepted_at: Set(invitation.accepted_at),
                declined_at: Set(invitation.declined_at),
                decline_reason: Set(invitation.decline_reason.clone()),
                created_at: Set(invitation.created_at),
                updated_at: Set(invitation.updated_at),
            };
            active_models.push(active_model);
        }

        for active_model in active_models {
            active_model.insert(&self.db).await.map_err(map_db_error)?;
        }

        Ok(invitations.to_vec())
    }

    pub async fn find_by_id(&self, invitation_id: Uuid) -> AppResult<Option<TeamInvitation>> {
        let model = TeamInvitationEntity::find_by_id(invitation_id)
            .one(&self.db)
            .await
            .map_err(map_db_error)?;
        Ok(model)
    }

    pub async fn find_by_team_id(&self, team_id: Uuid) -> AppResult<Vec<TeamInvitation>> {
        let models = TeamInvitationEntity::find()
            .filter(TeamInvitationColumn::TeamId.eq(team_id))
            .order_by_desc(TeamInvitationColumn::CreatedAt)
            .all(&self.db)
            .await
            .map_err(map_db_error)?;

        Ok(models)
    }

    pub async fn find_by_team_id_with_status(
        &self,
        team_id: Uuid,
        status: TeamInvitationStatus,
    ) -> AppResult<Vec<TeamInvitation>> {
        let models = TeamInvitationEntity::find()
            .filter(TeamInvitationColumn::TeamId.eq(team_id))
            .filter(TeamInvitationColumn::Status.eq(status.to_string()))
            .order_by_desc(TeamInvitationColumn::CreatedAt)
            .all(&self.db)
            .await
            .map_err(map_db_error)?;

        Ok(models)
    }

    pub async fn find_by_email(&self, email: &str) -> AppResult<Vec<TeamInvitation>> {
        let models = TeamInvitationEntity::find()
            .filter(TeamInvitationColumn::InvitedEmail.eq(email))
            .order_by_desc(TeamInvitationColumn::CreatedAt)
            .all(&self.db)
            .await
            .map_err(map_db_error)?;

        Ok(models)
    }

    pub async fn find_pending_by_email(&self, email: &str) -> AppResult<Vec<TeamInvitation>> {
        let models = TeamInvitationEntity::find()
            .filter(TeamInvitationColumn::InvitedEmail.eq(email))
            .filter(TeamInvitationColumn::Status.eq(TeamInvitationStatus::Pending.to_string()))
            .order_by_desc(TeamInvitationColumn::CreatedAt)
            .all(&self.db)
            .await
            .map_err(map_db_error)?;

        Ok(models)
    }

    pub async fn find_by_team_and_email(
        &self,
        team_id: Uuid,
        email: &str,
    ) -> AppResult<Option<TeamInvitation>> {
        let model = TeamInvitationEntity::find()
            .filter(TeamInvitationColumn::TeamId.eq(team_id))
            .filter(TeamInvitationColumn::InvitedEmail.eq(email))
            .order_by_desc(TeamInvitationColumn::CreatedAt)
            .one(&self.db)
            .await
            .map_err(map_db_error)?;

        Ok(model)
    }

    pub async fn find_pending_by_team_and_email(
        &self,
        team_id: Uuid,
        email: &str,
    ) -> AppResult<Option<TeamInvitation>> {
        let model = TeamInvitationEntity::find()
            .filter(TeamInvitationColumn::TeamId.eq(team_id))
            .filter(TeamInvitationColumn::InvitedEmail.eq(email))
            .filter(TeamInvitationColumn::Status.eq(TeamInvitationStatus::Pending.to_string()))
            .one(&self.db)
            .await
            .map_err(map_db_error)?;

        Ok(model)
    }

    pub async fn find_expired_invitations(&self) -> AppResult<Vec<TeamInvitation>> {
        let now = Utc::now();
        let models = TeamInvitationEntity::find()
            .filter(TeamInvitationColumn::Status.eq(TeamInvitationStatus::Pending.to_string()))
            .filter(TeamInvitationColumn::ExpiresAt.is_not_null())
            .filter(TeamInvitationColumn::ExpiresAt.lt(now))
            .all(&self.db)
            .await
            .map_err(map_db_error)?;

        Ok(models)
    }

    pub async fn find_with_pagination(
        &self,
        team_id: Uuid,
        page: u64,
        page_size: u64,
        status_filter: Option<TeamInvitationStatus>,
    ) -> AppResult<(Vec<TeamInvitation>, u64)> {
        let mut query =
            TeamInvitationEntity::find().filter(TeamInvitationColumn::TeamId.eq(team_id));

        if let Some(status) = status_filter {
            query = query.filter(TeamInvitationColumn::Status.eq(status.to_string()));
        }

        let paginator = query
            .order_by_desc(TeamInvitationColumn::CreatedAt)
            .paginate(&self.db, page_size);

        let total_items = paginator.num_items().await.map_err(map_db_error)?;
        let models = paginator.fetch_page(page - 1).await.map_err(map_db_error)?;

        Ok((models, total_items))
    }

    pub async fn update_invitation(
        &self,
        invitation: &TeamInvitation,
    ) -> AppResult<TeamInvitation> {
        let active_model = TeamInvitationActiveModel {
            id: Set(invitation.id),
            team_id: Set(invitation.team_id),
            invited_email: Set(invitation.invited_email.clone()),
            invited_user_id: Set(invitation.invited_user_id),
            invited_by_user_id: Set(invitation.invited_by_user_id),
            status: Set(invitation.status.clone()),
            message: Set(invitation.message.clone()),
            expires_at: Set(invitation.expires_at),
            accepted_at: Set(invitation.accepted_at),
            declined_at: Set(invitation.declined_at),
            decline_reason: Set(invitation.decline_reason.clone()),
            created_at: Set(invitation.created_at),
            updated_at: Set(invitation.updated_at),
        };

        let _result = active_model.update(&self.db).await.map_err(map_db_error)?;
        Ok(invitation.clone())
    }

    pub async fn accept_invitation(
        &self,
        invitation_id: Uuid,
        user_id: Option<Uuid>,
    ) -> AppResult<Option<TeamInvitation>> {
        let invitation = self.find_by_id(invitation_id).await?;

        if let Some(mut invitation) = invitation {
            if invitation.can_accept() {
                invitation.accept(user_id);
                let updated = self.update_invitation(&invitation).await?;
                return Ok(Some(updated));
            }
        }

        Ok(None)
    }

    pub async fn decline_invitation(
        &self,
        invitation_id: Uuid,
        reason: Option<String>,
    ) -> AppResult<Option<TeamInvitation>> {
        let invitation = self.find_by_id(invitation_id).await?;

        if let Some(mut invitation) = invitation {
            if invitation.can_decline() {
                invitation.decline(reason);
                let updated = self.update_invitation(&invitation).await?;
                return Ok(Some(updated));
            }
        }

        Ok(None)
    }

    pub async fn cancel_invitation(
        &self,
        invitation_id: Uuid,
    ) -> AppResult<Option<TeamInvitation>> {
        let invitation = self.find_by_id(invitation_id).await?;

        if let Some(mut invitation) = invitation {
            if invitation.is_pending() {
                invitation.cancel();
                let updated = self.update_invitation(&invitation).await?;
                return Ok(Some(updated));
            }
        }

        Ok(None)
    }

    pub async fn mark_expired_invitations(&self) -> AppResult<Vec<TeamInvitation>> {
        let expired_invitations = self.find_expired_invitations().await?;
        let mut updated_invitations = Vec::new();

        for mut invitation in expired_invitations {
            invitation.mark_expired();
            let updated = self.update_invitation(&invitation).await?;
            updated_invitations.push(updated);
        }

        Ok(updated_invitations)
    }

    pub async fn delete_invitation(&self, invitation_id: Uuid) -> AppResult<bool> {
        let result = TeamInvitationEntity::delete_by_id(invitation_id)
            .exec(&self.db)
            .await
            .map_err(map_db_error)?;
        Ok(result.rows_affected > 0)
    }

    pub async fn count_invitations_by_team(&self, team_id: Uuid) -> AppResult<u64> {
        let count = TeamInvitationEntity::find()
            .filter(TeamInvitationColumn::TeamId.eq(team_id))
            .count(&self.db)
            .await
            .map_err(map_db_error)?;
        Ok(count)
    }

    pub async fn count_pending_invitations_by_team(&self, team_id: Uuid) -> AppResult<u64> {
        let count = TeamInvitationEntity::find()
            .filter(TeamInvitationColumn::TeamId.eq(team_id))
            .filter(TeamInvitationColumn::Status.eq(TeamInvitationStatus::Pending.to_string()))
            .count(&self.db)
            .await
            .map_err(map_db_error)?;
        Ok(count)
    }

    pub async fn count_invitations_by_email(&self, email: &str) -> AppResult<u64> {
        let count = TeamInvitationEntity::find()
            .filter(TeamInvitationColumn::InvitedEmail.eq(email))
            .count(&self.db)
            .await
            .map_err(map_db_error)?;
        Ok(count)
    }

    pub async fn find_by_invited_by_user_id(
        &self,
        user_id: Uuid,
    ) -> AppResult<Vec<TeamInvitation>> {
        let models = TeamInvitationEntity::find()
            .filter(TeamInvitationColumn::InvitedByUserId.eq(user_id))
            .order_by_desc(TeamInvitationColumn::CreatedAt)
            .all(&self.db)
            .await
            .map_err(map_db_error)?;

        Ok(models)
    }

    pub async fn find_invitations_created_between(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> AppResult<Vec<TeamInvitation>> {
        let models = TeamInvitationEntity::find()
            .filter(TeamInvitationColumn::CreatedAt.gte(start_date))
            .filter(TeamInvitationColumn::CreatedAt.lte(end_date))
            .order_by_desc(TeamInvitationColumn::CreatedAt)
            .all(&self.db)
            .await
            .map_err(map_db_error)?;

        Ok(models)
    }

    pub async fn bulk_update_status(
        &self,
        invitation_ids: &[Uuid],
        new_status: TeamInvitationStatus,
    ) -> AppResult<u64> {
        let mut updated_count = 0;

        for &invitation_id in invitation_ids {
            if let Some(mut invitation) = self.find_by_id(invitation_id).await? {
                invitation.status = new_status.to_string();
                invitation.updated_at = Utc::now();
                self.update_invitation(&invitation).await?;
                updated_count += 1;
            }
        }

        Ok(updated_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::team_invitation_model::Model;

    fn create_test_invitation(team_id: Uuid, email: &str) -> Model {
        Model::new(
            team_id,
            email.to_string(),
            Uuid::new_v4(),
            Some("Test invitation".to_string()),
            Some(Utc::now() + chrono::Duration::days(7)),
        )
    }

    #[test]
    fn test_create_test_invitation() {
        let team_id = Uuid::new_v4();
        let email = "test@example.com";

        let invitation = create_test_invitation(team_id, email);

        assert_eq!(invitation.team_id, team_id);
        assert_eq!(invitation.invited_email, email);
        assert!(invitation.is_pending());
        assert!(invitation.can_accept());
        assert!(invitation.can_decline());
    }

    #[test]
    fn test_invitation_operations() {
        let team_id = Uuid::new_v4();
        let email = "test@example.com";

        let mut invitation = create_test_invitation(team_id, email);

        // Test acceptance
        assert!(invitation.can_accept());
        invitation.accept(Some(Uuid::new_v4()));
        assert_eq!(invitation.get_status(), TeamInvitationStatus::Accepted);
        assert!(!invitation.can_accept());
        assert!(!invitation.can_decline());

        // Create new invitation for decline test
        let mut invitation2 = create_test_invitation(team_id, "test2@example.com");

        // Test decline
        assert!(invitation2.can_decline());
        invitation2.decline(Some("Not interested".to_string()));
        assert_eq!(invitation2.get_status(), TeamInvitationStatus::Declined);
        assert!(!invitation2.can_accept());
        assert!(!invitation2.can_decline());
    }

    #[test]
    fn test_invitation_expiration() {
        let team_id = Uuid::new_v4();
        let email = "test@example.com";

        let mut invitation = Model::new(
            team_id,
            email.to_string(),
            Uuid::new_v4(),
            None,
            Some(Utc::now() - chrono::Duration::days(1)), // Expired
        );

        assert!(invitation.is_expired());
        assert!(!invitation.can_accept());
        assert!(!invitation.can_decline());

        invitation.mark_expired();
        assert_eq!(invitation.get_status(), TeamInvitationStatus::Expired);
    }

    #[test]
    fn test_invitation_cancellation() {
        let team_id = Uuid::new_v4();
        let email = "test@example.com";

        let mut invitation = create_test_invitation(team_id, email);

        assert!(invitation.is_pending());
        invitation.cancel();
        assert_eq!(invitation.get_status(), TeamInvitationStatus::Cancelled);
        assert!(!invitation.can_accept());
        assert!(!invitation.can_decline());
    }
}
