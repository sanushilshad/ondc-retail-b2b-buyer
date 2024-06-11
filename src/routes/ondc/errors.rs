use crate::general_utils::error_chain_fmt;
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};

use super::{ONDCErrorCode, ONDCResponse, ONDCResponseErrorBody, ONDErrorType};

#[allow(dead_code)]
#[derive(thiserror::Error)]
pub enum ONDCError {
    #[error("Stale Request")]
    StaleError { path: Option<String> },
    #[error("Internal Server Error")]
    InternalServerError { path: Option<String> },
    #[error("Response out of sequence")]
    ResponseSequence { path: Option<String> },
}

impl std::fmt::Debug for ONDCError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for ONDCError {
    fn status_code(&self) -> StatusCode {
        match self {
            ONDCError::StaleError { .. } => StatusCode::BAD_REQUEST,
            ONDCError::InternalServerError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            ONDCError::ResponseSequence { .. } => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();

        let (message, code, path, r#type) = match self {
            ONDCError::StaleError { path } => (
                "Stale Request",
                ONDCErrorCode::StaleRequestCode,
                path,
                ONDErrorType::CoreError,
            ),
            ONDCError::InternalServerError { path } => (
                "Internal Server Error",
                ONDCErrorCode::InternalErrorCode,
                path,
                ONDErrorType::CoreError,
            ),
            ONDCError::ResponseSequence { path } => (
                "Response out of sequence",
                ONDCErrorCode::ResponseSequenceCode,
                path,
                ONDErrorType::CoreError,
            ),
        };

        let error_obj = ONDCResponseErrorBody {
            message: message.to_string(),
            code: code,
            path: path.to_owned(),
            r#type: r#type,
        };
        HttpResponse::build(status_code).json(ONDCResponse::error_response(None, error_obj))
    }
}
