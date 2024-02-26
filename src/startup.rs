use crate::configuration::{DatabaseSettings, SecretSetting, UserSettings};
use crate::email_client::GenericEmailService;
use crate::middleware::SaveRequestResponse;
// use crate::middleware::tracing_middleware;
use crate::routes::main_route;
use crate::schemas::CommunicationType;
use crate::utils::create_email_type_pool;

// use actix_session::storage::RedisSessionStore;
// use actix_session::SessionMiddleware;
// use actix_web::cookie::Key;
use actix_web::dev::Server;
// use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use sqlx::postgres;
use sqlx::postgres::PgPool;
use std::collections::HashMap;
use std::net::TcpListener;
use std::sync::Arc;
use tracing_actix_web::TracingLogger;

use crate::configuration::Settings;

pub struct Application {
    port: u16,
    server: Server,
}
impl Application {
    // We have converted the `build` function into a constructor for
    // `Application`.
    pub async fn build(configuration: Settings) -> Result<Self, anyhow::Error> {
        let connection_pool = get_connection_pool(&configuration.database);
        // let email_client =
        //     SmtpEmailClient::new(configuration.email_client).expect("SMTP connection Failed");
        let email_type_pool = create_email_type_pool(configuration.email_client);
        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        println!("Listening {}", address);
        let listener = TcpListener::bind(&address)?;
        let port = listener.local_addr().unwrap().port();
        let server = run(
            listener,
            connection_pool,
            email_type_pool,
            configuration.secret,
            configuration.user,
        )
        .await?;
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

pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    postgres::PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.with_db())
}

async fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_type_pool: HashMap<CommunicationType, Arc<dyn GenericEmailService>>,
    secret: SecretSetting,
    user_setting: UserSettings,
) -> Result<Server, anyhow::Error> {
    let db_pool = web::Data::new(db_pool);
    let email_client = web::Data::new(email_type_pool);
    let secret_obj = web::Data::new(secret);
    let user_setting_obj = web::Data::new(user_setting);
    // let _secret_key = Key::from(hmac_secret.expose_secret().as_bytes());
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .wrap(SaveRequestResponse)
            // .wrap(ErrorHandlers::new().handler(StatusCode::BAD_REQUEST, add_error_header))
            // .wrap(Logger::default())  // for minimal logs
            // Register the connection as part of the application state
            // .wrap_fn(tracing_middleware)
            .app_data(db_pool.clone())
            .app_data(email_client.clone())
            .app_data(secret_obj.clone())
            .app_data(user_setting_obj.clone())
            .configure(main_route)
    })
    .workers(4)
    .listen(listener)?
    .run();

    Ok(server)
}
