pub use sea_orm_migration::prelude::*;

mod m20241228_000001_create_orders;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20241228_000001_create_orders::Migration)]
    }
}
