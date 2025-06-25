// task-backend/src/domain/team_invitation_model.rs

use chrono::{DateTime, Utc};
use sea_orm::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "team_invitations")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub team_id: Uuid,
    pub invited_email: String,
    pub invited_user_id: Option<Uuid>,
    pub invited_by_user_id: Uuid,
    pub status: String,
    pub message: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub accepted_at: Option<DateTime<Utc>>,
    pub declined_at: Option<DateTime<Utc>>,
    pub decline_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::team_model::Entity",
        from = "Column::TeamId",
        to = "super::team_model::Column::Id"
    )]
    Team,
    #[sea_orm(
        belongs_to = "crate::domain::user_model::Entity",
        from = "Column::InvitedUserId",
        to = "crate::domain::user_model::Column::Id"
    )]
    InvitedUser,
    #[sea_orm(
        belongs_to = "crate::domain::user_model::Entity",
        from = "Column::InvitedByUserId",
        to = "crate::domain::user_model::Column::Id"
    )]
    InvitedByUser,
}

impl Related<super::team_model::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Team.def()
    }
}

impl Related<crate::domain::user_model::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::InvitedUser.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TeamInvitationStatus {
    Pending,
    Accepted,
    Declined,
    Expired,
    Cancelled,
}

impl std::fmt::Display for TeamInvitationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TeamInvitationStatus::Pending => write!(f, "pending"),
            TeamInvitationStatus::Accepted => write!(f, "accepted"),
            TeamInvitationStatus::Declined => write!(f, "declined"),
            TeamInvitationStatus::Expired => write!(f, "expired"),
            TeamInvitationStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl std::str::FromStr for TeamInvitationStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(TeamInvitationStatus::Pending),
            "accepted" => Ok(TeamInvitationStatus::Accepted),
            "declined" => Ok(TeamInvitationStatus::Declined),
            "expired" => Ok(TeamInvitationStatus::Expired),
            "cancelled" => Ok(TeamInvitationStatus::Cancelled),
            _ => Err(format!("Invalid team invitation status: {}", s)),
        }
    }
}

