#![allow(dead_code)] // Repository methods for user settings

use crate::domain::user_settings_model::{self, Entity as UserSettings};
use crate::error::AppResult;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter,
    Set,
};
use uuid::Uuid;

pub struct UserSettingsRepository {
    db: DatabaseConnection,
}

impl UserSettingsRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn get_or_create(&self, user_id: Uuid) -> AppResult<user_settings_model::Model> {
        // 既存の設定を検索
        let existing = UserSettings::find()
            .filter(user_settings_model::Column::UserId.eq(user_id))
            .one(&self.db)
            .await?;

        match existing {
            Some(settings) => Ok(settings),
            None => {
                // 新規作成
                let model = user_settings_model::Model::new(user_id);
                let active_model = user_settings_model::ActiveModel {
                    user_id: Set(model.user_id),
                    language: Set(model.language.clone()),
                    timezone: Set(model.timezone.clone()),
                    notifications_enabled: Set(model.notifications_enabled),
                    email_notifications: Set(model.email_notifications.clone()),
                    ui_preferences: Set(model.ui_preferences.clone()),
                    created_at: Set(model.created_at),
                    updated_at: Set(model.updated_at),
                };

                let result = active_model.insert(&self.db).await?;
                Ok(result)
            }
        }
    }

    pub async fn update(
        &self,
        user_id: Uuid,
        input: user_settings_model::UserSettingsInput,
    ) -> AppResult<user_settings_model::Model> {
        // 既存の設定を取得または作成
        let mut settings = self.get_or_create(user_id).await?;

        // 設定を更新
        settings.update(input);

        // データベースに保存
        let mut active_model = settings.clone().into_active_model();
        active_model.language = Set(settings.language.clone());
        active_model.timezone = Set(settings.timezone.clone());
        active_model.notifications_enabled = Set(settings.notifications_enabled);
        active_model.email_notifications = Set(settings.email_notifications.clone());
        active_model.ui_preferences = Set(settings.ui_preferences.clone());
        active_model.updated_at = Set(settings.updated_at);

        let result = active_model.update(&self.db).await?;
        Ok(result)
    }

    pub async fn get_by_user_id(
        &self,
        user_id: Uuid,
    ) -> AppResult<Option<user_settings_model::Model>> {
        let settings = UserSettings::find()
            .filter(user_settings_model::Column::UserId.eq(user_id))
            .one(&self.db)
            .await?;

        Ok(settings)
    }

    pub async fn get_users_by_language(&self, language: &str) -> AppResult<Vec<Uuid>> {
        use sea_orm::QuerySelect;

        let results: Vec<Uuid> = UserSettings::find()
            .filter(user_settings_model::Column::Language.eq(language))
            .select_only()
            .column(user_settings_model::Column::UserId)
            .into_tuple()
            .all(&self.db)
            .await?;

        Ok(results)
    }

    pub async fn get_users_with_notification_enabled(
        &self,
        notification_type: &str,
    ) -> AppResult<Vec<Uuid>> {
        use sea_orm::sea_query::Expr;
        use sea_orm::QuerySelect;

        // email_notifications->'task_updates' = true のようなクエリを実行
        let query = format!("email_notifications->'{}' = true", notification_type);

        let results: Vec<Uuid> = UserSettings::find()
            .filter(user_settings_model::Column::NotificationsEnabled.eq(true))
            .filter(Expr::cust(&query))
            .select_only()
            .column(user_settings_model::Column::UserId)
            .into_tuple()
            .all(&self.db)
            .await?;

        Ok(results)
    }

    pub async fn delete(&self, user_id: Uuid) -> AppResult<bool> {
        let result = UserSettings::delete_many()
            .filter(user_settings_model::Column::UserId.eq(user_id))
            .exec(&self.db)
            .await?;

        Ok(result.rows_affected > 0)
    }
}
