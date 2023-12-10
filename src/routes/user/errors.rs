use crate::schemas::GenericResponse;
use crate::utils::error_chain_fmt;
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};

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

    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();
        let status_code_str = status_code.as_str();
        let response = match self {
            AuthError::InvalidCredentials(inner_error) => {
                GenericResponse::error(&inner_error.to_string(), status_code_str, None)
            }
            AuthError::UnexpectedError(inner_error) => {
                GenericResponse::error(&inner_error.to_string(), status_code_str, None)
            }
            AuthError::ValidationError(inner_error) => {
                GenericResponse::error(&inner_error.to_string(), status_code_str, None)
            }
            AuthError::ValidationStringError(message) => {
                GenericResponse::error(message, status_code_str, Some(()))
            }
        };

        HttpResponse::build(self.status_code()).json(response)
    }
}
