use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(UserSettings::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UserSettings::UserId)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(UserSettings::Language)
                            .string()
                            .string_len(10)
                            .not_null()
                            .default("ja"),
                    )
                    .col(
                        ColumnDef::new(UserSettings::Timezone)
                            .string()
                            .string_len(50)
                            .not_null()
                            .default("Asia/Tokyo"),
                    )
                    .col(
                        ColumnDef::new(UserSettings::NotificationsEnabled)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(UserSettings::EmailNotifications)
                            .json_binary()
                            .not_null()
                            .extra("DEFAULT '{}'::jsonb"),
                    )
                    .col(
                        ColumnDef::new(UserSettings::UiPreferences)
                            .json_binary()
                            .not_null()
                            .extra("DEFAULT '{}'::jsonb"),
                    )
                    .col(
                        ColumnDef::new(UserSettings::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .extra("DEFAULT NOW()"),
                    )
                    .col(
                        ColumnDef::new(UserSettings::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .extra("DEFAULT NOW()"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_settings_user_id")
                            .from(UserSettings::Table, UserSettings::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UserSettings::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum UserSettings {
    Table,
    UserId,
    Language,
    Timezone,
    NotificationsEnabled,
    EmailNotifications,
    UiPreferences,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}
