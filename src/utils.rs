use crate::configuration::{DatabaseConfig, EmailClientConfig};
use crate::constants::AUTHORIZATION_PATTERN;
use crate::email_client::{GenericEmailService, SmtpEmailClient};
// use crate::kafka_client::TopicType;
use crate::models::{RegisteredNetworkParticipantModel, SeriesNoModel};
use crate::routes::order::schemas::{PaymentSettlementPhase, PaymentSettlementType};
use crate::schemas::{
    CommunicationType, FeeType, ONDCNetworkType, RegisteredNetworkParticipant, SeriesNoType, Status,
};
use crate::schemas::{KycStatus, ONDCAuthParams};
use crate::user_client::{BusinessAccount, SettingData, SettingKey};
use crate::{kafka_client, migration};
use actix_http::h1;
use actix_web::dev::Payload;
use actix_web::dev::ServiceRequest;
use actix_web::rt::task::JoinHandle;
use actix_web::web;
use anyhow::anyhow;
use base64::engine::general_purpose;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use bigdecimal::BigDecimal;
use blake2::{Blake2b512, Digest};
use chrono::Utc;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use secrecy::ExposeSecret;
use serde::{Deserialize, Deserializer, Serialize};
use sqlx::{Connection, Executor, PgConnection, PgPool, Postgres, Transaction};
use std::collections::HashMap;
use std::str::FromStr;
use std::{fmt, fs, io, sync::Arc};

pub fn get_ondc_params_from_header(header: &str) -> Result<ONDCAuthParams, anyhow::Error> {
    let captures = AUTHORIZATION_PATTERN
        .captures(header)
        .ok_or_else(|| anyhow::anyhow!("Invalid Authorization Header"))?;

    let groups: Vec<String> = captures
        .iter()
        .skip(1)
        .filter_map(|m| m.map(|m| m.as_str().to_owned()))
        .collect();

    if groups.len() != 6 {
        return Err(anyhow::anyhow!(
            "Invalid number of captured groups in Authorization Token"
        ));
    }

    let created_time = groups[3]
        .parse::<i64>()
        .map_err(|err| anyhow::anyhow!("Invalid created time format: {}", err))?;
    let expires_time = groups[4]
        .parse::<i64>()
        .map_err(|err| anyhow::anyhow!("Invalid expired time format: {}", err))?;
    let subscriber_id = groups[0].clone();
    let uk_id = groups[1].clone();
    let algorithm = groups[2].clone();
    let signature = groups[5].clone();

    Ok(ONDCAuthParams {
        created_time,
        expires_time,
        subscriber_id,
        uk_id,
        algorithm,
        signature,
    })
}

pub fn bytes_to_payload(buf: web::Bytes) -> Payload {
    let (_, mut pl) = h1::Payload::create(true);
    pl.unread_data(buf);
    Payload::from(pl)
}

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

pub async fn configure_database_using_sqlx(config: &DatabaseConfig) -> PgPool {
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

    let test_connection_pool = PgPool::connect_with(config.test_with_db())
        .await
        .expect("Failed to connect to Postgres.");

    sqlx::migrate!("./migrations")
        .run(&test_connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}
#[tracing::instrument(name = "Confiure Database")]
pub async fn configure_database(config: &DatabaseConfig) -> PgPool {
    create_database(config).await;
    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres.");
    let test_connection_pool = PgPool::connect_with(config.test_with_db())
        .await
        .expect("Failed to connect to Postgres.");

    let _ = execute_query("./migrations", &connection_pool).await;
    let _ = execute_query("./migrations", &test_connection_pool).await;
    connection_pool
}
#[tracing::instrument(name = "Create Database")]
pub async fn create_database(config: &DatabaseConfig) {
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
                tracing::info!("Database {} already exists.", &config.name);
            } else {
                connection
                    .execute(format!(r#"CREATE DATABASE "{}";"#, config.name).as_str())
                    .await
                    .expect("Failed to create database.");
                eprintln!("Database created.");
            }
        }
        Ok(_) => eprintln!("No rows found."),
        Err(err) => eprintln!("Error: {}", err),
    }

    let test_db_count: Result<Option<i64>, sqlx::Error> =
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM pg_database WHERE datname = $1")
            .bind(&config.test_name)
            .fetch_optional(&mut connection)
            .await;

    match test_db_count {
        Ok(Some(count)) => {
            if count > 0 {
                eprintln!("Test database {} already exists.", &config.test_name);
            } else {
                connection
                    .execute(format!(r#"CREATE DATABASE "{}";"#, config.test_name).as_str())
                    .await
                    .expect("Failed to create test database.");
                eprintln!("Test database {} created.", &config.test_name);
            }
        }
        Ok(_) => eprintln!("No rows found for the test database check."),
        Err(err) => eprintln!("Error checking test database existence: {}", err),
    }
}

#[tracing::instrument(name = "Execute Queries")]
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
                eprintln!("Migration applied: {:?}", statement);
            }
        }

        eprintln!("Migration applied: {:?}", migration_path);
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
    email_config: EmailClientConfig,
) -> HashMap<CommunicationType, Arc<dyn GenericEmailService>> {
    let smtp_client =
        Arc::new(SmtpEmailClient::new(&email_config).expect("Failed to create SmtpEmailClient"))
            as Arc<dyn GenericEmailService>;

    let mut email_services = HashMap::new();
    email_services.insert(CommunicationType::Type1, smtp_client.clone());

    email_services
}

