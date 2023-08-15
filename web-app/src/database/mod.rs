use std::ops::Add;

use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher,
};
use chrono::{Days, Duration, Local, NaiveDateTime};
use rand::{distributions::Alphanumeric, Rng};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    Related,
};

use shared::data::Registration;
use sqlx::{Pool, Postgres};

use crate::error::{Error, RegistrationError};

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

pub async fn insert_registration(
    db: &Pool<Postgres>,
    registration: Registration,
) -> Result<(), Error> {
    if get_registration(db, &registration.registration_number)
        .await?
        .is_some()
    {
        Err(RegistrationError::AlreadyExists)?
    } else {
        sqlx::query!("insert into car_registration (issuer_state, issuer_authority, document_number, registration_number, date_of_first_registration, vehicle_identification_number, vehicle_mass_with_body, period_of_validity, date_of_registration, type_approval_number, power_weight_ratio, vechicle_category, colour, maximum_speed, vehicles_owner, surname_or_business_name, other_name_or_initials, address, make, vehicle_type, commercial_descriptons, maximum_technically_laden_mass, maximum_laden_mass_of_the_vehicle_in_service, maximum_laden_mass_of_the_whole_vehicle_in_service, capacity, max_net_power, fuel_type, number_of_seats, nunmber_of_standing_places, braked, unbraked, environmental_category)
        values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29, $30, $31, $32)", registration.issuer_state, registration.issuer_authority, registration.document_number, registration.registration_number, registration.date_of_first_registration, registration.vehicle_identification_number, registration.vehicle_mass_with_body, registration.period_of_validity, registration.date_of_registration, registration.type_approval_number, registration.power_weight_ratio, registration.vechicle_category, registration.colour, registration.maximum_speed, registration.personal_data.vehicles_owner.to_string(), registration.personal_data.certificate_holder.surname_or_business_name, registration.personal_data.certificate_holder.other_name_or_initials, registration.personal_data.certificate_holder.address, registration.vehicle.make, registration.vehicle.vehicle_type, registration.vehicle.commercial_descriptons, registration.mass.maximum_technically_permissible_laden_mass, registration.mass.maximum_permissible_laden_mass_of_the_vehicle_in_service, registration.mass.maximum_permissible_laden_mass_of_the_whole_vehicle_in_service, registration.engine.capacity, registration.engine.max_net_power, registration.engine.fuel_type, registration.seating_capacity.number_of_seats, registration.seating_capacity.nunmber_of_standing_places, registration.maximum_towable_mass.braked, registration.maximum_towable_mass.unbraked, registration.exhaust_emissions.environmental_category).execute(db).await?;
        Ok(())
    }
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
    notes_id: Option<i32>,
}

pub async fn update_or_insert_notes(
    db: &Pool<Postgres>,
    reg_num: &str,
    notes: &str,
) -> Result<(), Error> {
    let Some(query) = sqlx::query_as!(
        UpdateInsertNote,
        "select cr.id as car_id, vn.id as notes_id 
        from car_registration cr 
        left join vehicle_notes vn on vn.car_id = cr.id 
        where cr.registration_number = $1",
        reg_num
    )
    .fetch_optional(db)
    .await? else {
        return Err(Error::RegistrationNotFound(reg_num.to_string()))
    };
    if let Some(notes_id) = query.notes_id {
        // Update
        sqlx::query!(
            "update vehicle_notes
            set body = $1
            where id = $2",
            notes,
            notes_id
        )
        .execute(db)
        .await?;
    } else {
        // Insert
        sqlx::query!(
            "insert into vehicle_notes (car_id, body)
            values ($1, $2)",
            query.car_id,
            notes
        )
        .execute(db)
        .await?;
    }

    Ok(())
}

pub async fn insert_maintenance_item(
    db: &Pool<Postgres>,
    car_id: i32,
    date_time: NaiveDateTime,
    subject: &str,
    body: &str,
    mileage: i32,
) -> Result<(), Error> {
    sqlx::query!(
        "insert into maintenance_history (car_id, date_time, subject, body, mileage)
         values ($1, $2, $3, $4, $5)",
        car_id,
        date_time,
        subject.into(),
        body.into(),
        mileage
    )
    .execute(db)
    .await?;
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

pub async fn delete_user(db: &Pool<Postgres>, user_id: i32) -> Result<(), Error> {
    let mut trans = db.begin().await?;

    sqlx::query!("delete from active_session where user_id = $1", user_id)
        .execute(&mut *trans)
        .await?;
    sqlx::query!("delete from \"user\" where id = $1", user_id)
        .execute(&mut *trans)
        .await?;

    trans.commit().await?;
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
