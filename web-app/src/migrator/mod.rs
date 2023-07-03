use sea_orm_migration::prelude::*;

pub mod m20230622_000001_create_registraition;
pub mod m20230629_000002_create_maintenance_history;
pub mod m20230701_000003_create_notes;
pub mod m20230703_000004_create_user;
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20230622_000001_create_registraition::Migration),
            Box::new(m20230629_000002_create_maintenance_history::Migration),
            Box::new(m20230701_000003_create_notes::Migration),
            Box::new(m20230703_000004_create_user::Migration),
        ]
    }
}
