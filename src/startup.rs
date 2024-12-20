use crate::configuration::DatabaseSetting;
use crate::email_client::{GenericEmailService, SmtpEmailClient};
use crate::middleware::SaveRequestResponse;
use crate::redis::RedisClient;
// use crate::middleware::tracing_middleware;

use crate::routes::main_route;
// use actix_session::storage::RedisSessionStore;
// use actix_session::SessionMiddleware;
// use actix_web::cookie::Key;
use actix_web::dev::Server;
// use actix_web::middleware::Logger;
use crate::configuration::Setting;

use actix_web::{web, App, HttpServer};
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::net::TcpListener;
use std::sync::Arc;
use tracing_actix_web::TracingLogger;
pub struct Application {
    port: u16,
    server: Server,
}
impl Application {
    pub async fn build(configuration: Setting) -> Result<Self, anyhow::Error> {
        let connection_pool = get_connection_pool(&configuration.database);
        let email_pool = Arc::new(
            SmtpEmailClient::new(&configuration.email).expect("Failed to create SmtpEmailClient"),
        );
        // UNCOMMENT BELOW CODE TO ENABLE DUMMY EMAIL SERVICE
        // let email_pool =
        //     Arc::new(DummyEmailClient::new().expect("Failed to create SmtpEmailClient"));
        let address = format!(
            "{}:{}",
            &configuration.application.host, &configuration.application.port
        );
        let redis_obj = RedisClient::new(&configuration.redis).await?;
        let listener = TcpListener::bind(&address)?;
        let port = listener.local_addr().unwrap().port();
        let server = run(
            listener,
            connection_pool,
            email_pool,
            redis_obj,
            configuration,
        )
        .await?;
        Ok(Self { port, server })
    }
    pub fn port(&self) -> u16 {
        self.port
    }
    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub fn get_connection_pool(configuration: &DatabaseSetting) -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(
            configuration.acquire_timeout,
        ))
        .max_connections(configuration.max_connections)
        .min_connections(configuration.min_connections)
        .connect_lazy_with(configuration.with_db())
}

async fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_obj: Arc<dyn GenericEmailService>,
    redis_client: RedisClient,
    configuration: Setting,
) -> Result<Server, anyhow::Error> {
    let db_pool = web::Data::new(db_pool);
    let email_client: web::Data<dyn GenericEmailService> = web::Data::from(email_obj);
    // let secret_obj = web::Data::new(configuration.secret);
    // let user_setting_obj = web::Data::new(configuration.user);
    let ondc_obj = web::Data::new(configuration.ondc);
    let ws_client = web::Data::new(configuration.websocket.client());
    let user_client = web::Data::new(configuration.user_obj.client());
    let chat_client = web::Data::new(configuration.chat.client());
    let redis_app = web::Data::new(redis_client);
    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::PayloadConfig::new(1 << 25))
            .wrap(SaveRequestResponse)
            .wrap(TracingLogger::default())
            .app_data(db_pool.clone())
            .app_data(email_client.clone())
            .app_data(redis_app.clone())
            .app_data(ondc_obj.clone())
            .app_data(ws_client.clone())
            .app_data(user_client.clone())
            .app_data(chat_client.clone())
            .configure(main_route)
    })
    .workers(configuration.application.workers)
    .listen(listener)?
    .run();

    Ok(server)
}
