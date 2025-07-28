use axum::{body::Body, extract::State, http::Request, middleware::Next, response::Response};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    domain::activity_log_model, middleware::auth::AuthenticatedUser,
    repository::activity_log_repository::ActivityLogRepository,
};

#[derive(Clone)]
pub struct ActivityLogger {
    activity_log_repo: Arc<ActivityLogRepository>,
}

impl ActivityLogger {
    pub fn new(activity_log_repo: Arc<ActivityLogRepository>) -> Self {
        Self { activity_log_repo }
    }
}

/// アクティビティログを記録するミドルウェア
pub async fn log_activity(
    State(logger): State<ActivityLogger>,
    request: Request<Body>,
    next: Next,
) -> Response {
    // 認証済みユーザー情報を取得
    let user = request.extensions().get::<AuthenticatedUser>().cloned();

    // リクエスト情報を記録
    let method = request.method().clone();
    let uri = request.uri().clone();
    let path = uri.path().to_string();

    // ヘッダーからIP情報とユーザーエージェントを取得
    let ip_address = request
        .headers()
        .get("x-forwarded-for")
        .or_else(|| request.headers().get("x-real-ip"))
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    let user_agent = request
        .headers()
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    // レスポンスを処理
    let response = next.run(request).await;

    // ログを記録（ユーザーが認証されている場合のみ）
    if let Some(user) = user {
        // APIパスからアクションとリソースタイプを推定
        let (action, resource_type, resource_id) = parse_api_path(method.as_ref(), &path);

        // エラーレスポンスの場合は詳細を記録
        let mut details = None;
        if !response.status().is_success() {
            details = Some(serde_json::json!({
                "status": response.status().as_u16(),
                "method": method.to_string(),
                "path": path,
            }));
        }

        let activity_log = activity_log_model::Model {
            id: Uuid::new_v4(),
            user_id: user.claims.user_id,
            action,
            resource_type,
            resource_id,
            ip_address,
            user_agent,
            details,
            created_at: chrono::Utc::now(),
        };

        // 非同期でログを記録（エラーがあってもレスポンスには影響しない）
        let logger_clone = logger.clone();
        tokio::spawn(async move {
            if let Err(e) = logger_clone.activity_log_repo.create(&activity_log).await {
                tracing::error!("Failed to create activity log: {:?}", e);
            } else {
                tracing::debug!(
                    "Activity log created successfully for action: {}",
                    activity_log.action
                );
            }
        });
    }

    response
}

/// APIパスからアクション、リソースタイプ、リソースIDを推定
fn parse_api_path(method: &str, path: &str) -> (String, String, Option<Uuid>) {
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    // デフォルト値
    let mut action = method.to_lowercase();
    let mut resource_type = "unknown".to_string();
    let mut resource_id = None;

    // パスパターンのマッチング
    match segments.as_slice() {
        // タスク関連
        ["tasks"] => {
            resource_type = "task".to_string();
            action = match method {
                "GET" => "list_tasks",
                "POST" => "create_task",
                _ => &action,
            }
            .to_string();
        }
        ["tasks", id] => {
            resource_type = "task".to_string();
            resource_id = id.parse().ok();
            action = match method {
                "GET" => "view_task",
                "PATCH" | "PUT" => "update_task",
                "DELETE" => "delete_task",
                _ => &action,
            }
            .to_string();
        }
        ["tasks", id, "assign"] => {
            resource_type = "task".to_string();
            resource_id = id.parse().ok();
            action = "assign_task".to_string();
        }
        ["tasks", id, "multi-tenant"] => {
            resource_type = "task".to_string();
            resource_id = id.parse().ok();
            action = match method {
                "PATCH" => "update_multi_tenant_task",
                "DELETE" => "delete_multi_tenant_task",
                _ => &action,
            }
            .to_string();
        }

        // チーム関連
        ["teams"] => {
            resource_type = "team".to_string();
            action = match method {
                "GET" => "list_teams",
                "POST" => "create_team",
                _ => &action,
            }
            .to_string();
        }
        ["teams", id] => {
            resource_type = "team".to_string();
            resource_id = id.parse().ok();
            action = match method {
                "GET" => "view_team",
                "PATCH" | "PUT" => "update_team",
                "DELETE" => "delete_team",
                _ => &action,
            }
            .to_string();
        }
        ["teams", _id, "tasks"] => {
            resource_type = "task".to_string();
            action = "create_team_task".to_string();
        }
        ["teams", id, "members"] => {
            resource_type = "team".to_string();
            resource_id = id.parse().ok();
            action = match method {
                "GET" => "list_team_members",
                "POST" => "add_team_member",
                _ => &action,
            }
            .to_string();
        }

        // 組織関連
        ["organizations", _id, "tasks"] => {
            resource_type = "task".to_string();
            action = "create_organization_task".to_string();
        }

        // 認証関連
        ["auth", action_name] => {
            resource_type = "auth".to_string();
            action = (*action_name).to_string();
        }

        // 管理者関連
        ["admin", sub_resource, ..] => {
            resource_type = format!("admin_{}", sub_resource);
            action = format!("admin_{}_{}", action.to_lowercase(), sub_resource);
        }

        // アクティビティログ関連
        ["activity-logs", "me"] => {
            resource_type = "activity_log".to_string();
            action = "view_my_activity_logs".to_string();
        }

        _ => {
            // その他のパスはそのまま記録
            if !segments.is_empty() {
                resource_type = segments[0].to_string();
            }
        }
    }

    (action, resource_type, resource_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_api_path() {
        // タスク関連
        assert_eq!(
            parse_api_path("GET", "/tasks"),
            ("list_tasks".to_string(), "task".to_string(), None)
        );

        let task_id = Uuid::new_v4();
        assert_eq!(
            parse_api_path("PATCH", &format!("/tasks/{}", task_id)),
            ("update_task".to_string(), "task".to_string(), Some(task_id))
        );

        assert_eq!(
            parse_api_path("POST", &format!("/tasks/{}/assign", task_id)),
            ("assign_task".to_string(), "task".to_string(), Some(task_id))
        );

        // チーム関連
        assert_eq!(
            parse_api_path("POST", "/teams"),
            ("create_team".to_string(), "team".to_string(), None)
        );

        let team_id = Uuid::new_v4();
        assert_eq!(
            parse_api_path("POST", &format!("/teams/{}/tasks", team_id)),
            ("create_team_task".to_string(), "task".to_string(), None)
        );

        // 認証関連
        assert_eq!(
            parse_api_path("POST", "/auth/signin"),
            ("signin".to_string(), "auth".to_string(), None)
        );

        // 管理者関連
        assert_eq!(
            parse_api_path("GET", "/admin/users"),
            (
                "admin_get_users".to_string(),
                "admin_users".to_string(),
                None
            )
        );
    }
}
