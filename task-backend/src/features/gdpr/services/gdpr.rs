// task-backend/src/features/gdpr/service.rs

use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::features::auth::repositories::refresh_token_repository::RefreshTokenRepository;
use crate::features::gdpr::dto::*;
use crate::features::gdpr::models::user_consent::ConsentType;
use crate::features::subscription::repositories::history::SubscriptionHistoryRepository;
use crate::features::task::repositories::task_repository::TaskRepository;
use crate::features::team::repositories::team::TeamRepository;
use crate::features::user::repositories::user::UserRepository;
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};
use sea_orm_migration::sea_orm::IntoActiveModel;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

#[derive(Clone)]
pub struct GdprService {
    db: DbPool,
    user_repo: Arc<UserRepository>,
    task_repo: Arc<TaskRepository>,
    team_repo: Arc<TeamRepository>,
    subscription_history_repo: Arc<SubscriptionHistoryRepository>,
    refresh_token_repo: Arc<RefreshTokenRepository>,
}

impl GdprService {
    pub fn new(db: DbPool) -> Self {
        Self {
            db: db.clone(),
            user_repo: Arc::new(UserRepository::new(db.clone())),
            task_repo: Arc::new(TaskRepository::new(db.clone())),
            team_repo: Arc::new(TeamRepository::new(db.clone())),
            subscription_history_repo: Arc::new(SubscriptionHistoryRepository::new(db.clone())),
            refresh_token_repo: Arc::new(RefreshTokenRepository::new(db.clone())),
        }
    }

