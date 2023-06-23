#![allow(clippy::no_effect_underscore_binding)]
use std::io::Cursor;

use include_dir::{include_dir, Dir};
use lazy_static::lazy_static;
use migrator::Migrator;
use rocket::{
    http::{ContentType, Status},
    response::{
        self,
        content::{RawCss, RawHtml},
        Responder,
    },
    serde::json::Json,
    tokio::sync::Mutex,
    Request, Response, State,
};
use sea_orm::Database;
use sea_orm_migration::{MigratorTrait, SchemaManager};
use serde::{Deserialize, Serialize};
use shared::data::Registration;
use tera::{Context, Tera};
use thiserror::Error;

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

struct AppData {
    registrations: Vec<Registration>,
}

impl AppData {
    fn new() -> Self {
        Self {
            registrations: vec![],
        }
    }
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
    #[error("An error occured whilst rendering")]
    TeraRendering(#[from] tera::Error),
}

#[get("/style.css")]
fn get_style() -> RawCss<&'static str> {
    RawCss(STYLE)
}

#[get("/registration/<reg_num>")]
async fn get_registration(
    reg_num: &str,
    state: &State<Mutex<AppData>>,
) -> Option<Result<RawHtml<String>, Error>> {
    let data = state.lock().await;
    let registration = data
        .registrations
        .iter()
        .find(|reg| reg.registration_number == reg_num);
    if let Some(registration) = registration {
        let mut context = Context::new();
        context.insert("registration", registration);
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
    state: &State<Mutex<AppData>>,
) -> Result<RegistrationResult, RegistrationError> {
    let mut lock = state.lock().await;
    if lock
        .registrations
        .iter()
        .any(|reg| reg.registration_number == registration.registration_number)
    {
        Err(RegistrationError::AlreadyExists)
    } else {
        lock.registrations.push(registration.0);
        Ok(RegistrationResult::NoContent)
    }
}

#[put("/registration", format = "application/json", data = "<registration>")]
async fn put_registration(
    registration: Json<Registration>,
    state: &State<Mutex<AppData>>,
) -> Result<RegistrationResult, RegistrationError> {
    let mut lock = state.lock().await;
    if let Some(index) = lock
        .registrations
        .iter()
        .position(|reg| reg.registration_number == registration.registration_number)
    {
        lock.registrations[index] = registration.0;
        Ok(RegistrationResult::NoContent)
    } else {
        Err(RegistrationError::DoesNotExist)
    }
}

enum RegistrationResult {
    NoContent,
}

#[derive(Debug)]
enum RegistrationError {
    AlreadyExists,
    DoesNotExist,
}

impl ErrorResponder for Error {
    fn response(&self) -> (Status, String) {
        match self {
            Error::TeraRendering(tera_err) => (
                Status::InternalServerError,
                format!("Tera could not render the template: {tera_err:#?}"),
            ),
        }
    }
}

impl ErrorResponder for RegistrationError {
    fn response(&self) -> (Status, String) {
        match self {
            RegistrationError::AlreadyExists => (
                Status::Conflict,
                "A registration with that registration number already exists.".to_string(),
            ),
            RegistrationError::DoesNotExist => (
                Status::BadRequest,
                "Can not update as no registration exits".to_string(),
            ),
        }
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
    match Migrator::refresh(&db).await {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Failed to refresh database schema: {e}");
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

    rocket::build().manage(Mutex::new(AppData::new())).mount(
        "/",
        routes![
            get_style,
            get_status,
            get_registration,
            post_registration,
            put_registration
        ],
    )
}
