use crate::{errors::GenericError, utils::error_chain_fmt};

#[allow(clippy::enum_variant_names)]
#[derive(thiserror::Error)]
pub enum OrderError {
    #[error("{0}")]
    ValidationError(String),
    #[error("{0}")]
    InvalidDataError(String),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
    #[error("{0}")]
    DatabaseError(String, anyhow::Error),
    #[error("{0}")]
    NotImplemented(String),
}

impl std::fmt::Debug for OrderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl From<OrderError> for GenericError {
    fn from(err: OrderError) -> GenericError {
        match err {
            OrderError::ValidationError(message) => GenericError::ValidationError(message),
            OrderError::UnexpectedError(error) => GenericError::UnexpectedError(error),
            OrderError::DatabaseError(message, error) => {
                GenericError::DatabaseError(message, error)
            }
            OrderError::NotImplemented(message) => GenericError::NotImplemented(message),
            OrderError::InvalidDataError(message) => GenericError::SerializationError(message),
        }
    }
}
