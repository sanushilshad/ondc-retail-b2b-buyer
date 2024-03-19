use actix_web::{error::ErrorInternalServerError, FromRequest, HttpMessage};
use chrono::{DateTime, NaiveTime, Utc};
use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgHasArrayType;
use std::{
    fmt::{self, Debug},
    future::{ready, Ready},
};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    domain::{subscriber_email::deserialize_subscriber_email, EmailObject},
    schemas::Status,
};

use super::errors::AuthError;

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
#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticateRequest {
    pub scope: AuthenticationScope,
    pub identifier: String,
    // #[serde(with = "SecretString")]
    #[schema(value_type = String)]
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

#[derive(Serialize, Deserialize, Debug, sqlx::Type, PartialEq, ToSchema)]
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

impl fmt::Display for UserType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
        // or, alternatively:
        // fmt::Debug::fmt(self, f)
    }
}

impl UserType {
    pub fn to_lowercase_string(&self) -> String {
        format!("{:?}", self).to_lowercase()
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum CreateUserType {
    Guest,
    User,
    Member,
    Agent,
}
impl From<CreateUserType> for UserType {
    fn from(create_user_type: CreateUserType) -> Self {
        match create_user_type {
            CreateUserType::Guest => UserType::Guest,
            CreateUserType::User => UserType::User,
            CreateUserType::Member => UserType::Member,
            CreateUserType::Agent => UserType::Agent,
        }
    }
}

#[sqlx(type_name = "user_auth_identifier_scope", rename_all = "lowercase")]
#[derive(Serialize, Deserialize, Debug, sqlx::Type, ToSchema)]
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

#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserAccount {
    pub username: String,
    pub mobile_no: String,
    pub international_dialing_code: String,
    #[schema(value_type = String)]
    pub password: Secret<String>,
    #[serde(deserialize_with = "deserialize_subscriber_email")]
    pub email: EmailObject, //NOTE: email_address crate can be used if needed,
    pub display_name: String,
    pub is_test_user: bool,
    pub user_type: UserType,
    pub source: DataSource,
}

#[derive(Serialize, Deserialize, Debug, sqlx::Type, Clone, ToSchema)]
#[sqlx(rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum MaskingType {
    NA,
    Encrypt,
    PartialMask,
    FullMask,
}

// impl PgHasArrayType for UserVectors {
//     fn array_type_info() -> sqlx::postgres::PgTypeInfo {
//         sqlx::postgres::PgTypeInfo::with_name("_header_pair")
//     }
// }

#[derive(Serialize, Deserialize, Debug, sqlx::Type, PartialEq, Clone, ToSchema)]
#[sqlx(type_name = "vector_type", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum VectorType {
    PanCardNo,
    Gstin,
    AadhaarCardNo,
    MobileNo,
    Email,
    InternationalDialingCode,
    UpiId,
    BankAccountNumber,
    IfscCode,
    LicenseNumber,
    PassportNo,
    VoterIdNo,
    Ssn,
    Tin,
    ExportLicenseNo,
    FssaiLicenseNumber,
}

#[derive(Serialize, Deserialize, Debug, sqlx::Type, Clone, ToSchema)]
#[sqlx(type_name = "vectors")]
pub struct UserVectors {
    pub key: VectorType,
    pub value: String,
    pub masking: MaskingType,
    pub verified: bool,
}

impl PgHasArrayType for UserVectors {
    fn array_type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("_vectors")
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, ToSchema)]
pub struct UserAccount {
    #[schema(value_type = String)]
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
    pub is_deleted: bool,
    pub user_role: String,
}

impl FromRequest for UserAccount {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    /// Implement the `from_request` method to extract and wrap the authenticated user.
    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        // Attempt to retrieve the user information from request extensions.
        let value = req.extensions().get::<UserAccount>().cloned();

        // Check if the user information was successfully retrieved.
        let result = match value {
            Some(user) => Ok(user),
            None => Err(ErrorInternalServerError(AuthError::UnexpectedStringError(
                "Somrthing went wrong".to_string(),
            ))),
        };

