use crate::{errors::RequestMetaError, routes::user::schemas::AuthData};
use actix_web::{error::ErrorInternalServerError, FromRequest, HttpMessage};
use futures_util::future::{ready, Ready};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgHasArrayType;
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
            data: data,
        }
    }

    // Associated function for creating an error response
    pub fn error(message: &str, code: &str, data: Option<D>) -> Self {
        Self {
            status: false,
            customer_message: String::from(message),
            code: String::from(code),
            data: data,
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
