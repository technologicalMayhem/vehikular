use std::collections::HashMap;

use rocket::futures::StreamExt;
use sqlx::{Pool, Postgres};

use crate::error::Error;

struct Migration {
    name: &'static str,
    statements: &'static str,
}

macro_rules! migration {
    ($name:literal) => {
        Migration {
            name: $name,
            statements: include_str!($name),
        }
    };
}

static MIGRATIONS: &[Migration] = &[migration!("migration_000001_initial.sql")];

async fn migrate(db: &Pool<Postgres>) -> Result<bool, Error> {



    let succesful = sqlx::query("")
        .execute_many(db)
        .await
        .all(|f| async {
            if let Err(e) = f {
                error!("Encountered an error whilst applying migration: {e}");
                false
            } else {
                true
            }
        })
        .await;

    Ok(succesful)
}