#[tracing::instrument(name = "Run custom command")]
pub async fn run_custom_commands(args: Vec<String>) -> Result<(), anyhow::Error> {
    if args.len() < 2 {
        eprintln!("Invalid command. Please provide a valid command.");
        return Ok(());
    }
    let command = args[1].as_str();
    match command {
        "migrate" => {
            migration::run_migrations().await;
        }
        "sqlx_migrate" => {
            migration::migrate_using_sqlx().await;
        }
        "generate_kafka_topic" => {
            // let arg = args.get(2).unwrap_or(&TopicType::Search.to_string());
            kafka_client::create_kafka_topic_command().await;
        }
        _ => {
            eprintln!("Unknown command: {}. Please use a valid command.", command);
        }
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

#[tracing::instrument(name = "get GPS string")]
pub fn get_gps_string(latitude: f64, longitude: f64) -> String {
    format!("{},{}", latitude, longitude)
}

#[tracing::instrument(name = "Get header value")]
pub fn get_header_value<'a>(req: &'a ServiceRequest, header_name: &'a str) -> Option<&'a str> {
    req.headers().get(header_name).and_then(|h| h.to_str().ok())
}

pub fn pascal_to_snake_case(pascal_case: &str) -> String {
    let mut snake_case = String::new();
    let mut is_first_word = true;

    for c in pascal_case.chars() {
        if c.is_uppercase() {
            if !is_first_word {
                snake_case.push('_');
            }
            is_first_word = false;
        }
        snake_case.push(c.to_ascii_lowercase());
    }

    snake_case
}

pub fn pascal_to_uppercase(pascal_case: &str) -> String {
    let mut uppercase_with_underscores = String::new();
    let mut is_first_word = true;

    for c in pascal_case.chars() {
        if c.is_uppercase() {
            if !is_first_word {
                uppercase_with_underscores.push('_');
            }
            is_first_word = false;
        }
        uppercase_with_underscores.push(c.to_ascii_uppercase());
    }

    uppercase_with_underscores
}

#[tracing::instrument(name = "Get network participant detail model", skip(pool))]
pub async fn get_network_participant_detail_model(
    pool: &PgPool,
    subscriber_id: &str,
    network_participant_type: &ONDCNetworkType,
) -> Result<Option<RegisteredNetworkParticipantModel>, anyhow::Error> {
    let row: Option<RegisteredNetworkParticipantModel> = sqlx::query_as!(
        RegisteredNetworkParticipantModel,
        r#"SELECT id, code, name, logo, unique_key_id, fee_type as "fee_type: FeeType",
        fee_value, signing_key, subscriber_id, subscriber_uri, long_description,
        settlement_phase as "settlement_phase: PaymentSettlementPhase", settlement_type as "settlement_type: PaymentSettlementType",
        bank_account_no, bank_ifsc_code, bank_beneficiary_name, bank_name, short_description 
        FROM registered_network_participant WHERE subscriber_id = $1 AND network_participant_type = $2"#,
        subscriber_id,
        &network_participant_type as &ONDCNetworkType,
    )
    .fetch_optional(pool)
    .await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e).context("failed to fetch network participant detail from database")
    })?;
    Ok(row)
}

