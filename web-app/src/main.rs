#![allow(clippy::no_effect_underscore_binding)]
use authentication::Authentication;
use rocket::{
    form::Form,
    response::{content::RawCss, Redirect},
    serde::json::Json,
    State,
};
use sea_orm::{ActiveValue, DatabaseConnection, EntityTrait};
use shared::data::Registration;
use sqlx::{Pool, Postgres};
use templates::{TemplateFairing, Webpage};

use database::{
    entities::{
        car_registration,
        maintenance_history::{self, ActiveModel},
    },
    fairing::DatabaseFairing,
    get_all_registrations, get_registration as db_get_registration,
    get_registration_with_history_and_notes, update_or_insert_notes,
};
use error::{Error, RegistrationError, RegistrationResult};

use crate::templates::PageRenderer;

mod authentication;
mod database;
mod error;
mod migrator;
mod templates;
mod user;

#[macro_use]
extern crate rocket;

#[get("/style.css")]
async fn get_style(renderer: PageRenderer<'_>) -> RawCss<String> {
    renderer.style().await
}

#[get("/registration/<reg_num>")]
async fn get_registration(
    reg_num: &str,
    db: &State<Pool<Postgres>>,
    mut renderer: PageRenderer<'_>,
) -> Result<Webpage, Error> {
    let (registration, notes, history) =
        get_registration_with_history_and_notes(db, reg_num).await?;

    let notes = notes.map_or(String::new(), |f| f.body);

    let registration = match Registration::try_from(registration) {
        Ok(reg) => reg,
        Err(e) => return Err(Error::InternalConversionFailed(e)),
    };

    renderer.registration(&registration, &notes, &history).await
}

#[post("/registration", format = "application/json", data = "<registration>")]
async fn post_registration(
    registration: Json<Registration>,
    db: &State<DatabaseConnection>,
    sqlx: &State<Pool<Postgres>>,
) -> Result<RegistrationResult, Error> {
    let db = db as &DatabaseConnection;

    if db_get_registration(sqlx, &registration.registration_number)
        .await?
        .is_some()
    {
        Err(RegistrationError::AlreadyExists)?
    } else {
        let model = car_registration::ActiveModel::from(registration.0);
        car_registration::Entity::insert(model).exec(db).await?;
        Ok(RegistrationResult::NoContent)
    }
}

#[put(
    "/registration",
    format = "application/json",
    data = "<new_registration>"
)]
async fn put_registration(
    new_registration: Json<Registration>,
    db: &State<Pool<Postgres>>,
) -> Result<RegistrationResult, Error> {
    let registration = db_get_registration(db, &new_registration.registration_number).await?;
    if let Some(old_registration) = registration {
        let mut active_model: car_registration::ActiveModel = new_registration.0.into();
        active_model.id = ActiveValue::Set(old_registration.id);
        Ok(RegistrationResult::NoContent)
    } else {
        Err(RegistrationError::DoesNotExist)?
    }
}

#[derive(Debug, FromForm)]
struct NewMaintenanceItemForm<'r> {
    registration_number: &'r str,
    datetime: time::PrimitiveDateTime,
    subject: &'r str,
    body: &'r str,
    mileage: i32,
}

#[post("/maintenance", data = "<form>")]
async fn post_maintenance_item(
    form: Form<NewMaintenanceItemForm<'_>>,
    db: &State<DatabaseConnection>,
    sqlx: &State<Pool<Postgres>>,
) -> Result<Redirect, Error> {
    let db = db as &DatabaseConnection;
    let registration = db_get_registration(sqlx, form.registration_number).await?;
    if let Some(registration) = registration {
        let date_time = chrono::naive::NaiveDateTime::from_timestamp_millis(
            form.datetime.assume_utc().unix_timestamp() * 1000,
        )
        .ok_or(Error::DateParseFailure(false))?;
        let maintenance_item = ActiveModel {
            id: ActiveValue::NotSet,
            car_id: ActiveValue::Set(registration.id),
            date_time: ActiveValue::Set(date_time),
            subject: ActiveValue::Set(form.subject.into()),
            body: ActiveValue::Set(form.body.into()),
            mileage: ActiveValue::Set(Some(form.mileage)),
        };

        maintenance_history::Entity::insert(maintenance_item)
            .exec(db)
            .await?;

        Ok(Redirect::to(uri!(get_registration(
            form.registration_number
        ))))
    } else {
        Err(Error::RegistrationNotFound(form.registration_number.into()))
    }
}

#[derive(FromForm)]
struct UpdateNotesForm<'r> {
    registration_number: &'r str,
    body: &'r str,
}

#[post("/updateNotes", data = "<form>")]
async fn update_notes(
    form: Form<UpdateNotesForm<'_>>,
    db: &State<DatabaseConnection>,
    sqlx: &State<Pool<Postgres>>,
) -> Result<Redirect, Error> {
    update_or_insert_notes(db, sqlx, form.registration_number, form.body).await?;
    Ok(Redirect::to(uri!(get_registration(
        form.registration_number
    ))))
}

#[get("/")]
async fn index(
    db: &State<Pool<Postgres>>,
    mut renderer: PageRenderer<'_>,
) -> Result<Webpage, Error> {
    renderer.index(get_all_registrations(db).await?).await
}

const DATABASE_URL: &str = "postgres://vehikular:vehikular@localhost:5432/vehikular";

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(DatabaseFairing::fairing(DATABASE_URL))
        .attach(TemplateFairing::fairing())
        .attach(Authentication::fairing())
        .mount(
            "/",
            routes![
                get_style,
                index,
                get_registration,
                post_registration,
                put_registration,
                post_maintenance_item,
                update_notes,
            ],
        )
}
