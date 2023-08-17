// use env_logger::Env;
use rust_test::{
    configuration::get_configuration,
    startup::Application,
    telemetry::{get_subscriber, init_subscriber},
};
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();  // using logging crate
    let configuration = get_configuration().expect("Failed to read configuration.");
    let subscriber = get_subscriber("rust_test".into(), "info".into(), std::io::stdout); // set sink  to `std::io::stdout` to print trace in terminal
    init_subscriber(subscriber);
    let application = Application::build(configuration).await?;
    application.run_until_stopped().await?;
    Ok(())
}
