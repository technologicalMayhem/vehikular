use std::ops::Add;

use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher,
};
use chrono::{Days, Duration, Local};
use rand::{distributions::Alphanumeric, Rng};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, ModelTrait,
    QueryFilter, Related,
};

use entities::prelude::*;
use sqlx::{Pool, Postgres};

use crate::error::Error;

use self::entities::{active_session, car_registration, maintenance_history, user, vehicle_notes};

pub mod convert;
pub mod entities;
pub mod fairing;

pub async fn get_registration(
    db: &Pool<Postgres>,
    reg_num: &str,
) -> Result<Option<car_registration::Model>, Error> {
    sqlx::query_as!(
        car_registration::Model,
        "SELECT * FROM car_registration WHERE registration_number = $1",
        reg_num
    )
    .fetch_optional(db)
    .await
    .map_err(Error::SqlxError)
}

pub async fn get_all_registrations(
    db: &Pool<Postgres>,
) -> Result<Vec<car_registration::Model>, Error> {
    sqlx::query_as!(car_registration::Model, "SELECT * FROM car_registration;")
        .fetch_all(db)
        .await
        .map_err(Error::SqlxError)
}

pub async fn get_registration_with_history_and_notes(
    db: &Pool<Postgres>,
    reg_num: &str,
) -> Result<
    (
        car_registration::Model,
        Option<vehicle_notes::Model>,
        Vec<maintenance_history::Model>,
    ),
    Error,
> {
    let registration = sqlx::query_as!(
        car_registration::Model,
        "select *
        from car_registration cr
        where cr.registration_number = $1",
        reg_num
    )
    .fetch_optional(db)
    .await?
    .ok_or_else(|| Error::RegistrationNotFound(reg_num.into()))?;

    let notes = sqlx::query_as!(
        vehicle_notes::Model,
        "select vn.*
        from car_registration cr
        left join vehicle_notes vn on vn.car_id = cr.id
        where cr.registration_number = $1",
        reg_num
    )
    .fetch_optional(db)
    .await?;
    let history = sqlx::query_as!(
        maintenance_history::Model,
        "select mh.*
        from car_registration cr 
        left join maintenance_history mh ON mh.car_id = cr.id
        where cr.registration_number = $1",
        reg_num
    )
    .fetch_all(db)
    .await?;

    Ok((registration, notes, history))
}

struct UpdateInsertNote {
    car_id: i32,
    notes_id: i32,
}

pub async fn update_or_insert_notes(
    db: &DatabaseConnection,
    sqlx: &Pool<Postgres>,
    reg_num: &str,
    notes: &str,
) -> Result<(), Error> {
    let query = sqlx::query_as!(UpdateInsertNote, "select cr.id as car_id, vn.id as notes_id 
    from car_registration cr 
    left join vehicle_notes vn on vn.car_id = cr.id 
    where cr.registration_number = $1", reg_num).fetch_optional(sqlx).await;
    if let Some(db_notes) = VehicleNotes::find()
        .filter(vehicle_notes::Column::CarId.eq(registration.id))
        .one(db)
        .await?
    {
        let notes = vehicle_notes::ActiveModel {
            id: ActiveValue::Unchanged(db_notes.id),
            body: ActiveValue::Set(notes.into()),
            ..Default::default()
        };
        notes.update(db).await?;
    } else {
        let notes = vehicle_notes::ActiveModel {
            car_id: ActiveValue::Set(registration.id),
            body: ActiveValue::Set(notes.into()),
            ..Default::default()
        };
        notes.insert(db).await?;
    }

    Ok(())
}

pub async fn create_user(
    db: &DatabaseConnection,
    email: &str,
    display_name: &str,
    password: &str,
) -> Result<(), Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();

    user::ActiveModel {
        email: ActiveValue::Set(email.into()),
        display_name: ActiveValue::Set(display_name.into()),
        password_hash: ActiveValue::Set(password_hash),
        ..Default::default()
    }
    .insert(db)
    .await?;

    Ok(())
}

pub async fn delete_user(db: &DatabaseConnection, user_id: i32) -> Result<(), Error> {
    user::Entity::delete_by_id(user_id).exec(db).await?;
    Ok(())
}

pub async fn get_users_by_display_name(
    db: &DatabaseConnection,
    display_name: &str,
) -> Result<Vec<user::Model>, Error> {
    Ok(user::Entity::find()
        .filter(user::Column::DisplayName.like(display_name))
        .all(db)
        .await?)
}

pub async fn get_user_by_email(
    db: &DatabaseConnection,
    email: &str,
) -> Result<Option<user::Model>, Error> {
    Ok(user::Entity::find()
        .filter(user::Column::Email.like(email))
        .one(db)
        .await?)
}

pub async fn get_user_by_token(
    db: &DatabaseConnection,
    token: &str,
) -> Result<Option<user::Model>, Error> {
    active_session::Entity::find_related()
        .filter(active_session::Column::Token.like(token))
        .one(db)
        .await
        .map_err(Error::DatabaseError)
}

pub async fn create_token(
    db: &DatabaseConnection,
    user_id: i32,
) -> Result<active_session::Model, Error> {
    let token: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    active_session::ActiveModel {
        user_id: ActiveValue::Set(user_id),
        token: ActiveValue::Set(token),
        idle_timeout: ActiveValue::Set(Local::now().naive_local().add(Duration::hours(2))),
        absolute_timeout: ActiveValue::Set(Local::now().naive_local().add(Days::new(1))),
        ..Default::default()
    }
    .insert(db)
    .await
    .map_err(Error::DatabaseError)
}
