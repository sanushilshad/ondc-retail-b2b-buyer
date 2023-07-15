use once_cell::sync::Lazy;
use rust_test::{
    configuration::{get_configuration, DatabaseSettings},
    telemetry::{get_subscriber, init_subscriber},
};
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
// use uuid::Uuid;
pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

#[actix_web::test]
async fn health_check_works() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();
    // Act
    let response = client
        .get(&format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(7), response.content_length());
}
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber)
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    };
});

// #[actix_web::test]
// async fn health_check_works2() {
//     let app = spawn_app().await;

//     let client = reqwest::Client::new();
//     // Act
//     let response = client
//         .get(&format!("{}/health_check", &app.address))
//         .send()
//         .await
//         .expect("Failed to execute request.");
//     // Assert
//     assert!(response.status().is_success());
//     assert_eq!(Some(7), response.content_length());
// }
async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);
    let mut configuration = get_configuration().expect("Failed to read configuration.");
    // configuration.database.name = Uuid::new_v4().to_string();
    configuration.database.name = "rust_test_db".to_string();
    let connection_pool = configure_database(&configuration.database).await;
    let server =
        rust_test::startup::run(listener, connection_pool.clone()).expect("Failed to bind address");
    let _ = tokio::spawn(server);
    // format!("http://127.0.0.1:{}", port)
    TestApp {
        address,
        db_pool: connection_pool,
    }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create database
    let mut connection = PgConnection::connect(&config.connection_string_without_db())
        .expose_secret()
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.name).as_str())
        .await
        .expect("Failed to create database.");
    // Migrate database
    let connection_pool = PgPool::connect(&config.connection_string())
        .expose_secret()
        .await
        .expect("Failed to connect to Postgres.");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");
    connection_pool
}
