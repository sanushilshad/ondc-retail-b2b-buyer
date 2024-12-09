use crate::{
    chat_client::ChatClient, domain::EmailObject, user_client::UserClient,
    websocket_client::WebSocketClient,
};
use config::{self, ConfigError, Environment};
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use sqlx::{postgres::PgConnectOptions, ConnectOptions};

#[derive(Debug, Deserialize, Clone)]
pub struct JWT {
    pub secret: SecretString,
    pub expiry: i64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UserSetting {
    token: SecretString,
    base_url: String,
    timeout_milliseconds: u64,
}

impl UserSetting {
    pub fn client(self) -> UserClient {
        let timeout = self.timeout();
        UserClient::new(self.base_url, self.token, timeout)
    }
    fn timeout(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.timeout_milliseconds)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ONDCSetting {
    pub gateway_uri: String,
    pub registry_base_url: String,
}
#[derive(Debug, Deserialize, Clone)]
pub struct Setting {
    pub database: DatabaseSetting,
    pub application: ApplicationSetting,
    pub redis: RedisSetting,
    pub email: EmailClientSetting,
    pub user_obj: UserSetting,
    pub ondc: ONDCSetting,
    pub websocket: WebSocketSetting,
    pub chat: ChatSetting,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ApplicationSetting {
    pub port: u16,
    pub host: String,
    pub workers: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RedisSetting {
    pub port: u16,
    pub host: String,
    pub password: SecretString,
}

impl RedisSetting {
    pub fn get_string(&self) -> SecretString {
        SecretString::new(
            format!(
                "redis://{}:{}/{}",
                self.host,
                self.port,
                self.password.expose_secret()
            )
            .into(),
        )
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct WebSocketSetting {
    token: SecretString,
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
pub struct ChatSetting {
    token: SecretString,
    base_url: String,
    timeout_milliseconds: u64,
}

impl ChatSetting {
    pub fn client(self) -> ChatClient {
        let timeout = self.timeout();
        ChatClient::new(self.base_url, self.token, timeout)
    }
    fn timeout(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.timeout_milliseconds)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseSetting {
    pub username: String,
    pub password: SecretString,
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
    pub password: SecretString,
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

/// do not add any env variable starting with user, this crate doesn't support it
pub fn get_configuration() -> Result<Setting, ConfigError> {
    // let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    let builder = config::Config::builder()
        .add_source(Environment::default().separator("__"))
        // .add_source(config::File::from(base_path.join("configuration.yaml")))
        .build()?;
    builder.try_deserialize::<Setting>()
}

// pub fn get_configuration_by_custom() -> Result<Setting, anyhow::Error> {
//     let database = DatabaseSetting {
//         username: env::var("DATABASE__USERNAME")
//             .unwrap_or_else(|_| panic!("DATABASE__USERNAME is missing")),
//         password: SecretString::new(
//             env::var("DATABASE__PASSWORD")
//                 .unwrap_or_else(|_| panic!("DATABASE__PASSWORD is missing"))
//                 .into(),
//         ),
//         port: env::var("DATABASE__PORT")
//             .unwrap_or_else(|_| panic!("DATABASE__PORT is missing"))
//             .parse()
//             .map_err(|e| anyhow!("DATABASE__PORT must be a valid u16: {}", e))?,
//         host: env::var("DATABASE__HOST").unwrap_or_else(|_| panic!("DATABASE__HOST is missing")),
//         name: env::var("DATABASE__NAME").unwrap_or_else(|_| panic!("DATABASE__NAME is missing")),
//         test_name: env::var("DATABASE__NAME")
//             .unwrap_or_else(|_| panic!("DATABASE__NAME is missing"))
//             + "_test",
//         max_connections: env::var("DATABASE__MAX_CONNECTIONS")
//             .unwrap_or_else(|_| panic!("DATABASE__MAX_CONNECTIONS is missing"))
//             .parse()
//             .map_err(|e| anyhow!("DATABASE__MAX_CONNECTIONS must be a valid u32: {}", e))?,
//         min_connections: env::var("DATABASE__MIN_CONNECTIONS")
//             .unwrap_or_else(|_| panic!("DATABASE__MIN_CONNECTIONS is missing"))
//             .parse()
//             .map_err(|e| anyhow!("DATABASE__MIN_CONNECTIONS must be a valid u32: {}", e))?,
//         acquire_timeout: env::var("DATABASE__ACQUIRE_TIMEOUT")
//             .unwrap_or_else(|_| panic!("DATABASE__ACQUIRE_TIMEOUT is missing"))
//             .parse()
//             .map_err(|e| anyhow!("DATABASE__ACQUIRE_TIMEOUT must be a valid u64: {}", e))?,
//     };

//     let application = ApplicationSetting {
//         port: env::var("APPLICATION__PORT")
//             .unwrap_or_else(|_| panic!("APPLICATION__PORT is missing"))
//             .parse()
//             .map_err(|e| anyhow!("APPLICATION__PORT must be a valid u16: {}", e))?,
//         host: env::var("APPLICATION__HOST")
//             .unwrap_or_else(|_| panic!("APPLICATION__HOST is missing")),
//         workers: env::var("APPLICATION__WORKERS")
//             .unwrap_or_else(|_| panic!("APPLICATION__WORKERS is missing"))
//             .parse()
//             .map_err(|e| anyhow!("APPLICATION__WORKERS must be a valid usize: {}", e))?,
//     };

//     let redis = RedisSetting {
//         port: env::var("REDIS__PORT")
//             .unwrap_or_else(|_| panic!("REDIS__PORT is missing"))
//             .parse()
//             .map_err(|e| anyhow!("REDIS__PORT must be a valid u16: {}", e))?,
//         host: env::var("REDIS__HOST").unwrap_or_else(|_| panic!("REDIS__HOST is missing")),
//         password: SecretString::new(
//             env::var("REDIS_PASSWORD").unwrap_or_else(|_| panic!("REDIS_PASSWORD is missing")),
//         ),
//     };

//     let email_client = EmailClientSetting {
//         base_url: env::var("EMAIL_CLIENT__BASE_URL")
//             .unwrap_or_else(|_| panic!("EMAIL_CLIENT__BASE_URL is missing")),
//         username: env::var("EMAIL_CLIENT__USERNAME")
//             .unwrap_or_else(|_| panic!("EMAIL_CLIENT__USERNAME is missing")),
//         password: SecretString::new(
//             env::var("EMAIL_CLIENT__PASSWORD")
//                 .unwrap_or_else(|_| panic!("EMAIL_CLIENT__PASSWORD is missing")),
//         ),
//         sender_email: env::var("EMAIL_CLIENT__SENDER_EMAIL")
//             .unwrap_or_else(|_| panic!("EMAIL_CLIENT__SENDER_EMAIL is missing")),
//         timeout_milliseconds: env::var("EMAIL_CLIENT__TIMEOUT_MILLISECONDS")
//             .unwrap_or_else(|_| panic!("EMAIL_CLIENT__TIMEOUT_MILLISECONDS is missing"))
//             .parse()
//             .map_err(|e| {
//                 anyhow!(
//                     "EMAIL_CLIENT__TIMEOUT_MILLISECONDS must be a valid u64: {}",
//                     e
//                 )
//             })?,
//     };

//     let user = UserSetting {
//         token: SecretString::new(
//             env::var("USER__TOKEN").unwrap_or_else(|_| panic!("USER__TOKEN is missing")),
//         ),
//         base_url: env::var("USER__BASE_URL")
//             .unwrap_or_else(|_| panic!("USER__BASE_URL is missing")),
//         timeout_milliseconds: env::var("USER__TIMEOUT_MILLISECONDS")
//             .unwrap_or_else(|_| panic!("USER__TIMEOUT_MILLISECONDS is missing"))
//             .parse()
//             .map_err(|e| anyhow!("USER__TIMEOUT_MILLISECONDS must be a valid u64: {}", e))?,
//     };

//     let websocket = WebSocketSetting {
//         token: SecretString::new(
//             env::var("WEBSOCKET__TOKEN").unwrap_or_else(|_| panic!("WEBSOCKET__TOKEN is missing")),
//         ),
//         base_url: env::var("WEBSOCKET__BASE_URL")
//             .unwrap_or_else(|_| panic!("WEBSOCKET__BASE_URL is missing")),
//         timeout_milliseconds: env::var("WEBSOCKET__TIMEOUT_MILLISECONDS")
//             .unwrap_or_else(|_| panic!("WEBSOCKET__TIMEOUT_MILLISECONDS is missing"))
//             .parse()
//             .map_err(|e| anyhow!("WEBSOCKET__TIMEOUT_MILLISECONDS must be a valid u64: {}", e))?,
//     };

//     let ondc = ONDCSetting {
//         gateway_uri: env::var("ONDC__GATEWAY_URI")
//             .unwrap_or_else(|_| panic!("ONDC__GATEWAY_URI is missing")),
//         registry_base_url: env::var("ONDC__REGISTRY_BASE_URL")
//             .unwrap_or_else(|_| panic!("ONDC__REGISTRY_BASE_URL is missing")),
//     };

//     Ok(Setting {
//         database,
//         application,
//         redis,
//         email_client,
//         user,
//         ondc,
//         websocket,
//     })
// }
