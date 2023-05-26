#![allow(clippy::no_effect_underscore_binding)]
use std::io::Cursor;

use rocket::{
    http::{ContentType, Status},
    response::{self, Responder},
    serde::json::Json,
    tokio::sync::Mutex,
    Request, Response, State,
};
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
) -> Result<(), RegistrationError> {
    let mut lock = state.lock().await;
    if lock
        .registrations
        .iter()
        .any(|reg| reg.registration_number == registration.registration_number)
    {
        Err(RegistrationError::AlreadyExists)
    } else {
        lock.registrations.push(registration.0);
        Ok(())
    }
}

#[derive(Debug)]
enum RegistrationError {
    AlreadyExists,
}

impl ErrorResponder for RegistrationError {
    fn response(&self) -> (Status, String) {
        match self {
            RegistrationError::AlreadyExists => (Status::Conflict, "A registration with that registration number already exists.".to_string()),
        }
    }
}

trait ErrorResponder {
    fn response(&self) -> (Status, String);
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

#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(Mutex::new(AppData::new()))
        .mount("/", routes![get_registration, post_registration])
}
