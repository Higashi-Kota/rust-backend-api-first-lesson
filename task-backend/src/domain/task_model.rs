// src/domain/task_model.rs
use crate::domain::task_visibility::TaskVisibility;
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*; // Uuid, ActiveModelBehavior, ActiveModelTrait などを含む
use sea_orm::{ConnectionTrait, DbErr, Set}; // ActiveValue, Set, ConnectionTrait, DbErr を明示的にインポート
use serde::{Deserialize, Serialize}; // Utc をインポート

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "tasks")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(nullable)]
    pub user_id: Option<Uuid>,
    pub title: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub description: Option<String>,
    pub status: String,
    pub priority: String, // 'low', 'medium', 'high'
    #[sea_orm(nullable)]
    pub due_date: Option<DateTime<Utc>>,
    #[sea_orm(nullable)]
    pub completed_at: Option<DateTime<Utc>>,
    #[sea_orm(nullable)]
    pub completion_duration_hours: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    // マルチテナント対応フィールド
    #[sea_orm(nullable)]
    pub team_id: Option<Uuid>,
    #[sea_orm(nullable)]
    pub organization_id: Option<Uuid>,
    pub visibility: TaskVisibility,
    #[sea_orm(nullable)]
    pub assigned_to: Option<Uuid>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::domain::user_model::Entity",
        from = "Column::UserId",
        to = "crate::domain::user_model::Column::Id"
    )]
    User,

    #[sea_orm(
        belongs_to = "crate::domain::team_model::Entity",
        from = "Column::TeamId",
        to = "crate::domain::team_model::Column::Id"
    )]
    Team,

    #[sea_orm(
        belongs_to = "crate::domain::organization_model::Entity",
        from = "Column::OrganizationId",
        to = "crate::domain::organization_model::Column::Id"
    )]
    Organization,

    #[sea_orm(
        belongs_to = "crate::domain::user_model::Entity",
        from = "Column::AssignedTo",
        to = "crate::domain::user_model::Column::Id"
    )]
    AssignedUser,
}

impl Related<crate::domain::user_model::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

#[async_trait::async_trait] // async fn をトレイト内で使うために追加
impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            id: Set(Uuid::new_v4()),                   // Uuid は prelude::* から
            priority: Set("medium".to_string()),       // デフォルトはmedium
            visibility: Set(TaskVisibility::Personal), // デフォルトはPersonal
            created_at: Set(Utc::now()),               // Utc は chrono::Utc
            updated_at: Set(Utc::now()),               // Utc は chrono::Utc
            ..ActiveModelTrait::default()
        }
    }

    // before_save メソッドのシグネチャをSeaORM 1.x.y系に合わせて修正
    async fn before_save<C>(mut self, _db: &C, insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if !insert {
            // 更新の場合のみ updated_at を更新
            self.updated_at = Set(Utc::now()); // Utc は chrono::Utc
        }
        Ok(self)
    }
}

// マルチテナント操作のためのヘルパーメソッド
impl Model {
    /// タスクが特定のユーザーに属しているかチェック
    pub fn is_owned_by(&self, user_id: &Uuid) -> bool {
        self.user_id.as_ref() == Some(user_id)
    }

    /// タスクの所有者情報を取得（表示用）
    pub fn get_owner_info(&self) -> String {
        match self.visibility {
            TaskVisibility::Personal => "Personal task".to_string(),
            TaskVisibility::Team => {
                if self.team_id.is_some() {
                    "Team task".to_string()
                } else {
                    "Invalid team task".to_string()
                }
            }
            TaskVisibility::Organization => {
                if self.organization_id.is_some() {
                    "Organization task".to_string()
                } else {
                    "Invalid organization task".to_string()
                }
            }
        }
    }
}

// ActiveModel用のヘルパーメソッド
impl ActiveModel {
    /// チームタスクとして設定
    pub fn set_as_team_task(&mut self, team_id: Uuid, organization_id: Uuid) {
        self.team_id = Set(Some(team_id));
        self.organization_id = Set(Some(organization_id));
        self.visibility = Set(TaskVisibility::Team);
    }

    /// 組織タスクとして設定
    pub fn set_as_organization_task(&mut self, organization_id: Uuid) {
        self.organization_id = Set(Some(organization_id));
        self.visibility = Set(TaskVisibility::Organization);
        self.team_id = Set(None);
    }

    /// タスクを特定のユーザーに割り当て
    pub fn assign_to(&mut self, user_id: Option<Uuid>) {
        self.assigned_to = Set(user_id);
    }
}
