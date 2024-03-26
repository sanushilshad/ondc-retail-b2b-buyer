use crate::domain::EmailObject;
use config::{self, ConfigError, Environment};
use dotenv::dotenv;
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use sqlx::{postgres::PgConnectOptions, ConnectOptions};

#[derive(Debug, Deserialize, Clone)]
pub struct JWT {
    pub secret: Secret<String>,
    pub expiry: i64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SecretSetting {
    pub jwt: JWT,
}
#[derive(Debug, Deserialize, Clone)]
pub struct UserSettings {
    pub admin_list: Vec<String>,
}
#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
    pub redis: RedisSettings,
    pub email_client: EmailClientSettings,
    pub secret: SecretSetting,
    pub user: UserSettings,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ApplicationSettings {
    pub port: u16,
    pub host: String,
    pub hmac_secret: Secret<String>,
    pub workers: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RedisSettings {
    pub port: u16,
    pub host: String,
    pub password: Secret<String>,
}

impl RedisSettings {
    pub fn get_string(&self) -> Secret<String> {
        Secret::new(format!(
            "redis://{}:{}/{}",
            self.host,
            self.port,
            self.password.expose_secret()
        ))
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    pub port: u16,
    pub host: String,
    pub name: String,
}

impl DatabaseSettings {
    // Renamed from `connection_string_without_db`
    pub fn without_db(&self) -> PgConnectOptions {
        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(&self.password.expose_secret())
            .port(self.port)
    }
    // Renamed from `connection_string`
    pub fn with_db(&self) -> PgConnectOptions {
        self.without_db()
            .database(&self.name)
            .log_statements(tracing::log::LevelFilter::Trace)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct EmailClientSettings {
    pub base_url: String,
    pub username: String,
    pub password: Secret<String>,
    pub sender_email: String,
    pub timeout_milliseconds: u64,
}
impl EmailClientSettings {
    pub fn sender(&self) -> Result<EmailObject, String> {
        EmailObject::parse(self.sender_email.clone())
    }

    pub fn timeout(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.timeout_milliseconds)
    }
}

pub fn get_configuration() -> Result<Settings, ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    dotenv().ok();
    let builder = config::Config::builder()
        .add_source(config::File::from(base_path.join("configuration.yaml")))
        .add_source(Environment::default().separator("__"))
        .add_source(
            Environment::with_prefix("LIST")
                .try_parsing(true)
                .separator("__")
                .keep_prefix(false)
                .list_separator(","),
        )
        .build()?;
    builder.try_deserialize::<Settings>()
}

// pub fn get_configuration_by_custom() -> Result<Settings, ConfigError> {
//     // todo!()
//     let base_path = std::env::current_dir().expect("Failed to determine the current directory");
//     dotenv().ok();
//     let builder = config::Config::builder()
//         .add_source(config::File::from(base_path.join("configuration.yaml")))
//         .add_source(Environment::default().separator("_"))
//         .add_source(
//             Environment::with_prefix("LIST")
//                 .try_parsing(true)
//                 .separator("_")
//                 .keep_prefix(false)
//                 .list_separator(","),
//         )
//         .build()?;
//     let database = DatabaseSettings {
//         username: todo!(),
//         password: todo!(),
//         port: todo!(),
//         host: todo!(),
//         name: todo!(),
//     };
//     let application = ApplicationSettings {
//         port: todo!(),
//         host: todo!(),
//         hmac_secret: todo!(),
//     };
//     let redis = RedisSettings {
//         port: todo!(),
//         host: todo!(),
//         password: todo!(),
//     };
//     let email_client = EmailClientSettings {
//         base_url: todo!(),
//         username: todo!(),
//         password: todo!(),
//         sender_email: todo!(),
//         timeout_milliseconds: todo!(),
//     };
//     let secret = SecretSetting { jwt: todo!() };
//     let user = UserSettings {
//         admin_list: todo!(),
//     };
//     Ok(Settings {
//         database,
//         application,
//         redis,
//         email_client,
//         secret,
//         user,
//     })
// }
