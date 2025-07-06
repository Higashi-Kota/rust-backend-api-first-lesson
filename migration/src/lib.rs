// migration/src/lib.rs
pub use sea_orm_migration::prelude::*;

// マイグレーションモジュール
mod m20250511_073638_create_task_table;
mod m20250512_000001_add_task_indexes;

// 認証関連マイグレーション
mod m20250612_000001_create_users_table;
mod m20250612_000002_create_refresh_tokens_table;
mod m20250612_000003_create_password_reset_tokens_table;
mod m20250612_000004_add_user_id_to_tasks;

// ロール関連マイグレーション
mod m20250613_000001_create_roles_table;
mod m20250613_000002_add_role_id_to_users;
mod m20250613_000003_create_initial_admin;

// サブスクリプション関連マイグレーション
mod m20250616_000001_add_subscription_tier_to_users;
mod m20250616_000002_create_subscription_histories_table;

// チーム・組織関連マイグレーション
mod m20250616_000003_create_teams_table;
mod m20250616_000004_create_organizations_table;
mod m20250616_000005_create_team_members_table;
mod m20250616_000006_create_organization_members_table;

// メール認証関連マイグレーション
mod m20250621_140000_create_email_verification_tokens_table;

// 組織階層管理関連マイグレーション
mod m20250624_000001_create_organization_departments_table;
mod m20250624_000002_create_permission_matrices_table;
mod m20250624_000003_create_organization_analytics_table;
mod m20250624_000004_create_department_members_table;

// チーム招待・権限管理関連マイグレーション
mod m20250625_000001_create_team_invitations_table;

// セキュリティ分析関連マイグレーション
mod m20250630_000001_create_activity_logs_table;
mod m20250630_000002_create_security_incidents_table;
mod m20250630_000003_create_login_attempts_table;

// 分析・設定関連マイグレーション
mod m20250702_000001_create_feature_usage_metrics_table;
mod m20250702_000002_create_daily_activity_summaries_table;
mod m20250702_000003_create_user_settings_table;
mod m20250702_000004_create_bulk_operation_histories_table;

// パフォーマンス最適化マイグレーション
mod m20250702_100001_add_performance_indexes;

// GDPR関連マイグレーション
mod m20250702_113002_create_user_consents_table;

// 外部キー制約修正マイグレーション
mod m20250703_013738_fix_teams_owner_cascade;

// ファイルアップロード関連マイグレーション
mod m20250703_150000_create_task_attachments_table;
mod m20250704_180000_create_attachment_share_links;
mod m20250704_180001_create_share_link_access_logs;

// Stripe決済関連マイグレーション
mod m20250704_180002_add_stripe_support;

// データ品質改善マイグレーション
mod m20250706_000001_add_priority_completion_to_tasks;
mod m20250706_000002_add_ip_tracking_to_password_reset;
mod m20250706_000003_add_session_analytics_to_refresh_tokens;
mod m20250706_000004_enhance_login_attempts_tracking;
mod m20250706_000005_create_session_analytics_summaries;
mod m20250706_000006_create_task_analytics_summaries;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            // 1. 基本テーブル作成（依存関係なし）
            Box::new(m20250511_073638_create_task_table::Migration),
            Box::new(m20250612_000001_create_users_table::Migration),
            // 2. 基本テーブルのインデックス追加
            Box::new(m20250512_000001_add_task_indexes::Migration),
            // 3. 依存テーブル作成（usersテーブルに依存）
            Box::new(m20250612_000002_create_refresh_tokens_table::Migration),
            Box::new(m20250612_000003_create_password_reset_tokens_table::Migration),
            // 4. 既存テーブルの変更（usersテーブルへの外部キー追加）
            Box::new(m20250612_000004_add_user_id_to_tasks::Migration),
            // 5. ロール関連テーブル作成とユーザーテーブル更新
            Box::new(m20250613_000001_create_roles_table::Migration),
            Box::new(m20250613_000002_add_role_id_to_users::Migration),
            // 6. 初期管理者ユーザー作成
            Box::new(m20250613_000003_create_initial_admin::Migration),
            // 7. サブスクリプション階層システム
            Box::new(m20250616_000001_add_subscription_tier_to_users::Migration),
            Box::new(m20250616_000002_create_subscription_histories_table::Migration),
            // 8. チーム・組織管理システム
            Box::new(m20250616_000004_create_organizations_table::Migration),
            Box::new(m20250616_000006_create_organization_members_table::Migration),
            Box::new(m20250616_000003_create_teams_table::Migration),
            Box::new(m20250616_000005_create_team_members_table::Migration),
            // 9. メール認証システム
            Box::new(m20250621_140000_create_email_verification_tokens_table::Migration),
            // 10. 組織階層管理システム
            Box::new(m20250624_000001_create_organization_departments_table::Migration),
            Box::new(m20250624_000002_create_permission_matrices_table::Migration),
            Box::new(m20250624_000003_create_organization_analytics_table::Migration),
            Box::new(m20250624_000004_create_department_members_table::Migration),
            // 11. チーム招待・権限管理システム
            Box::new(m20250625_000001_create_team_invitations_table::Migration),
            // 12. セキュリティ分析システム
            Box::new(m20250630_000001_create_activity_logs_table::Migration),
            Box::new(m20250630_000002_create_security_incidents_table::Migration),
            Box::new(m20250630_000003_create_login_attempts_table::Migration),
            // 13. 分析・設定システム
            Box::new(m20250702_000001_create_feature_usage_metrics_table::Migration),
            Box::new(m20250702_000002_create_daily_activity_summaries_table::Migration),
            Box::new(m20250702_000003_create_user_settings_table::Migration),
            Box::new(m20250702_000004_create_bulk_operation_histories_table::Migration),
            // 14. パフォーマンス最適化
            Box::new(m20250702_100001_add_performance_indexes::Migration),
            // 15. GDPR関連テーブル
            Box::new(m20250702_113002_create_user_consents_table::Migration),
            // 16. 外部キー制約修正
            Box::new(m20250703_013738_fix_teams_owner_cascade::Migration),
            // 17. ファイルアップロードシステム
            Box::new(m20250703_150000_create_task_attachments_table::Migration),
            // 18. 外部共有リンクシステム
            Box::new(m20250704_180000_create_attachment_share_links::Migration),
            Box::new(m20250704_180001_create_share_link_access_logs::Migration),
            // 19. Stripe決済システム
            Box::new(m20250704_180002_add_stripe_support::Migration),
            // 20. データ品質改善（プレースホルダー実装の解消）
            Box::new(m20250706_000001_add_priority_completion_to_tasks::Migration),
            Box::new(m20250706_000002_add_ip_tracking_to_password_reset::Migration),
            Box::new(m20250706_000003_add_session_analytics_to_refresh_tokens::Migration),
            Box::new(m20250706_000004_enhance_login_attempts_tracking::Migration),
            Box::new(m20250706_000005_create_session_analytics_summaries::Migration),
            Box::new(m20250706_000006_create_task_analytics_summaries::Migration),
        ]
    }
}
