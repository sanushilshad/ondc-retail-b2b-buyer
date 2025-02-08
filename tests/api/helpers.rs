use once_cell::sync::Lazy;
// use ondc_b2b_buyer::constants::TEST_DB;
use ondc_retail_b2b_buyer::{
    configuration::get_configuration,
    database::get_connection_pool,
    startup::Application,
    telemetry::{get_subscriber, init_subscriber},
};
use sqlx::PgPool;
#[allow(dead_code)]
pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
    pub port: u16,
}
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    // let test_log = parse_bool_env_var("TEST_LOG", false);
    let test_log = std::env::var("TEST_LOG")
        .map(|value| value == "true")
        .unwrap_or(false);
    if test_log {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    }
});

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration.");
        // c.database.name = TEST_DB.to_string();
        c.application.port = 0;
        c
    };
    // configure_database(&configuration.database).await;
    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to build application.");
    let application_port = application.port();

    let address = format!("http://127.0.0.1:{}", application_port);
    let _ = tokio::spawn(application.run_until_stopped());

    TestApp {
        address: address,
        db_pool: get_connection_pool(&configuration.database),
        port: application_port,
    }
}