impl Model {
    pub fn new(
        team_id: Uuid,
        invited_email: String,
        invited_by_user_id: Uuid,
        message: Option<String>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            team_id,
            invited_email,
            invited_user_id: None,
            invited_by_user_id,
            status: TeamInvitationStatus::Pending.to_string(),
            message,
            expires_at,
            accepted_at: None,
            declined_at: None,
            decline_reason: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn get_status(&self) -> TeamInvitationStatus {
        self.status.parse().unwrap_or(TeamInvitationStatus::Pending)
    }

    pub fn is_pending(&self) -> bool {
        self.get_status() == TeamInvitationStatus::Pending
    }

    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            expires_at < Utc::now()
        } else {
            false
        }
    }

    pub fn can_accept(&self) -> bool {
        self.is_pending() && !self.is_expired()
    }

    pub fn can_decline(&self) -> bool {
        self.is_pending() && !self.is_expired()
    }

    pub fn accept(&mut self, user_id: Option<Uuid>) {
        if self.can_accept() {
            self.status = TeamInvitationStatus::Accepted.to_string();
            self.accepted_at = Some(Utc::now());
            self.invited_user_id = user_id;
            self.updated_at = Utc::now();
        }
    }

    pub fn decline(&mut self, reason: Option<String>) {
        if self.can_decline() {
            self.status = TeamInvitationStatus::Declined.to_string();
            self.declined_at = Some(Utc::now());
            self.decline_reason = reason;
            self.updated_at = Utc::now();
        }
    }

    pub fn cancel(&mut self) {
        if self.is_pending() {
            self.status = TeamInvitationStatus::Cancelled.to_string();
            self.updated_at = Utc::now();
        }
    }

    pub fn mark_expired(&mut self) {
        if self.is_pending() && self.is_expired() {
            self.status = TeamInvitationStatus::Expired.to_string();
            self.updated_at = Utc::now();
        }
    }

    #[allow(dead_code)]
    pub fn update_message(&mut self, message: Option<String>) {
        self.message = message;
        self.updated_at = Utc::now();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamInvitation {
    pub id: Uuid,
    pub team_id: Uuid,
    pub invited_email: String,
    pub invited_user_id: Option<Uuid>,
    pub invited_by_user_id: Uuid,
    pub status: TeamInvitationStatus,
    pub message: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub accepted_at: Option<DateTime<Utc>>,
    pub declined_at: Option<DateTime<Utc>>,
    pub decline_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[allow(dead_code)]
impl Model {
    pub fn to_team_invitation(&self) -> Result<TeamInvitation, String> {
        Ok(TeamInvitation {
            id: self.id,
            team_id: self.team_id,
            invited_email: self.invited_email.clone(),
            invited_user_id: self.invited_user_id,
            invited_by_user_id: self.invited_by_user_id,
            status: self.get_status(),
            message: self.message.clone(),
            expires_at: self.expires_at,
            accepted_at: self.accepted_at,
            declined_at: self.declined_at,
            decline_reason: self.decline_reason.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }

    pub fn from_team_invitation(invitation: &TeamInvitation) -> Self {
        Self {
            id: invitation.id,
            team_id: invitation.team_id,
            invited_email: invitation.invited_email.clone(),
            invited_user_id: invitation.invited_user_id,
            invited_by_user_id: invitation.invited_by_user_id,
            status: invitation.status.to_string(),
            message: invitation.message.clone(),
            expires_at: invitation.expires_at,
            accepted_at: invitation.accepted_at,
            declined_at: invitation.declined_at,
            decline_reason: invitation.decline_reason.clone(),
            created_at: invitation.created_at,
            updated_at: invitation.updated_at,
        }
    }
}

#[allow(dead_code)]
impl TeamInvitation {
    pub fn new(
        team_id: Uuid,
        invited_email: String,
        invited_by_user_id: Uuid,
        message: Option<String>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            team_id,
            invited_email,
            invited_user_id: None,
            invited_by_user_id,
            status: TeamInvitationStatus::Pending,
            message,
            expires_at,
            accepted_at: None,
            declined_at: None,
            decline_reason: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn is_pending(&self) -> bool {
        self.status == TeamInvitationStatus::Pending
    }

    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            expires_at < Utc::now()
        } else {
            false
        }
    }

    pub fn can_accept(&self) -> bool {
        self.is_pending() && !self.is_expired()
    }

    pub fn can_decline(&self) -> bool {
        self.is_pending() && !self.is_expired()
    }

    pub fn accept(&mut self, user_id: Option<Uuid>) {
        if self.can_accept() {
            self.status = TeamInvitationStatus::Accepted;
            self.accepted_at = Some(Utc::now());
            self.invited_user_id = user_id;
            self.updated_at = Utc::now();
        }
    }

    pub fn decline(&mut self, reason: Option<String>) {
        if self.can_decline() {
            self.status = TeamInvitationStatus::Declined;
            self.declined_at = Some(Utc::now());
            self.decline_reason = reason;
            self.updated_at = Utc::now();
        }
    }

    #[allow(dead_code)]
    pub fn cancel(&mut self) {
        if self.is_pending() {
            self.status = TeamInvitationStatus::Cancelled;
            self.updated_at = Utc::now();
        }
    }

    #[allow(dead_code)]
    pub fn mark_expired(&mut self) {
        if self.is_pending() && self.is_expired() {
            self.status = TeamInvitationStatus::Expired;
            self.updated_at = Utc::now();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_team_invitation_status_string_conversion() {
        assert_eq!(TeamInvitationStatus::Pending.to_string(), "pending");
        assert_eq!(TeamInvitationStatus::Accepted.to_string(), "accepted");
        assert_eq!(TeamInvitationStatus::Declined.to_string(), "declined");
        assert_eq!(TeamInvitationStatus::Expired.to_string(), "expired");
        assert_eq!(TeamInvitationStatus::Cancelled.to_string(), "cancelled");

        assert_eq!(
            "pending".parse::<TeamInvitationStatus>().unwrap(),
            TeamInvitationStatus::Pending
        );
        assert_eq!(
            "accepted".parse::<TeamInvitationStatus>().unwrap(),
            TeamInvitationStatus::Accepted
        );
        assert_eq!(
            "declined".parse::<TeamInvitationStatus>().unwrap(),
            TeamInvitationStatus::Declined
        );
        assert_eq!(
            "expired".parse::<TeamInvitationStatus>().unwrap(),
            TeamInvitationStatus::Expired
        );
        assert_eq!(
            "cancelled".parse::<TeamInvitationStatus>().unwrap(),
            TeamInvitationStatus::Cancelled
        );

        assert!("invalid".parse::<TeamInvitationStatus>().is_err());
    }

    #[test]
    fn test_model_creation() {
        let team_id = Uuid::new_v4();
        let invited_by_user_id = Uuid::new_v4();
        let invited_email = "test@example.com".to_string();
        let message = Some("Join our team!".to_string());
        let expires_at = Some(Utc::now() + chrono::Duration::days(7));

        let invitation = Model::new(
            team_id,
            invited_email.clone(),
            invited_by_user_id,
            message.clone(),
            expires_at,
        );

        assert_eq!(invitation.team_id, team_id);
        assert_eq!(invitation.invited_email, invited_email);
        assert_eq!(invitation.invited_by_user_id, invited_by_user_id);
        assert_eq!(invitation.message, message);
        assert_eq!(invitation.expires_at, expires_at);
        assert_eq!(invitation.get_status(), TeamInvitationStatus::Pending);
        assert!(invitation.is_pending());
        assert!(!invitation.is_expired());
        assert!(invitation.can_accept());
        assert!(invitation.can_decline());
    }

    #[test]
    fn test_invitation_acceptance() {
        let team_id = Uuid::new_v4();
        let invited_by_user_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let mut invitation = Model::new(
            team_id,
            "test@example.com".to_string(),
            invited_by_user_id,
            None,
            None,
        );

        assert!(invitation.can_accept());
        invitation.accept(Some(user_id));

        assert_eq!(invitation.get_status(), TeamInvitationStatus::Accepted);
        assert_eq!(invitation.invited_user_id, Some(user_id));
        assert!(invitation.accepted_at.is_some());
        assert!(!invitation.can_accept());
        assert!(!invitation.can_decline());
    }

    #[test]
    fn test_invitation_decline() {
        let team_id = Uuid::new_v4();
        let invited_by_user_id = Uuid::new_v4();
        let decline_reason = Some("Not interested".to_string());

        let mut invitation = Model::new(
            team_id,
            "test@example.com".to_string(),
            invited_by_user_id,
            None,
            None,
        );

        assert!(invitation.can_decline());
        invitation.decline(decline_reason.clone());

        assert_eq!(invitation.get_status(), TeamInvitationStatus::Declined);
        assert_eq!(invitation.decline_reason, decline_reason);
        assert!(invitation.declined_at.is_some());
        assert!(!invitation.can_accept());
        assert!(!invitation.can_decline());
    }

    #[test]
    fn test_invitation_expiration() {
        let team_id = Uuid::new_v4();
        let invited_by_user_id = Uuid::new_v4();
        let past_time = Utc::now() - chrono::Duration::days(1);

        let mut invitation = Model::new(
            team_id,
            "test@example.com".to_string(),
            invited_by_user_id,
            None,
            Some(past_time),
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
        let invited_by_user_id = Uuid::new_v4();

        let mut invitation = Model::new(
            team_id,
            "test@example.com".to_string(),
            invited_by_user_id,
            None,
            None,
        );

        assert!(invitation.is_pending());
        invitation.cancel();

        assert_eq!(invitation.get_status(), TeamInvitationStatus::Cancelled);
        assert!(!invitation.can_accept());
        assert!(!invitation.can_decline());
    }

    #[test]
    fn test_team_invitation_struct_creation() {
        let team_id = Uuid::new_v4();
        let invited_by_user_id = Uuid::new_v4();
        let invited_email = "test@example.com".to_string();
        let message = Some("Join our team!".to_string());

        let invitation = TeamInvitation::new(
            team_id,
            invited_email.clone(),
            invited_by_user_id,
            message.clone(),
            None,
        );

        assert_eq!(invitation.team_id, team_id);
        assert_eq!(invitation.invited_email, invited_email);
        assert_eq!(invitation.invited_by_user_id, invited_by_user_id);
        assert_eq!(invitation.message, message);
        assert_eq!(invitation.status, TeamInvitationStatus::Pending);
        assert!(invitation.is_pending());
        assert!(!invitation.is_expired());
        assert!(invitation.can_accept());
        assert!(invitation.can_decline());
    }

    #[test]
    fn test_model_to_struct_conversion() {
        let team_id = Uuid::new_v4();
        let invited_by_user_id = Uuid::new_v4();

        let model = Model::new(
            team_id,
            "test@example.com".to_string(),
            invited_by_user_id,
            Some("Welcome!".to_string()),
            None,
        );

        let invitation = model.to_team_invitation().unwrap();
        assert_eq!(invitation.id, model.id);
        assert_eq!(invitation.team_id, model.team_id);
        assert_eq!(invitation.invited_email, model.invited_email);
        assert_eq!(invitation.message, model.message);
        assert_eq!(invitation.status, model.get_status());

        let converted_model = Model::from_team_invitation(&invitation);
        assert_eq!(converted_model.id, invitation.id);
        assert_eq!(converted_model.team_id, invitation.team_id);
        assert_eq!(converted_model.invited_email, invitation.invited_email);
        assert_eq!(converted_model.message, invitation.message);
        assert_eq!(converted_model.status, invitation.status.to_string());
    }

    #[test]
    fn test_team_invitation_operations() {
        let team_id = Uuid::new_v4();
        let invited_by_user_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let mut invitation = TeamInvitation::new(
            team_id,
            "test@example.com".to_string(),
            invited_by_user_id,
            None,
            None,
        );

        // Test acceptance
        assert!(invitation.can_accept());
        invitation.accept(Some(user_id));
        assert_eq!(invitation.status, TeamInvitationStatus::Accepted);
        assert_eq!(invitation.invited_user_id, Some(user_id));
        assert!(invitation.accepted_at.is_some());

        // Create new invitation for decline test
        let mut invitation2 = TeamInvitation::new(
            team_id,
            "test2@example.com".to_string(),
            invited_by_user_id,
            None,
            None,
        );

        // Test decline
        assert!(invitation2.can_decline());
        invitation2.decline(Some("Not available".to_string()));
        assert_eq!(invitation2.status, TeamInvitationStatus::Declined);
        assert_eq!(
            invitation2.decline_reason,
            Some("Not available".to_string())
        );
        assert!(invitation2.declined_at.is_some());
    }
}
