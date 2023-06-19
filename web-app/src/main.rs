#![allow(clippy::no_effect_underscore_binding)]
use std::io::Cursor;

use lazy_static::lazy_static;
use rocket::{
    http::{ContentType, Status},
    response::{self, content::RawHtml, Responder},
    serde::json::Json,
    tokio::sync::Mutex,
    Request, Response, State,
};
use serde::{Deserialize, Serialize};
use shared::data::Registration;
use tera::{Context, Tera};
use thiserror::Error;

#[macro_use]
extern crate rocket;

static INDEX: &str = include_str!("../templates/index.html");

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let mut tera = Tera::default();

        match tera.add_raw_template("index", INDEX) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Encountered errors whilst loading templates: {e}");
                std::process::exit(1);
            }
        };

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
                .map(|s| RawHtml(s)),
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

#[launch]
fn rocket() -> _ {
    rocket::build().manage(Mutex::new(AppData::new())).mount(
        "/",
        routes![
            get_status,
            get_registration,
            post_registration,
            put_registration
        ],
    )
}