pub fn get_network_participant_detail_from_model(
    network_model: RegisteredNetworkParticipantModel,
) -> RegisteredNetworkParticipant {
    RegisteredNetworkParticipant {
        id: network_model.id,
        name: network_model.name,
        code: network_model.code,
        logo: network_model.logo,
        signing_key: network_model.signing_key,
        subscriber_id: network_model.subscriber_id,
        subscriber_uri: network_model.subscriber_uri,
        long_description: network_model.long_description,
        short_description: network_model.short_description,
        fee_type: network_model.fee_type,
        fee_value: BigDecimal::from_str(&network_model.fee_value.to_string()).unwrap(),
        unique_key_id: network_model.unique_key_id,
        settlement_phase: network_model.settlement_phase,
        settlement_type: network_model.settlement_type,
        bank_account_no: network_model.bank_account_no,
        bank_ifsc_code: network_model.bank_ifsc_code,
        bank_beneficiary_name: network_model.bank_beneficiary_name,
        bank_name: network_model.bank_name,
    }
}

#[tracing::instrument(name = "Get network participany detail", skip(pool))]
pub async fn get_np_detail(
    pool: &PgPool,
    subscriber_id: &str,
    participant_type: &ONDCNetworkType,
) -> Result<Option<RegisteredNetworkParticipant>, anyhow::Error> {
    let network_model =
        get_network_participant_detail_model(pool, subscriber_id, participant_type).await?;
    return match network_model {
        Some(model) => Ok(Some(get_network_participant_detail_from_model(model))),
        None => Ok(None),
    };
}

pub fn create_signing_string(
    digest_base64: &str,
    created: Option<i64>,
    expires: Option<i64>,
) -> String {
    format!(
        "(created): {}\n(expires): {}\ndigest: BLAKE-512={}",
        created.unwrap_or_else(|| Utc::now().timestamp()),
        expires.unwrap_or_else(|| (Utc::now() + chrono::Duration::hours(1)).timestamp()),
        digest_base64
    )
}

pub fn hash_message(msg: &str) -> String {
    let mut hasher = Blake2b512::new();
    hasher.update(msg.as_bytes());
    let digest = hasher.finalize();
    general_purpose::STANDARD.encode(digest.as_slice())
}

fn sign_response(msg: &str, private_key: &str) -> Result<String, anyhow::Error> {
    let decoded_bytes = BASE64.decode(private_key)?;
    let secret_key_bytes: &[u8; 32] = decoded_bytes.as_slice().try_into()?;
    let signing_key: SigningKey = SigningKey::from_bytes(secret_key_bytes);
    let singed_value = signing_key.sign(msg.as_bytes());
    Ok(BASE64.encode(singed_value.to_bytes()))
}

pub fn verify_response(signature: &str, msg: &str, public_key: &str) -> Result<(), anyhow::Error> {
    let decoded_public_key = BASE64.decode(public_key)?;
    let secret_key_bytes: &[u8; 32] = decoded_public_key.as_slice().try_into()?;
    let public_key = VerifyingKey::from_bytes(secret_key_bytes)?;
    let decoded_signature = BASE64.decode(signature)?;
    let decoded_signature_bytes: &[u8; 64] = decoded_signature.as_slice().try_into()?;
    let signature_obj = Signature::from_bytes(decoded_signature_bytes);
    public_key.verify(msg.as_bytes(), &signature_obj)?;
    Ok(())
}

