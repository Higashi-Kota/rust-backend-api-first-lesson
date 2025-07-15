// task-backend/src/service/team_invitation_service.rs

use crate::domain::team_invitation_model::{Model as TeamInvitationModel, TeamInvitationStatus};
use crate::error::{AppError, AppResult};
use crate::repository::team_invitation_repository::TeamInvitationRepository;
use crate::repository::team_repository::TeamRepository;
use crate::repository::user_repository::UserRepository;
use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

pub struct TeamInvitationService {
    team_invitation_repository: TeamInvitationRepository,
    team_repository: TeamRepository,
    user_repository: UserRepository,
}

impl TeamInvitationService {
    pub fn new(
        team_invitation_repository: TeamInvitationRepository,
        team_repository: TeamRepository,
        user_repository: UserRepository,
    ) -> Self {
        Self {
            team_invitation_repository,
            team_repository,
            user_repository,
        }
    }

    pub async fn create_bulk_member_invite(
        &self,
        team_id: Uuid,
        emails: Vec<String>,
        message: Option<String>,
        inviter_id: Uuid,
    ) -> AppResult<Vec<TeamInvitationModel>> {
        let _team = self
            .team_repository
            .find_by_id(team_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Team not found".to_string()))?;

        let expires_at = Some(Utc::now() + Duration::days(7));
        let mut invitations = Vec::new();

        for email in emails {
            if self
                .team_invitation_repository
                .find_pending_by_team_and_email(team_id, &email)
                .await?
                .is_some()
            {
                continue;
            }

            // Check if user with this email already exists
            let invited_user_id = self
                .user_repository
                .find_by_email(&email)
                .await?
                .map(|u| u.id);

            let mut invitation =
                TeamInvitationModel::new(team_id, email, inviter_id, message.clone(), expires_at);
            invitation.invited_user_id = invited_user_id;
            invitations.push(invitation);
        }

        if invitations.is_empty() {
            return Err(AppError::BadRequest(
                "No valid invitations to create".to_string(),
            ));
        }

        let created_invitations = self
            .team_invitation_repository
            .create_bulk_invitations(&invitations)
            .await?;

        Ok(created_invitations)
    }

    pub async fn get_team_invitations(
        &self,
        team_id: Uuid,
        status_filter: Option<TeamInvitationStatus>,
    ) -> AppResult<Vec<TeamInvitationModel>> {
        self.team_repository
            .find_by_id(team_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Team not found".to_string()))?;

        match status_filter {
            Some(status) => {
                self.team_invitation_repository
                    .find_by_team_id_with_status(team_id, status)
                    .await
            }
            None => {
                self.team_invitation_repository
                    .find_by_team_id(team_id)
                    .await
            }
        }
    }

    pub async fn decline_invitation(
        &self,
        team_id: Uuid,
        invitation_id: Uuid,
        reason: Option<String>,
    ) -> AppResult<TeamInvitationModel> {
        let invitation = self
            .team_invitation_repository
            .find_by_id(invitation_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Invitation not found".to_string()))?;

        if invitation.team_id != team_id {
            return Err(AppError::BadRequest(
                "Invitation does not belong to the specified team".to_string(),
            ));
        }

        if !invitation.can_decline() {
            return Err(AppError::BadRequest(
                "Invitation cannot be declined".to_string(),
            ));
        }

        let updated_invitation = self
            .team_invitation_repository
            .decline_invitation(invitation_id, reason)
            .await?
            .ok_or_else(|| {
                AppError::InternalServerError("Failed to decline invitation".to_string())
            })?;

        Ok(updated_invitation)
    }

    pub async fn accept_invitation(
        &self,
        invitation_id: Uuid,
        user_id: Option<Uuid>,
    ) -> AppResult<TeamInvitationModel> {
        let invitation = self
            .team_invitation_repository
            .find_by_id(invitation_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Invitation not found".to_string()))?;

        if !invitation.can_accept() {
            return Err(AppError::BadRequest(
                "Invitation cannot be accepted".to_string(),
            ));
        }

        let updated_invitation = self
            .team_invitation_repository
            .accept_invitation(invitation_id, user_id)
            .await?
            .ok_or_else(|| {
                AppError::InternalServerError("Failed to accept invitation".to_string())
            })?;

        Ok(updated_invitation)
    }

    pub async fn cancel_invitation(
        &self,
        team_id: Uuid,
        invitation_id: Uuid,
        requester_id: Uuid,
    ) -> AppResult<TeamInvitationModel> {
        let invitation = self
            .team_invitation_repository
            .find_by_id(invitation_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Invitation not found".to_string()))?;

        if invitation.team_id != team_id {
            return Err(AppError::BadRequest(
                "Invitation does not belong to the specified team".to_string(),
            ));
        }

        let team = self
            .team_repository
            .find_by_id(team_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Team not found".to_string()))?;

        if team.owner_id != requester_id && invitation.invited_by_user_id != requester_id {
            return Err(AppError::Forbidden(
                "Only team owner or invitation creator can cancel invitation".to_string(),
            ));
        }

        let updated_invitation = self
            .team_invitation_repository
            .cancel_invitation(invitation_id)
            .await?
            .ok_or_else(|| {
                AppError::InternalServerError("Failed to cancel invitation".to_string())
            })?;

        Ok(updated_invitation)
    }

    pub async fn get_user_invitations(&self, email: &str) -> AppResult<Vec<TeamInvitationModel>> {
        self.team_invitation_repository
            .find_pending_by_email(email)
            .await
    }

    pub async fn mark_expired_invitations(&self) -> AppResult<Vec<TeamInvitationModel>> {
        self.team_invitation_repository
            .mark_expired_invitations()
            .await
    }

    pub async fn get_invitation_statistics(
        &self,
        team_id: Uuid,
    ) -> AppResult<TeamInvitationStatistics> {
        let total = self
            .team_invitation_repository
            .count_invitations_by_team(team_id)
            .await?;

        let pending = self
            .team_invitation_repository
            .count_pending_invitations_by_team(team_id)
            .await?;

        let accepted = self
            .team_invitation_repository
            .find_by_team_id_with_status(team_id, TeamInvitationStatus::Accepted)
            .await?
            .len() as u64;

        let declined = self
            .team_invitation_repository
            .find_by_team_id_with_status(team_id, TeamInvitationStatus::Declined)
            .await?
            .len() as u64;

        let expired = self
            .team_invitation_repository
            .find_by_team_id_with_status(team_id, TeamInvitationStatus::Expired)
            .await?
            .len() as u64;

        let cancelled = self
            .team_invitation_repository
            .find_by_team_id_with_status(team_id, TeamInvitationStatus::Cancelled)
            .await?
            .len() as u64;

        Ok(TeamInvitationStatistics {
            total,
            pending,
            accepted,
            declined,
            expired,
            cancelled,
        })
    }

    pub async fn cleanup_old_invitations(&self, older_than_days: u32) -> AppResult<u64> {
        let cutoff_date = Utc::now() - Duration::days(older_than_days as i64);
        let old_invitations = self
            .team_invitation_repository
            .find_invitations_created_between(DateTime::<Utc>::MIN_UTC, cutoff_date)
            .await?;

        let mut deleted_count = 0;
        for invitation in old_invitations {
            if matches!(
                invitation.get_status(),
                TeamInvitationStatus::Declined
                    | TeamInvitationStatus::Expired
                    | TeamInvitationStatus::Cancelled
            ) && self
                .team_invitation_repository
                .delete_invitation(invitation.id)
                .await?
            {
                deleted_count += 1;
            }
        }

        Ok(deleted_count)
    }

    pub async fn resend_invitation(
        &self,
        invitation_id: Uuid,
        new_message: Option<String>,
        requester_id: Uuid,
    ) -> AppResult<TeamInvitationModel> {
        let mut invitation = self
            .team_invitation_repository
            .find_by_id(invitation_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Invitation not found".to_string()))?;

        let team = self
            .team_repository
            .find_by_id(invitation.team_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Team not found".to_string()))?;

        if team.owner_id != requester_id && invitation.invited_by_user_id != requester_id {
            return Err(AppError::Forbidden(
                "Only team owner or invitation creator can resend invitation".to_string(),
            ));
        }

        if !invitation.is_pending() {
            return Err(AppError::BadRequest(
                "Only pending invitations can be resent".to_string(),
            ));
        }

        if let Some(message) = new_message {
            invitation.update_message(Some(message));
        }

        invitation.expires_at = Some(Utc::now() + Duration::days(7));
        invitation.updated_at = Utc::now();

        let updated_invitation = self
            .team_invitation_repository
            .update_invitation(&invitation)
            .await?;

        Ok(updated_invitation)
    }

    pub async fn get_invitations_by_creator(
        &self,
        creator_id: Uuid,
    ) -> AppResult<Vec<TeamInvitationModel>> {
        self.team_invitation_repository
            .find_by_invited_by_user_id(creator_id)
            .await
    }

    /// ユーザーのメール宛て招待一覧を取得
    pub async fn get_invitations_by_email(
        &self,
        email: &str,
    ) -> AppResult<Vec<TeamInvitationModel>> {
        self.team_invitation_repository.find_by_email(email).await
    }

    /// 特定チーム・メールの招待を確認
    pub async fn check_team_invitation(
        &self,
        team_id: Uuid,
        email: &str,
    ) -> AppResult<Option<TeamInvitationModel>> {
        self.team_invitation_repository
            .find_by_team_and_email(team_id, email)
            .await
    }

    /// 招待一覧をページング付きで取得
    pub async fn get_invitations_with_pagination(
        &self,
        team_id: Uuid,
        page: u64,
        page_size: u64,
        status_filter: Option<TeamInvitationStatus>,
        requester_id: Uuid,
    ) -> AppResult<(Vec<TeamInvitationModel>, u64)> {
        // チーム権限確認
        if !self
            .validate_invitation_permissions(team_id, requester_id)
            .await?
        {
            return Err(AppError::Forbidden(
                "You do not have permission to view team invitations".to_string(),
            ));
        }

        self.team_invitation_repository
            .find_with_pagination(team_id, page, page_size, status_filter)
            .await
    }

    /// ユーザーの招待数を取得
    pub async fn count_user_invitations(&self, email: &str) -> AppResult<u64> {
        self.team_invitation_repository
            .count_invitations_by_email(email)
            .await
    }

    /// 招待の一括ステータス更新
    pub async fn bulk_update_invitation_status(
        &self,
        invitation_ids: &[Uuid],
        new_status: TeamInvitationStatus,
        requester_id: Uuid,
    ) -> AppResult<u64> {
        // 各招待の権限確認
        for &invitation_id in invitation_ids {
            let invitation = self
                .team_invitation_repository
                .find_by_id(invitation_id)
                .await?
                .ok_or_else(|| AppError::NotFound("Invitation not found".to_string()))?;

            if !self
                .validate_invitation_permissions(invitation.team_id, requester_id)
                .await?
            {
                return Err(AppError::Forbidden(
                    "You do not have permission to update these invitations".to_string(),
                ));
            }
        }

        self.team_invitation_repository
            .bulk_update_status(invitation_ids, new_status)
            .await
    }

    /// 単一招待を作成
    pub async fn create_single_invitation(
        &self,
        team_id: Uuid,
        email: String,
        message: Option<String>,
        inviter_id: Uuid,
    ) -> AppResult<TeamInvitationModel> {
        // チーム存在確認
        let _team = self
            .team_repository
            .find_by_id(team_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Team not found".to_string()))?;

        // 権限確認
        if !self
            .validate_invitation_permissions(team_id, inviter_id)
            .await?
        {
            return Err(AppError::Forbidden(
                "You do not have permission to invite members to this team".to_string(),
            ));
        }

        // 既存招待確認
        if self
            .team_invitation_repository
            .find_pending_by_team_and_email(team_id, &email)
            .await?
            .is_some()
        {
            return Err(AppError::BadRequest(
                "Pending invitation already exists for this email".to_string(),
            ));
        }

        // Check if user with this email already exists
        let invited_user_id = self
            .user_repository
            .find_by_email(&email)
            .await?
            .map(|u| u.id);

        let expires_at = Some(chrono::Utc::now() + chrono::Duration::days(7));
        let mut invitation =
            TeamInvitationModel::new(team_id, email, inviter_id, message, expires_at);
        invitation.invited_user_id = invited_user_id;

        self.team_invitation_repository
            .create_invitation(&invitation)
            .await
    }

    pub async fn validate_invitation_permissions(
        &self,
        team_id: Uuid,
        requester_id: Uuid,
    ) -> AppResult<bool> {
        let team = self
            .team_repository
            .find_by_id(team_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Team not found".to_string()))?;

        let member = self
            .team_repository
            .find_member_by_user_and_team(requester_id, team_id)
            .await?;

        match member {
            Some(member) => {
                let role = member
                    .role
                    .parse::<crate::domain::team_model::TeamRole>()
                    .map_err(|_| AppError::BadRequest("Invalid team role".to_string()))?;
                Ok(role.can_invite())
            }
            None => Ok(team.owner_id == requester_id),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TeamInvitationStatistics {
    pub total: u64,
    pub pending: u64,
    pub accepted: u64,
    pub declined: u64,
    pub expired: u64,
    pub cancelled: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_team_invitation_statistics() {
        let stats = TeamInvitationStatistics {
            total: 100,
            pending: 20,
            accepted: 65,
            declined: 10,
            expired: 5,
            cancelled: 0,
        };

        assert_eq!(stats.total, 100);
        assert_eq!(stats.pending, 20);
        assert_eq!(stats.accepted, 65);
        assert_eq!(stats.declined, 10);
        assert_eq!(stats.expired, 5);
        assert_eq!(
            stats.pending + stats.accepted + stats.declined + stats.expired,
            100
        );
    }

    #[test]
    fn test_team_invitation_model_operations() {
        let team_id = Uuid::new_v4();
        let email = "test@example.com".to_string();
        let inviter_id = Uuid::new_v4();

        let mut invitation = TeamInvitationModel::new(
            team_id,
            email.clone(),
            inviter_id,
            Some("Welcome to our team!".to_string()),
            Some(Utc::now() + Duration::days(7)),
        );

        assert_eq!(invitation.team_id, team_id);
        assert_eq!(invitation.invited_email, email);
        assert_eq!(invitation.invited_by_user_id, inviter_id);
        assert!(invitation.is_pending());
        assert!(invitation.can_accept());
        assert!(invitation.can_decline());
        assert!(!invitation.is_expired());

        invitation.accept(Some(Uuid::new_v4()));
        assert!(!invitation.is_pending());
        assert!(!invitation.can_decline());
        assert_eq!(invitation.get_status(), TeamInvitationStatus::Accepted);
    }

    #[test]
    fn test_expired_invitation() {
        let team_id = Uuid::new_v4();
        let email = "test@example.com".to_string();
        let inviter_id = Uuid::new_v4();

        let mut invitation = TeamInvitationModel::new(
            team_id,
            email,
            inviter_id,
            None,
            Some(Utc::now() - Duration::days(1)), // Already expired
        );

        assert!(invitation.is_expired());
        assert!(!invitation.can_accept());
        assert!(!invitation.can_decline());

        invitation.mark_expired();
        assert_eq!(invitation.get_status(), TeamInvitationStatus::Expired);
    }
}
