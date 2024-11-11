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

#[allow(clippy::enum_variant_names)]
#[derive(thiserror::Error)]
pub enum InitOrderError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for InitOrderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl From<InitOrderError> for GenericError {
    fn from(err: InitOrderError) -> GenericError {
        match err {
            InitOrderError::UnexpectedError(error) => GenericError::UnexpectedError(error),
        }
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(thiserror::Error)]
pub enum ConfirmOrderError {
    #[error("{0}")]
    ValidationError(String),
    #[error("{0}")]
    InvalidDataError(String),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for ConfirmOrderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl From<ConfirmOrderError> for GenericError {
    fn from(err: ConfirmOrderError) -> GenericError {
        match err {
            ConfirmOrderError::ValidationError(message) => GenericError::ValidationError(message),
            ConfirmOrderError::UnexpectedError(error) => GenericError::UnexpectedError(error),
            ConfirmOrderError::InvalidDataError(message) => {
                GenericError::SerializationError(message)
            }
        }
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(thiserror::Error)]
pub enum OrderStatusError {
    #[error("{0}")]
    ValidationError(String),

    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
    #[error("{0}")]
    DatabaseError(String, anyhow::Error),
}

impl std::fmt::Debug for OrderStatusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl From<OrderStatusError> for GenericError {
    fn from(err: OrderStatusError) -> GenericError {
        match err {
            OrderStatusError::ValidationError(message) => GenericError::ValidationError(message),
            OrderStatusError::UnexpectedError(error) => GenericError::UnexpectedError(error),
            OrderStatusError::DatabaseError(message, error) => {
                GenericError::DatabaseError(message, error)
            }
        }
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(thiserror::Error)]
pub enum OrderCancelError {
    #[error("{0}")]
    ValidationError(String),

    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for OrderCancelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl From<OrderCancelError> for GenericError {
    fn from(err: OrderCancelError) -> GenericError {
        match err {
            OrderCancelError::ValidationError(message) => GenericError::ValidationError(message),
            OrderCancelError::UnexpectedError(error) => GenericError::UnexpectedError(error),
        }
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(thiserror::Error)]
pub enum OrderUpdateError {
    #[error("{0}")]
    ValidationError(String),

    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
    #[error("{0}")]
    NotImplemented(String),
}

impl std::fmt::Debug for OrderUpdateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl From<OrderUpdateError> for GenericError {
    fn from(err: OrderUpdateError) -> GenericError {
        match err {
            OrderUpdateError::ValidationError(message) => GenericError::ValidationError(message),
            OrderUpdateError::UnexpectedError(error) => GenericError::UnexpectedError(error),
            OrderUpdateError::NotImplemented(message) => GenericError::NotImplemented(message),
        }
    }
}
