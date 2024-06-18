use crate::general_utils::error_chain_fmt;
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};

use super::{ONDCBuyerErrorCode, ONDCResponse, ONDCResponseErrorBody, ONDErrorType};

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
    InvalidResponseError { path: Option<String> },
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
            ONDCBuyerError::InvalidResponseError { path } => (
                "Invalid request",
                ONDCBuyerErrorCode::InvalidResponseCode,
                path,
                ONDErrorType::JsonSchemaError,
            ),
        };

        let error_obj = ONDCResponseErrorBody {
            message: message.to_string(),
            code,
            path: path.to_owned(),
            r#type,
        };
        HttpResponse::build(status_code).json(ONDCResponse::error_response(None, error_obj))
    }
}
