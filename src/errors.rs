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
