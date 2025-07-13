use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "user_settings")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: Uuid,
    pub language: String,
    pub timezone: String,
    pub notifications_enabled: bool,
    pub email_notifications: serde_json::Value,
    pub ui_preferences: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::features::user::models::user::Entity",
        from = "Column::UserId",
        to = "crate::features::user::models::user::Column::Id"
    )]
    User,
}

impl Related<crate::features::user::models::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub theme: String,
    pub layout: String,
    pub sidebar_collapsed: bool,
    pub display_density: String,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            theme: "light".to_string(),
            layout: "default".to_string(),
            sidebar_collapsed: false,
            display_density: "comfortable".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSettings {
    pub task_updates: bool,
    pub team_invites: bool,
    pub organization_updates: bool,
    pub security_alerts: bool,
    pub newsletter: bool,
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            task_updates: true,
            team_invites: true,
            organization_updates: true,
            security_alerts: true,
            newsletter: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettingsInput {
    pub language: Option<String>,
    pub timezone: Option<String>,
    pub notifications_enabled: Option<bool>,
    pub email_notifications: Option<NotificationSettings>,
    pub ui_preferences: Option<UserPreferences>,
}

impl Model {
    pub fn new(user_id: Uuid) -> Self {
        let now = Utc::now();
        Self {
            user_id,
            language: "ja".to_string(),
            timezone: "Asia/Tokyo".to_string(),
            notifications_enabled: true,
            email_notifications: serde_json::to_value(NotificationSettings::default())
                .unwrap_or_else(|_| serde_json::json!({})),
            ui_preferences: serde_json::to_value(UserPreferences::default())
                .unwrap_or_else(|_| serde_json::json!({})),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn update(&mut self, input: UserSettingsInput) {
        if let Some(language) = input.language {
            self.language = language;
        }
        if let Some(timezone) = input.timezone {
            self.timezone = timezone;
        }
        if let Some(notifications_enabled) = input.notifications_enabled {
            self.notifications_enabled = notifications_enabled;
        }
        if let Some(email_notifications) = input.email_notifications {
            self.email_notifications = serde_json::to_value(email_notifications)
                .unwrap_or_else(|_| self.email_notifications.clone());
        }
        if let Some(ui_preferences) = input.ui_preferences {
            self.ui_preferences = serde_json::to_value(ui_preferences)
                .unwrap_or_else(|_| self.ui_preferences.clone());
        }
        self.updated_at = Utc::now();
    }

    pub fn get_email_notifications(&self) -> NotificationSettings {
        serde_json::from_value(self.email_notifications.clone()).unwrap_or_default()
    }

    pub fn get_ui_preferences(&self) -> UserPreferences {
        serde_json::from_value(self.ui_preferences.clone()).unwrap_or_default()
    }
}
