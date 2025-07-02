use crate::api::AppState;
use uuid::Uuid;

/// 機能使用状況を追跡するヘルパー関数
pub async fn track_feature_usage(
    app_state: &AppState,
    user_id: Uuid,
    feature_name: &str,
    action_type: &str,
) {
    // エラーが発生しても処理を継続（ログのみ）
    if let Err(e) = app_state
        .feature_tracking_service
        .track_feature_usage(user_id, feature_name, action_type, None)
        .await
    {
        tracing::warn!(
            "Failed to track feature usage: feature={}, action={}, error={}",
            feature_name,
            action_type,
            e
        );
    }
}

/// 機能使用状況を追跡するマクロ
#[macro_export]
macro_rules! track_feature {
    ($app_state:expr, $user_id:expr, $feature:expr, $action:expr) => {
        tokio::spawn(async move {
            $crate::utils::feature_tracking::track_feature_usage(
                &$app_state,
                $user_id,
                $feature,
                $action,
            )
            .await;
        });
    };
}
