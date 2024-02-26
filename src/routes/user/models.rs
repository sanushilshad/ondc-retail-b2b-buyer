use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{types::Json, FromRow};
use uuid::Uuid;

use crate::schemas::Status;

use super::schemas::{AuthenticationScope, UserVectors};

#[derive(Serialize, FromRow)]
pub struct RapidorCustomerModel {
    domain: String,
    database: String,
}

#[derive(Debug, FromRow)]
pub struct AuthMechanismModel {
    pub id: Uuid,
    pub user_id: Uuid,
    pub auth_scope: AuthenticationScope,
    pub auth_identifier: String,
    pub secret: Option<String>,
    pub is_active: bool,
    pub valid_upto: Option<DateTime<Utc>>,
}

#[derive(Debug, FromRow)]
pub struct UserAccountModel {
    pub id: Uuid,
    pub username: String,
    pub mobile_no: String,
    pub email: String,
    pub is_active: Status,
    pub display_name: String,
    pub vectors: Json<Vec<Option<UserVectors>>>,
    pub international_dialing_code: String,
    pub user_account_number: String,
    pub alt_user_account_number: String,
    pub is_test_user: bool,
    pub is_deleted: bool,
    pub role_name: String,
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

#[derive(Debug, FromRow)]
pub struct UserRoleModel {
    pub id: Uuid,
    pub role_name: String,
    pub role_status: Status,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub is_deleted: bool,
}
