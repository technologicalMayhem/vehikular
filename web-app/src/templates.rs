use std::{convert::Into, env, fs, path::PathBuf};

use include_dir::{include_dir, Dir};
use rocket::{
    fairing::{self, Fairing, Info, Kind},
    http::Status,
    request::{self, FromRequest, Outcome},
    response::{
        content::{RawCss, RawHtml},
        Responder,
    },
    tokio::sync::RwLock,
    Build, Request, Rocket, State,
};
use shared::data::Registration;
use tera::{Context, Tera};
use thiserror::Error;

use crate::{
    database::entities::{car_registration, maintenance_history, user},
    error::Error,
};

static TEMPLATE_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates");
static STYLE: &str = include_str!("../webroot/style.css");

#[derive(Debug, Error)]
pub enum TemplateError {
    #[error("Could not read directory '{0}'. {1}")]
    FailedToReadDirectory(PathBuf, std::io::Error),
    #[error("Tera encountered an error. {0}")]
    TeraError(#[from] tera::Error),
    #[error("Failed to read file. {0}")]
    FileReadError(std::io::Error),
}

pub struct TemplateFairing;

impl TemplateFairing {
    pub fn fairing() -> Self {
        Self {}
    }
}

#[rocket::async_trait]
impl Fairing for TemplateFairing {
    fn info(&self) -> Info {
        Info {
            name: "Template",
            kind: Kind::Ignite | Kind::Singleton,
        }
    }

    async fn on_ignite(&self, rocket: Rocket<Build>) -> fairing::Result {
        let debug_mode = if let Ok(path) = env::var("TEMPLATE_DIR") {
            if let Ok(path) = PathBuf::try_from(&path) {
                Some(path)
            } else {
                error!("Could not load alternative templates. '{path}' is not a valid path.");
                return Err(rocket);
            }
        } else {
            None
        };

        let rocket = if debug_mode.is_some() {
            rocket.mount("/template", routes![refresh])
        } else {
            rocket
        };

        let page_renderer = match Templates::new(debug_mode) {
            Ok(page_renderer) => page_renderer,
            Err(e) => {
                error!("Could not create page renderer. {e}");
                return Err(rocket);
            }
        };

        Ok(rocket.manage(page_renderer))
    }
}

#[get("/refresh")]
async fn refresh(template: &State<Templates>) -> Result<(), Error> {
    template.refresh().await?;
    Ok(())
}

pub struct Webpage(RawHtml<String>);

impl From<String> for Webpage {
    fn from(value: String) -> Self {
        Self(RawHtml(value))
    }
}

impl<'r> Responder<'r, 'static> for Webpage {
    fn respond_to(self, request: &'r Request<'_>) -> rocket::response::Result<'static> {
        self.0.respond_to(request)
    }
}

pub struct Templates {
    debug_mode: Option<PathBuf>,
    tera: RwLock<Tera>,
    style: RwLock<String>,
}

impl Templates {
    fn new(debug_mode: Option<PathBuf>) -> Result<Self, Error> {
        let tera = RwLock::new(load_templates(&debug_mode)?);
        let style = RwLock::new(load_styling(&debug_mode)?);

        Ok(Self {
            debug_mode,
            tera,
            style,
        })
    }

    async fn refresh(&self) -> Result<(), Error> {
        let mut tera = self.tera.write().await;
        *tera = load_templates(&self.debug_mode)?;

        let mut style = self.style.write().await;
        *style = load_styling(&self.debug_mode)?;
        Ok(())
    }
}

pub struct PageRenderer<'r> {
    templates: &'r Templates,
    context: Context,
}

impl<'r> PageRenderer<'r> {
    pub async fn style(&self) -> RawCss<String> {
        RawCss(self.templates.style.read().await.clone())
    }

    pub async fn index(
        &mut self,
        vehicles: Vec<car_registration::Model>,
    ) -> Result<Webpage, Error> {
        self.context.insert("registrations", &vehicles);

        Ok(self
            .templates
            .tera
            .read()
            .await
            .render("index", &self.context)
            .map(Into::into)?)
    }

    pub async fn registration(
        &mut self,
        registration: &Registration,
        notes: &str,
        history: &Vec<maintenance_history::Model>,
    ) -> Result<Webpage, Error> {
        self.context.insert("registration", &registration);
        self.context.insert("notes", &notes);
        self.context.insert("history", &history);

        Ok(self
            .templates
            .tera
            .read()
            .await
            .render("vehicle", &self.context)
            .map(Into::into)?)
    }

    pub async fn register(&self) -> Result<Webpage, Error> {
        Ok(self
            .templates
            .tera
            .read()
            .await
            .render("register", &self.context)
            .map(Into::into)?)
    }

    pub async fn login(&self) -> Result<Webpage, Error> {
        Ok(self
            .templates
            .tera
            .read()
            .await
            .render("login", &self.context)
            .map(Into::into)?)
    }

    pub async fn account_page(&self) -> Result<Webpage, Error> {
        Ok(self
            .templates
            .tera
            .read()
            .await
            .render("account_page", &self.context)
            .map(Into::into)?)
    }
}

fn load_styling(debug_mode: &Option<PathBuf>) -> Result<String, Error> {
    if let Some(path) = debug_mode {
        Ok(fs::read_to_string(path.join("webroot/style.css"))
            .map_err(TemplateError::FileReadError)?)
    } else {
        Ok(STYLE.to_string())
    }
}

fn load_templates(debug_mode: &Option<PathBuf>) -> Result<Tera, Error> {
    let mut templates = Vec::new();
    if let Some(path) = debug_mode {
        let files = path
            .join("templates")
            .read_dir()
            .map_err(|e| TemplateError::FailedToReadDirectory(path.clone(), e))?
            .flatten();
        for file in files {
            if let Some(name) = file.path().file_stem() {
                let contents =
                    fs::read_to_string(file.path()).map_err(TemplateError::FileReadError)?;
                templates.push((name.to_string_lossy().to_string(), contents));
            }
        }
    } else {
        for file in TEMPLATE_DIR.files() {
            if let Some(filename) = file.path().file_stem() {
                let filename = filename.to_string_lossy();
                let template = String::from_utf8_lossy(file.contents());
                templates.push((filename.to_string(), template.to_string()));
            }
        }
    }

    let mut tera = Tera::default();
    tera.add_raw_templates(templates)?;
    Ok(tera)
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for PageRenderer<'r> {
    type Error = Error;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let mut context = Context::default();
        let user = req.guard::<user::Model>().await;
        match user {
            Outcome::Success(user) => context.insert("user", &user),
            Outcome::Failure(_) | Outcome::Forward(_) => {}
        }

        let guard = req.guard::<&State<Templates>>().await;
        let templates = match guard {
            Outcome::Success(templates) => templates,
            Outcome::Failure(_) => {
                return Outcome::Failure((Status::InternalServerError, Error::TemplateNotFound))
            }
            Outcome::Forward(f) => return Outcome::Forward(f),
        };

        Outcome::Success(PageRenderer { templates, context })
    }
}
