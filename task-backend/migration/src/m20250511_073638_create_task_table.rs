use sea_orm_migration::prelude::*; // schema::* は ColumnDef を直接使う場合は必須ではないですが、あっても問題ありません。

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // todo!(); // この行を削除

        manager
            .create_table(
                Table::create()
                    .table(Alias::new("tasks")) // <<<--- ここを Alias::new("tasks") に変更
                    .if_not_exists()    // テーブルが存在しない場合のみ作成
                    .col(
                        ColumnDef::new(Task::Id) // "Post::Id" から "Task::Id" に変更、型定義も変更
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Task::Title) // "Post::Title" から "Task::Title" に変更、型定義も変更
                            .text() // string() ヘルパーの代わりに text() を使用
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Task::Description).text(), // "Post::Text" から "Task::Description" に変更
                    )
                    .col(
                        ColumnDef::new(Task::Status) // 新しいカラム
                            .string() // string() は TEXT 型にマッピングされることが多い
                            .not_null()
                            .default("todo"), // デフォルト値
                    )
                    .col(
                        ColumnDef::new(Task::DueDate).timestamp_with_time_zone(), // 新しいカラム
                    )
                    .col(
                        ColumnDef::new(Task::CreatedAt) // 新しいカラム
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()), // DEFAULT NOW()
                    )
                    .col(
                        ColumnDef::new(Task::UpdatedAt) // 新しいカラム
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()), // DEFAULT NOW()
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // todo!(); // この行を削除

        manager
            .drop_table(
                Table::drop()
                    .table(Alias::new("tasks")) // <<<--- ここも Alias::new("tasks") に変更
                    .to_owned()
            )
            .await
    }
}

/// Iden Enum for the 'tasks' table and its columns
#[derive(DeriveIden)]
enum Task { // "Post" から "Task" に変更
    // Table, // テーブル名は Alias で直接指定するので、このバリアントは必須ではなくなります。
              // もし他の箇所で Task::Table を参照していなければ削除してもOK。
              // カラム名を定義するためにはこの enum は有用です。
    Id,
    Title,
    Description,
    Status,
    DueDate,
    CreatedAt,
    UpdatedAt,
}