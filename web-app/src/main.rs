#![allow(clippy::no_effect_underscore_binding)]
use authentication::Authentication;
use rocket::{
    form::Form,
    response::{content::RawCss, Redirect},
    serde::json::Json,
    State,
};
use shared::data::Registration;
use sqlx::{Pool, Postgres};
use templates::{TemplateFairing, Webpage};

use database::{self as db, entities::user};
use db::fairing::DatabaseFairing;
use error::{Error, RegistrationResult};

use crate::templates::PageRenderer;

mod authentication;
mod database;
mod error;
mod templates;

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
        db::get_registration_with_history_and_notes(db, reg_num).await?;

    let notes = notes.map_or(String::new(), |f| f.body);

    renderer.registration(&registration, &notes, &history).await
}

#[post("/registration", format = "application/json", data = "<registration>")]
async fn post_registration(
    registration: Json<Registration>,
    db: &State<Pool<Postgres>>,
) -> Result<RegistrationResult, Error> {
    db::insert_registration(db, registration.0).await?;
    Ok(RegistrationResult::NoContent)
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
    user: user::Model,
    form: Form<NewMaintenanceItemForm<'_>>,
    db: &State<Pool<Postgres>>,
) -> Result<Redirect, Error> {
    let registration = db::get_registration(db, form.registration_number).await?;
    if let Some(registration) = registration {
        let date_time = chrono::naive::NaiveDateTime::from_timestamp_millis(
            form.datetime.assume_utc().unix_timestamp() * 1000,
        )
        .ok_or(Error::DateParseFailure(false))?;

        db::insert_maintenance_item(
            db,
            registration.id,
            date_time,
            form.subject,
            form.body,
            form.mileage,
            user.id,
        )
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
    db: &State<Pool<Postgres>>,
) -> Result<Redirect, Error> {
    db::update_or_insert_notes(db, form.registration_number, form.body).await?;
    Ok(Redirect::to(uri!(get_registration(
        form.registration_number
    ))))
}

#[get("/")]
async fn index(
    db: &State<Pool<Postgres>>,
    mut renderer: PageRenderer<'_>,
) -> Result<Webpage, Error> {
    renderer.index(db::get_all_registrations(db).await?).await
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
                post_maintenance_item,
                update_notes,
            ],
        )
}
