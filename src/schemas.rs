use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use crate::{errors::RequestMetaError, routes::user::schemas::AuthData};
use actix_web::{error::ErrorInternalServerError, FromRequest, HttpMessage};
//use bigdecimal::BigDecimal;
use futures_util::future::{ready, Ready};
use reqwest::{header, Client};
use secrecy::Secret;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{postgres::PgHasArrayType, types::BigDecimal};
use tokio::time::sleep;
use utoipa::{openapi::Object, ToSchema};
use uuid::Uuid;

#[derive(Serialize, Debug, ToSchema)]
#[aliases(EmptyGenericResponse = GenericResponse<Object>, AuthResponse = GenericResponse<AuthData>)]
pub struct GenericResponse<D> {
    pub status: bool,
    pub customer_message: String,
    pub code: String,
    pub data: Option<D>,
}

// impl Responder for GenericResponse {
//     fn respond_to(self, _req: &web::HttpRequest) -> HttpResponse {
//         HttpResponse::Ok().json(self)
//     }
// }
// impl<D: Serialize> std::fmt::Display for GenericResponse<D> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}", serde_json::json!(&self))
//     }
// }
impl<D> GenericResponse<D> {
    // Associated function for creating a success response
    pub fn success(message: &str, data: Option<D>) -> Self {
        Self {
            status: true,
            customer_message: String::from(message),
            code: String::from("200"),
            data,
        }
    }

