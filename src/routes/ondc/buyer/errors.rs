use crate::general_utils::error_chain_fmt;
use crate::routes::ondc::{ONDCResponse, ONDCResponseErrorBody, ONDErrorType};
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};

use super::schemas::ONDCBuyerErrorCode;
#[allow(dead_code)]
#[derive(thiserror::Error)]
pub enum ONDCProductSearchError {
    #[error("{0}")]
    ValidationError(String),

    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for ONDCProductSearchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for ONDCProductSearchError {
    fn status_code(&self) -> StatusCode {
        match self {
            ONDCProductSearchError::ValidationError(_) => StatusCode::BAD_REQUEST,
            ONDCProductSearchError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
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

#[allow(dead_code)]
#[allow(clippy::enum_variant_names)]
#[derive(thiserror::Error)]
pub enum ONDCBuyerError {
    #[error("Stale Request")]
    BuyerStaleError { path: Option<String> },
    #[error("Internal Server Error")]
    BuyerInternalServerError { path: Option<String> },
    #[error("Response out of sequence")]
    BuyerResponseSequenceError { path: Option<String> },
    #[error("Invalid Response")]
    InvalidResponseError {
        path: Option<String>,
        message: String,
    },
}

impl std::fmt::Debug for ONDCBuyerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for ONDCBuyerError {
    fn status_code(&self) -> StatusCode {
        match self {
            ONDCBuyerError::BuyerStaleError { .. } => StatusCode::BAD_REQUEST,
            ONDCBuyerError::BuyerInternalServerError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            ONDCBuyerError::BuyerResponseSequenceError { .. } => StatusCode::BAD_REQUEST,
            ONDCBuyerError::InvalidResponseError { .. } => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();

        let (message, code, path, r#type) = match self {
            ONDCBuyerError::BuyerStaleError { path } => (
                "Stale Request",
                ONDCBuyerErrorCode::StaleRequestCode,
                path,
                ONDErrorType::CoreError,
            ),
            ONDCBuyerError::BuyerInternalServerError { path } => (
                "Internal Server Error",
                ONDCBuyerErrorCode::InternalErrorCode,
                path,
                ONDErrorType::CoreError,
            ),
            ONDCBuyerError::BuyerResponseSequenceError { path } => (
                "Response out of sequence",
                ONDCBuyerErrorCode::ResponseSequenceCode,
                path,
                ONDErrorType::CoreError,
            ),
            ONDCBuyerError::InvalidResponseError { path, message } => (
                message.as_str(),
                ONDCBuyerErrorCode::InvalidResponseCode,
                path,
                ONDErrorType::JsonSchemaError,
            ),
        };

        let error_obj: ONDCResponseErrorBody<ONDCBuyerErrorCode> = ONDCResponseErrorBody {
            message: message.to_string(),
            code,
            path: path.to_owned(),
            r#type,
        };
        HttpResponse::build(status_code).json(ONDCResponse::error_response(None, error_obj))
    }
}
