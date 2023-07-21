use crate::errors::DatabaseError;
use config::{self, ConfigError, Environment};
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use sqlx::{postgres::PgConnectOptions, ConnectOptions};

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application_port: u16,
}
#[derive(Debug, Deserialize)]
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
        let mut options = self.without_db().database(&self.name);
        options.log_statements(tracing::log::LevelFilter::Trace);
        options
    }
}

impl DatabaseSettings {
    pub fn from_env() -> Result<Self, DatabaseError> {
        let username: String = "postgres".to_string();

        let database_name = "rapidor_master".to_string();

        let password: String = std::env::var("RAPIDOR_DB_PASSWORD")
            .map_err(|_| DatabaseError::MissingDatabasePassword)?;
        println!("{:?}", &password);
        let port: u16 = std::env::var("RAPIDOR_DB_PORT")
            .map_err(|_| DatabaseError::MissingDatabasePort)?
            .parse()
            .map_err(|_| DatabaseError::DatabasePortMustbeNumber)?;

        let host: String =
            std::env::var("RAPIDOR_DB_IP").map_err(|_| DatabaseError::MissingDatabaseIP)?;
        let password_secret = Secret::new(password);
        Ok(DatabaseSettings {
            username,
            password: password_secret,
            port,
            host,
            name: database_name,
        })
    }
}

// impl DatabaseCredentials {
//     fn from_env() -> Result<Self> {
//         dotenv::dotenv().ok();

//         let host = env::var("DB_HOST")?;
//         let port = env::var("DB_PORT")?.parse()?;
//         let name = env::var("DB_NAME")?;
//         let user = env::var("DB_USER")?;
//         let password = env::var("DB_PASSWORD")?;

//         Ok(DatabaseCredentials {
//             host,
//             port,
//             name,
//             user,
//             password,
//         })
//     }
// }

// pub fn get_configuration() -> Result<Settings, config::ConfigError> {
//     // Initialise our configuration reader
//     let mut settings = config::Config::default();

//     let mut builder = config::ConfigBuilder::new();
//     // Add configuration values from a file named `configuration`.
//     // It will look for any top-level file with an extension
//     // that `config` knows how to parse: yaml, json, etc.
//     settings.merge(config::File::with_name("configuration"))?;
//     // Try to convert the configuration values it read into
//     // our Settings type
//     settings.try_into()
// }
// impl From<Config> for Settings {
//     fn from(config: Config) -> Self {
//         config.try_into().unwrap()
//     }
// }

// impl From<Config> for Settings {
//     fn from(config: Config) -> Self {
//         let application_port = config.get::<u16>("application_port").unwrap();
//         // Retrieve other configuration fields using `config.get` or other methods

//         // Create a `Settings` instance
//         let settings = Settings {
//             application_port,
//             // Assign other configuration fields
//         };

//         settings
//     }
// }

// impl TryFrom<Config> for Settings {
//     type Error = ConfigError;

//     fn try_from(config: Config) -> Result<Self, Self::Error> {
//         let application_port = config.get::<u16>("application_port")?;
//         // Retrieve other configuration fields using `config.get` or other methods

//         // Create a `Settings` instance
//         let settings = Settings {
//             application_port,
//             // Assign other configuration fields
//         };

//         Ok(settings)
//     }
// }
pub fn get_configuration() -> Result<Settings, ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    let configuration_directory = base_path.join("configuration");
    let builder = config::Config::builder()
        .add_source(config::File::from(
            configuration_directory.join("configuration.yaml"),
        ))
        .add_source(Environment::default().separator("_"))
        .build()?;
    builder.try_deserialize::<Settings>()
}
// pub fn get_configuration() -> Result<Settings, config::ConfigError> {
//     let builder = config::Config::builder()
//         .set_default("default", "1")?
//         .add_source(config::File::new(
//             "configuration.yaml",
//             config::FileFormat::Yaml,
//         ))
//         //  .add_async_source(...)
//         .set_override("override", "1")?;
//     println!("First step");
//     builder
//         .build()
//         .map(|config| config.try_into().unwrap())
//         .map_err(|err| {
//             // Convert ConfigError to Infallible
//             match err {
//                 ConfigError::NotFound(_) => unreachable!(),
//                 ConfigError::Message(_) => unreachable!(),
//                 ConfigError::Foreign(_) => unreachable!(),
//                 ConfigError::PathParse(_) => unreachable!(),
//                 ConfigError::Frozen => unreachable!(),
//                 ConfigError::FileParse { uri: _, cause: _ } => unreachable!(),
//                 ConfigError::Type {
//                     origin: _,
//                     unexpected: _,
//                     expected: _,
//                     key: _,
//                 } => unreachable!(),
//             }
//         })
// }

// pub fn get_configuration() -> Result<Settings, config::ConfigError> {
//     let settings = config::Config::builder()
//         .set_default("default", "1")?
//         .add_source().
//         // .source(config::File::with_name("configuration"))
//         .try_into()?;
//     Ok(settings)
// }

// use std::env;

// fn main() {
//     let v = env::var("USER").expect("$USER is not set");
// }

// struct EnvVar {
//     MASTER_DB_NAME:
// }

// pub enum Environment {
//     Local,
//     Production,
// }
// impl Environment {
//     pub fn as_str(&self) -> &'static str {
//         match self {
//             Environment::Local => "local",
//             Environment::Production => "production",
//         }
//     }
// }
