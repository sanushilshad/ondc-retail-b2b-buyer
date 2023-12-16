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
        let inner_error_msg = match self {
            AuthError::InvalidCredentials(inner_error)
            | AuthError::UnexpectedError(inner_error)
            | AuthError::ValidationError(inner_error) => inner_error.to_string(),
            AuthError::ValidationStringError(message) => message.to_string(),
        };

        HttpResponse::build(status_code).json(GenericResponse::error(
            &inner_error_msg,
            status_code_str,
            Some(()),
        ))
    }
}

#[derive(thiserror::Error)]
pub enum UserRegistrationError {
    #[error("Duplicate email")]
    DuplicateEmail(#[source] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
    #[error("Duplicate mobile no")]
    DuplicateMobileNo(#[source] anyhow::Error),
}

impl std::fmt::Debug for UserRegistrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for UserRegistrationError {
    fn status_code(&self) -> StatusCode {
        match self {
            UserRegistrationError::DuplicateEmail(_) => StatusCode::BAD_REQUEST,
            UserRegistrationError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            UserRegistrationError::DuplicateMobileNo(_) => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();
        let status_code_str = status_code.as_str();
        let inner_error_msg = match self {
            UserRegistrationError::DuplicateEmail(inner_error)
            | UserRegistrationError::UnexpectedError(inner_error)
            | UserRegistrationError::DuplicateMobileNo(inner_error) => inner_error.to_string(),
        };

        HttpResponse::build(status_code).json(GenericResponse::error(
            &inner_error_msg,
            status_code_str,
            Some(()),
        ))
    }
}
