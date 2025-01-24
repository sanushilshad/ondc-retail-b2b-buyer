use crate::errors::GenericError;
use crate::utils::error_chain_fmt;
#[allow(clippy::enum_variant_names)]
#[derive(thiserror::Error)]
pub enum PaymentOrderError {
    #[error("{0}")]
    ValidationError(String),
    #[error("{0}")]
    UnexpectedCustomError(String),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
    #[error("{0}")]
    DatabaseError(String, anyhow::Error),
}

impl std::fmt::Debug for PaymentOrderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl From<PaymentOrderError> for GenericError {
    fn from(err: PaymentOrderError) -> GenericError {
        match err {
            PaymentOrderError::ValidationError(message) => GenericError::ValidationError(message),
            PaymentOrderError::UnexpectedError(error) => GenericError::UnexpectedError(error),
            PaymentOrderError::UnexpectedCustomError(error) => {
                GenericError::UnexpectedCustomError(error)
            }
            PaymentOrderError::DatabaseError(message, error) => {
                GenericError::DatabaseError(message, error)
            }
        }
    }
}
