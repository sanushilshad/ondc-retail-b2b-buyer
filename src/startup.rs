use crate::configuration::DatabaseSettings;
use crate::email_client::EmailClient;
use crate::routes::fetch_inventory;
use crate::routes::get_customer_dbs_api;
use crate::routes::health_check;
use crate::routes::send_email;
use actix_web::dev::Server;
// use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use sea_orm::ConnectOptions;
use sea_orm::Database;
use sea_orm::DatabaseConnection;
use sea_orm::DbErr;
// use sqlx::postgres;
// use sqlx::postgres::PgPool;
use std::net::TcpListener;
use std::time::Duration;
use tracing_actix_web::TracingLogger;
use tracing_log::log;

use crate::configuration::Settings;

pub struct Application {
    port: u16,
    server: Server,
}
impl Application {
    // We have converted the `build` function into a constructor for
    // `Application`.
    pub async fn build(configuration: Settings) -> Result<Self, std::io::Error> {
        let db_conn = get_connection_connection(&configuration.database)
            .await
            .unwrap();
        let email_client =
            EmailClient::new(configuration.email_client).expect("SMTP connection Failed");
        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        println!("Lisetening to {}", address);
        let listener = TcpListener::bind(&address)?;
        let port = listener.local_addr().unwrap().port();
        let server = run(listener, db_conn, email_client)?;
        // We "save" the bound port in one of `Application`'s fields
        Ok(Self { port, server })
    }
    pub fn port(&self) -> u16 {
        self.port
    }
    // A more expressive name that makes it clear that
    // this function only returns when the application is stopped.
    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub async fn get_connection_connection(
    configuration: &DatabaseSettings,
) -> Result<DatabaseConnection, DbErr> {
    // postgres::PgPoolOptions::new()
    //     .acquire_timeout(std::time::Duration::from_secs(2))
    //     .connect_lazy_with(configuration.with_db())

    let mut opt = ConnectOptions::new(configuration.connection_string());
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(true)
        .sqlx_logging_level(log::LevelFilter::Info)
        .set_schema_search_path("my_schema"); // Setting default PostgreSQL schema

    let db = Database::connect(opt).await;
    db
}

pub fn run(
    listener: TcpListener,
    db_conn: DatabaseConnection,
    email_client: EmailClient,
) -> Result<Server, std::io::Error> {
    // let db_pool = web::Data::new(db_pool);
    let email_pool = web::Data::new(email_client);
    let server = HttpServer::new(move || {
        App::new()
            // .wrap(Logger::default())  // for minimal logs
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/customer_database", web::post().to(get_customer_dbs_api))
            .route("/inventory_fetch", web::post().to(fetch_inventory))
            .route("/send_email", web::post().to(send_email))
            // Register the connection as part of the application state
            .app_data(db_conn.clone())
            .app_data(email_pool.clone())
    })
    .workers(4)
    .listen(listener)?
    .run();

    Ok(server)
}
