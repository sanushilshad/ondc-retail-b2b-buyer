use actix_web::{web, HttpResponse, Responder, ResponseError};
use anyhow::Context;
use bigdecimal::BigDecimal;
use sea_orm::DatabaseConnection;
// use log::{info, warn};
// use bigdecimal::BigDecimal;
use crate::email_client::EmailClient;
use actix_web::http::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::fmt;
use std::fmt::{Debug, Display};
// use tracing::Instrument;
// use uuid::Uuid;
#[allow(dead_code)]
#[derive(Serialize, sqlx::FromRow)]
struct RapidorCustomer {
    domain: String,
    database: String,
}

#[tracing::instrument(name = "Fetching customer data from database.", skip(pool), fields())]
async fn get_customer_dbs(
    pool: web::Data<DatabaseConnection>,
) -> Result<Vec<RapidorCustomer>, sqlx::Error> {
    let rapidor_customers =
        sqlx::query_as::<_, RapidorCustomer>("SELECT domain, database FROM customer_customer")
            .fetch_all(pool.get_ref())
            .await
            .map_err(|e| {
                tracing::error!("Failed to execute query: {:?}", e);
                e
                // Using the `?` operator to return early
                // if the function failed, returning a sqlx::Error
                // We will talk about error handling in depth later!
            })?;
    tracing::info!("successfully fetched databases from database.");
    Ok(rapidor_customers)
}
#[tracing::instrument(
name = "Fetching Customer List",
skip(pool),
fields(
// request_id = %Uuid::new_v4()
)
)]
pub async fn get_customer_dbs_api(pool: web::Data<PgPool>) -> impl Responder {
    let _request_span = tracing::info_span!("starting fetching of databases.");
    // tracing::info!("request_id {} - fetching databases.", request_id);
    let db_domain_mapping = get_customer_dbs(pool)
        .await
        .expect("Something went wrong with the db connection");
    // match get_customer_dbs().await {
    //     Ok(data) => {
    //         log::info!("Sucessfully fetched data");
    //         web::Json(db_domain_mapping)
    //     }
    //     Err(err) => {
    //         log::error!("Operation failed: {}", err)
    //     }
    // }

    web::Json(db_domain_mapping)
}

fn fmt_json<T: Serialize>(value: &T, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", serde_json::to_string(value).unwrap())
}

macro_rules! impl_serialize_format {
    ($struct_name:ident, $trait_name:path) => {
        impl $trait_name for $struct_name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt_json(self, f)
            }
        }
    };
}
impl_serialize_format!(InventoryRequest, Debug);
#[derive(Deserialize, Serialize)]
pub struct InventoryRequest {
    username: String,
    session_id: String,
    product_codes: Vec<String>,
}

impl_serialize_format!(InventoryRequest, Display);
#[derive(sqlx::FromRow, Serialize, Deserialize)]
struct ProductInventory {
    #[sqlx(rename = "code")]
    product_code: String,
    #[sqlx(rename = "no_of_items")]
    qty: BigDecimal,
}
impl_serialize_format!(MyResponse, Display);
#[derive(Serialize, Deserialize)]
struct MyResponse {
    status: bool,
    customer_message: String,
    success_code: String,
    data: Vec<ProductInventory>,
}
// #[derive(thiserror::Error)]
// pub enum InventoryError {
//     #[error("{0}")]
//     ValidationError(String),
//     #[error("Failed to acquire data from database")]
//     DatabaseFetchError(#[source] sqlx::Error),
//     #[error("Failed to acquire a Postgres connection from the pool")]
//     PoolError(#[source] sqlx::Error),
//     #[error("Failed to insert new subscriber in the database.")]
//     InsertSubscriberError(#[source] sqlx::Error),
//     #[error("Failed to commit SQL transaction to store a new subscriber.")]
//     TransactionCommitError(#[source] sqlx::Error),
//     #[error("Failed to send a confirmation email.")]
//     SendEmailError(#[from] reqwest::Error),
// }

#[derive(thiserror::Error)]
pub enum InventoryError {
    #[error("{0}")]
    ValidationError(String),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

fn error_chain_fmt(
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

impl std::fmt::Debug for InventoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for InventoryError {
    fn status_code(&self) -> StatusCode {
        match self {
            InventoryError::ValidationError(_) => StatusCode::BAD_REQUEST,
            InventoryError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[tracing::instrument(ret(Debug), name = "Fetching Inventory List", skip(pool), fields())]
pub async fn fetch_inventory(
    _body: web::Json<InventoryRequest>,
    pool: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, InventoryError> {
    let rapidor_invetory = sqlx::query_as::<_, ProductInventory>(
        "SELECT code, no_of_items FROM product_product limit 100",
    )
    .fetch_all(pool.get_ref())
    .await
    .context("Failed to fetch data from database")?;

    let _response = MyResponse {
        status: true,
        customer_message: "Inventory Fetched Sucessfully".to_string(),
        data: rapidor_invetory,
        success_code: "PYWS0000".to_string(),
    };
    // web::Json(response)
    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(name = "Sending Email", skip(email_client), fields())]
pub async fn send_email(email_client: web::Data<EmailClient>) -> impl Responder {
    let _responsed = email_client
        .send_email_smtp("kevin.norbert@acelrtech.com", "SANU", "apple".to_owned())
        .await;

    HttpResponse::Ok().body("Successfully send data")
}
