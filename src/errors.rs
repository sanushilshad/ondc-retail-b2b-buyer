use actix_web::{HttpResponse, ResponseError};
use reqwest::StatusCode;

use crate::{schemas::GenericResponse, utils::error_chain_fmt};

#[derive(Debug)]
pub enum DatabaseError {
    MissingDatabasePassword,
    MissingDatabasePort,
    MissingDatabaseIP, // Add more error variants as needed
    DatabasePortMustbeNumber,
}

impl std::fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DatabaseError::MissingDatabasePassword => {
                write!(f, "Missing database password")
            } // Handle other error variants here

            DatabaseError::MissingDatabasePort => {
                write!(f, "Missing database port")
            }

            DatabaseError::MissingDatabaseIP => {
                write!(f, "Missing database IP")
            }

            DatabaseError::DatabasePortMustbeNumber => {
                write!(f, "Missing Port should be a numbers")
            }
        }
    }
}

impl std::error::Error for DatabaseError {}

#[derive(thiserror::Error)]

pub enum CustomJWTTokenError {
    #[error("Token expired")]
    Expired,
    #[error("{0}")]
    Invalid(String),
}

impl std::fmt::Debug for CustomJWTTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

#[derive(thiserror::Error)]
pub enum RequestMetaError {
    #[error("{0}")]
    ValidationStringError(String),
}

impl std::fmt::Debug for RequestMetaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for RequestMetaError {
    fn status_code(&self) -> StatusCode {
        match self {
            RequestMetaError::ValidationStringError(_) => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();
        let status_code_str = status_code.as_str();
        let inner_error_msg = match self {
            RequestMetaError::ValidationStringError(message) => message.to_string(),
        };

        HttpResponse::build(status_code).json(GenericResponse::error(
            &inner_error_msg,
            status_code_str,
            Some(()),
        ))
    }
}
