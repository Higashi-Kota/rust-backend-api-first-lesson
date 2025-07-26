// src/service/audit_log_service.rs
use crate::domain::audit_log_model::{AuditAction, AuditLogBuilder, AuditResult};
use crate::error::AppResult;
use crate::log_with_context;
use crate::repository::audit_log_repository::AuditLogRepository;
use crate::utils::error_helper::internal_server_error;
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

// 監査ログ記録のためのパラメータ構造体
pub struct LogActionParams {
    pub user_id: Uuid,
    pub action: AuditAction,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub team_id: Option<Uuid>,
    pub organization_id: Option<Uuid>,
    pub details: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub result: AuditResult,
}

// タスク引き継ぎログのためのパラメータ構造体
pub struct TaskTransferParams {
    pub user_id: Uuid,
    pub task_id: Uuid,
    pub previous_assignee: Option<Uuid>,
    pub new_assignee: Uuid,
    pub team_id: Option<Uuid>,
    pub organization_id: Option<Uuid>,
    pub reason: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

// タスク作成ログのためのパラメータ構造体
pub struct TaskCreationParams {
    pub user_id: Uuid,
    pub task_id: Uuid,
    pub task_title: String,
    pub team_id: Option<Uuid>,
    pub organization_id: Option<Uuid>,
    pub visibility: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

pub struct AuditLogService {
    audit_log_repo: Arc<AuditLogRepository>,
}

impl AuditLogService {
    pub fn new(audit_log_repo: Arc<AuditLogRepository>) -> Self {
        Self { audit_log_repo }
    }

    // 監査ログを記録
    pub async fn log_action(&self, params: LogActionParams) -> AppResult<()> {
        let action_str = format!("{:?}", params.action);
        let result_str = format!("{:?}", params.result);

        log_with_context!(
            tracing::Level::DEBUG,
            "Recording audit log",
            "user_id" => params.user_id,
            "action" => &action_str,
            "resource_type" => &params.resource_type,
            "resource_id" => params.resource_id,
            "result" => &result_str
        );
        let mut builder =
            AuditLogBuilder::new(params.user_id, params.action, &params.resource_type)
                .result(params.result);

        if let Some(id) = params.resource_id {
            builder = builder.resource_id(id);
        }
        if let Some(id) = params.team_id {
            builder = builder.team_id(id);
        }
        if let Some(id) = params.organization_id {
            builder = builder.organization_id(id);
        }
        if let Some(d) = params.details {
            builder = builder.details(d);
        }
        if let Some(ip) = params.ip_address {
            builder = builder.ip_address(ip);
        }
        if let Some(agent) = params.user_agent {
            builder = builder.user_agent(agent);
        }

        let audit_log = builder.build();

        self.audit_log_repo.create(audit_log).await.map_err(|e| {
            log_with_context!(
                tracing::Level::ERROR,
                "Failed to create audit log",
                "error" => &e.to_string()
            );
            internal_server_error(
                e,
                "audit_log_service::log_action",
                "Failed to create audit log",
            )
        })?;

        log_with_context!(
            tracing::Level::INFO,
            "Audit log recorded successfully",
            "user_id" => params.user_id,
            "action" => &action_str,
            "resource_type" => &params.resource_type
        );

        Ok(())
    }

    // タスク引き継ぎの監査ログを記録
    pub async fn log_task_transfer(&self, params: TaskTransferParams) -> AppResult<()> {
        log_with_context!(
            tracing::Level::DEBUG,
            "Recording task transfer audit log",
            "user_id" => params.user_id,
            "task_id" => params.task_id,
            "previous_assignee" => params.previous_assignee,
            "new_assignee" => params.new_assignee
        );
        let details = serde_json::json!({
            "previous_assignee": params.previous_assignee,
            "new_assignee": params.new_assignee,
            "reason": params.reason,
            "transferred_by": params.user_id,
        });

        self.log_action(LogActionParams {
            user_id: params.user_id,
            action: AuditAction::TaskTransferred,
            resource_type: "task".to_string(),
            resource_id: Some(params.task_id),
            team_id: params.team_id,
            organization_id: params.organization_id,
            details: Some(details),
            ip_address: params.ip_address,
            user_agent: params.user_agent,
            result: AuditResult::Success,
        })
        .await
    }

    // タスク作成の監査ログを記録
    pub async fn log_task_creation(&self, params: TaskCreationParams) -> AppResult<()> {
        log_with_context!(
            tracing::Level::DEBUG,
            "Recording task creation audit log",
            "user_id" => params.user_id,
            "task_id" => params.task_id,
            "task_title" => &params.task_title,
            "visibility" => &params.visibility
        );
        let details = serde_json::json!({
            "title": params.task_title,
            "visibility": params.visibility,
            "created_by": params.user_id,
        });

        self.log_action(LogActionParams {
            user_id: params.user_id,
            action: AuditAction::TaskCreated,
            resource_type: "task".to_string(),
            resource_id: Some(params.task_id),
            team_id: params.team_id,
            organization_id: params.organization_id,
            details: Some(details),
            ip_address: params.ip_address,
            user_agent: params.user_agent,
            result: AuditResult::Success,
        })
        .await
    }

