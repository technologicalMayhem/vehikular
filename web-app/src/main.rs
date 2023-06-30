#![allow(clippy::no_effect_underscore_binding)]
use std::{io::Cursor, str::FromStr};

use chrono::{Local, NaiveDate, NaiveTime};
use include_dir::{include_dir, Dir};
use lazy_static::lazy_static;
use migrator::Migrator;
use rocket::{
    form::{Form, Strict},
    http::{ContentType, Status},
    log::private::info,
    response::{
        self,
        content::{RawCss, RawHtml},
        Responder, Redirect, status::NotFound,
    },
    serde::json::Json,
    Request, Response, State,
};
use sea_orm::{
    prelude::ChronoDateTime, ActiveValue, ColumnTrait, Database, DatabaseConnection, EntityTrait,
    QueryFilter,
};
use sea_orm_migration::MigratorTrait;
use sea_orm_migration::SchemaManager;
use serde::{Deserialize, Serialize};
use shared::data::Registration;
use tera::{Context, Tera};
use thiserror::Error;

use entities::{
    car_registration,
    maintenance_history::{self, ActiveModel},
    prelude::*,
};

mod entities;
mod migrator;

#[macro_use]
extern crate rocket;

static TEMPLATE_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates");
static STYLE: &str = include_str!("../webroot/style.css");

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let mut tera = Tera::default();

        for file in TEMPLATE_DIR.files() {
            if let Some(filename) = file.path().file_stem() {
                let filename = filename.to_string_lossy();
                let template = String::from_utf8_lossy(file.contents());
                let result = tera.add_raw_template(&filename, &template);
                if let Err(e) = result {
                    eprintln!("Encountered errors whilst loading templates: {e}");
                    std::process::exit(1);
                }
            }
        }

        tera
    };
}

#[derive(Debug, Serialize, Deserialize)]
struct StatusData {
    pub is_debug: bool,
}

impl StatusData {
    fn new() -> Self {
        StatusData {
            is_debug: cfg!(debug_assertions),
        }
    }
}

#[get("/status")]
fn get_status() -> Json<StatusData> {
    Json(StatusData::new())
}

