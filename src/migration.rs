use crate::{
    configuration::get_configuration,
    utils::{configure_database, configure_database_using_sqlx},
};

// use rust_test::configuration::get_configuration;
pub async fn run_migrations() {
    let configuration = get_configuration().expect("Failed to read configuration.");
    configure_database(&configuration.database).await;
}

pub async fn migrate_using_sqlx() {
    let configuration = get_configuration().expect("Failed to read configuration.");
    configure_database_using_sqlx(&configuration.database).await;
}