    // ユーザーの監査ログを取得
    pub async fn get_user_audit_logs(
        &self,
        user_id: Uuid,
        page: u64,
        per_page: u64,
    ) -> AppResult<PaginatedAuditLogs> {
        log_with_context!(
            tracing::Level::DEBUG,
            "Retrieving user audit logs",
            "user_id" => user_id,
            "page" => page,
            "per_page" => per_page
        );
        let offset = (page - 1) * per_page;
        let limit = per_page;

        let logs = self
            .audit_log_repo
            .find_by_user(user_id, limit, offset)
            .await
            .map_err(|e| {
                log_with_context!(
                    tracing::Level::ERROR,
                    "Failed to retrieve user audit logs",
                    "user_id" => user_id,
                    "error" => &e.to_string()
                );
                internal_server_error(
                    e,
                    "audit_log_service::get_user_audit_logs",
                    "Failed to retrieve user audit logs",
                )
            })?;

        let total = self
            .audit_log_repo
            .count_by_user(user_id)
            .await
            .map_err(|e| {
                log_with_context!(
                    tracing::Level::ERROR,
                    "Failed to count user audit logs",
                    "user_id" => user_id,
                    "error" => &e.to_string()
                );
                internal_server_error(
                    e,
                    "audit_log_service::get_user_audit_logs",
                    "Failed to count user audit logs",
                )
            })?;

        Ok(PaginatedAuditLogs {
            logs: logs.into_iter().map(AuditLogDto::from).collect(),
            total,
            page,
            per_page,
            total_pages: total.div_ceil(per_page),
        })
    }

    // チームの監査ログを取得
    pub async fn get_team_audit_logs(
        &self,
        team_id: Uuid,
        page: u64,
        per_page: u64,
    ) -> AppResult<PaginatedAuditLogs> {
        log_with_context!(
            tracing::Level::DEBUG,
            "Retrieving team audit logs",
            "team_id" => team_id,
            "page" => page,
            "per_page" => per_page
        );
        let offset = (page - 1) * per_page;
        let limit = per_page;

        let logs = self
            .audit_log_repo
            .find_by_team(team_id, limit, offset)
            .await
            .map_err(|e| {
                log_with_context!(
                    tracing::Level::ERROR,
                    "Failed to retrieve team audit logs",
                    "team_id" => team_id,
                    "error" => &e.to_string()
                );
                internal_server_error(
                    e,
                    "audit_log_service::get_team_audit_logs",
                    "Failed to retrieve team audit logs",
                )
            })?;

        let total = self
            .audit_log_repo
            .count_by_team(team_id)
            .await
            .map_err(|e| {
                log_with_context!(
                    tracing::Level::ERROR,
                    "Failed to count team audit logs",
                    "team_id" => team_id,
                    "error" => &e.to_string()
                );
                internal_server_error(
                    e,
                    "audit_log_service::get_team_audit_logs",
                    "Failed to count team audit logs",
                )
            })?;

        Ok(PaginatedAuditLogs {
            logs: logs.into_iter().map(AuditLogDto::from).collect(),
            total,
            page,
            per_page,
            total_pages: total.div_ceil(per_page),
        })
    }

    // 古いログの削除（デフォルトは90日以上前のログ）
    pub async fn cleanup_old_logs(&self, days_to_keep: i64) -> AppResult<u64> {
        log_with_context!(
            tracing::Level::DEBUG,
            "Cleaning up old audit logs",
            "days_to_keep" => days_to_keep
        );
        let cutoff_date = Utc::now() - Duration::days(days_to_keep);

        let deleted_count = self
            .audit_log_repo
            .delete_old_logs(cutoff_date)
            .await
            .map_err(|e| {
                log_with_context!(
                    tracing::Level::ERROR,
                    "Failed to delete old audit logs",
                    "error" => &e.to_string()
                );
                internal_server_error(
                    e,
                    "audit_log_service::cleanup_old_logs",
                    "Failed to delete old audit logs",
                )
            })?;

        log_with_context!(
            tracing::Level::INFO,
            "Old audit logs cleaned up successfully",
            "deleted_count" => deleted_count,
            "days_kept" => days_to_keep
        );

        Ok(deleted_count)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuditLogDto {
    pub id: Uuid,
    pub user_id: Uuid,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub team_id: Option<Uuid>,
    pub organization_id: Option<Uuid>,
    pub details: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub result: String,
    pub created_at: i64, // Unix timestamp
}

impl From<crate::domain::audit_log_model::Model> for AuditLogDto {
    fn from(model: crate::domain::audit_log_model::Model) -> Self {
        Self {
            id: model.id,
            user_id: model.user_id,
            action: model.action,
            resource_type: model.resource_type,
            resource_id: model.resource_id,
            team_id: model.team_id,
            organization_id: model.organization_id,
            details: model.details,
            ip_address: model.ip_address,
            user_agent: model.user_agent,
            result: model.result,
            created_at: model.created_at.timestamp(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedAuditLogs {
    pub logs: Vec<AuditLogDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}
