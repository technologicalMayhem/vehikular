use rocket::{
    fairing::{self, Fairing, Info, Kind},
    Build, Rocket,
};
use sqlx::postgres::PgPoolOptions;

pub struct DatabaseFairing {
    connection_string: String,
}

impl DatabaseFairing {
    pub fn fairing(connection_string: &str) -> Self {
        Self {
            connection_string: connection_string.into(),
        }
    }
}

#[rocket::async_trait]
impl Fairing for DatabaseFairing {
    fn info(&self) -> Info {
        Info {
            name: "Database",
            kind: Kind::Ignite | Kind::Singleton,
        }
    }

    async fn on_ignite(&self, rocket: Rocket<Build>) -> fairing::Result {
        let db = match PgPoolOptions::new()
            .max_connections(5)
            .connect(&self.connection_string)
            .await
        {
            Ok(pool) => pool,
            Err(e) => {
                error!("Could not establish connection to database: {e}");
                return Err(rocket);
            }
        };

        Ok(rocket.manage(db))
    }
}
