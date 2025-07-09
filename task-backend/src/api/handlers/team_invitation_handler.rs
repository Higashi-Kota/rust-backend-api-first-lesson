// task-backend/src/api/handlers/team_invitation_handler.rs

use crate::api::dto::common::ApiResponse;
use crate::api::dto::team_invitation_dto::*;
use crate::error::{AppError, AppResult};
use crate::features::auth::middleware::AuthenticatedUser;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use uuid::Uuid;
use validator::Validate;

pub async fn bulk_member_invite(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Path(team_id): Path<Uuid>,
    Json(request): Json<BulkTeamInviteRequest>,
) -> AppResult<Json<ApiResponse<BulkInviteResponse>>> {
    let service = &app_state.team_invitation_service;
    request
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    if !service
        .validate_invitation_permissions(team_id, user.user_id())
        .await?
    {
        return Err(AppError::Forbidden(
            "You do not have permission to invite members to this team".to_string(),
        ));
    }

    let mut success_count = 0;
    let mut invitations = Vec::new();
    let mut failed_emails = Vec::new();

    for email in &request.emails {
        match service
            .create_bulk_member_invite(
                team_id,
                vec![email.clone()],
                request.message.clone(),
                user.user_id(),
            )
            .await
        {
            Ok(mut created_invitations) => {
                if let Some(invitation) = created_invitations.pop() {
                    invitations.push(TeamInvitationResponse::from(invitation));
                    success_count += 1;
                }
            }
            Err(_) => {
                failed_emails.push(email.clone());
            }
        }
    }

    let response = BulkInviteResponse {
        success_count,
        invitations,
        failed_emails,
    };

    Ok(Json(ApiResponse::success(
        "Operation completed successfully",
        response,
    )))
}

pub async fn get_team_invitations(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Path(team_id): Path<Uuid>,
    Query(query): Query<TeamInvitationQuery>,
) -> AppResult<Json<ApiResponse<TeamInvitationsListResponse>>> {
    let service = &app_state.team_invitation_service;
    if !service
        .validate_invitation_permissions(team_id, user.user_id())
        .await?
    {
        return Err(AppError::Forbidden(
            "You do not have permission to view team invitations".to_string(),
        ));
    }

    let invitations = service
        .get_team_invitations(team_id, query.status.clone())
        .await?;

    let invitation_responses: Vec<TeamInvitationResponse> = invitations
        .into_iter()
        .map(TeamInvitationResponse::from)
        .collect();

    let stats = service.get_invitation_statistics(team_id).await?;
    let status_counts = TeamInvitationStatusCounts {
        pending: stats.pending,
        accepted: stats.accepted,
        declined: stats.declined,
        expired: stats.expired,
        cancelled: stats.cancelled,
    };

    let response = TeamInvitationsListResponse {
        invitations: invitation_responses,
        total_count: stats.total,
        status_counts,
    };

    Ok(Json(ApiResponse::success(
        "Operation completed successfully",
        response,
    )))
}

