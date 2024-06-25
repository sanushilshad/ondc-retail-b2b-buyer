// use env_logger::Env;
use rust_test::{
    configuration::get_configuration,
    startup::Application,
    telemetry::{get_subscriber_with_jeager, init_subscriber},
    utils::run_custom_commands,
};
#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    // env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();  // using logging crate
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        run_custom_commands(args).await?;
    } else {
        let configuration = get_configuration().expect("Failed to read configuration.");
        let subscriber =
            get_subscriber_with_jeager("rust_test".into(), "info".into(), std::io::stdout); // set sink  to `std::io::stdout` to print trace in terminal
        init_subscriber(subscriber);
        let application = Application::build(configuration).await?;
        application.run_until_stopped().await?;
    }
    Ok(())
}
