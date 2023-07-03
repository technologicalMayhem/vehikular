use rocket::{
    fairing::{self, Fairing, Info, Kind},
    Build, Rocket,
};
use sea_orm::Database;
use sea_orm_migration::MigratorTrait;

use crate::migrator::Migrator;

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
        let db = match Database::connect(&self.connection_string).await {
            Ok(db) => db,
            Err(e) => {
                error!(
                    "Failed to connect to database ({}): {e}",
                    self.connection_string
                );
                return Err(rocket);
            }
        };

        match Migrator::get_pending_migrations(&db).await {
            Ok(migrations) => {
                info!("{} database migrations pending.", migrations.len());
                #[allow(clippy::cast_possible_truncation)]
                // Truncating is fine as there should never be more than 4294967295 pending migrations. I hope...
                let result = Migrator::up(&db, Some(migrations.len() as u32)).await;

                if let Err(e) = result {
                    error!("Failed to apply pending migrations: {e}");
                    std::process::exit(1);
                } else {
                    info!("Database migrations succesfully applied!");
                }
            }
            Err(e) => {
                error!("Failed to get pending migrations: {e}");
                return Err(rocket);
            }
        };

        Ok(rocket.manage(db))
    }
}
