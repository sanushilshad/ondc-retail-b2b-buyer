use crate::configuration::get_configuration;
use crate::utils::configure_database;
// use rust_test::configuration::get_configuration;
pub async fn run_migrations() -> Result<(), Box<dyn std::error::Error>> {
    let configuration = get_configuration().expect("Failed to read configuration.");
    configure_database(&configuration.database).await;
    Ok(())
}