    // Associated function for creating an error response
    pub fn error(message: &str, code: &str, data: Option<D>) -> Self {
        Self {
            status: false,
            customer_message: String::from(message),
            code: String::from(code),
            data,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, sqlx::Type, Clone, PartialEq, ToSchema)]
#[sqlx(rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "status")]
pub enum Status {
    Active,
    Inactive,
    Pending,
    Archived,
}

impl PgHasArrayType for Status {
    fn array_type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("_status")
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CommunicationType {
    Type1,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct JWTClaims {
    pub sub: Uuid,
    pub exp: usize,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub enum CountryCode {
    IND,
    SGP,
}

#[derive(Debug, Clone)]
pub struct RequestMetaData {
    pub device_id: String,
    pub request_id: String,
    pub domain_uri: String,
}

impl FromRequest for RequestMetaData {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    /// Implement the `from_request` method to extract and wrap the authenticated user.
    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        // Attempt to retrieve the user information from request extensions.
        let value = req.extensions().get::<RequestMetaData>().cloned();

        // Check if the user information was successfully retrieved.
        let result = match value {
            Some(user) => Ok(user),
            None => Err(ErrorInternalServerError(
                RequestMetaError::ValidationStringError("Something went wrong".to_string()),
            )),
        };

        // Return a ready future with the result.
        ready(result)
    }
}

#[derive(Serialize, Deserialize, Debug, sqlx::Type, Clone, PartialEq)]
#[sqlx(type_name = "kyc_status", rename_all = "snake_case")] // Match the type name in PostgreSQL
#[serde(rename_all = "snake_case")] // Ensure JSON serialization matches PostgreSQL naming
pub enum KycStatus {
    Pending,
    OnHold,
    Rejected,
    Completed,
}

#[derive(Debug)]
struct RetryPolicy {
    max_retries: u32,
    backoff_value: f64,
}

impl RetryPolicy {
    fn new() -> Self {
        RetryPolicy {
            max_retries: 3,
            backoff_value: 5.0,
        }
    }
}

#[derive(Debug)]
struct NetworkResponse {
    status_code: u16,
    body: String,
}

#[derive(Debug, thiserror::Error)]
pub enum NetworkError {
    #[error("Request error: {0}")]
    Request(reqwest::Error),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Request timed out")]
    Timeout,
}

impl From<reqwest::Error> for NetworkError {
    fn from(err: reqwest::Error) -> Self {
        NetworkError::Request(err)
    }
}

#[derive(Debug)]
pub struct NetworkCall {
    pub client: Client,
}

impl NetworkCall {
    #[tracing::instrument(name = "Async Post Call", skip(), fields())]
    pub async fn async_post_call(
        &self,
        url: &str,
        payload: Option<&str>,
        headers: Option<HashMap<&str, &str>>,
    ) -> Result<NetworkResponse, NetworkError> {
        let mut req_headers = header::HeaderMap::new();
        if let Some(headers) = headers {
            for (key, value) in headers {
                req_headers.insert(
                    header::HeaderName::from_bytes(key.as_bytes()).unwrap(),
                    header::HeaderValue::from_str(value).unwrap(),
                );
            }
        }

        if payload.is_some() {
            req_headers.insert(
                header::CONTENT_TYPE,
                header::HeaderValue::from_static("application/json"),
            );
        }

        let body = payload.unwrap_or_default().to_string();
        let response = self
            .client
            .post(url)
            .headers(req_headers)
            .timeout(Duration::from_secs(10))
            .body(body)
            .send()
            .await?;

        let status_code = response.status();
        let body = response.text().await?;
        print!("Response: {}", body);
        Ok(NetworkResponse {
            status_code: status_code.as_u16(),
            body,
        })
    }
    #[tracing::instrument(name = "Async Post Call With Retry", skip(), fields())]
    pub async fn async_post_call_with_retry(
        &self,
        url: &str,
        payload: Option<&str>,
        headers: Option<HashMap<&str, &str>>,
    ) -> Result<Value, NetworkError> {
        let retry_policy = RetryPolicy::new();
        let mut response: Option<Value> = None;
        let mut current_backoff = 1.0;

        let start_time = Instant::now();
        for current_retry in 0..retry_policy.max_retries {
            tracing::info!("Retry attempt {}...", current_retry + 1);
            match self.async_post_call(url, payload, headers.to_owned()).await {
                Ok(network_response) => {
                    if network_response.status_code > 500 {
                        let error_message = network_response.body;
                        return Err(NetworkError::Validation(error_message));
                    }

                    match serde_json::from_str(&network_response.body) {
                        Ok(value) => response = Some(value),
                        Err(err) => {
                            tracing::error!("Deserialization error: {}", err);
                            return Err(NetworkError::Validation(format!(
                                "Failed to deserialize response: {}",
                                err
                            )));
                        }
                    }
                    break;
                }
                Err(NetworkError::Timeout) => {
                    tracing::warn!("Request timed out. Retrying...");
                }
                Err(NetworkError::Request(err)) => {
                    tracing::error!("Request error: {}", err);
                }
                Err(NetworkError::Validation(validation_error)) => {
                    tracing::error!("Validation error: {}", validation_error);
                }
            }

            let elapsed_time = start_time.elapsed().as_secs_f64();
            if elapsed_time > retry_policy.max_retries as f64 * retry_policy.backoff_value {
                tracing::warn!("Maximum retry attempts reached.");
                break;
            }

            sleep(Duration::from_secs_f64(current_backoff)).await;
            current_backoff *= retry_policy.backoff_value;
        }

        response
            .ok_or_else(|| NetworkError::Validation("No successful response received".to_string()))
    }
}

#[derive(Debug, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "ondc_np_fee_type", rename_all = "snake_case")]
pub enum FeeType {
    Percent,
    Amount,
}

impl std::fmt::Display for FeeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            FeeType::Percent => "percent",
            FeeType::Amount => "amount",
        };

        write!(f, "{}", s)
    }
}

#[derive(Debug)]
pub struct RegisteredNetworkParticipant {
    pub code: String,
    pub name: String,
    pub logo: String,
    pub signing_key: Secret<String>,
    pub id: Uuid,
    pub subscriber_id: String,
    pub subscriber_uri: String,
    pub long_description: String,
    pub short_description: String,
    pub fee_type: FeeType,
    pub fee_value: BigDecimal,
    pub unique_key_id: String,
}

#[derive(Debug, Serialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "ondc_network_participant_type", rename_all = "snake_case")]
pub enum ONDCNPType {
    Buyer,
    Seller,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct WebSocketParam {
    pub user_id: Uuid,
    pub business_id: Uuid,
    pub device_id: String,
}

impl WebSocketParam {
    pub fn get_key(&self) -> String {
        format!("{}#{}#{}", self.user_id, self.business_id, self.device_id)
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ONDCNetworkType {
    Bap,
    Bpp,
}
