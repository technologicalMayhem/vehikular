use std::{error::Error, io::Cursor};

use rocket::{serde::json::Json, tokio::sync::Mutex, State, response::{Responder, self}, Request, http::ContentType, Response};
use shared::data::Registration;
use thiserror::Error;

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
async fn post_registration(registration: Json<Registration>, state: &State<Mutex<AppData>>) -> Result<(), RegistrationError> {
    let mut lock = state.lock().await;
    if lock.registrations.iter().any(|reg| reg.registration_number == registration.registration_number) {
        Err(RegistrationError::AlreadyExists)
    } else {
        lock.registrations.push(registration.0);
        Ok(())
    }
}

#[derive(Debug, Error)]
enum RegistrationError {
    #[error("A registration with that registration number already exists.")]
    AlreadyExists
}

#[rocket::async_trait]
impl<'r> Responder<'r, 'static> for RegistrationError {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        let s =self.to_string();
        Response::build()
            .header(ContentType::Plain)
            .sized_body(s.len(), Cursor::new(s))
            .ok()
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(Mutex::new(AppData::new()))
        .mount("/", routes![get_registration, post_registration])
}
