// use crate::email_client::{GenericEmailService, SmtpEmailClient};
// use crate::kafka_client::TopicType;
use crate::{middleware::SaveRequestResponse, schemas::StartUpMap};
// use crate::middleware::tracing_middleware;

use crate::routes::main_route;
// use actix_session::storage::RedisSessionStore;
// use actix_session::SessionMiddleware;
// use actix_web::cookie::Key;
use actix_web::dev::Server;
use tokio::try_join;
// use actix_web::middleware::Logger;
use crate::configuration::Config;

use crate::database::get_connection_pool;
use actix_web::{web, App, HttpServer};
use sqlx::postgres::PgPool;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;
pub struct Application {
    port: u16,
    server: Server,
}
impl Application {
    pub async fn build(configuration: Config) -> Result<Self, anyhow::Error> {
        let connection_pool = get_connection_pool(&configuration.database);
        // let email_pool = Arc::new(
        //     SmtpEmailClient::new(&configuration.email).expect("Failed to create SmtpEmailClient"),
        // );
        // UNCOMMENT BELOW CODE TO ENABLE DUMMY EMAIL SERVICE
        // let email_pool =
        //     Arc::new(DummyEmailClient::new().expect("Failed to create SmtpEmailClient"));
        let address = format!(
            "{}:{}",
            &configuration.application.host, &configuration.application.port
        );
        let listener = TcpListener::bind(&address)?;
        let port = listener.local_addr().unwrap().port();
        let server = run(listener, connection_pool, configuration).await?;
        Ok(Self { port, server })
    }
    pub fn port(&self) -> u16 {
        self.port
    }
    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

async fn run(
    listener: TcpListener,
    db_pool: PgPool,
    // email_obj: Arc<dyn GenericEmailService>,
    configuration: Config,
) -> Result<Server, anyhow::Error> {
    let db_pool = web::Data::new(db_pool);
    let email_client = web::Data::new(configuration.email.client());
    let kafka_client = configuration.kafka.client();
    let ondc_obj = web::Data::new(configuration.ondc);
    let ws_client = web::Data::new(configuration.websocket.client());
    let user_client = web::Data::new(configuration.user_obj.client());
    let chat_client = web::Data::new(configuration.chat.client());
    let redis_app = web::Data::new(configuration.redis.client());
    let es_client = web::Data::new(configuration.elastic_search.client());
    let payment_client = web::Data::new(configuration.payment.client());
    // es_client.send().await;
    // let kafka_producer = kafka_client.create_producer().await;
    let workers = configuration.application.workers;
    let application_obj = web::Data::new(configuration.application);
    let secret_obj = web::Data::new(configuration.secret);
    let start_up_map = web::Data::new(StartUpMap::default());
    try_join!(
        kafka_client.kafka_client_search_consumer(
            ws_client.clone(),
            db_pool.clone(),
            es_client.clone(),
        ),
        kafka_client.kafka_observability_consumer(
            db_pool.clone(),
            start_up_map.clone(),
            ondc_obj.clone()
        )
    )?;

    let kafka_client = web::Data::new(kafka_client);
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
            .app_data(kafka_client.clone())
            .app_data(es_client.clone())
            .app_data(payment_client.clone())
            .app_data(secret_obj.clone())
            .app_data(application_obj.clone())
            .app_data(start_up_map.clone())
            .configure(main_route)
    })
    .workers(workers)
    .listen(listener)?
    .run();

    Ok(server)
}
