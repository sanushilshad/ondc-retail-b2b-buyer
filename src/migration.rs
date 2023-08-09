use clap::{App, Arg};
use config::{Config, Environment};
use sqlx::migrate::MigrateDatabase;
use sqlx::postgres::PgPoolOptions;

#[derive(Debug, Deserialize)]
struct DatabaseSettings {
    url: String,
}

#[derive(Debug, Deserialize)]
struct Settings {
    database: DatabaseSettings,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let configuration = get_configuration().expect("Failed to read configuration.");

    let matches = App::new("Migration Runner")
        .version("1.0")
        .author("Your Name")
        .about("Run database migrations")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .takes_value(true)
                .required(false)
                .about("Sets the configuration file"),
        )
        .get_matches();
    configure_database(&configuration.database)
}
