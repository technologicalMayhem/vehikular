#![allow(clippy::no_effect_underscore_binding)]
use std::io::Cursor;

use rocket::{
    http::{ContentType, Status},
    response::{self, Responder},
    serde::json::Json,
    tokio::sync::Mutex,
    Request, Response, State,
};
use serde::{Deserialize, Serialize};
use shared::data::Registration;

#[macro_use]
extern crate rocket;

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
        StatusData { is_debug: cfg!(debug_assertions) }
    }
}

#[get("/status")]
fn get_status() -> Json<StatusData> {
    Json(StatusData::new())
}

#[get("/registration/<reg_num>")]
async fn get_registration(
    reg_num: &str,
    state: &State<Mutex<AppData>>,
) -> Option<Json<Registration>> {
    let data = state.lock().await;
    data.registrations
        .iter()
        .filter_map(|reg| {
            if reg.registration_number == reg_num {
                Some(Json(reg.clone()))
            } else {
                None
            }
        })
        .collect::<Vec<Json<Registration>>>()
        .pop()
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

impl ErrorResponder for RegistrationError {
    fn response(&self) -> (Status, &str) {
        match self {
            RegistrationError::AlreadyExists => (
                Status::Conflict,
                "A registration with that registration number already exists.",
            ),
            RegistrationError::DoesNotExist => (
                Status::BadRequest,
                "Can not update as no registration exits",
            ),
        }
    }
}

trait ErrorResponder {
    fn response(&self) -> (Status, &str);
}

#[rocket::async_trait]
impl<'r> Responder<'r, 'static> for RegistrationError {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        let (status, body) = self.response();
        Response::build()
            .status(status)
            .header(ContentType::Plain)
            .sized_body(body.len(), Cursor::new(String::from(body)))
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
