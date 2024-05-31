use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};

use crate::errors::GenericError;
use crate::general_utils::error_chain_fmt;
use crate::schemas::GenericResponse;
use serde_json::Error as SerdeError;

#[allow(clippy::enum_variant_names)]
#[derive(thiserror::Error)]
pub enum ProductSearchError {
    #[error("{0}")]
    ValidationError(String),

    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
    #[error("{0}")]
    DatabaseError(String, anyhow::Error),
}

impl std::fmt::Debug for ProductSearchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl From<GenericError> for ProductSearchError {
    fn from(err: GenericError) -> Self {
        match err {
            GenericError::ValidationStringError(msg) => ProductSearchError::ValidationError(msg),
        }
    }
}

impl From<SerdeError> for ProductSearchError {
    fn from(err: SerdeError) -> Self {
        ProductSearchError::ValidationError(err.to_string())
    }
}

impl ResponseError for ProductSearchError {
    fn status_code(&self) -> StatusCode {
        match self {
            ProductSearchError::ValidationError(_) => StatusCode::BAD_REQUEST,
            ProductSearchError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ProductSearchError::DatabaseError(_, _) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();
        let status_code_str = status_code.as_str();
        let inner_error_msg = match self {
            ProductSearchError::ValidationError(message) => message.to_string(),
            ProductSearchError::UnexpectedError(error_msg) => error_msg.to_string(),
            ProductSearchError::DatabaseError(message, _err) => message.to_string(),
        };

        HttpResponse::build(status_code).json(GenericResponse::error(
            &inner_error_msg,
            status_code_str,
            Some(()),
        ))
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