        // Return a ready future with the result.
        ready(result)
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct BasicBusinessAccount {
    pub company_name: String,
    #[schema(value_type = String)]
    pub id: Uuid,
    pub customer_type: CustomerType,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AuthData {
    pub user: UserAccount,
    #[serde(serialize_with = "round_serialize")]
    #[schema(value_type = String)]
    pub token: Secret<String>,
    pub business_account_list: Vec<BasicBusinessAccount>,
}

fn round_serialize<S>(x: &Secret<String>, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    s.serialize_str(x.expose_secret())
}

// impl<'__s> utoipa::ToSchema<'__s> for Secret<String> {
//     fn schema() -> (
//         &'__s str,
//         utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
//     ) {
//     }
// }

// impl ToSchema for Secret<String> {
// fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
//     schema_for!(String)
// }

// fn schema_name() -> String {
//     "Secret<String>".to_string()
// }

// fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
//     schema_for!(String)
// }
// }

#[derive(Serialize, Deserialize, Debug, sqlx::Type)]
#[sqlx(type_name = "auth_context_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AuthContextType {
    UserAccount,
    BusinessAccount,
}

impl PgHasArrayType for AuthContextType {
    fn array_type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("_auth_context_type")
    }
}

#[derive(Debug)]
pub struct AuthMechanism {
    pub id: Uuid,
    pub user_id: Uuid,
    pub auth_scope: AuthenticationScope,
    pub auth_identifier: String,
    pub secret: Option<Secret<String>>,
    pub is_active: Status,
    pub valid_upto: Option<DateTime<Utc>>,
    pub auth_context: AuthContextType,
}

pub struct AccountRole {
    pub id: Uuid,
    pub role_name: String,
    pub role_status: Status,
    pub is_deleted: bool,
}

#[derive(Serialize, Deserialize, Debug, sqlx::Type, PartialEq, Copy, Clone, ToSchema)]
#[sqlx(type_name = "customer_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum CustomerType {
    NA,
    Buyer,
    Seller,
    Brand,
    LogisticPartner,
    PaymentAggregator,
    VirtualOperator,
    ExternalPartner,
}

#[derive(Serialize, Deserialize, Debug, sqlx::Type, PartialEq, ToSchema)]
#[sqlx(type_name = "data_source", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum DataSource {
    PlaceOrder,
    Ondc,
    Rapidor,
}

#[derive(Serialize, Deserialize, Debug, sqlx::Type, PartialEq, ToSchema)]
#[sqlx(type_name = "trade_type", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum TradeType {
    Domestic,
    Export,
}

impl PgHasArrayType for TradeType {
    fn array_type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("_trade_type")
    }
}
#[derive(Serialize, Deserialize, Debug, sqlx::Type, PartialEq, ToSchema)]
#[sqlx(type_name = "merchant_type", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum MerchantType {
    FPO,
    Manufacturer,
    Restaurant,
    Grocery,
    Mall,
    Supermart,
    Store,
    Other,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BulkAuthMechanismInsert<'a> {
    pub id: Vec<Uuid>,
    pub user_id_list: Vec<Uuid>,
    pub auth_scope: Vec<AuthenticationScope>,
    #[serde(borrow)]
    pub auth_identifier: Vec<&'a str>,
    pub secret: Vec<String>,
    pub is_active: Vec<Status>,
    pub created_on: Vec<DateTime<Utc>>,
    pub created_by: Vec<Uuid>,
    pub auth_context: Vec<AuthContextType>,
}

#[derive(Serialize, Deserialize, Debug, sqlx::Type, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct KYCProof {
    pub key: VectorType,
    pub kyc_id: String,
    pub value: Vec<String>,
}

#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateBusinessAccount {
    pub company_name: String,
    pub is_test_account: bool,
    pub customer_type: CustomerType,
    pub source: DataSource,
    pub mobile_no: String,
    pub email: EmailObject,
    pub trade_type: Vec<TradeType>,
    pub merchant_type: MerchantType,
    pub opening_time: Option<NaiveTime>,
    pub closing_time: Option<NaiveTime>,
    pub proofs: Vec<KYCProof>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BusinessAccount {
    pub id: Uuid,
    pub company_name: String,
}