pub async fn decline_invitation(
    State(app_state): State<crate::api::AppState>,
    Path((team_id, invitation_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<DeclineInvitationRequest>,
) -> AppResult<Json<ApiResponse<TeamInvitationResponse>>> {
    let service = &app_state.team_invitation_service;
    request
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    let updated_invitation = service
        .decline_invitation(team_id, invitation_id, request.reason)
        .await?;

    let response = TeamInvitationResponse::from(updated_invitation);
    Ok(Json(ApiResponse::success(
        "Operation completed successfully",
        response,
    )))
}

pub async fn accept_invitation(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Path(invitation_id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<TeamInvitationResponse>>> {
    let service = &app_state.team_invitation_service;
    let updated_invitation = service
        .accept_invitation(invitation_id, Some(user.user_id()))
        .await?;

    let response = TeamInvitationResponse::from(updated_invitation);
    Ok(Json(ApiResponse::success(
        "Operation completed successfully",
        response,
    )))
}

pub async fn cancel_invitation(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Path((team_id, invitation_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<ApiResponse<TeamInvitationResponse>>> {
    let service = &app_state.team_invitation_service;
    let updated_invitation = service
        .cancel_invitation(team_id, invitation_id, user.user_id())
        .await?;

    let response = TeamInvitationResponse::from(updated_invitation);
    Ok(Json(ApiResponse::success(
        "Operation completed successfully",
        response,
    )))
}

pub async fn resend_invitation(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Path(invitation_id): Path<Uuid>,
    Json(request): Json<ResendInvitationRequest>,
) -> AppResult<Json<ApiResponse<TeamInvitationResponse>>> {
    let service = &app_state.team_invitation_service;
    request
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    let updated_invitation = service
        .resend_invitation(invitation_id, request.message, user.user_id())
        .await?;

    let response = TeamInvitationResponse::from(updated_invitation);
    Ok(Json(ApiResponse::success(
        "Operation completed successfully",
        response,
    )))
}

pub async fn get_user_invitations(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
) -> AppResult<Json<ApiResponse<Vec<TeamInvitationResponse>>>> {
    let service = &app_state.team_invitation_service;
    let invitations = service.get_user_invitations(&user.claims.email).await?;

    let responses: Vec<TeamInvitationResponse> = invitations
        .into_iter()
        .map(TeamInvitationResponse::from)
        .collect();

    Ok(Json(ApiResponse::success(
        "Operation completed successfully",
        responses,
    )))
}

pub async fn get_invitation_statistics(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Path(team_id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<InvitationStatisticsResponse>>> {
    let service = &app_state.team_invitation_service;
    if !service
        .validate_invitation_permissions(team_id, user.user_id())
        .await?
    {
        return Err(AppError::Forbidden(
            "You do not have permission to view team invitation statistics".to_string(),
        ));
    }

    let stats = service.get_invitation_statistics(team_id).await?;
    let response = InvitationStatisticsResponse::from(stats);

    Ok(Json(ApiResponse::success(
        "Operation completed successfully",
        response,
    )))
}

pub async fn cleanup_expired_invitations(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
) -> AppResult<Json<ApiResponse<Vec<TeamInvitationResponse>>>> {
    let service = &app_state.team_invitation_service;
    if !user.is_admin() {
        return Err(AppError::Forbidden(
            "Only administrators can cleanup expired invitations".to_string(),
        ));
    }

    let expired_invitations = service.mark_expired_invitations().await?;
    let responses: Vec<TeamInvitationResponse> = expired_invitations
        .into_iter()
        .map(TeamInvitationResponse::from)
        .collect();

    Ok(Json(ApiResponse::success(
        "Operation completed successfully",
        responses,
    )))
}

pub async fn get_invitations_by_creator(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
) -> AppResult<Json<ApiResponse<Vec<TeamInvitationResponse>>>> {
    let service = &app_state.team_invitation_service;
    let invitations = service.get_invitations_by_creator(user.user_id()).await?;

    let responses: Vec<TeamInvitationResponse> = invitations
        .into_iter()
        .map(TeamInvitationResponse::from)
        .collect();

    Ok(Json(ApiResponse::success(
        "Operation completed successfully",
        responses,
    )))
}

pub async fn delete_old_invitations(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let service = &app_state.team_invitation_service;
    if !user.is_admin() {
        return Err(AppError::Forbidden(
            "Only administrators can delete old invitations".to_string(),
        ));
    }

    let days = params
        .get("days")
        .and_then(|d| d.parse::<u32>().ok())
        .unwrap_or(30);

    if days < 7 {
        return Err(AppError::ValidationError(
            "Cannot delete invitations less than 7 days old".to_string(),
        ));
    }

    let deleted_count = service.cleanup_old_invitations(days).await?;

    Ok(Json(ApiResponse::success(
        "Old invitations deleted successfully",
        serde_json::json!({
            "deleted_count": deleted_count,
            "days": days
        }),
    )))
}

/// 単一招待を作成
pub async fn create_single_invitation(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Path(team_id): Path<Uuid>,
    Json(request): Json<CreateInvitationRequest>,
) -> AppResult<Json<ApiResponse<TeamInvitationResponse>>> {
    let service = &app_state.team_invitation_service;
    request
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    let invitation = service
        .create_single_invitation(team_id, request.email, request.message, user.user_id())
        .await?;

    let response = TeamInvitationResponse::from(invitation);
    Ok(Json(ApiResponse::success(
        "Invitation created successfully",
        response,
    )))
}

/// ユーザーのメール宛て招待一覧を取得
pub async fn get_invitations_by_email(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Query(query): Query<UserInvitationQuery>,
) -> AppResult<Json<ApiResponse<Vec<TeamInvitationResponse>>>> {
    let service = &app_state.team_invitation_service;

    // ユーザーは自分のメールアドレスの招待のみ閲覧可能
    if query.email != user.claims.email && !user.is_admin() {
        return Err(AppError::Forbidden(
            "You can only view your own invitations".to_string(),
        ));
    }

    let invitations = service.get_invitations_by_email(&query.email).await?;

    let responses: Vec<TeamInvitationResponse> = invitations
        .into_iter()
        .map(TeamInvitationResponse::from)
        .collect();

    Ok(Json(ApiResponse::success(
        "Invitations retrieved successfully",
        responses,
    )))
}

/// 特定チーム・メールの招待を確認
pub async fn check_team_invitation(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Path((team_id, email)): Path<(Uuid, String)>,
) -> AppResult<Json<ApiResponse<CheckInvitationResponse>>> {
    let service = &app_state.team_invitation_service;

    // 権限確認
    if !service
        .validate_invitation_permissions(team_id, user.user_id())
        .await?
    {
        return Err(AppError::Forbidden(
            "You do not have permission to check invitations for this team".to_string(),
        ));
    }

    let invitation = service.check_team_invitation(team_id, &email).await?;

    let response = CheckInvitationResponse {
        exists: invitation.is_some(),
        invitation: invitation.map(TeamInvitationResponse::from),
    };

    Ok(Json(ApiResponse::success(
        "Invitation check completed",
        response,
    )))
}

/// 招待一覧をページング付きで取得
pub async fn get_invitations_with_pagination(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Path(team_id): Path<Uuid>,
    Query(query): Query<TeamInvitationQuery>,
) -> AppResult<Json<ApiResponse<InvitationPaginationResponse>>> {
    let service = &app_state.team_invitation_service;

    let page = query.get_page();
    let page_size = query.get_page_size();

    let (invitations, total_count) = service
        .get_invitations_with_pagination(
            team_id,
            page,
            page_size,
            query.status.clone(),
            user.user_id(),
        )
        .await?;

    let responses: Vec<TeamInvitationResponse> = invitations
        .into_iter()
        .map(TeamInvitationResponse::from)
        .collect();

    let response = InvitationPaginationResponse::new(responses, total_count, page, page_size);

    Ok(Json(ApiResponse::success(
        "Invitations retrieved successfully",
        response,
    )))
}

/// ユーザーの招待数を取得
pub async fn count_user_invitations(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Query(query): Query<UserInvitationQuery>,
) -> AppResult<Json<ApiResponse<UserInvitationStatsResponse>>> {
    let service = &app_state.team_invitation_service;

    // ユーザーは自分のメールアドレスの招待のみ閲覧可能
    if query.email != user.claims.email && !user.is_admin() {
        return Err(AppError::Forbidden(
            "You can only view your own invitation statistics".to_string(),
        ));
    }

    let total_invitations = service.count_user_invitations(&query.email).await?;

    // 詳細統計を取得
    let all_invitations = service.get_invitations_by_email(&query.email).await?;

    let pending_invitations = all_invitations
        .iter()
        .filter(|inv| {
            inv.get_status() == crate::domain::team_invitation_model::TeamInvitationStatus::Pending
        })
        .count() as u64;

    let accepted_invitations = all_invitations
        .iter()
        .filter(|inv| {
            inv.get_status() == crate::domain::team_invitation_model::TeamInvitationStatus::Accepted
        })
        .count() as u64;

    let declined_invitations = all_invitations
        .iter()
        .filter(|inv| {
            inv.get_status() == crate::domain::team_invitation_model::TeamInvitationStatus::Declined
        })
        .count() as u64;

    let response = UserInvitationStatsResponse {
        total_invitations,
        pending_invitations,
        accepted_invitations,
        declined_invitations,
    };

    Ok(Json(ApiResponse::success(
        "Invitation statistics retrieved successfully",
        response,
    )))
}

/// 招待の一括ステータス更新
pub async fn bulk_update_invitation_status(
    State(app_state): State<crate::api::AppState>,
    user: AuthenticatedUser,
    Json(request): Json<BulkUpdateStatusRequest>,
) -> AppResult<Json<ApiResponse<BulkUpdateStatusResponse>>> {
    let service = &app_state.team_invitation_service;
    request
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    let updated_count = service
        .bulk_update_invitation_status(&request.invitation_ids, request.new_status, user.user_id())
        .await?;

    let response = BulkUpdateStatusResponse {
        updated_count,
        failed_ids: vec![], // All succeeded if no error was thrown
    };

    Ok(Json(ApiResponse::success(
        "Invitation status updated successfully",
        response,
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::team_invitation_model::{
        Model as TeamInvitationModel, TeamInvitationStatus,
    };

    #[test]
    fn test_bulk_invite_request_validation() {
        let valid_request = BulkTeamInviteRequest {
            emails: vec!["test@example.com".to_string()],
            message: Some("Welcome!".to_string()),
        };
        assert!(valid_request.validate().is_ok());

        let invalid_request = BulkTeamInviteRequest {
            emails: vec!["invalid-email".to_string()],
            message: None,
        };
        assert!(invalid_request.validate().is_err());
    }

    #[test]
    fn test_decline_invitation_request_validation() {
        let valid_request = DeclineInvitationRequest {
            reason: Some("Not interested".to_string()),
        };
        assert!(valid_request.validate().is_ok());

        let invalid_request = DeclineInvitationRequest {
            reason: Some("a".repeat(501)),
        };
        assert!(invalid_request.validate().is_err());
    }

    #[test]
    fn test_team_invitation_response_creation() {
        let model = TeamInvitationModel::new(
            Uuid::new_v4(),
            "test@example.com".to_string(),
            Uuid::new_v4(),
            Some("Test message".to_string()),
            None,
        );

        let response = TeamInvitationResponse::from(model.clone());
        assert_eq!(response.invited_email, model.invited_email);
        assert_eq!(response.message, model.message);
        assert_eq!(response.status, TeamInvitationStatus::Pending);
    }

    #[test]
    fn test_invitation_statistics_response_creation() {
        let stats = crate::service::team_invitation_service::TeamInvitationStatistics {
            total: 50,
            pending: 10,
            accepted: 30,
            declined: 8,
            expired: 2,
            cancelled: 0,
        };

        let response = InvitationStatisticsResponse::from(stats);
        assert_eq!(response.total, 50);
        assert_eq!(response.pending, 10);
        assert_eq!(response.accepted, 30);
        assert_eq!(response.declined, 8);
        assert_eq!(response.expired, 2);
    }
}
