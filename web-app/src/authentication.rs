use argon2::Argon2;
use argon2::PasswordHash;
use argon2::PasswordVerifier;
use rocket::http::Cookie;
use rocket::{
    fairing::{self, Fairing, Info, Kind},
    form::Form,
    http::{CookieJar, Status},
    request::{FromRequest, Outcome},
    response::Redirect,
    Build, Request, Rocket, State,
};
use sqlx::Pool;
use sqlx::Postgres;

use crate::database::create_token;
use crate::{
    database::{self, entities::user, get_user_by_email, get_user_by_token},
    error::Error,
    templates::{PageRenderer, Webpage},
};

pub struct Authentication {}

impl Authentication {
    pub(crate) fn fairing() -> Self {
        Self {}
    }
}

#[rocket::async_trait]
impl Fairing for Authentication {
    fn info(&self) -> Info {
        Info {
            name: "Authentication",
            kind: Kind::Ignite | Kind::Singleton,
        }
    }

    async fn on_ignite(&self, rocket: Rocket<Build>) -> fairing::Result {
        Ok(rocket.mount(
            "/account",
            routes![
                get,
                register_get,
                register_post,
                login_get,
                login_post,
                logout
            ],
        ))
    }
}

#[derive(Debug, FromForm)]
struct RegistrationForm<'r> {
    email: &'r str,
    username: &'r str,
    password: &'r str,
}

#[derive(FromForm)]
struct LoginForm<'r> {
    email: &'r str,
    password: &'r str,
}

#[get("/")]
async fn get(_user: user::Model, renderer: PageRenderer<'_>) -> Result<Webpage, Error> {
    renderer.account_page().await
}

#[get("/register")]
async fn register_get(renderer: PageRenderer<'_>) -> Result<Webpage, Error> {
    renderer.register().await
}

#[post("/register", data = "<form>")]
async fn register_post(form: Form<RegistrationForm<'_>>, db: &State<Pool<Postgres>>) -> Redirect {
    match database::create_user(db, form.email, form.username, form.password).await {
        Ok(_) => Redirect::to("/"),
        Err(_) => Redirect::to("/account/register"),
    }
}

#[get("/login")]
async fn login_get(renderer: PageRenderer<'_>) -> Result<Webpage, Error> {
    renderer.login().await
}

#[post("/login", data = "<form>")]
async fn login_post(
    form: Form<LoginForm<'_>>,
    db: &State<Pool<Postgres>>,
    cookies: &CookieJar<'_>,
) -> Result<Redirect, Error> {
    let Some(user) = get_user_by_email(db, form.email).await? else {
        return Err(Error::UserNotFoundEmail(form.email.to_string()))
    };

    let argon2 = Argon2::default();
    if argon2
        .verify_password(
            form.password.as_bytes(),
            &PasswordHash::new(&user.password_hash)?,
        )
        .is_ok()
    {
        let token = create_token(db, user.id).await?;
        cookies.add(Cookie::build("LoginToken", token.token).finish());
        Ok(Redirect::to(uri!("/")))
    } else {
        Ok(Redirect::to(uri!("/account/login")))
    }
}

#[get("/logout")]
fn logout(cookies: &CookieJar<'_>) -> Redirect {
    cookies.remove(Cookie::named("LoginToken"));
    Redirect::to("/")
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for user::Model {
    type Error = crate::error::Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let Some(cookie) = req.cookies().get("LoginToken") else {
            return Outcome::Forward(());
        };
        let Some(db) = req.rocket().state::<Pool<Postgres>>() else {
            return Outcome::Failure((Status::InternalServerError, Error::DatabaseNotFound))
        };

        match get_user_by_token(db, cookie.value()).await {
            Ok(user) => match user {
                Some(user) => Outcome::Success(user),
                None => Outcome::Forward(()),
            },
            Err(e) => Outcome::Failure((Status::InternalServerError, e)),
        }
    }
}
