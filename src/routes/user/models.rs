use secrecy::Secret;
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

use super::schemas::AuthenticationScope;

#[derive(Serialize, FromRow)]
pub struct RapidorCustomer {
    domain: String,
    database: String,
}

#[derive(Debug, FromRow)]
pub struct AuthMechanism {
    pub user_id: Uuid,
    pub auth_scope: AuthenticationScope,
    pub auth_identifier: String,
    pub secret: Secret<String>,
}
