// use env_logger::Env;
use rust_test::{
    configuration::get_configuration,
    email_client::EmailClient,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};
use sqlx::postgres;
use std::net::TcpListener;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();  // using logging crate
    let configuration = get_configuration().expect("Failed to read configuration.");
    let subscriber = get_subscriber("rust_test".into(), "info".into(), std::io::stdout); // set sink  to `std::io::stdout` to print trace in terminal
    init_subscriber(subscriber);

    let email_client =
        EmailClient::new(configuration.email_client).expect("SMTP connection Failed");
    let address = &format!("0.0.0.0:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;
    let connection_pool = postgres::PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.database.with_db());
    println!("Listening in {}", address);
    run(listener, connection_pool, email_client)?.await
}
