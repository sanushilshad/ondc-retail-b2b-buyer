// use env_logger::Env;
use rust_test::{
    configuration::get_configuration,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};
use secrecy::ExposeSecret;
use sqlx::postgres;
use std::net::TcpListener;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();  // using logging crate
    let subscriber = get_subscriber("rust_test".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");
    let address = &format!("0.0.0.0:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;
    let connection_pool =
        postgres::PgPool::connect(configuration.database.connection_string().expose_secret())
            .await
            .expect("Failed to connect to Postgres.");
    println!("Listening in {}", address);
    run(listener, connection_pool)?.await
}
