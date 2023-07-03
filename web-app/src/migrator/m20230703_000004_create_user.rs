use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        file!()
    }
}

#[rustfmt::skip]
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    // Define how to apply this migration: Create the Bakery table.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .col(
                        ColumnDef::new(User::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(User::DisplayName).string().not_null().unique_key())
                    .col(ColumnDef::new(User::Email).string().not_null())
                    .col(ColumnDef::new(User::PasswordHash).string().not_null())
                    .clone(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(ActiveSession::Table)
                    .col(
                        ColumnDef::new(ActiveSession::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-activesession-user")
                            .from(ActiveSession::Table, ActiveSession::UserId)
                            .to(User::Table, User::Id),
                    )
                    .col(ColumnDef::new(ActiveSession::UserId).integer().not_null())
                    .col(ColumnDef::new(ActiveSession::Token).string().not_null())
                    .col(ColumnDef::new(ActiveSession::IdleTimeout).date_time().not_null())
                    .col(ColumnDef::new(ActiveSession::AbsoluteTimeout).date_time().not_null())
                    .clone(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Bakery table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(User::Table).clone())
            .await?;
        manager
            .drop_table(Table::drop().table(ActiveSession::Table).clone())
            .await
    }
}

#[derive(Iden)]
pub enum User {
    Table,
    Id,
    DisplayName,
    Email,
    PasswordHash,
}

#[derive(Iden)]
pub enum ActiveSession {
    Table,
    Id,
    UserId,
    Token,
    IdleTimeout,
    AbsoluteTimeout,
}
