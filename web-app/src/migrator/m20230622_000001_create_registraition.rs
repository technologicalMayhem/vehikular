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
                    .table(CarRegistration::Table)
                    .col(
                        ColumnDef::new(CarRegistration::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(CarRegistration::IssuerState).string().not_null())
                    .col(ColumnDef::new(CarRegistration::IssuerAuthority).string().not_null())
                    .col(ColumnDef::new(CarRegistration::DocumentNumber).string().not_null())
                    .col(ColumnDef::new(CarRegistration::RegistrationNumber).string().not_null())
                    .col(ColumnDef::new(CarRegistration::DateOfFirstRegistration).string().not_null())
                    .col(ColumnDef::new(CarRegistration::VehicleIdentificationNumber).string().not_null())
                    .col(ColumnDef::new(CarRegistration::VehicleMassWithBody).string().not_null())
                    .col(ColumnDef::new(CarRegistration::PeriodOfValidity).string().not_null())
                    .col(ColumnDef::new(CarRegistration::DateOfRegistration).string().not_null())
                    .col(ColumnDef::new(CarRegistration::TypeApprovalNumber).string().not_null())
                    .col(ColumnDef::new(CarRegistration::PowerWeightRatio).string().not_null())
                    .col(ColumnDef::new(CarRegistration::VechicleCategory).string().not_null())
                    .col(ColumnDef::new(CarRegistration::Colour).string().not_null())
                    .col(ColumnDef::new(CarRegistration::MaximumSpeed).string().not_null(),)
                    .col(ColumnDef::new(CarRegistration::VehiclesOwner).string().not_null(),)
                    .col(ColumnDef::new(CarRegistration::SurnameOrBusinessName).string().not_null())
                    .col(ColumnDef::new(CarRegistration::OtherNameOrInitials).string().not_null())
                    .col(ColumnDef::new(CarRegistration::Address).string().not_null())
                    .col(ColumnDef::new(CarRegistration::Make).string().not_null())
                    .col(ColumnDef::new(CarRegistration::VehicleType).string().not_null())
                    .col(ColumnDef::new(CarRegistration::CommercialDescriptons).string().not_null())
                    .col(ColumnDef::new(CarRegistration::MaximumTechnicallyLadenMass).string().not_null())
                    .col(ColumnDef::new(CarRegistration::MaximumLadenMassOfTheVehicleInService,).string().not_null())
                    .col(ColumnDef::new(CarRegistration::MaximumLadenMassOfTheWholeVehicleInService,).string().not_null())
                    .col(ColumnDef::new(CarRegistration::Capacity).string().not_null())
                    .col(ColumnDef::new(CarRegistration::MaxNetPower).string().not_null())
                    .col(ColumnDef::new(CarRegistration::FuelType).string().not_null())
                    .col(ColumnDef::new(CarRegistration::NumberOfSeats).string().not_null())
                    .col(ColumnDef::new(CarRegistration::NunmberOfStandingPlaces).string().not_null())
                    .col(ColumnDef::new(CarRegistration::Braked).string().not_null())
                    .col(ColumnDef::new(CarRegistration::Unbraked).string().not_null())
                    .col(ColumnDef::new(CarRegistration::EnvironmentalCategory).string().not_null())
                    .clone(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Bakery table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(CarRegistration::Table).clone())
            .await
    }
}

#[derive(Iden)]
pub enum CarRegistration {
    Table,
    Id,
    IssuerState,
    IssuerAuthority,
    DocumentNumber,
    RegistrationNumber,
    DateOfFirstRegistration,
    VehicleIdentificationNumber,
    VehicleMassWithBody,
    PeriodOfValidity,
    DateOfRegistration,
    TypeApprovalNumber,
    PowerWeightRatio,
    VechicleCategory,
    Colour,
    MaximumSpeed,
    VehiclesOwner,
    SurnameOrBusinessName,
    OtherNameOrInitials,
    Address,
    Make,
    VehicleType,
    CommercialDescriptons,
    MaximumTechnicallyLadenMass,
    MaximumLadenMassOfTheVehicleInService,
    MaximumLadenMassOfTheWholeVehicleInService,
    Capacity,
    MaxNetPower,
    FuelType,
    NumberOfSeats,
    NunmberOfStandingPlaces,
    Braked,
    Unbraked,
    EnvironmentalCategory,
}
