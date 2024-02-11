use secrecy::Secret;
use serde::Serialize;
use sqlx::{types::Json, FromRow};
use uuid::Uuid;

use super::schemas::{AuthenticationScope, UserVectors};

#[derive(Serialize, FromRow)]
pub struct RapidorCustomerModel {
    domain: String,
    database: String,
}

#[derive(Debug, FromRow)]
pub struct AuthMechanismModel {
    pub user_id: Uuid,
    pub auth_scope: AuthenticationScope,
    pub auth_identifier: String,
    pub secret: Option<Secret<String>>,
}

#[derive(Debug, FromRow)]
pub struct UserAccountModel {
    pub id: Uuid,
    pub username: String,
    pub mobile_no: String,
    pub email: String,
    pub is_active: bool,

    pub vectors: Json<Option<Vec<UserVectors>>>,
}

// impl FromRow<'_, PgRow> for UserAccount {
//     fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
//         Ok(UserAccount {
//             id: row.try_get("id")?,
//             username: row.try_get("username")?,
//             email: row.try_get("email")?,
//             is_active: row.try_get("is_active")?, // Ensure that "is_active" column name matches your database schema
//             vectors: Json(row.try_get::<UserVectors, _>("vectors")?),
//         })
//     }
// }