#[derive(Debug, Error)]
enum Error {
    #[error("An error occured whilst trying to access the database: {0}")]
    DatabaseError(#[from] sea_orm::error::DbErr),
    #[error("An error occured whilst rendering")]
    TeraRendering(#[from] tera::Error),
    #[error("Could not convert database types to internal types: {0}")]
    InternalConversionFailed(#[from] shared::data::Error),
    #[error("An error occured whilst registering: {0}")]
    RegistrationError(#[from] RegistrationError),
    #[error("Failed to parse date. Was it the first one: {0}")]
    DateParseFailure(bool),
    #[error("No car with {0} as it's registration could be found.")]
    RegistrationNotFound(String)
}

#[get("/style.css")]
fn get_style() -> RawCss<&'static str> {
    RawCss(STYLE)
}

#[get("/registration/<reg_num>")]
async fn get_registration(
    reg_num: &str,
    db: &State<DatabaseConnection>,
) -> Option<Result<RawHtml<String>, Error>> {
    let db = db as &DatabaseConnection;
    let registration = match db_get_registration_with_history(db, reg_num).await {
        Ok(reg) => reg,
        Err(e) => return Some(Err(e)),
    };

    if let Some((registration, history)) = registration {
        let registration = match Registration::try_from(registration) {
            Ok(reg) => reg,
            Err(e) => return Some(Err(Error::InternalConversionFailed(e))),
        };
        let mut context = Context::new();
        context.insert("registration", &registration);
        context.insert("history", &history);
        //println!("{context:#?}");
        Some(
            TEMPLATES
                .render("index", &context)
                .map_err(Error::TeraRendering)
                .map(RawHtml),
        )
    } else {
        None
    }
}

#[post("/registration", format = "application/json", data = "<registration>")]
async fn post_registration(
    registration: Json<Registration>,
    db: &State<DatabaseConnection>,
) -> Result<RegistrationResult, Error> {
    let db = db as &DatabaseConnection;

    if db_get_registration(db, &registration.registration_number)
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
    db: &State<DatabaseConnection>,
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
) -> Result<Redirect, Error> {
    let db = db as &DatabaseConnection;
    let registration = db_get_registration(db, form.registration_number).await?;
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

        Ok(Redirect::to(uri!(get_registration(form.registration_number))))
    } else {
        Err(Error::RegistrationNotFound(form.registration_number.into()))
    }
}

async fn db_get_registration(
    db: &DatabaseConnection,
    reg_num: &str,
) -> Result<Option<car_registration::Model>, Error> {
    CarRegistration::find()
        .filter(car_registration::Column::RegistrationNumber.like(reg_num))
        .one(db)
        .await
        .map_err(Error::DatabaseError)
}

async fn db_get_registration_with_history(
    db: &DatabaseConnection,
    reg_num: &str,
) -> Result<Option<(car_registration::Model, Vec<maintenance_history::Model>)>, Error> {
    CarRegistration::find()
        .filter(car_registration::Column::RegistrationNumber.like(reg_num))
        .find_with_related(maintenance_history::Entity)
        .all(db)
        .await
        .map(|mut r| r.pop())
        .map_err(Error::DatabaseError)
}

enum RegistrationResult {
    NoContent,
}

#[derive(Debug, Error)]
enum RegistrationError {
    #[error("A registration with that registration number already exists.")]
    AlreadyExists,
    #[error("Can not update as no registration exits")]
    DoesNotExist,
}

impl ErrorResponder for Error {
    fn response(&self) -> (Status, String) {
        (
            match self {
                Error::TeraRendering(_)
                | Error::DatabaseError(_)
                | Error::DateParseFailure(_)
                | Error::InternalConversionFailed(_) => Status::InternalServerError,
                Error::RegistrationNotFound(_) => Status::NotFound,
                Error::RegistrationError(reg) => return reg.response(),
            },
            self.to_string(),
        )
    }
}

impl ErrorResponder for RegistrationError {
    fn response(&self) -> (Status, String) {
        (
            match self {
                RegistrationError::AlreadyExists => Status::Conflict,
                RegistrationError::DoesNotExist => Status::BadRequest,
            },
            self.to_string(),
        )
    }
}

trait ErrorResponder {
    fn response(&self) -> (Status, String);
}

#[rocket::async_trait]
impl<'r> Responder<'r, 'static> for Error {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        let (status, body) = self.response();
        Response::build()
            .status(status)
            .header(ContentType::Plain)
            .sized_body(body.len(), Cursor::new(body))
            .ok()
    }
}

#[rocket::async_trait]
impl<'r> Responder<'r, 'static> for RegistrationError {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        let (status, body) = self.response();
        Response::build()
            .status(status)
            .header(ContentType::Plain)
            .sized_body(body.len(), Cursor::new(body))
            .ok()
    }
}

impl<'r> Responder<'r, 'static> for RegistrationResult {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        match self {
            RegistrationResult::NoContent => Response::build()
                .status(Status::NoContent)
                .header(ContentType::Plain)
                .ok(),
        }
    }
}

const DATABASE_URL: &str = "postgres://vehikular:vehikular@localhost:5432/vehikular";

#[launch]
async fn rocket() -> _ {
    let db = match Database::connect(DATABASE_URL).await {
        Ok(db) => db,
        Err(e) => {
            eprint!("Failed to connect to database ({DATABASE_URL}): {e}");
            std::process::exit(1);
        }
    };

    let schema_manager = SchemaManager::new(&db);
    match Migrator::get_pending_migrations(&db).await {
        Ok(migrations) => {
            let result = Migrator::up(&db, Some(migrations.len() as u32)).await;

            if let Err(e) = result {
                eprintln!("Failed to get pending migrations: {e}");
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Failed to get pending migrations: {e}");
            std::process::exit(1);
        }
    };

    if !schema_manager
        .has_table("car_registration")
        .await
        .unwrap_or(false)
    {
        eprintln!("No car_registration table found. Something went wrong with the database.");
        std::process::exit(1);
    }

    rocket::build().manage(db).mount(
        "/",
        routes![
            get_style,
            get_status,
            get_registration,
            post_registration,
            put_registration,
            post_maintenance_item
        ],
    )
}
