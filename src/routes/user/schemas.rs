use chrono::{DateTime, Utc};
use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgHasArrayType;
use std::fmt::Debug;
use uuid::Uuid;

use crate::{
    domain::{subscriber_email::deserialize_subscriber_email, EmailObject},
    schemas::Status,
};

// macro_rules! impl_serialize_format {
//     ($struct_name:ident, $trait_name:path) => {
//         impl $trait_name for $struct_name {
//             fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//                 fmt_json(self, f)
//             }
//         }
//     };
// }

// #[derive(Debug)]
// struct SecretString(Secret<String>);

// impl_serialize_format!(AuthenticateRequest, Debug);
// #[strum(serialize_all = "snake_case")]
// #[derive(Debug, Deserialize, Serialize)]
// #[serde(rename_all = "lowercase")]

// pub enum AuthenticationScope {
//     OTP,
//     Password,
// }
// impl_serialize_format!(AuthenticateRequest, Display);
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticateRequest {
    pub scope: AuthenticationScope,
    pub identifier: String,
    // #[serde(with = "SecretString")]
    pub secret: Secret<String>,
}

// impl Serialize for SecretString {
//     fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
//         serializer.serialize_str(&self.0.expose_secret())
//     }
// }

// pub struct AuthData {
//     token: String,
// }

// pub struct AuthResponse {
//     data: AuthData,
// }

#[derive(Serialize, Deserialize, Debug, sqlx::Type)]
#[sqlx(type_name = "user_type", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum UserType {
    Guest,
    User,
    Member,
    Agent,
    Superadmin,
    Admin,
}

#[derive(Serialize, Deserialize, Debug, sqlx::Type)]
#[sqlx(type_name = "user_auth_identifier_scope", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum AuthenticationScope {
    Otp,
    Password,
    Google,
    Facebook,
    Microsoft,
    Apple,
    Token,
    AuthApp,
    Qr,
    Email,
}

impl PgHasArrayType for AuthenticationScope {
    fn array_type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("_user_auth_identifier_scope")
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserAccount {
    pub username: String,
    pub mobile_no: String,
    pub international_dialing_code: String,
    pub password: Secret<String>,
    #[serde(deserialize_with = "deserialize_subscriber_email")]
    pub email: EmailObject, //NOTE: email_address crate cah be used if needed,
    pub display_name: String,
    pub is_test_user: bool,
}

#[derive(Serialize, Deserialize, Debug, sqlx::Type)]
#[sqlx(rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum MaskingType {
    NA,
    Encrypt,
    PartialMask,
    FullMask,
}

#[derive(Serialize, Deserialize, Debug, sqlx::Type)]
#[sqlx(type_name = "vectors")]
pub struct UserVectors {
    pub key: String,
    pub value: String,
    pub masking: MaskingType,
    pub verified: bool,
}

// impl PgHasArrayType for UserVectors {
//     fn array_type_info() -> sqlx::postgres::PgTypeInfo {
//         sqlx::postgres::PgTypeInfo::with_name("_header_pair")
//     }
// }

#[derive(Debug, Deserialize, Serialize)]
pub struct UserAccount {
    pub id: Uuid,
    pub username: String,
    pub mobile_no: String,
    pub email: String,
    pub is_active: Status,
    pub display_name: String,
    pub vectors: Vec<Option<UserVectors>>,
    pub international_dialing_code: String,
    pub user_account_number: String,
    pub alt_user_account_number: String,
    pub is_test_user: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthData {
    pub user: UserAccount,
    #[serde(serialize_with = "round_serialize")]
    pub token: Secret<String>,
    pub business_account_list: Vec<Option<String>>,
}

fn round_serialize<S>(x: &Secret<String>, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    s.serialize_str(x.expose_secret())
}

#[derive(Debug)]
pub struct AuthMechanism {
    pub id: Uuid,
    pub user_id: Uuid,
    pub auth_scope: AuthenticationScope,
    pub auth_identifier: String,
    pub secret: Option<Secret<String>>,
    pub is_active: bool,
    pub valid_upto: Option<DateTime<Utc>>,
}
