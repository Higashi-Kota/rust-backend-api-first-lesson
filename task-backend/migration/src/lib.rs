// migration/src/lib.rs
pub use sea_orm_migration::prelude::*;

// STEP 1: 個々のマイグレーションファイルをモジュールとして宣言します。
// `sea-orm-cli migrate generate` を実行すると、ここに自動的に追加されます。
mod m20250511_073638_create_task_table;
// 例: mod m20220101_000002_create_another_table;

// STEP 2: `Migrator` 構造体を定義します。
// この構造体は、プロジェクト全体のマイグレーションを管理します。
// `#[derive(DeriveMigrationName)]` はここでは不要です。
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator { // `MigrationTrait` ではなく `MigratorTrait` を実装します。
    // `migrations` メソッドは、適用する全てのマイグレーションのリストを返します。
    // 各マイグレーションは `MigrationTrait` を実装した構造体である必要があります。
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            // STEP 3: 各マイグレーション構造体をBox化してベクターに追加します。
            Box::new(m20250511_073638_create_task_table::Migration),
            // 例: Box::new(m20220101_000002_create_another_table::Migration),
        ]
    }
}