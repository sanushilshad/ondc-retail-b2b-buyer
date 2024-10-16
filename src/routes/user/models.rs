use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{types::Json, FromRow};
use uuid::Uuid;

use crate::schemas::{KycStatus, Status};

use super::schemas::{AuthContextType, AuthenticationScope, CustomerType, UserVector, VectorType};

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
    pub is_active: Status,
    pub valid_upto: Option<DateTime<Utc>>,
    pub auth_context: AuthContextType,
}

#[derive(Debug, FromRow)]
pub struct UserAccountModel {
    pub id: Uuid,
    pub username: String,
    pub mobile_no: String,
    pub email: String,
    pub is_active: Status,
    pub display_name: String,
    pub vectors: Json<Vec<Option<UserVector>>>,
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
#[allow(dead_code)]
#[derive(Debug, FromRow)]
pub struct UserRoleModel {
    pub id: Uuid,
    pub role_name: String,
    pub role_status: Status,
    pub created_on: DateTime<Utc>,
    pub created_by: String,
    pub is_deleted: bool,
}
#[allow(dead_code)]
#[derive(Debug, FromRow)]
pub struct BusinessAccountModel {
    pub id: Uuid,
    pub company_name: String,
    pub customer_type: CustomerType,
    pub vectors: Json<Vec<UserVector>>,
    pub kyc_status: KycStatus,
    pub is_active: Status,
}
#[allow(dead_code)]
#[derive(Debug, FromRow)]
pub struct UserBusinessRelationAccountModel {
    pub id: Uuid,
    pub company_name: String,
    pub customer_type: CustomerType,
    pub vectors: Json<Vec<UserVector>>,
    pub kyc_status: KycStatus,
    pub is_active: Status,
    pub verified: bool,
    pub is_deleted: bool,
    pub default_vector_type: VectorType,
}
