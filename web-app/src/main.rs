#![allow(clippy::no_effect_underscore_binding)]
use include_dir::{include_dir, Dir};
use lazy_static::lazy_static;
use migrator::Migrator;
use rocket::{
    form::Form,
    response::{
        content::{RawCss, RawHtml},
        Redirect,
    },
    serde::json::Json,
    State,
};
use sea_orm::{ActiveValue, Database, DatabaseConnection, EntityTrait};
use sea_orm_migration::MigratorTrait;
use shared::data::Registration;
use tera::{Context, Tera};

use database::{
    entities::{
        car_registration,
        maintenance_history::{self, ActiveModel},
    },
    get_registration as db_get_registration, get_registration_with_history_and_notes,
    update_or_insert_notes,
};
use error::{Error, RegistrationError, RegistrationResult};

mod database;
mod error;
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

#[get("/style.css")]
fn get_style() -> RawCss<&'static str> {
    RawCss(STYLE)
}

#[get("/registration/<reg_num>")]
async fn get_registration(
    reg_num: &str,
    db: &State<DatabaseConnection>,
) -> Result<RawHtml<String>, Error> {
    let db = db as &DatabaseConnection;
    let (registration, notes, history) =
        get_registration_with_history_and_notes(db, reg_num).await?;

    let registration = match Registration::try_from(registration) {
        Ok(reg) => reg,
        Err(e) => return Err(Error::InternalConversionFailed(e)),
    };
    let mut context = Context::new();
    context.insert("registration", &registration);
    context.insert("notes", &notes);
    context.insert("history", &history);

    TEMPLATES
        .render("index", &context)
        .map_err(Error::TeraRendering)
        .map(RawHtml)
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
) -> Result<Redirect, Error> {
    update_or_insert_notes(db, form.registration_number, form.body).await?;
    Ok(Redirect::to(uri!(get_registration(
        form.registration_number
    ))))
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

    match Migrator::get_pending_migrations(&db).await {
        Ok(migrations) => {
            let result = Migrator::up(&db, Some(migrations.len() as u32)).await;

            if let Err(e) = result {
                eprintln!("Failed to apply pending migrations: {e}");
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Failed to get pending migrations: {e}");
            std::process::exit(1);
        }
    };

    rocket::build().manage(db).mount(
        "/",
        routes![
            get_style,
            get_registration,
            post_registration,
            put_registration,
            post_maintenance_item,
            update_notes,
        ],
    )
}