pub fn create_authorization_header(
    request_body: &str,
    np_detail: &RegisteredNetworkParticipant,
    created: Option<i64>,
    expires: Option<i64>,
) -> Result<String, anyhow::Error> {
    let signing_key = create_signing_string(&hash_message(request_body), created, expires);
    // println!("{}", signing_key);
    // let  signing_key = "digest: BLAKE-512=n11lI7rMbBysTm60EL5ALC4rlSB3bnd9510qrH9g5eh2idHdghW1Z6zxChE6ozn42UybQQowSQ7pEuTMrM3rYg==";
    // let a = "xPwEy7bD3SWw0UBAG+SpznAS5xjgNUlBPD0GqKj/pz4=";
    // let signature = sign_response(&signing_key, a)?;
    let signature = sign_response(&signing_key, np_detail.signing_key.expose_secret())?;
    Ok(format!(
            "Signature keyId=\"{}|{}|ed25519\",algorithm=\"ed25519\", created=\"{}\", expires=\"{}\", headers=\"(created) (expires) digest\",signature=\"{}\"",
            &np_detail.subscriber_id, &np_detail.unique_key_id,
            created.unwrap_or_else(|| Utc::now().timestamp()),
            expires.unwrap_or_else(|| (Utc::now() + chrono::Duration::hours(1)).timestamp()),
            signature
    ))
}

pub fn deserialize_non_empty_vector<'de, T, D>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    let vec: Vec<T> = Deserialize::deserialize(deserializer)?;
    if vec.is_empty() {
        Err(serde::de::Error::custom(
            "Vector field must contain at least one value",
        ))
    } else {
        Ok(vec)
    }
}

// pub fn get_default_vector_value<'a>(
//     default_vector_type: &'a VectorType,
//     vectors: &'a Vec<UserVector>,
// ) -> Option<&'a str> {
//     for vector in vectors {
//         if vector.key == *default_vector_type {
//             return Some(&vector.value);
//         }
//     }
//     None
// }

// pub fn get_vector_from_list<'a>(
//     default_vector_type: &'a VectorType,
//     vectors: &'a [UserVector],
// ) -> Option<&'a UserVector> {
//     vectors
//         .iter()
//         .find(|&vector| vector.key == *default_vector_type)
// }

pub fn validate_business_account_active(business_obj: &BusinessAccount) -> Option<String> {
    match (
        &business_obj.kyc_status,
        &business_obj.is_active,
        business_obj.is_deleted,
        business_obj.verified,
    ) {
        (KycStatus::Pending, _, _, _) => Some("KYC is still pending".to_string()),
        (KycStatus::OnHold, _, _, _) => Some("KYC is On-hold".to_string()),
        (KycStatus::Rejected, _, _, _) => Some("KYC is Rejected".to_string()),
        (_, Status::Inactive, _, _) => Some("Business Account is inactive".to_string()),
        (_, _, true, _) => Some("Business Account is deleted".to_string()),
        (_, _, _, false) => Some("Business User relation is not verified".to_string()),
        _ => None,
    }
}

#[tracing::instrument(name = "generating series")]
async fn generate_series(
    transaction: &mut Transaction<'_, Postgres>,
    subscriber_id: &str,
    series_type: SeriesNoType,
    prefix: &str,
) -> Result<SeriesNoModel, anyhow::Error> {
    let record = sqlx::query_as!(
        SeriesNoModel,
        r#"
            INSERT INTO series_no_generator (subscriber_id, series_type, prefix, series_no)
            VALUES ($1, $2, $3, COALESCE(
                (SELECT series_no + 1 FROM series_no_generator 
                 WHERE subscriber_id = $1 AND series_type = $2 AND prefix = $3 
                 LIMIT 1 FOR UPDATE), 1))
            ON CONFLICT (subscriber_id, series_type, prefix) 
            DO UPDATE 
            SET series_no = series_no_generator.series_no + 1
            RETURNING prefix, series_no;
    "#,
        subscriber_id,
        series_type as SeriesNoType,
        prefix
    )
    .fetch_one(&mut **transaction)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query for series creation: {:?}", e);
        anyhow::Error::new(e).context("Something went wrong while generating series NO")
    })?;

    Ok(record)
}

#[tracing::instrument(name = "get series")]
pub async fn get_series_no(
    transaction: &mut Transaction<'_, Postgres>,
    setting: &SettingData,
    subscriber_id: &str,
    series_type: SeriesNoType,
) -> Result<String, anyhow::Error> {
    let order_no_prefix = setting
        .get_setting(SettingKey::OrderNoPrefix)
        .ok_or_else(|| anyhow!("Order No Prefix is not configured"))?;

    let series_model = generate_series(transaction, subscriber_id, series_type, &order_no_prefix)
        .await
        .map_err(|e| anyhow!(e))?;
    Ok(series_model.get_final_no())
}
