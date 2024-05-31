use actix_web::{HttpResponse, ResponseError};
use reqwest::StatusCode;

use crate::schemas::GenericResponse;
use crate::utils::error_chain_fmt;

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

#[derive(thiserror::Error)]
pub enum GenericError {
    #[error("{0}")]
    ValidationStringError(String),
}

impl std::fmt::Debug for GenericError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

// impl From<anyhow::Error> for GenericError {
//     fn from(err: anyhow::Error) -> Self {
//         // Convert the error details from anyhow::Error to your GenericError format
//         GenericError {
//             message: err.to_string(),
//         } // Example conversion
//     }
// }

// impl From<GenericError> for ProductSearchError {
//     fn from(err: GenericError) -> Self {
//         ProductSearchError::ValidationError(err) // Replace with the appropriate variant
//     }
// }

// impl From<GenericError> for ProductSearchError {
//     fn from(err: GenericError) -> Self {
//         // Implement conversion logic here (e.g., match on specific error types within GenericError)
//         ProductSearchError::ValidationError(err.to_string())
//     }
// }

// impl ResponseError for GenericError {
//     fn status_code(&self) -> StatusCode {
//         match self {
//             GenericError::ValidationStringError(_) => StatusCode::BAD_REQUEST,
//         }
//     }

//     fn error_response(&self) -> HttpResponse {
//         let status_code = self.status_code();
//         let status_code_str = status_code.as_str();
//         let inner_error_msg = match self {
//             GenericError::ValidationStringError(message) => message.to_string(),
//         };

//         HttpResponse::build(status_code).json(GenericResponse::error(
//             &inner_error_msg,
//             status_code_str,
//             Some(()),
//         ))
//     }
// }
