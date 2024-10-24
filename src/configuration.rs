use crate::{domain::EmailObject, websocket::WebSocketClient};
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
pub struct UserSetting {
    pub admin_list: Vec<String>,
}

// pub struct ONDCSeller{

// }
#[derive(Debug, Deserialize, Clone)]
pub struct ONDCBuyer {
    pub id: String,
    pub uri: String,
    pub signing_key: Secret<String>,
}
#[derive(Debug, Deserialize, Clone)]
pub struct ONDCSetting {
    //pub bap: ONDCBuyer, // pub seller:ONDCSeller
    //pub gateway_key: String,
    pub gateway_uri: String,
    pub registry_base_url: String,
}
#[derive(Debug, Deserialize, Clone)]
pub struct Setting {
    pub database: DatabaseSetting,
    pub application: ApplicationSetting,
    pub redis: RedisSettings,
    pub email_client: EmailClientSetting,
    pub secret: SecretSetting,
    pub user: UserSetting,
    pub ondc: ONDCSetting,
    pub websocket: WebSocketSetting,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ApplicationSetting {
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
pub struct WebSocketSetting {
    token: Secret<String>,
    base_url: String,
    timeout_milliseconds: u64,
}

impl WebSocketSetting {
    pub fn client(self) -> WebSocketClient {
        let timeout = self.timeout();
        WebSocketClient::new(self.base_url, self.token, timeout)
    }
    fn timeout(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.timeout_milliseconds)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseSetting {
    pub username: String,
    pub password: Secret<String>,
    pub port: u16,
    pub host: String,
    pub name: String,
    pub test_name: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout: u64,
}

impl DatabaseSetting {
    // Renamed from `connection_string_without_db`
    pub fn without_db(&self) -> PgConnectOptions {
        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(self.password.expose_secret())
            .port(self.port)
    }
    // Renamed from `connection_string`
    pub fn with_db(&self) -> PgConnectOptions {
        self.without_db()
            .database(&self.name)
            .log_statements(tracing::log::LevelFilter::Trace)
    }

    pub fn test_with_db(&self) -> PgConnectOptions {
        self.without_db()
            .database(&self.test_name)
            .log_statements(tracing::log::LevelFilter::Trace)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct EmailClientSetting {
    pub base_url: String,
    pub username: String,
    pub password: Secret<String>,
    pub sender_email: String,
    pub timeout_milliseconds: u64,
}
impl EmailClientSetting {
    pub fn sender(&self) -> Result<EmailObject, String> {
        EmailObject::parse(self.sender_email.clone())
    }

    pub fn timeout(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.timeout_milliseconds)
    }
}

pub fn get_configuration() -> Result<Setting, ConfigError> {
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
    builder.try_deserialize::<Setting>()
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
