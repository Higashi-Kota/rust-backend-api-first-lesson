// src/domain/task_model.rs
use sea_orm::entity::prelude::*; // Uuid, ActiveModelBehavior, ActiveModelTrait などを含む
use sea_orm::{ConnectionTrait, DbErr, Set}; // ActiveValue, Set, ConnectionTrait, DbErr を明示的にインポート
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc}; // Utc をインポート

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "tasks")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub title: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub description: Option<String>,
    pub status: String,
    #[sea_orm(nullable)]
    pub due_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

#[async_trait::async_trait] // async fn をトレイト内で使うために追加
impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            id: Set(Uuid::new_v4()), // Uuid は prelude::* から
            created_at: Set(Utc::now()), // Utc は chrono::Utc
            updated_at: Set(Utc::now()), // Utc は chrono::Utc
            ..ActiveModelTrait::default()
        }
    }

    // before_save メソッドのシグネチャをSeaORM 1.x.y系に合わせて修正
    async fn before_save<C>(mut self, _db: &C, insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if !insert { // 更新の場合のみ updated_at を更新
            self.updated_at = Set(Utc::now()); // Utc は chrono::Utc
        }
        Ok(self)
    }
}