// task-backend/src/service/gdpr_service.rs

use crate::api::dto::gdpr_dto::*;
use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::repository::refresh_token_repository::RefreshTokenRepository;
use crate::repository::subscription_history_repository::SubscriptionHistoryRepository;
use crate::repository::task_repository::TaskRepository;
use crate::repository::team_repository::TeamRepository;
use crate::repository::user_repository::UserRepository;
use chrono::Utc;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

#[derive(Clone)]
pub struct GdprService {
    user_repo: Arc<UserRepository>,
    task_repo: Arc<TaskRepository>,
    team_repo: Arc<TeamRepository>,
    subscription_history_repo: Arc<SubscriptionHistoryRepository>,
    refresh_token_repo: Arc<RefreshTokenRepository>,
}

impl GdprService {
    pub fn new(db: DbPool) -> Self {
        Self {
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

        // Remove from teams
        let teams = self.team_repo.find_teams_by_member(user_id).await?;
        for team in &teams {
            // Get member info for this team to get the member_id
            if let Ok(Some(member)) = self
                .team_repo
                .find_member_by_user_and_team(user_id, team.id)
                .await
            {
                self.team_repo.remove_member(member.id).await?;
            }
        }
        deleted_records.teams_count = teams.len() as u64;

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
}
