// src/api/dto/team_task_dto.rs

use crate::domain::task_status::TaskStatus;
use crate::domain::task_visibility::TaskVisibility;
use crate::utils::validation::common;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// チームタスク作成リクエスト
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateTeamTaskRequest {
    #[validate(
        length(
            min = common::task::TITLE_MIN_LENGTH,
            max = common::task::TITLE_MAX_LENGTH,
            message = "Task title must be between 1 and 200 characters"
        ),
        custom(function = common::validate_task_title)
    )]
    pub title: String,

    #[validate(length(
        max = common::task::DESCRIPTION_MAX_LENGTH,
        message = "Task description must not exceed 2000 characters"
    ))]
    pub description: Option<String>,

    pub status: Option<TaskStatus>,

    pub priority: Option<String>, // 'low', 'medium', 'high'

    pub due_date: Option<i64>, // Unix timestamp

    #[serde(skip_deserializing)]
    pub team_id: Uuid,

    pub visibility: Option<TaskVisibility>,

    pub assigned_to: Option<Uuid>,
}

/// 組織タスク作成リクエスト
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateOrganizationTaskRequest {
    #[validate(
        length(
            min = common::task::TITLE_MIN_LENGTH,
            max = common::task::TITLE_MAX_LENGTH,
            message = "Task title must be between 1 and 200 characters"
        ),
        custom(function = common::validate_task_title)
    )]
    pub title: String,

    #[validate(length(
        max = common::task::DESCRIPTION_MAX_LENGTH,
        message = "Task description must not exceed 2000 characters"
    ))]
    pub description: Option<String>,

    pub status: Option<TaskStatus>,

    pub priority: Option<String>, // 'low', 'medium', 'high'

    pub due_date: Option<i64>, // Unix timestamp

    #[serde(skip_deserializing)]
    pub organization_id: Uuid,

    pub assigned_to: Option<Uuid>,
}

/// タスク割り当てリクエスト
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AssignTaskRequest {
    pub assigned_to: Option<Uuid>,
}

/// タスク可視性更新リクエスト
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateTaskVisibilityRequest {
    pub visibility: TaskVisibility,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<Uuid>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<Uuid>,
}

/// タスクフィルタリングパラメータ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskFilterParams {
    pub visibility: Option<TaskVisibility>,
    pub team_id: Option<Uuid>,
    pub organization_id: Option<Uuid>,
    pub assigned_to: Option<Uuid>,
    pub include_assigned: Option<bool>, // 自分に割り当てられたタスクも含むか
}
