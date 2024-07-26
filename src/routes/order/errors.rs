use crate::{errors::GenericError, utils::error_chain_fmt};

#[allow(clippy::enum_variant_names)]
#[derive(thiserror::Error)]
pub enum SelectOrderError {
    #[error("{0}")]
    ValidationError(String),
    #[error("{0}")]
    InvalidDataError(String),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
    #[error("{0}")]
    DatabaseError(String, anyhow::Error),
}

impl std::fmt::Debug for SelectOrderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl From<SelectOrderError> for GenericError {
    fn from(err: SelectOrderError) -> GenericError {
        match err {
            SelectOrderError::ValidationError(message) => GenericError::ValidationError(message),
            SelectOrderError::UnexpectedError(error) => GenericError::UnexpectedError(error),
            SelectOrderError::DatabaseError(message, error) => {
                GenericError::DatabaseError(message, error)
            }
            SelectOrderError::InvalidDataError(message) => {
                GenericError::SerializationError(message)
            }
        }
    }
}
