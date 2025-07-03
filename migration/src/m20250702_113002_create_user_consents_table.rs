use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(UserConsents::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UserConsents::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(UserConsents::UserId).uuid().not_null())
                    .col(
                        ColumnDef::new(UserConsents::ConsentType)
                            .string_len(50)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserConsents::IsGranted)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(UserConsents::GrantedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(UserConsents::RevokedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(UserConsents::IpAddress).string_len(45))
                    .col(ColumnDef::new(UserConsents::UserAgent).text())
                    .col(
                        ColumnDef::new(UserConsents::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserConsents::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_consents_user")
                            .from(UserConsents::Table, UserConsents::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_user_consents_user_id")
                    .table(UserConsents::Table)
                    .col(UserConsents::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_user_consents_consent_type")
                    .table(UserConsents::Table)
                    .col(UserConsents::ConsentType)
                    .to_owned(),
            )
            .await?;

        // Create unique index for user_id + consent_type
        manager
            .create_index(
                Index::create()
                    .name("idx_user_consents_user_consent_unique")
                    .table(UserConsents::Table)
                    .col(UserConsents::UserId)
                    .col(UserConsents::ConsentType)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UserConsents::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum UserConsents {
    Table,
    Id,
    UserId,
    ConsentType,
    IsGranted,
    GrantedAt,
    RevokedAt,
    IpAddress,
    UserAgent,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}
