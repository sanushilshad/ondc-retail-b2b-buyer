use crate::utils::error_chain_fmt;
use actix_web::http::StatusCode;
use actix_web::ResponseError;

#[derive(thiserror::Error)]
pub enum AuthError {
    #[error("Invalid credentials.")]
    InvalidCredentials(#[source] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
    #[error("{0}")]
    ValidationStringError(String),
    #[error("Authentication failed")]
    ValidationError(#[source] anyhow::Error),
}

impl std::fmt::Debug for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for AuthError {
    fn status_code(&self) -> StatusCode {
        match self {
            AuthError::InvalidCredentials(_) => StatusCode::BAD_REQUEST,
            AuthError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AuthError::ValidationError(_) => StatusCode::BAD_REQUEST,
            AuthError::ValidationStringError(_) => StatusCode::BAD_REQUEST,
        }
    }
}