    /// Export all user data for GDPR compliance
    pub async fn export_user_data(
        &self,
        user_id: Uuid,
        request: DataExportRequest,
    ) -> AppResult<DataExportResponse> {
        // Get user data
        let user_with_role = self
            .user_repo
            .find_by_id_with_role(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        let user_data = UserDataExport {
            id: user_with_role.id,
            email: user_with_role.email,
            username: user_with_role.username,
            is_active: user_with_role.is_active,
            email_verified: user_with_role.email_verified,
            role_name: user_with_role.role.name.to_string(),
            subscription_tier: user_with_role.subscription_tier,
            last_login_at: user_with_role.last_login_at,
            created_at: user_with_role.created_at,
            updated_at: user_with_role.updated_at,
        };

        // Export tasks if requested
        let tasks = if request.include_tasks {
            let user_tasks = self.task_repo.find_all_for_user(user_id).await?;
            let task_exports: Vec<TaskDataExport> = user_tasks
                .into_iter()
                .map(|task| TaskDataExport {
                    id: task.id,
                    title: task.title,
                    description: task.description,
                    status: task.status,
                    due_date: task.due_date,
                    created_at: task.created_at,
                    updated_at: task.updated_at,
                })
                .collect();
            Some(task_exports)
        } else {
            None
        };

        // Export teams if requested
        let teams = if request.include_teams {
            let user_teams = self.team_repo.find_teams_by_member(user_id).await?;
            let mut team_exports = Vec::new();

            for team in user_teams {
                // Get member info for this team
                if let Ok(Some(member)) = self
                    .team_repo
                    .find_member_by_user_and_team(user_id, team.id)
                    .await
                {
                    team_exports.push(TeamDataExport {
                        id: team.id,
                        name: team.name,
                        description: team.description,
                        role_in_team: member.role,
                        joined_at: member.joined_at,
                    });
                }
            }
            Some(team_exports)
        } else {
            None
        };

        // Export subscription history if requested
        let subscription_history = if request.include_subscription_history {
            let histories = self
                .subscription_history_repo
                .find_by_user_id(user_id)
                .await?;
            let history_exports: Vec<SubscriptionHistoryExport> = histories
                .into_iter()
                .map(|h| SubscriptionHistoryExport {
                    id: h.id,
                    previous_tier: h.previous_tier,
                    new_tier: h.new_tier,
                    changed_at: h.changed_at,
                    reason: h.reason,
                })
                .collect();
            Some(history_exports)
        } else {
            None
        };

        // For now, activity logs are not implemented, so return None
        let activity_logs = None;

        Ok(DataExportResponse {
            user_data,
            tasks,
            teams,
            subscription_history,
            activity_logs,
            exported_at: Utc::now(),
        })
    }

    /// Delete all user data for GDPR compliance
    pub async fn delete_user_data(
        &self,
        user_id: Uuid,
        request: DataDeletionRequest,
    ) -> AppResult<DataDeletionResponse> {
        if !request.confirm_deletion {
            return Err(AppError::ValidationError(
                "Deletion must be confirmed".to_string(),
            ));
        }

        // Verify user exists
        let user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        info!(
            "Starting GDPR data deletion for user: {} ({})",
            user_id, user.email
        );

        let mut deleted_records = DeletedRecordsSummary {
            user_data: false,
            tasks_count: 0,
            teams_count: 0,
            subscription_history_count: 0,
            activity_logs_count: 0,
            refresh_tokens_count: 0,
        };

        // Delete subscription history
        let subscription_history_count = self
            .subscription_history_repo
            .delete_by_user_id(user_id)
            .await?;
        deleted_records.subscription_history_count = subscription_history_count;

        // Delete refresh tokens
        let refresh_tokens_count = self
            .refresh_token_repo
            .revoke_all_user_tokens(user_id)
            .await?;
        deleted_records.refresh_tokens_count = refresh_tokens_count;

        // Delete tasks
        let tasks = self.task_repo.find_all_for_user(user_id).await?;
        for task in &tasks {
            self.task_repo.delete(task.id).await?;
        }
        deleted_records.tasks_count = tasks.len() as u64;

        // Handle teams owned by the user
        let owned_teams = self.team_repo.find_by_owner_id(user_id).await?;
        for team in &owned_teams {
            // Delete the team (this will cascade delete team members and invitations)
            self.team_repo.delete_team(team.id).await?;
        }

        // Remove from teams where user is a member but not owner
        let member_teams = self.team_repo.find_teams_by_member(user_id).await?;
        for team in &member_teams {
            // Skip if this is an owned team (already deleted)
            if owned_teams.iter().any(|t| t.id == team.id) {
                continue;
            }

            // Get member info for this team to get the member_id
            if let Ok(Some(member)) = self
                .team_repo
                .find_member_by_user_and_team(user_id, team.id)
                .await
            {
                self.team_repo.remove_member(member.id).await?;
            }
        }
        deleted_records.teams_count = (owned_teams.len() + member_teams.len()) as u64;

        // Finally, delete the user
        self.user_repo.delete(user_id).await?;
        deleted_records.user_data = true;

        info!(
            "Completed GDPR data deletion for user: {}. Deleted records: {:?}",
            user_id, deleted_records
        );

        Ok(DataDeletionResponse {
            user_id,
            deleted_at: Utc::now(),
            deleted_records,
        })
    }

    /// Get GDPR compliance status for a user
    pub async fn get_compliance_status(
        &self,
        user_id: Uuid,
    ) -> AppResult<ComplianceStatusResponse> {
        // Verify user exists
        self.user_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // Data retention is 90 days by default
        let data_retention_days = 90;

        // Check if deletion has been requested (not implemented in this example)
        let deletion_requested = false;
        let deletion_scheduled_for = None;

        // For now, we don't track last export date
        let last_data_export = None;

        Ok(ComplianceStatusResponse {
            user_id,
            data_retention_days,
            last_data_export,
            deletion_requested,
            deletion_scheduled_for,
        })
    }

    /// Get user consent status
    pub async fn get_consent_status(&self, user_id: Uuid) -> AppResult<ConsentStatusResponse> {
        // Verify user exists
        self.user_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // Get all consent types
        let consent_types = vec![
            ConsentType::DataProcessing,
            ConsentType::Marketing,
            ConsentType::Analytics,
            ConsentType::ThirdPartySharing,
        ];

        let mut consents = Vec::new();

        for consent_type in consent_types {
            // Find existing consent
            let existing_consent = crate::features::gdpr::models::user_consent::Entity::find()
                .filter(crate::features::gdpr::models::user_consent::Column::UserId.eq(user_id))
                .filter(
                    crate::features::gdpr::models::user_consent::Column::ConsentType
                        .eq::<String>(consent_type.clone().into()),
                )
                .one(&self.db)
                .await?;

            let consent_status = if let Some(consent) = existing_consent {
                let consent_type_enum = consent.get_consent_type().unwrap_or(consent_type.clone());
                ConsentStatus {
                    consent_type: consent_type_enum.clone(),
                    is_granted: consent.is_granted,
                    granted_at: consent.granted_at,
                    revoked_at: consent.revoked_at,
                    last_updated: consent.updated_at,
                    display_name: consent_type_enum.display_name().to_string(),
                    description: consent_type_enum.description().to_string(),
                    is_required: consent_type_enum.is_required(),
                }
            } else {
                // No consent record exists, create default status
                ConsentStatus {
                    consent_type: consent_type.clone(),
                    is_granted: false,
                    granted_at: None,
                    revoked_at: None,
                    last_updated: Utc::now(),
                    display_name: consent_type.display_name().to_string(),
                    description: consent_type.description().to_string(),
                    is_required: consent_type.is_required(),
                }
            };

            consents.push(consent_status);
        }

        // Get the most recent update time
        let last_updated = consents
            .iter()
            .map(|c| c.last_updated)
            .max()
            .unwrap_or_else(Utc::now);

        Ok(ConsentStatusResponse {
            user_id,
            consents,
            last_updated,
        })
    }

    /// Update user consents
    pub async fn update_consents(
        &self,
        user_id: Uuid,
        request: ConsentUpdateRequest,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> AppResult<ConsentStatusResponse> {
        // Verify user exists
        self.user_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // Validate required consents
        if let Some(&false) = request.consents.get(&ConsentType::DataProcessing) {
            return Err(AppError::ValidationError(
                "Data processing consent is required and cannot be revoked".to_string(),
            ));
        }

        // Update each consent
        for (consent_type, is_granted) in request.consents {
            // Find existing consent
            let existing_consent = crate::features::gdpr::models::user_consent::Entity::find()
                .filter(crate::features::gdpr::models::user_consent::Column::UserId.eq(user_id))
                .filter(
                    crate::features::gdpr::models::user_consent::Column::ConsentType
                        .eq::<String>(consent_type.clone().into()),
                )
                .one(&self.db)
                .await?;

            if let Some(existing) = existing_consent {
                // Update existing consent
                let mut active_model = existing.into_active_model();
                active_model.is_granted = Set(is_granted);
                active_model.updated_at = Set(Utc::now());
                if is_granted {
                    active_model.granted_at = Set(Some(Utc::now()));
                    active_model.revoked_at = Set(None);
                } else {
                    active_model.revoked_at = Set(Some(Utc::now()));
                }
                active_model.ip_address = Set(ip_address.clone());
                active_model.user_agent = Set(user_agent.clone());
                active_model.update(&self.db).await?;
            } else {
                // Create new consent
                let new_consent = crate::features::gdpr::models::user_consent::Model::new(
                    user_id,
                    consent_type,
                    is_granted,
                    ip_address.clone(),
                    user_agent.clone(),
                );
                crate::features::gdpr::models::user_consent::ActiveModel {
                    id: Set(new_consent.id),
                    user_id: Set(new_consent.user_id),
                    consent_type: Set(new_consent.consent_type),
                    is_granted: Set(new_consent.is_granted),
                    granted_at: Set(new_consent.granted_at),
                    revoked_at: Set(new_consent.revoked_at),
                    ip_address: Set(new_consent.ip_address),
                    user_agent: Set(new_consent.user_agent),
                    created_at: Set(new_consent.created_at),
                    updated_at: Set(new_consent.updated_at),
                }
                .insert(&self.db)
                .await?;
            }
        }

        info!("Updated consents for user: {}", user_id);

        // Return updated consent status
        self.get_consent_status(user_id).await
    }

    /// Update single consent
    pub async fn update_single_consent(
        &self,
        user_id: Uuid,
        request: SingleConsentUpdateRequest,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> AppResult<ConsentStatusResponse> {
        let mut consents = HashMap::new();
        consents.insert(request.consent_type, request.is_granted);

        let update_request = ConsentUpdateRequest {
            consents,
            reason: request.reason,
        };

        self.update_consents(user_id, update_request, ip_address, user_agent)
            .await
    }

    /// Get consent history
    pub async fn get_consent_history(
        &self,
        user_id: Uuid,
        limit: Option<u64>,
    ) -> AppResult<ConsentHistoryResponse> {
        // Verify user exists
        self.user_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        let limit = limit.unwrap_or(100).min(1000);

        // Get consent history
        let consents = crate::features::gdpr::models::user_consent::Entity::find()
            .filter(crate::features::gdpr::models::user_consent::Column::UserId.eq(user_id))
            .order_by_desc(crate::features::gdpr::models::user_consent::Column::UpdatedAt)
            .limit(limit)
            .all(&self.db)
            .await?;

        let total_count = crate::features::gdpr::models::user_consent::Entity::find()
            .filter(crate::features::gdpr::models::user_consent::Column::UserId.eq(user_id))
            .count(&self.db)
            .await?;

        let history: Vec<ConsentHistoryEntry> = consents
            .into_iter()
            .map(|consent| ConsentHistoryEntry {
                id: consent.id,
                consent_type: consent
                    .get_consent_type()
                    .unwrap_or(ConsentType::DataProcessing),
                action: if consent.is_granted {
                    "granted".to_string()
                } else {
                    "revoked".to_string()
                },
                is_granted: consent.is_granted,
                timestamp: consent.updated_at,
                ip_address: consent.ip_address,
                user_agent: consent.user_agent,
            })
            .collect();

        Ok(ConsentHistoryResponse {
            user_id,
            history,
            total_count,
        })
    }
}
