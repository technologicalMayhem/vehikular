use std::io::Cursor;

use rocket::{
    http::{ContentType, Status},
    response::{self, Responder},
    Request, Response,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
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
    RegistrationNotFound(String),
}

#[derive(Debug, Error)]
pub enum RegistrationError {
    #[error("A registration with that registration number already exists.")]
    AlreadyExists,
    #[error("Can not update as no registration exits")]
    DoesNotExist,
}

pub enum RegistrationResult {
    NoContent,
}

pub trait ErrorResponder {
    fn response(&self) -> (Status, String);
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
