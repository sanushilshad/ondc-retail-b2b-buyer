use crate::configuration::DatabaseSettings;
use crate::configuration::EmailClientSettings;
use crate::email_client::GenericEmailService;
use crate::email_client::SmtpEmailClient;
use crate::errors::CustomJWTTokenError;
use crate::migration;
use crate::schemas::CommunicationType;
use crate::schemas::JWTClaims;
use actix_web::rt::task::JoinHandle;
use chrono::DateTime;
use chrono::Duration;
use chrono::Utc;
use jsonwebtoken::{
    decode, encode, Algorithm as JWTAlgorithm, DecodingKey, EncodingKey, Header, Validation,
};
use secrecy::ExposeSecret;
use secrecy::Secret;
use serde::Deserialize;
use serde::Serialize;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::io;
use std::sync::Arc;
use uuid::Uuid;
pub fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}

pub async fn configure_database_using_sqlx(config: &DatabaseSettings) -> PgPool {
    // Create database
    create_database(config).await;
    // Migrate database
    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres.");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");
    connection_pool
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres.");
    create_database(config).await;
    match execute_query("./migrations", &connection_pool).await {
        Ok(_) => {}
        Err(_) => {}
    }
    connection_pool
}

pub async fn create_database(config: &DatabaseSettings) {
    // Create database
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres");

    let db_count: Result<Option<i64>, sqlx::Error> =
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM pg_database WHERE datname = $1")
            .bind(&config.name)
            .fetch_optional(&mut connection)
            .await;
    match db_count {
        Ok(Some(count)) => {
            if count > 0 {
                println!("Database {} already exists.", &config.name);
            } else {
                connection
                    .execute(format!(r#"CREATE DATABASE "{}";"#, config.name).as_str())
                    .await
                    .expect("Failed to create database.");
                println!("Database created.");
            }
        }
        Ok(_) => println!("No rows found."),
        Err(err) => eprintln!("Error: {}", err),
    }
}

async fn execute_query(path: &str, pool: &PgPool) -> io::Result<()> {
    let migration_files = fs::read_dir(path)?;
    for migration_file in migration_files {
        let migration_file = migration_file?;
        let migration_path = migration_file.path();
        let migration_sql = fs::read_to_string(&migration_path)?;
        let statements: String = migration_sql.replace('\n', "");
        let new_statement: Vec<&str> = statements
            .split(';')
            .filter(|s| !s.trim().is_empty() & !s.starts_with("--"))
            .collect();
        for statement in new_statement {
            if let Err(err) = sqlx::query(statement).execute(pool).await {
                eprintln!("Error executing statement {:?}: {} ", statement, err);
            } else {
                println!("Migration applied: {:?}", statement);
            }
        }

        println!("Migration applied: {:?}", migration_path);
    }

    Ok(())
}

pub fn spawn_blocking_with_tracing<F, R>(f: F) -> JoinHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    let current_span = tracing::Span::current();
    actix_web::rt::task::spawn_blocking(move || current_span.in_scope(f))
}

pub fn fmt_json<T: Serialize>(value: &T, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", serde_json::to_string(value).unwrap())
}

#[macro_export]
macro_rules! impl_serialize_format {
    ($struct_name:ident, $trait_name:path) => {
        impl $trait_name for $struct_name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt_json(self, f)
            }
        }
    };
}

pub struct EmailTypeMapping {
    pub type_1: HashMap<CommunicationType, Arc<dyn GenericEmailService>>,
}
pub fn create_email_type_pool(
    email_config: EmailClientSettings,
) -> HashMap<CommunicationType, Arc<dyn GenericEmailService>> {
    let smtp_client =
        Arc::new(SmtpEmailClient::new(email_config).expect("Failed to create SmtpEmailClient"))
            as Arc<dyn GenericEmailService>;

    let mut email_services = HashMap::new();
    email_services.insert(CommunicationType::Type1, smtp_client.clone());

    email_services
}

pub fn generate_jwt_token_for_user(
    user_id: Uuid,
    expiry_date: Option<DateTime<Utc>>,
    secret: &Secret<String>,
) -> Result<Secret<String>, anyhow::Error> {
    let expiration = match expiry_date {
        Some(expiry) => expiry.timestamp() as usize,
        None => Utc::now()
            .checked_add_signed(Duration::hours(24))
            .expect("valid timestamp")
            .timestamp() as usize,
    };
    let claims: JWTClaims = JWTClaims {
        sub: user_id,
        exp: expiration as usize,
    };
    let header = Header::new(JWTAlgorithm::HS256);
    let encoding_key = EncodingKey::from_secret(secret.expose_secret().as_bytes());
    let token: String = encode(&header, &claims, &encoding_key).expect("Failed to generate token");
    return Ok(Secret::new(token));
}

pub fn decode_token<T: Into<String>>(
    token: T,
    secret: &Secret<String>,
) -> Result<Uuid, CustomJWTTokenError> {
    let decoding_key = DecodingKey::from_secret(secret.expose_secret().as_bytes());
    let decoded = decode::<JWTClaims>(
        &token.into(),
        &decoding_key,
        &Validation::new(JWTAlgorithm::HS256),
    );
    match decoded {
        Ok(token) => Ok(token.claims.sub),
        Err(e) => {
            // Map jsonwebtoken errors to custom AuthTokenError
            match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                    Err(CustomJWTTokenError::Expired.into())
                }
                _ => Err(CustomJWTTokenError::Invalid("Invalid Token".to_string())),
            }
        }
    }
}

pub async fn run_custom_commands(args: Vec<String>) -> Result<(), anyhow::Error> {
    if args.len() > 1 {
        if args[1] == "migrate" {
            migration::run_migrations().await;
        }

        if args[1] == "sqlx_migrate" {
            migration::migrate_using_sqlx().await;
        }
    } else {
        println!("Invalid command. Use Enter a valid command");
    }

    Ok(())
}

pub fn deserialize_config_list<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    // Deserialize the value as a String
    let config_str = String::deserialize(deserializer)?;

    // Parse the string as JSON array and extract Vec<String>
    serde_json::from_str::<Vec<String>>(&config_str).map_err(serde::de::Error::custom)
}
