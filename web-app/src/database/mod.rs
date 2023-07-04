use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, ModelTrait,
    QueryFilter,
};

use entities::prelude::*;

use crate::Error;

use self::entities::{car_registration, maintenance_history, vehicle_notes};

pub mod convert;
pub mod entities;

pub async fn get_registration(
    db: &DatabaseConnection,
    reg_num: &str,
) -> Result<Option<car_registration::Model>, Error> {
    CarRegistration::find()
        .filter(car_registration::Column::RegistrationNumber.like(reg_num))
        .one(db)
        .await
        .map_err(Error::DatabaseError)
}

pub async fn get_registration_with_history_and_notes(
    db: &DatabaseConnection,
    reg_num: &str,
) -> Result<
    (
        car_registration::Model,
        Option<vehicle_notes::Model>,
        Vec<maintenance_history::Model>,
    ),
    Error,
> {
    let registration = CarRegistration::find()
        .filter(car_registration::Column::RegistrationNumber.like(reg_num))
        .one(db)
        .await?
        .ok_or_else(|| Error::RegistrationNotFound(reg_num.into()))?;

    let notes = registration.find_related(VehicleNotes).one(db).await?;
    let history = registration
        .find_related(MaintenanceHistory)
        .all(db)
        .await?;

    Ok((registration, notes, history))
}

pub async fn update_or_insert_notes(
    db: &DatabaseConnection,
    reg_num: &str,
    notes: &str,
) -> Result<(), Error> {
    info!("Get registration");
    let Some(registration) = get_registration(db, reg_num).await? else {
        return Err(Error::RegistrationNotFound(reg_num.into()));
    };
    info!("Find vehicle notes");
    if let Some(db_notes) = VehicleNotes::find()
        .filter(vehicle_notes::Column::CarId.eq(registration.id))
        .one(db)
        .await?
    {
            info!("Found some updating");
            let notes = vehicle_notes::ActiveModel {
                id: ActiveValue::Unchanged(db_notes.id),
                body: ActiveValue::Set(notes.into()),
                ..Default::default()
            };
            notes.update(db).await?;}
     else {
            info!("Found none, inserting");
            let notes = vehicle_notes::ActiveModel {
                car_id: ActiveValue::Set(registration.id),
                body: ActiveValue::Set(notes.into()),
                ..Default::default()
            };
            notes.insert(db).await?;
        }

    Ok(())
}
