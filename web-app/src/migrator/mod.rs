use sea_orm_migration::prelude::*;

pub mod m20230622_000001_create_registraition;
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20230622_000001_create_registraition::Migration)]
    }
}
