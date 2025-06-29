use sea_orm_migration::prelude::*; // schema::* は ColumnDef を直接使う場合は必須ではないですが、あっても問題ありません。

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Tasks::Table) // <<<--- ここを Tasks::Table に変更
                    .if_not_exists() // テーブルが存在しない場合のみ作成
                    .col(
                        ColumnDef::new(Tasks::Id) // "Post::Id" から "Tasks::Id" に変更、型定義も変更
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Tasks::Title) // "Post::Title" から "Tasks::Title" に変更、型定義も変更
                            .text() // string() ヘルパーの代わりに text() を使用
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Tasks::Description).text(), // "Post::Text" から "Tasks::Description" に変更
                    )
                    .col(
                        ColumnDef::new(Tasks::Status) // 新しいカラム
                            .string() // string() は TEXT 型にマッピングされることが多い
                            .not_null()
                            .default("todo"), // デフォルト値
                    )
                    .col(
                        ColumnDef::new(Tasks::DueDate).timestamp_with_time_zone(), // 新しいカラム
                    )
                    .col(
                        ColumnDef::new(Tasks::CreatedAt) // 新しいカラム
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()), // DEFAULT NOW()
                    )
                    .col(
                        ColumnDef::new(Tasks::UpdatedAt) // 新しいカラム
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()), // DEFAULT NOW()
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(Tasks::Table) // <<<--- ここも Tasks::Table に変更
                    .to_owned(),
            )
            .await
    }
}

/// Iden Enum for the 'tasks' table and its columns
#[derive(DeriveIden)]
enum Tasks {
    Table,
    Id,
    Title,
    Description,
    Status,
    DueDate,
    CreatedAt,
    UpdatedAt,
}
