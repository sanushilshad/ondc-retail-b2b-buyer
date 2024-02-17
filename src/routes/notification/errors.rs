use actix_web::{HttpResponse, ResponseError};
use reqwest::StatusCode;

use crate::{schemas::GenericResponse, utils::error_chain_fmt};

#[derive(thiserror::Error)]
pub enum OTPError {
    #[error("{0}")]
    ValidationStringError(String),
    #[error("Authentication failed")]
    ValidationError(#[source] anyhow::Error),
    #[error("{0}")]
    UnexpectedStringError(String),
    #[error("{0}")]
    DatabaseError(String, anyhow::Error),
}

impl std::fmt::Debug for OTPError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for OTPError {
    fn status_code(&self) -> StatusCode {
        match self {
            OTPError::ValidationError(_) => StatusCode::BAD_REQUEST,
            OTPError::ValidationStringError(_) => StatusCode::BAD_REQUEST,
            OTPError::UnexpectedStringError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            OTPError::DatabaseError(_, _) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();
        let status_code_str = status_code.as_str();
        let inner_error_msg = match self {
            OTPError::ValidationError(inner_error) => inner_error.to_string(),
            OTPError::ValidationStringError(message) => message.to_string(),
            OTPError::UnexpectedStringError(message) => message.to_string(),
            OTPError::DatabaseError(message, _err) => message.to_string(),
        };

        HttpResponse::build(status_code).json(GenericResponse::error(
            &inner_error_msg,
            status_code_str,
            Some(()),
        ))
    }
}
