use chrono::{DateTime, NaiveDate, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "daily_activity_summaries")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub date: NaiveDate,
    pub total_users: i32,
    pub active_users: i32,
    pub new_users: i32,
    pub tasks_created: i32,
    pub tasks_completed: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyActivityInput {
    pub total_users: i32,
    pub active_users: i32,
    pub new_users: i32,
    pub tasks_created: i32,
    pub tasks_completed: i32,
}

impl Model {
    pub fn new(date: NaiveDate, input: DailyActivityInput) -> Self {
        let now = Utc::now();
        Self {
            date,
            total_users: input.total_users,
            active_users: input.active_users,
            new_users: input.new_users,
            tasks_created: input.tasks_created,
            tasks_completed: input.tasks_completed,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn update(&mut self, input: DailyActivityInput) {
        self.total_users = input.total_users;
        self.active_users = input.active_users;
        self.new_users = input.new_users;
        self.tasks_created = input.tasks_created;
        self.tasks_completed = input.tasks_completed;
        self.updated_at = Utc::now();
    }
}
