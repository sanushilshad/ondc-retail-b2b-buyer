use crate::{
    configuration::get_configuration,
    utils::{configure_database, configure_database_using_sqlx},
};

// use rust_test::configuration::get_configuration;
#[tracing::instrument(name = "Default Migration")]
pub async fn run_migrations() {
    let configuration = get_configuration().expect("Failed to read configuration.");
    configure_database(&configuration.database).await;
}
#[tracing::instrument(name = "Migrate using Sqlx")]
pub async fn migrate_using_sqlx() {
    let configuration = get_configuration().expect("Failed to read configuration.");
    configure_database_using_sqlx(&configuration.database).await;
}
