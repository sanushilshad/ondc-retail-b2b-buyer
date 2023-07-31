use crate::email_client::EmailClient;
use crate::routes::fetch_inventory;
use crate::routes::get_customer_dbs_api;
use crate::routes::health_check;
use crate::routes::send_email;
use actix_web::dev::Server;
// use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use sqlx::postgres::PgPool;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;
pub fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient,
) -> Result<Server, std::io::Error> {
    let db_pool = web::Data::new(db_pool);
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
            .app_data(db_pool.clone())
            .app_data(email_pool.clone())
    })
    .workers(4)
    .listen(listener)?
    .run();

    // get_customer_dbs().await;
    Ok(server)
}
