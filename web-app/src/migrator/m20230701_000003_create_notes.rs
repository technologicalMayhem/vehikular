use sea_orm_migration::prelude::*;

use super::m20230622_000001_create_registraition::CarRegistration;

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
                    .table(VehicleNotes::Table)
                    .col(
                        ColumnDef::new(VehicleNotes::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-notes-registration")
                            .from(VehicleNotes::Table, VehicleNotes::CarId)
                            .to(CarRegistration::Table, CarRegistration::Id),
                    )
                    .col(ColumnDef::new(VehicleNotes::CarId).integer().not_null())
                    .col(ColumnDef::new(VehicleNotes::Body).string().not_null())
                    .clone(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Bakery table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(VehicleNotes::Table).clone())
            .await
    }
}

#[derive(Iden)]
pub enum VehicleNotes {
    Table,
    Id,
    CarId,
    Body,
}
