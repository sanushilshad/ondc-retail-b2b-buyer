use actix_web::http::StatusCode;
use actix_web::ResponseError;

use crate::general_utils::error_chain_fmt;
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
