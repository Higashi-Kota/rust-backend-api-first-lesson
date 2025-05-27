// migration/src/lib.rs
pub use sea_orm_migration::prelude::*;

// マイグレーションモジュール
mod m20250511_073638_create_task_table;
mod m20250512_000001_add_task_indexes; // 追加したインデックスマイグレーション

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250511_073638_create_task_table::Migration),
            Box::new(m20250512_000001_add_task_indexes::Migration), // 追加
        ]
    }
}
